//! Global Healthcare Accreditation (GHA) directory of accredited and
//! certified healthcare organizations.
//!
//! Source: https://www.globalhealthcareaccreditation.com/accredited-and-certified-organizations
//! — a single server-rendered Webflow directory page. robots.txt only
//! excludes ia_archiver, so general crawling is permitted. Facilitator and
//! corporate-entity cards are filtered out; only healthcare delivery
//! organizations (hospitals, ambulatory centers) are collected. Descriptions
//! are generated, not copied, so no source prose is republished.

use crate::{
    collector::Collector,
    country,
    model::{CollectedClinic, CollectorError},
};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub const SOURCE: &str = "gha";
pub const LISTING_URL: &str =
    "https://www.globalhealthcareaccreditation.com/accredited-and-certified-organizations";

pub struct GhaCollector;

#[async_trait]
impl Collector for GhaCollector {
    fn source(&self) -> &'static str {
        SOURCE
    }

    async fn collect(&self, client: &Client) -> Result<Vec<CollectedClinic>, CollectorError> {
        let html = client
            .get(LISTING_URL)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        parse(&html)
    }
}

/// Only healthcare delivery organizations belong in the clinics table;
/// facilitators, corporate entities, and hotels are excluded.
fn is_provider_label(label: &str) -> bool {
    let l = label.to_lowercase();
    (l.contains("hospital")
        || l.contains("ambulatory center")
        || l.contains("healthcare organizations"))
        && !l.contains("facilitator")
        && !l.contains("corporate")
        && !l.contains("hotel")
}

pub fn parse(html: &str) -> Result<Vec<CollectedClinic>, CollectorError> {
    let doc = Html::parse_document(html);
    let card_sel = Selector::parse("div.directory-card").unwrap();
    let name_sel = Selector::parse("h2.gha-new-h2").unwrap();
    let label_sel = Selector::parse("div.gha-new-paragraph.bold").unwrap();
    let addr_sel =
        Selector::parse("div.address-container:not(.language):not(.location) div.paragraph-8")
            .unwrap();

    // Dedup by (name, city): the same organization can appear under several
    // program tabs; merge their accreditation labels into one record.
    let mut by_key: HashMap<(String, String), CollectedClinic> = HashMap::new();
    let mut skipped_country = 0usize;

    for card in doc.select(&card_sel) {
        let Some(label) = card
            .select(&label_sel)
            .next()
            .map(|l| l.text().collect::<String>().trim().to_string())
        else {
            continue;
        };
        if !is_provider_label(&label) {
            continue;
        }
        let Some(name) = card
            .select(&name_sel)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .filter(|n| !n.is_empty())
        else {
            continue;
        };

        // Address renders as "Country | City" paragraphs; the "|" separator
        // paragraph carries no text of interest.
        let parts: Vec<String> = card
            .select(&addr_sel)
            .map(|p| p.text().collect::<String>().trim().to_string())
            .filter(|p| !p.is_empty() && p != "|")
            .collect();
        let (mut country_raw, mut city) = match parts.as_slice() {
            [c, ci, ..] => (c.clone(), ci.clone()),
            _ => continue,
        };
        // One known record has country/city swapped ("Kyiv" / "Ukraine").
        if country::normalize(&country_raw).is_none() && country::normalize(&city).is_some() {
            std::mem::swap(&mut country_raw, &mut city);
        }
        let Some((country_code, country_name)) = country::normalize(&country_raw) else {
            skipped_country += 1;
            continue;
        };

        let key = (name.to_lowercase(), city.to_lowercase());
        by_key
            .entry(key)
            .and_modify(|c| {
                let label = format!("GHA {label}");
                if !c.accreditations.iter().any(|a| a == &label) {
                    c.accreditations.push(label);
                }
            })
            .or_insert_with(|| CollectedClinic {
                source: SOURCE,
                external_ref: format!("{}/{}", super::slugify(&name), super::slugify(&city)),
                name: name.clone(),
                country_code: country_code.into(),
                country_name: country_name.into(),
                city: city.clone(),
                accreditations: vec![format!("GHA {label}")],
                description: Some(format!(
                    "{name} is a GHA-accredited healthcare organization in {city}, {country_name}."
                )),
                source_url: LISTING_URL.into(),
            });
    }

    if skipped_country > 0 {
        tracing::warn!(skipped_country, "gha cards skipped: unrecognized country");
    }
    let mut out: Vec<CollectedClinic> = by_key.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if out.is_empty() {
        return Err(CollectorError::parse(
            SOURCE,
            "no directory-card entries found; page structure may have changed",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/gha_accredited_organizations.html");

    #[test]
    fn parses_provider_cards_from_fixture() {
        let clinics = parse(FIXTURE).unwrap();
        assert!(clinics.len() >= 25, "got {}", clinics.len());
        assert!(clinics.iter().all(|c| c.source == "gha"));
        assert!(clinics.iter().all(|c| c.country_code.len() == 2));
        assert!(clinics
            .iter()
            .all(|c| c.accreditations.iter().all(|a| a.starts_with("GHA "))));

        let bumrungrad = clinics
            .iter()
            .find(|c| c.name.contains("Bumrungrad"))
            .expect("Bumrungrad present");
        assert_eq!(bumrungrad.country_code, "TH");
        assert_eq!(bumrungrad.city, "Bangkok");
        // Appears under multiple program tabs; record deduped to one.
        assert_eq!(
            clinics
                .iter()
                .filter(|c| c.name.contains("Bumrungrad"))
                .count(),
            1
        );
        assert!(bumrungrad
            .accreditations
            .iter()
            .any(|a| a.contains("Excellence")));

        // Facilitators and corporate entities are not healthcare providers
        // (Bookimed, the swapped Kyiv/Ukraine record, is a facilitator).
        assert!(!clinics
            .iter()
            .any(|c| c.accreditations.iter().any(|a| a.contains("Facilitator"))));
        assert!(!clinics.iter().any(|c| c.name.contains("Bookimed")));

        let mut refs: Vec<&str> = clinics.iter().map(|c| c.external_ref.as_str()).collect();
        refs.sort_unstable();
        refs.dedup();
        assert_eq!(refs.len(), clinics.len());
    }

    #[test]
    fn filters_non_provider_labels() {
        assert!(is_provider_label(
            "Hospital Accreditation for Medical Travel"
        ));
        assert!(is_provider_label(
            "Ambulatory Center Accreditation with Excellence"
        ));
        assert!(is_provider_label(
            "GHA's Accreditation for Healthcare Organizations"
        ));
        assert!(!is_provider_label(
            "Medical Travel Facilitator Certification"
        ));
        assert!(!is_provider_label(
            "Corporate Entity: Certification for Excellence in Medical Travel Patient Experience"
        ));
    }

    #[test]
    fn repairs_swapped_country_city() {
        let html = r#"<div class="directory-card">
            <div class="gha-new-paragraph bold">Hospital Accreditation for Medical Travel</div>
            <h2 class="gha-new-h2">Test Hospital</h2>
            <div class="address-container">
              <div class="paragraph-8 padding-top">Kyiv</div>
              <div class="paragraph-8 padding-top margin">|</div>
              <div class="paragraph-8 padding-top">Ukraine</div>
            </div></div>"#;
        let clinics = parse(html).unwrap();
        assert_eq!(clinics.len(), 1);
        assert_eq!(clinics[0].country_code, "UA");
        assert_eq!(clinics[0].city, "Kyiv");
    }
}
