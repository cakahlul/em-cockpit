//! Unified Search Service
//!
//! Provides intelligent search across multiple data sources
//! (Jira tickets, PRs, incidents) with ranking and caching.

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::integrations::traits::{
    IntegrationError, PrFilter, PullRequest, Ticket, TicketRepository, TicketSearchQuery,
};
use crate::services::CacheService;

/// Type of search result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchResultType {
    Ticket,
    PullRequest,
    Incident,
    Document,
}

impl SearchResultType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchResultType::Ticket => "Ticket",
            SearchResultType::PullRequest => "PR",
            SearchResultType::Incident => "Incident",
            SearchResultType::Document => "Document",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SearchResultType::Ticket => "🎫",
            SearchResultType::PullRequest => "🔀",
            SearchResultType::Incident => "🚨",
            SearchResultType::Document => "📄",
        }
    }
}

/// A unified search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub result_type: SearchResultType,
    pub title: String,
    pub subtitle: Option<String>,
    pub url: Option<String>,
    pub relevance_score: f32,
    pub updated_at: DateTime<Utc>,
    pub metadata: SearchResultMetadata,
}

/// Additional metadata for search results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResultMetadata {
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    pub is_stale: Option<bool>,
}

impl SearchResult {
    pub fn from_ticket(ticket: &Ticket) -> Self {
        Self {
            id: ticket.key.clone(),
            result_type: SearchResultType::Ticket,
            title: ticket.summary.clone(),
            subtitle: Some(format!("{} • {}", ticket.key, ticket.status.name)),
            url: None, // Will be constructed from config
            relevance_score: 1.0,
            updated_at: ticket.updated_at,
            metadata: SearchResultMetadata {
                status: Some(ticket.status.name.clone()),
                assignee: ticket.assignee.as_ref().map(|a| a.name.clone()),
                priority: ticket.priority.as_ref().map(|p| p.as_str().to_string()),
                is_stale: None,
            },
        }
    }

    pub fn from_pr(pr: &PullRequest) -> Self {
        Self {
            id: pr.id.clone(),
            result_type: SearchResultType::PullRequest,
            title: pr.title.clone(),
            subtitle: Some(format!("{} → {} • {}", pr.source_branch, pr.target_branch, pr.state.as_str())),
            url: Some(pr.url.clone()),
            relevance_score: 1.0,
            updated_at: pr.updated_at,
            metadata: SearchResultMetadata {
                status: Some(pr.state.as_str().to_string()),
                assignee: None,
                priority: None,
                is_stale: Some(pr.is_stale),
            },
        }
    }

    /// Boost score for recent updates
    pub fn boost_for_recency(&mut self) {
        let age = Utc::now().signed_duration_since(self.updated_at);
        
        if age < Duration::hours(1) {
            self.relevance_score *= 1.5;
        } else if age < Duration::hours(24) {
            self.relevance_score *= 1.2;
        } else if age < Duration::days(7) {
            self.relevance_score *= 1.0;
        } else {
            self.relevance_score *= 0.8;
        }
    }

    /// Boost score for exact ID match
    pub fn boost_for_id_match(&mut self, query: &str) {
        if self.id.to_lowercase() == query.to_lowercase() {
            self.relevance_score *= 2.0;
        } else if self.id.to_lowercase().contains(&query.to_lowercase()) {
            self.relevance_score *= 1.5;
        }
    }
}

/// Search query options
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: String,
    pub types: Vec<SearchResultType>,
    pub limit: usize,
    pub include_closed: bool,
}

impl SearchQuery {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            types: vec![SearchResultType::Ticket, SearchResultType::PullRequest],
            limit: 10,
            include_closed: false,
        }
    }

    pub fn with_types(mut self, types: Vec<SearchResultType>) -> Self {
        self.types = types;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn include_closed(mut self) -> Self {
        self.include_closed = true;
        self
    }

    /// Check if query looks like a ticket ID (e.g., PROJ-123)
    pub fn is_ticket_id(&self) -> bool {
        let pattern = regex::Regex::new(r"^[A-Z]+-\d+$").unwrap();
        pattern.is_match(&self.text.to_uppercase())
    }

    /// Check if query looks like a PR number
    pub fn is_pr_number(&self) -> bool {
        self.text.starts_with('#') && self.text[1..].parse::<u32>().is_ok()
    }
}

/// Unified search service
pub struct SearchService<T: TicketRepository> {
    ticket_repo: Arc<T>,
    cache: Option<Arc<CacheService>>,
    cache_ttl: Duration,
}

impl<T: TicketRepository> SearchService<T> {
    pub fn new(ticket_repo: Arc<T>) -> Self {
        Self {
            ticket_repo,
            cache: None,
            cache_ttl: Duration::minutes(5),
        }
    }

    pub fn with_cache(mut self, cache: Arc<CacheService>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Perform a unified search across all sources
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, IntegrationError> {
        // Check cache first
        let cache_key = format!("search:{}", query.text);
        if let Some(ref cache) = self.cache {
            if let Ok(cached) = cache.get::<Vec<SearchResult>>(&cache_key) {
                log::debug!("Search cache hit for: {}", query.text);
                return Ok(cached);
            }
        }

        let mut results = Vec::new();

        // Search tickets if requested
        if query.types.contains(&SearchResultType::Ticket) {
            let ticket_results = self.search_tickets(query).await?;
            results.extend(ticket_results);
        }

        // Apply ranking
        for result in &mut results {
            result.boost_for_recency();
            result.boost_for_id_match(&query.text);
        }

        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Apply limit
        results.truncate(query.limit);

        // Cache results
        if let Some(ref cache) = self.cache {
            let _ = cache.set(&cache_key, &results, self.cache_ttl);
        }

        Ok(results)
    }

    async fn search_tickets(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, IntegrationError> {
        // If it looks like a ticket ID, try direct lookup first
        if query.is_ticket_id() {
            match self.ticket_repo.find_by_id(&query.text.to_uppercase()).await {
                Ok(ticket) => {
                    let mut result = SearchResult::from_ticket(&ticket);
                    result.relevance_score = 2.0; // Boost exact match
                    return Ok(vec![result]);
                }
                Err(IntegrationError::NotFound(_)) => {
                    // Fall through to regular search
                }
                Err(e) => return Err(e),
            }
        }

        // Regular text search
        let search_query = TicketSearchQuery::new()
            .with_text(&query.text)
            .with_limit(query.limit);

        let tickets = self.ticket_repo.search(&search_query).await?;
        
        Ok(tickets.iter().map(SearchResult::from_ticket).collect())
    }

    /// Get recent search suggestions
    pub fn get_suggestions(&self, _prefix: &str) -> Vec<String> {
        // In a real implementation, this would track recent searches
        Vec::new()
    }
}

// Allow Debug for SearchService
impl<T: TicketRepository> std::fmt::Debug for SearchService<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchService")
            .field("cache_ttl", &self.cache_ttl)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::traits::{StatusCategory, TicketStatus, User};
    use std::sync::Mutex;

    // Mock ticket repository for testing
    struct MockTicketRepo {
        tickets: Mutex<Vec<Ticket>>,
    }

    impl MockTicketRepo {
        fn new(tickets: Vec<Ticket>) -> Self {
            Self {
                tickets: Mutex::new(tickets),
            }
        }
    }

    #[async_trait]
    impl TicketRepository for MockTicketRepo {
        async fn find_by_id(&self, id: &str) -> Result<Ticket, IntegrationError> {
            let tickets = self.tickets.lock().unwrap();
            tickets
                .iter()
                .find(|t| t.key == id)
                .cloned()
                .ok_or_else(|| IntegrationError::NotFound(format!("Ticket {} not found", id)))
        }

        async fn search(&self, query: &TicketSearchQuery) -> Result<Vec<Ticket>, IntegrationError> {
            let tickets = self.tickets.lock().unwrap();
            let text = query.text.as_deref().unwrap_or("").to_lowercase();
            
            let results: Vec<Ticket> = tickets
                .iter()
                .filter(|t| {
                    t.summary.to_lowercase().contains(&text)
                        || t.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&text))
                })
                .take(query.limit)
                .cloned()
                .collect();
            
            Ok(results)
        }
    }

    fn create_test_ticket(key: &str, summary: &str) -> Ticket {
        Ticket {
            id: key.to_string(),
            key: key.to_string(),
            summary: summary.to_string(),
            description: None,
            status: TicketStatus {
                name: "Open".to_string(),
                category: StatusCategory::Todo,
            },
            assignee: None,
            reporter: None,
            priority: None,
            sprint: None,
            labels: vec![],
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new("test query");
        
        assert_eq!(query.text, "test query");
        assert_eq!(query.limit, 10);
        assert!(!query.include_closed);
    }

    #[test]
    fn test_search_query_is_ticket_id() {
        assert!(SearchQuery::new("PROJ-123").is_ticket_id());
        assert!(SearchQuery::new("ABC-1").is_ticket_id());
        assert!(SearchQuery::new("proj-123").is_ticket_id()); // to_uppercase handles lowercase
        assert!(!SearchQuery::new("just text").is_ticket_id());
        assert!(!SearchQuery::new("123").is_ticket_id());
        assert!(!SearchQuery::new("PROJ").is_ticket_id());
        assert!(!SearchQuery::new("-123").is_ticket_id());
    }

    #[test]
    fn test_search_query_is_pr_number() {
        assert!(SearchQuery::new("#123").is_pr_number());
        assert!(SearchQuery::new("#1").is_pr_number());
        assert!(!SearchQuery::new("123").is_pr_number());
        assert!(!SearchQuery::new("#abc").is_pr_number());
    }

    #[test]
    fn test_search_result_from_ticket() {
        let ticket = create_test_ticket("TEST-1", "Fix critical bug");
        let result = SearchResult::from_ticket(&ticket);

        assert_eq!(result.id, "TEST-1");
        assert_eq!(result.result_type, SearchResultType::Ticket);
        assert_eq!(result.title, "Fix critical bug");
        assert!(result.subtitle.unwrap().contains("TEST-1"));
    }

    #[test]
    fn test_search_result_boost_for_id_match() {
        let mut result = SearchResult::from_ticket(&create_test_ticket("TEST-123", "Test"));
        result.relevance_score = 1.0;

        result.boost_for_id_match("TEST-123");
        assert_eq!(result.relevance_score, 2.0); // Exact match

        let mut result2 = SearchResult::from_ticket(&create_test_ticket("TEST-123", "Test"));
        result2.relevance_score = 1.0;
        result2.boost_for_id_match("TEST");
        assert_eq!(result2.relevance_score, 1.5); // Partial match
    }

    #[test]
    fn test_search_result_boost_for_recency() {
        let mut recent = create_test_ticket("T-1", "Recent");
        recent.updated_at = Utc::now();
        let mut result = SearchResult::from_ticket(&recent);
        result.relevance_score = 1.0;
        result.boost_for_recency();
        assert!(result.relevance_score > 1.0);

        let mut old = create_test_ticket("T-2", "Old");
        old.updated_at = Utc::now() - Duration::days(30);
        let mut result2 = SearchResult::from_ticket(&old);
        result2.relevance_score = 1.0;
        result2.boost_for_recency();
        assert!(result2.relevance_score < 1.0);
    }

    #[test]
    fn test_search_result_type_display() {
        assert_eq!(SearchResultType::Ticket.as_str(), "Ticket");
        assert_eq!(SearchResultType::PullRequest.icon(), "🔀");
        assert_eq!(SearchResultType::Incident.icon(), "🚨");
    }

    #[tokio::test]
    async fn test_search_service_find_by_id() {
        let tickets = vec![
            create_test_ticket("PROJ-123", "First ticket"),
            create_test_ticket("PROJ-456", "Second ticket"),
        ];
        let repo = Arc::new(MockTicketRepo::new(tickets));
        let service = SearchService::new(repo);

        let query = SearchQuery::new("PROJ-123");
        let results = service.search(&query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "PROJ-123");
        assert!(results[0].relevance_score > 1.0); // ID match boost
    }

    #[tokio::test]
    async fn test_search_service_text_search() {
        let tickets = vec![
            create_test_ticket("T-1", "Fix login bug"),
            create_test_ticket("T-2", "Add feature"),
            create_test_ticket("T-3", "Another bug fix"),
        ];
        let repo = Arc::new(MockTicketRepo::new(tickets));
        let service = SearchService::new(repo);

        let query = SearchQuery::new("bug");
        let results = service.search(&query).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.title.to_lowercase().contains("bug")));
    }

    #[tokio::test]
    async fn test_search_service_results_sorted_by_relevance() {
        let mut old_ticket = create_test_ticket("T-1", "Old bug");
        old_ticket.updated_at = Utc::now() - Duration::days(30);
        
        let mut recent_ticket = create_test_ticket("T-2", "Recent bug");
        recent_ticket.updated_at = Utc::now();

        let tickets = vec![old_ticket, recent_ticket];
        let repo = Arc::new(MockTicketRepo::new(tickets));
        let service = SearchService::new(repo);

        let query = SearchQuery::new("bug");
        let results = service.search(&query).await.unwrap();

        // Recent should be first due to recency boost
        assert_eq!(results[0].id, "T-2");
        assert!(results[0].relevance_score > results[1].relevance_score);
    }

    #[tokio::test]
    async fn test_search_service_respects_limit() {
        let tickets: Vec<Ticket> = (1..=20)
            .map(|i| create_test_ticket(&format!("T-{}", i), &format!("Bug {}", i)))
            .collect();
        let repo = Arc::new(MockTicketRepo::new(tickets));
        let service = SearchService::new(repo);

        let query = SearchQuery::new("Bug").with_limit(5);
        let results = service.search(&query).await.unwrap();

        assert_eq!(results.len(), 5);
    }
}
