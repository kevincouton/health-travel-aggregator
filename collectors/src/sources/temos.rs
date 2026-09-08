//! Temos (Trust, Effective Medicine, Optimized Services) accredited-partner
//! directory.
//!
//! Source: https://temos-accreditation.com/AccreditedPartners/List.aspx — a
//! single server-rendered ASP.NET table of all Temos-accredited and -assessed
//! healthcare organizations with country, city, care level, and program.
//! The host serves no robots.txt (404), so no crawl restrictions are stated;
//! we fetch this one page per run with a descriptive User-Agent.

use crate::{
    collector::Collector,
    country,
    model::{CollectedClinic, CollectorError},
};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

pub const SOURCE: &str = "temos";
pub const LISTING_URL: &str = "https://temos-accreditation.com/AccreditedPartners/List.aspx";

pub struct TemosCollector;

#[async_trait]
impl Collector for TemosCollector {
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

fn cell_text(row: &scraper::ElementRef, sel: &Selector) -> Option<String> {
    row.select(sel)
        .next()
        .map(|c| c.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

/// "Ümraniye, Istanbul" → "Istanbul", "Büyükçekmece / Istanbul," → "Istanbul":
/// sources sometimes prefix districts or leave trailing separators.
fn normalize_city(raw: &str) -> String {
    raw.split(',')
        .rev()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn parse(html: &str) -> Result<Vec<CollectedClinic>, CollectorError> {
    let doc = Html::parse_document(html);
    let row_sel = Selector::parse("table#AccreditedPartnersList tr.hos").unwrap();
    let name_sel = Selector::parse("td.hos-name a").unwrap();
    let country_sel = Selector::parse("td.hos-country span").unwrap();
    let city_sel = Selector::parse("td.hos-city span").unwrap();
    let type_sel = Selector::parse("td.hos-type span").unwrap();
    let facility_sel = Selector::parse("td.hos-facility").unwrap();
    let status_img_sel = Selector::parse("td.hos-status img").unwrap();

    let mut out = Vec::new();
    let mut skipped_country = 0usize;
    for row in doc.select(&row_sel) {
        let name_link = row.select(&name_sel).next();
        let Some(link) = name_link else { continue };
        let name = link.text().collect::<String>().trim().to_string();
        if name.is_empty() {
            continue;
        }
        let external_ref = link
            .value()
            .attr("href")
            .and_then(|h| h.split("id=").nth(1))
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| super::slugify(&name));

        let country_raw = cell_text(&row, &country_sel).unwrap_or_default();
        let Some((country_code, country_name)) = country::normalize(&country_raw) else {
            skipped_country += 1;
            continue;
        };

        let city = cell_text(&row, &city_sel)
            .map(|c| normalize_city(&c))
            .filter(|c| !c.is_empty());
        let Some(city) = city else { continue };

        let care_type = cell_text(&row, &type_sel).unwrap_or_default();
        let facility = cell_text(&row, &facility_sel).unwrap_or_default();
        let excellence = row
            .select(&status_img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .is_some_and(|src| src.contains("excellence"));

        let accreditation = if excellence {
            "Temos Excellence"
        } else {
            "Temos Accredited"
        };

        out.push(CollectedClinic {
            source: SOURCE,
            external_ref,
            name: name.clone(),
            country_code: country_code.into(),
            country_name: country_name.into(),
            city: city.clone(),
            accreditations: vec![accreditation.into()],
            description: Some(format!(
                "{name} is a Temos-accredited healthcare organization in {city}, {country_name} ({facility}; {care_type})."
            )),
            source_url: LISTING_URL.into(),
        });
    }

    if skipped_country > 0 {
        tracing::warn!(skipped_country, "temos rows skipped: unrecognized country");
    }
    if out.is_empty() {
        return Err(CollectorError::parse(
            SOURCE,
            "no tr.hos rows found; page structure may have changed",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/temos_accredited_partners.html");

    #[test]
    fn parses_accredited_partners_from_fixture() {
        let clinics = parse(FIXTURE).unwrap();
        assert_eq!(clinics.len(), 83);
        assert!(clinics.iter().all(|c| c.source == "temos"));
        assert!(clinics.iter().all(|c| c.country_code.len() == 2));

        let first = clinics
            .iter()
            .find(|c| c.name.contains("Agios Charalambos"))
            .expect("first row present");
        assert_eq!(first.country_code, "GR");
        assert_eq!(first.city, "Iraklion");
        assert_eq!(first.external_ref, "16725");
        assert_eq!(first.accreditations, vec!["Temos Excellence"]);

        let tbilisi = clinics
            .iter()
            .find(|c| c.name.contains("Aleksandre Aladashvili"))
            .expect("georgian clinic present");
        assert_eq!(tbilisi.country_code, "GE");
        assert_eq!(tbilisi.accreditations, vec!["Temos Accredited"]);

        let turkish = clinics.iter().filter(|c| c.country_code == "TR").count();
        assert!(turkish > 0, "Türkiye rows parsed");

        let mut refs: Vec<&str> = clinics.iter().map(|c| c.external_ref.as_str()).collect();
        refs.sort_unstable();
        refs.dedup();
        assert_eq!(refs.len(), clinics.len());
    }

    #[test]
    fn normalizes_district_prefixed_cities() {
        assert_eq!(normalize_city("Ümraniye, Istanbul"), "Istanbul");
        assert_eq!(normalize_city("Büyükçekmece / Istanbul,"), "Istanbul");
        assert_eq!(normalize_city("Tbilisi"), "Tbilisi");
    }
}
