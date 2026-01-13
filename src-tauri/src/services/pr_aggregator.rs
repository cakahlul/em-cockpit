//! PR Aggregator Service
//!
//! Aggregates pull requests from multiple repositories,
//! providing stale detection and smart grouping.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::integrations::traits::{IntegrationError, PrFilter, PrState, PullRequest, PullRequestRepository};
use crate::services::CacheService;
use crate::system::TrayState;

/// Summary of PR status across repositories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrSummary {
    /// Total open PRs
    pub total_open: usize,
    /// PRs pending review (assigned to user)
    pub pending_review: usize,
    /// Stale PRs (exceeding threshold)
    pub stale_count: usize,
    /// PRs by repository
    pub by_repository: HashMap<String, usize>,
    /// Oldest stale PR age in hours
    pub oldest_stale_hours: Option<i64>,
    /// Tray state based on PR status
    pub tray_state: TrayState,
}

impl PrSummary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate tray state based on PR counts
    pub fn calculate_tray_state(stale_count: usize, pending_review: usize) -> TrayState {
        if stale_count > 0 {
            TrayState::Amber
        } else if pending_review == 0 {
            TrayState::Green
        } else {
            TrayState::Neutral
        }
    }
}

/// PR grouping options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrGrouping {
    ByRepository,
    ByAuthor,
    ByAge,
}

/// Grouped PR result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedPrs {
    pub label: String,
    pub prs: Vec<PullRequest>,
    pub stale_count: usize,
}

/// PR Aggregator configuration
#[derive(Debug, Clone)]
pub struct PrAggregatorConfig {
    pub stale_threshold_hours: i64,
    pub refresh_interval: Duration,
    pub repositories: Vec<String>,
}

impl Default for PrAggregatorConfig {
    fn default() -> Self {
        Self {
            stale_threshold_hours: 48,
            refresh_interval: Duration::minutes(2),
            repositories: Vec::new(),
        }
    }
}

impl PrAggregatorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stale_threshold(mut self, hours: i64) -> Self {
        self.stale_threshold_hours = hours;
        self
    }

    pub fn with_repositories(mut self, repos: Vec<String>) -> Self {
        self.repositories = repos;
        self
    }
}

/// PR Aggregator Service
pub struct PrAggregator<R: PullRequestRepository> {
    repo: Arc<R>,
    config: PrAggregatorConfig,
    cache: Option<Arc<CacheService>>,
    user_id: Option<String>,
}

impl<R: PullRequestRepository> PrAggregator<R> {
    pub fn new(repo: Arc<R>, config: PrAggregatorConfig) -> Self {
        Self {
            repo,
            config,
            cache: None,
            user_id: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<CacheService>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Get summary of all PRs
    pub async fn get_summary(&self) -> Result<PrSummary, IntegrationError> {
        // Check cache
        let cache_key = "pr_summary";
        if let Some(ref cache) = self.cache {
            if let Ok(cached) = cache.get::<PrSummary>(cache_key) {
                return Ok(cached);
            }
        }

        let prs = self.fetch_all_prs().await?;
        let summary = self.compute_summary(&prs);

        // Cache result
        if let Some(ref cache) = self.cache {
            let _ = cache.set(cache_key, &summary, self.config.refresh_interval);
        }

        Ok(summary)
    }

    /// Fetch all open PRs
    pub async fn fetch_all_prs(&self) -> Result<Vec<PullRequest>, IntegrationError> {
        let filter = PrFilter::new()
            .with_repositories(self.config.repositories.clone());

        self.repo.get_open_prs(&filter).await
    }

    /// Fetch PRs pending review by the user
    pub async fn get_pending_review(&self) -> Result<Vec<PullRequest>, IntegrationError> {
        if let Some(ref user_id) = self.user_id {
            let filter = PrFilter::new()
                .with_repositories(self.config.repositories.clone());
            self.repo.find_by_reviewer(user_id, &filter).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get stale PRs
    pub async fn get_stale_prs(&self) -> Result<Vec<PullRequest>, IntegrationError> {
        let prs = self.fetch_all_prs().await?;
        let threshold = Duration::hours(self.config.stale_threshold_hours);
        let now = Utc::now();

        Ok(prs
            .into_iter()
            .filter(|pr| now.signed_duration_since(pr.updated_at) > threshold)
            .collect())
    }

    /// Group PRs by a specific criteria
    pub fn group_prs(&self, prs: &[PullRequest], grouping: PrGrouping) -> Vec<GroupedPrs> {
        let mut groups: HashMap<String, Vec<PullRequest>> = HashMap::new();

        for pr in prs {
            let key = match grouping {
                PrGrouping::ByRepository => pr.repository.clone(),
                PrGrouping::ByAuthor => pr.author.name.clone(),
                PrGrouping::ByAge => self.age_bucket(pr),
            };
            groups.entry(key).or_default().push(pr.clone());
        }

        let threshold = Duration::hours(self.config.stale_threshold_hours);
        let now = Utc::now();

        let mut result: Vec<GroupedPrs> = groups
            .into_iter()
            .map(|(label, prs)| {
                let stale_count = prs
                    .iter()
                    .filter(|pr| now.signed_duration_since(pr.updated_at) > threshold)
                    .count();
                GroupedPrs {
                    label,
                    prs,
                    stale_count,
                }
            })
            .collect();

        // Sort by stale count descending
        result.sort_by(|a, b| b.stale_count.cmp(&a.stale_count));
        result
    }

    fn age_bucket(&self, pr: &PullRequest) -> String {
        let age = Utc::now().signed_duration_since(pr.updated_at);
        
        if age < Duration::hours(24) {
            "< 24 hours".to_string()
        } else if age < Duration::hours(48) {
            "24-48 hours".to_string()
        } else if age < Duration::days(7) {
            "2-7 days".to_string()
        } else {
            "> 7 days".to_string()
        }
    }

    fn compute_summary(&self, prs: &[PullRequest]) -> PrSummary {
        let threshold = Duration::hours(self.config.stale_threshold_hours);
        let now = Utc::now();

        let stale_prs: Vec<&PullRequest> = prs
            .iter()
            .filter(|pr| now.signed_duration_since(pr.updated_at) > threshold)
            .collect();

        let pending_review = if let Some(ref user_id) = self.user_id {
            prs.iter()
                .filter(|pr| pr.reviewers.iter().any(|r| r.user.id == *user_id || r.user.name == *user_id))
                .count()
        } else {
            0
        };

        let mut by_repository: HashMap<String, usize> = HashMap::new();
        for pr in prs {
            *by_repository.entry(pr.repository.clone()).or_default() += 1;
        }

        let oldest_stale_hours = stale_prs
            .iter()
            .map(|pr| now.signed_duration_since(pr.updated_at).num_hours())
            .max();

        PrSummary {
            total_open: prs.len(),
            pending_review,
            stale_count: stale_prs.len(),
            by_repository,
            oldest_stale_hours,
            tray_state: PrSummary::calculate_tray_state(stale_prs.len(), pending_review),
        }
    }
}

// Debug implementation
impl<R: PullRequestRepository> std::fmt::Debug for PrAggregator<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrAggregator")
            .field("config", &self.config)
            .field("user_id", &self.user_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::traits::{ChecksStatus, Reviewer, User};
    use std::sync::Mutex;

    struct MockPrRepo {
        prs: Mutex<Vec<PullRequest>>,
    }

    impl MockPrRepo {
        fn new(prs: Vec<PullRequest>) -> Self {
            Self { prs: Mutex::new(prs) }
        }
    }

    #[async_trait]
    impl PullRequestRepository for MockPrRepo {
        async fn find_by_id(&self, _repo: &str, id: &str) -> Result<PullRequest, IntegrationError> {
            let prs = self.prs.lock().unwrap();
            prs.iter()
                .find(|pr| pr.id == id)
                .cloned()
                .ok_or_else(|| IntegrationError::NotFound(format!("PR {} not found", id)))
        }

        async fn find_by_reviewer(&self, user_id: &str, _filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
            let prs = self.prs.lock().unwrap();
            Ok(prs
                .iter()
                .filter(|pr| pr.reviewers.iter().any(|r| r.user.id == user_id || r.user.name == user_id))
                .cloned()
                .collect())
        }

        async fn get_open_prs(&self, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
            let prs = self.prs.lock().unwrap();
            let result = if filter.repositories.is_empty() {
                prs.clone()
            } else {
                prs.iter()
                    .filter(|pr| filter.repositories.contains(&pr.repository))
                    .cloned()
                    .collect()
            };
            Ok(result.into_iter().take(filter.limit).collect())
        }
    }

    fn create_test_pr(id: &str, repo: &str, age_hours: i64) -> PullRequest {
        PullRequest {
            id: id.to_string(),
            repository: repo.to_string(),
            title: format!("PR {}", id),
            description: None,
            state: PrState::Open,
            author: User {
                id: "author1".to_string(),
                name: "Author".to_string(),
                email: None,
                avatar_url: None,
            },
            reviewers: vec![],
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            checks_status: ChecksStatus::Pass,
            is_stale: age_hours >= 48,
            updated_at: Utc::now() - Duration::hours(age_hours),
            created_at: Utc::now() - Duration::hours(age_hours + 10),
            url: format!("https://example.com/pr/{}", id),
        }
    }

    fn create_pr_with_reviewer(id: &str, reviewer_id: &str) -> PullRequest {
        let mut pr = create_test_pr(id, "repo1", 12);
        pr.reviewers = vec![Reviewer {
            user: User {
                id: reviewer_id.to_string(),
                name: reviewer_id.to_string(),
                email: None,
                avatar_url: None,
            },
            approved: false,
        }];
        pr
    }

    #[test]
    fn test_pr_summary_calculate_tray_state() {
        assert_eq!(PrSummary::calculate_tray_state(1, 0), TrayState::Amber);
        assert_eq!(PrSummary::calculate_tray_state(0, 0), TrayState::Green);
        assert_eq!(PrSummary::calculate_tray_state(0, 5), TrayState::Neutral);
    }

    #[test]
    fn test_pr_aggregator_config_builder() {
        let config = PrAggregatorConfig::new()
            .with_stale_threshold(24)
            .with_repositories(vec!["repo1".to_string()]);
        
        assert_eq!(config.stale_threshold_hours, 24);
        assert_eq!(config.repositories.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_all_prs() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),
            create_test_pr("2", "repo2", 20),
        ];
        let repo = Arc::new(MockPrRepo::new(prs));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config);

        let result = aggregator.fetch_all_prs().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_stale_prs() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),   // Not stale
            create_test_pr("2", "repo1", 50),   // Stale (>48h)
            create_test_pr("3", "repo1", 100),  // Stale
        ];
        let repo = Arc::new(MockPrRepo::new(prs));
        let config = PrAggregatorConfig::new().with_stale_threshold(48);
        let aggregator = PrAggregator::new(repo, config);

        let stale = aggregator.get_stale_prs().await.unwrap();
        assert_eq!(stale.len(), 2);
    }

    #[tokio::test]
    async fn test_get_pending_review() {
        let prs = vec![
            create_pr_with_reviewer("1", "user1"),
            create_pr_with_reviewer("2", "user2"),
            create_test_pr("3", "repo1", 10),
        ];
        let repo = Arc::new(MockPrRepo::new(prs));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config).with_user_id("user1");

        let pending = aggregator.get_pending_review().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "1");
    }

    #[tokio::test]
    async fn test_get_summary() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),
            create_test_pr("2", "repo1", 50),
            create_test_pr("3", "repo2", 100),
        ];
        let repo = Arc::new(MockPrRepo::new(prs));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config);

        let summary = aggregator.get_summary().await.unwrap();
        
        assert_eq!(summary.total_open, 3);
        assert_eq!(summary.stale_count, 2);
        assert_eq!(summary.by_repository.get("repo1"), Some(&2));
        assert_eq!(summary.by_repository.get("repo2"), Some(&1));
        assert_eq!(summary.tray_state, TrayState::Amber);
    }

    #[test]
    fn test_group_prs_by_repository() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),
            create_test_pr("2", "repo1", 20),
            create_test_pr("3", "repo2", 30),
        ];
        let repo = Arc::new(MockPrRepo::new(vec![]));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config);

        let groups = aggregator.group_prs(&prs, PrGrouping::ByRepository);
        
        assert_eq!(groups.len(), 2);
        let repo1_group = groups.iter().find(|g| g.label == "repo1");
        assert!(repo1_group.is_some());
        assert_eq!(repo1_group.unwrap().prs.len(), 2);
    }

    #[test]
    fn test_group_prs_by_age() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),   // < 24h
            create_test_pr("2", "repo1", 30),   // 24-48h
            create_test_pr("3", "repo1", 100),  // 2-7 days
        ];
        let repo = Arc::new(MockPrRepo::new(vec![]));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config);

        let groups = aggregator.group_prs(&prs, PrGrouping::ByAge);
        
        assert!(groups.len() >= 2);
    }

    #[test]
    fn test_groups_sorted_by_stale_count() {
        let prs = vec![
            create_test_pr("1", "repo1", 10),   // Not stale
            create_test_pr("2", "repo2", 100),  // Stale
            create_test_pr("3", "repo2", 100),  // Stale
        ];
        let repo = Arc::new(MockPrRepo::new(vec![]));
        let config = PrAggregatorConfig::new();
        let aggregator = PrAggregator::new(repo, config);

        let groups = aggregator.group_prs(&prs, PrGrouping::ByRepository);
        
        // repo2 should be first (more stale)
        assert_eq!(groups[0].label, "repo2");
        assert_eq!(groups[0].stale_count, 2);
    }
}
