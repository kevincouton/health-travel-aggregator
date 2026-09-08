//! Clinic-data collectors for the health-travel aggregator.
//!
//! Each collector fetches a single publicly accessible listing page from one
//! source, parses it into normalized [`model::CollectedClinic`] records
//! (mapping onto the platform's clinics/locations schema, with per-record
//! source attribution), and the `ingest` binary upserts them into Postgres.
//!
//! Politeness policy: one HTTP fetch per collector per run, descriptive
//! User-Agent, robots.txt reviewed per source (see docs/data-sources.md).

pub mod collector;
pub mod country;
pub mod ingest;
pub mod model;
pub mod sources;
