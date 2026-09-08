use thiserror::Error;

/// A clinic listing normalized from an external source, mapping onto the
/// platform's clinics/locations schema with per-record source attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedClinic {
    /// Collector identifier, e.g. "mhtc". Stored in clinics.source.
    pub source: &'static str,
    /// Stable per-source identifier (detail-page id, website URL, or a
    /// name/city slug). Stored in clinics.external_ref for idempotent re-runs.
    pub external_ref: String,
    pub name: String,
    /// ISO 3166-1 alpha-2, matches clinics.country_code.
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    /// Accreditation/membership labels, matches clinics.accreditations.
    pub accreditations: Vec<String>,
    pub description: Option<String>,
    /// Public URL of the listing the record was parsed from.
    pub source_url: String,
}

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("fetch failed: {0}")]
    Fetch(#[from] reqwest::Error),
    #[error("parse failed for {origin}: {reason}")]
    Parse {
        origin: &'static str,
        reason: String,
    },
}

impl CollectorError {
    pub fn parse(origin: &'static str, reason: impl Into<String>) -> Self {
        CollectorError::Parse {
            origin,
            reason: reason.into(),
        }
    }
}
