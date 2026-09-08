use crate::model::{CollectedClinic, CollectorError};
use async_trait::async_trait;
use reqwest::Client;

/// Descriptive User-Agent identifying the collector and its operator.
pub const USER_AGENT: &str = concat!(
    "health-travel-aggregator-collectors/",
    env!("CARGO_PKG_VERSION"),
    " (+https://health-travel.lucanian.app; contact: admin@lucanian.app)"
);

/// A clinic-data source. Implementations fetch exactly one listing page per
/// `collect` call (no crawling) and normalize it into `CollectedClinic`s.
/// Parsing is kept in a separate pure `parse(html)` function per source so
/// unit tests run against committed fixtures without network access.
#[async_trait]
pub trait Collector: Send + Sync {
    /// Stable source identifier stored as clinics.source.
    fn source(&self) -> &'static str;

    /// Fetch and parse the source's listing page.
    async fn collect(&self, client: &Client) -> Result<Vec<CollectedClinic>, CollectorError>;
}

/// Shared HTTP client: descriptive UA, generous timeout, no redirect loops.
pub fn http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
}
