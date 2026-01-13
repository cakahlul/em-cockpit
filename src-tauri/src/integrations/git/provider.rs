//! Git Hosting Provider Implementation
//!
//! Implements PullRequestRepository using Strategy Pattern for multiple providers.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::integrations::traits::{
    ChecksStatus, IntegrationError, PrFilter, PrState, PullRequest, PullRequestRepository,
    Reviewer, User,
};

/// Git provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitProviderType {
    Bitbucket,
    GitHub,
    GitLab,
}

/// Git hosting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub provider: GitProviderType,
    pub base_url: Option<String>,
    pub workspace: Option<String>,
    pub username: String,
    #[serde(skip)]
    pub token: Option<String>,
    pub repositories: Vec<String>,
}

impl GitConfig {
    pub fn bitbucket(workspace: &str, username: &str) -> Self {
        Self {
            provider: GitProviderType::Bitbucket,
            base_url: None,
            workspace: Some(workspace.to_string()),
            username: username.to_string(),
            token: None,
            repositories: Vec::new(),
        }
    }

    pub fn github(username: &str) -> Self {
        Self {
            provider: GitProviderType::GitHub,
            base_url: None,
            workspace: None,
            username: username.to_string(),
            token: None,
            repositories: Vec::new(),
        }
    }

    pub fn with_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    pub fn with_repositories(mut self, repos: Vec<String>) -> Self {
        self.repositories = repos;
        self
    }

    fn api_base_url(&self) -> String {
        if let Some(ref url) = self.base_url {
            return url.trim_end_matches('/').to_string();
        }
        match self.provider {
            GitProviderType::Bitbucket => "https://api.bitbucket.org/2.0".to_string(),
            GitProviderType::GitHub => "https://api.github.com".to_string(),
            GitProviderType::GitLab => "https://gitlab.com/api/v4".to_string(),
        }
    }
}

/// Git provider client using Strategy Pattern
#[derive(Debug)]
pub struct GitProvider {
    config: GitConfig,
    http_client: Client,
    stale_threshold: Duration,
}

impl GitProvider {
    pub fn new(config: GitConfig) -> Result<Self, IntegrationError> {
        if config.token.is_none() {
            return Err(IntegrationError::ConfigError(
                "Git token is required".to_string(),
            ));
        }

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| IntegrationError::Network(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            stale_threshold: Duration::hours(48),
        })
    }

    pub fn with_stale_threshold(mut self, hours: i64) -> Self {
        self.stale_threshold = Duration::hours(hours);
        self
    }

    fn auth_header(&self) -> (&'static str, String) {
        let token = self.config.token.as_deref().unwrap_or("");
        match self.config.provider {
            GitProviderType::Bitbucket => {
                use base64::Engine;
                let credentials = format!("{}:{}", self.config.username, token);
                (
                    "Authorization",
                    format!(
                        "Basic {}",
                        base64::engine::general_purpose::STANDARD.encode(credentials)
                    ),
                )
            }
            GitProviderType::GitHub => ("Authorization", format!("Bearer {}", token)),
            GitProviderType::GitLab => ("PRIVATE-TOKEN", token.to_string()),
        }
    }

    fn is_stale(&self, updated_at: &chrono::DateTime<chrono::Utc>) -> bool {
        Utc::now().signed_duration_since(*updated_at) > self.stale_threshold
    }

    async fn fetch_bitbucket_prs(&self, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
        let workspace = self.config.workspace.as_ref()
            .ok_or_else(|| IntegrationError::ConfigError("Workspace required for Bitbucket".to_string()))?;
        
        let repos = if filter.repositories.is_empty() {
            &self.config.repositories
        } else {
            &filter.repositories
        };

        let mut all_prs = Vec::new();
        let (header_name, header_value) = self.auth_header();

        for repo in repos {
            let url = format!(
                "{}/repositories/{}/{}/pullrequests?state=OPEN",
                self.config.api_base_url(), workspace, repo
            );

            let response = self.http_client
                .get(&url)
                .header(header_name, &header_value)
                .send()
                .await?;

            if response.status().as_u16() == 200 {
                let result: BitbucketPrList = response
                    .json()
                    .await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;

                for pr in result.values {
                    let mapped = self.map_bitbucket_pr(&pr, repo);
                    if !filter.stale_only || mapped.is_stale {
                        all_prs.push(mapped);
                    }
                }
            }
        }

        Ok(all_prs.into_iter().take(filter.limit).collect())
    }

    fn map_bitbucket_pr(&self, pr: &BitbucketPr, repo: &str) -> PullRequest {
        let updated_at = pr.updated_on;
        PullRequest {
            id: pr.id.to_string(),
            repository: repo.to_string(),
            title: pr.title.clone(),
            description: pr.description.clone(),
            state: PrState::Open,
            author: User {
                id: pr.author.uuid.clone(),
                name: pr.author.display_name.clone(),
                email: None,
                avatar_url: pr.author.links.avatar.as_ref().map(|l| l.href.clone()),
            },
            reviewers: pr.reviewers.iter().map(|r| Reviewer {
                user: User {
                    id: r.uuid.clone(),
                    name: r.display_name.clone(),
                    email: None,
                    avatar_url: None,
                },
                approved: false,
            }).collect(),
            source_branch: pr.source.branch.name.clone(),
            target_branch: pr.destination.branch.name.clone(),
            checks_status: ChecksStatus::None,
            is_stale: self.is_stale(&updated_at),
            updated_at,
            created_at: pr.created_on,
            url: pr.links.html.href.clone(),
        }
    }

    async fn fetch_github_prs(&self, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
        let repos = if filter.repositories.is_empty() {
            &self.config.repositories
        } else {
            &filter.repositories
        };

        let mut all_prs = Vec::new();
        let (header_name, header_value) = self.auth_header();

        for repo in repos {
            let url = format!(
                "{}/repos/{}/pulls?state=open",
                self.config.api_base_url(), repo
            );

            let response = self.http_client
                .get(&url)
                .header(header_name, &header_value)
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "em-cockpit")
                .send()
                .await?;

            if response.status().as_u16() == 200 {
                let prs: Vec<GitHubPr> = response
                    .json()
                    .await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;

                for pr in prs {
                    let mapped = self.map_github_pr(&pr, repo);
                    if !filter.stale_only || mapped.is_stale {
                        all_prs.push(mapped);
                    }
                }
            }
        }

        Ok(all_prs.into_iter().take(filter.limit).collect())
    }

    fn map_github_pr(&self, pr: &GitHubPr, repo: &str) -> PullRequest {
        let updated_at = pr.updated_at;
        PullRequest {
            id: pr.number.to_string(),
            repository: repo.to_string(),
            title: pr.title.clone(),
            description: pr.body.clone(),
            state: if pr.draft { PrState::Draft } else { PrState::Open },
            author: User {
                id: pr.user.id.to_string(),
                name: pr.user.login.clone(),
                email: None,
                avatar_url: Some(pr.user.avatar_url.clone()),
            },
            reviewers: pr.requested_reviewers.iter().map(|r| Reviewer {
                user: User {
                    id: r.id.to_string(),
                    name: r.login.clone(),
                    email: None,
                    avatar_url: Some(r.avatar_url.clone()),
                },
                approved: false,
            }).collect(),
            source_branch: pr.head.ref_name.clone(),
            target_branch: pr.base.ref_name.clone(),
            checks_status: ChecksStatus::None,
            is_stale: self.is_stale(&updated_at),
            updated_at,
            created_at: pr.created_at,
            url: pr.html_url.clone(),
        }
    }
}

#[async_trait]
impl PullRequestRepository for GitProvider {
    async fn find_by_id(&self, repo: &str, id: &str) -> Result<PullRequest, IntegrationError> {
        let (header_name, header_value) = self.auth_header();
        
        let url = match self.config.provider {
            GitProviderType::Bitbucket => {
                let workspace = self.config.workspace.as_ref()
                    .ok_or_else(|| IntegrationError::ConfigError("Workspace required".to_string()))?;
                format!("{}/repositories/{}/{}/pullrequests/{}", 
                    self.config.api_base_url(), workspace, repo, id)
            }
            GitProviderType::GitHub => {
                format!("{}/repos/{}/pulls/{}", self.config.api_base_url(), repo, id)
            }
            GitProviderType::GitLab => {
                format!("{}/projects/{}/merge_requests/{}", 
                    self.config.api_base_url(), urlencoding::encode(repo), id)
            }
        };

        let response = self.http_client
            .get(&url)
            .header(header_name, header_value)
            .header("User-Agent", "em-cockpit")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                match self.config.provider {
                    GitProviderType::Bitbucket => {
                        let pr: BitbucketPr = response.json().await
                            .map_err(|e| IntegrationError::ParseError(e.to_string()))?;
                        Ok(self.map_bitbucket_pr(&pr, repo))
                    }
                    GitProviderType::GitHub => {
                        let pr: GitHubPr = response.json().await
                            .map_err(|e| IntegrationError::ParseError(e.to_string()))?;
                        Ok(self.map_github_pr(&pr, repo))
                    }
                    GitProviderType::GitLab => {
                        Err(IntegrationError::ApiError("GitLab not fully implemented".to_string()))
                    }
                }
            }
            401 => Err(IntegrationError::Auth("Invalid credentials".to_string())),
            404 => Err(IntegrationError::NotFound(format!("PR {} not found", id))),
            429 => Err(IntegrationError::RateLimit),
            status => Err(IntegrationError::ApiError(format!("Status: {}", status))),
        }
    }

    async fn find_by_reviewer(&self, user_id: &str, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
        let all_prs = self.get_open_prs(filter).await?;
        
        Ok(all_prs
            .into_iter()
            .filter(|pr| pr.reviewers.iter().any(|r| r.user.id == user_id || r.user.name == user_id))
            .collect())
    }

    async fn get_open_prs(&self, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError> {
        match self.config.provider {
            GitProviderType::Bitbucket => self.fetch_bitbucket_prs(filter).await,
            GitProviderType::GitHub => self.fetch_github_prs(filter).await,
            GitProviderType::GitLab => {
                Err(IntegrationError::ApiError("GitLab not fully implemented".to_string()))
            }
        }
    }
}

// ===== Bitbucket API Types =====

#[derive(Debug, Deserialize)]
struct BitbucketPrList {
    values: Vec<BitbucketPr>,
}

#[derive(Debug, Deserialize)]
struct BitbucketPr {
    id: i64,
    title: String,
    description: Option<String>,
    author: BitbucketUser,
    #[serde(default)]
    reviewers: Vec<BitbucketUser>,
    source: BitbucketRef,
    destination: BitbucketRef,
    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: chrono::DateTime<chrono::Utc>,
    links: BitbucketLinks,
}

#[derive(Debug, Deserialize)]
struct BitbucketUser {
    uuid: String,
    display_name: String,
    links: BitbucketUserLinks,
}

#[derive(Debug, Deserialize)]
struct BitbucketUserLinks {
    avatar: Option<BitbucketLink>,
}

#[derive(Debug, Deserialize)]
struct BitbucketRef {
    branch: BitbucketBranch,
}

#[derive(Debug, Deserialize)]
struct BitbucketBranch {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketLinks {
    html: BitbucketLink,
}

#[derive(Debug, Deserialize)]
struct BitbucketLink {
    href: String,
}

// ===== GitHub API Types =====

#[derive(Debug, Deserialize)]
struct GitHubPr {
    number: i64,
    title: String,
    body: Option<String>,
    user: GitHubUser,
    #[serde(default)]
    requested_reviewers: Vec<GitHubUser>,
    head: GitHubRef,
    base: GitHubRef,
    #[serde(default)]
    draft: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRef {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_bitbucket() {
        let config = GitConfig::bitbucket("workspace", "user")
            .with_token("token")
            .with_repositories(vec!["repo1".to_string()]);

        assert_eq!(config.provider, GitProviderType::Bitbucket);
        assert_eq!(config.workspace, Some("workspace".to_string()));
        assert_eq!(config.repositories.len(), 1);
    }

    #[test]
    fn test_git_config_github() {
        let config = GitConfig::github("user").with_token("token");

        assert_eq!(config.provider, GitProviderType::GitHub);
        assert!(config.workspace.is_none());
    }

    #[test]
    fn test_api_base_url() {
        let config = GitConfig::bitbucket("ws", "user").with_token("t");
        assert_eq!(config.api_base_url(), "https://api.bitbucket.org/2.0");

        let config = GitConfig::github("user").with_token("t");
        assert_eq!(config.api_base_url(), "https://api.github.com");
    }

    #[test]
    fn test_provider_requires_token() {
        let config = GitConfig::github("user");
        let result = GitProvider::new(config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IntegrationError::ConfigError(_)));
    }

    #[test]
    fn test_stale_threshold() {
        let config = GitConfig::github("user").with_token("token");
        let provider = GitProvider::new(config).unwrap().with_stale_threshold(24);

        let old_date = Utc::now() - Duration::hours(25);
        assert!(provider.is_stale(&old_date));

        let recent_date = Utc::now() - Duration::hours(12);
        assert!(!provider.is_stale(&recent_date));
    }

    #[test]
    fn test_pr_filter_builder() {
        let filter = PrFilter::new()
            .with_repositories(vec!["repo1".to_string(), "repo2".to_string()])
            .stale_only();

        assert_eq!(filter.repositories.len(), 2);
        assert!(filter.stale_only);
    }
}
