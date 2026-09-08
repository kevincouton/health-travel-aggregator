//! Malaysia Healthcare Travel Council (MHTC) member-hospital directory.
//!
//! Source: https://www.malaysiahealthcare.org/find-hospital — a single
//! server-rendered Webflow CMS page listing all MHTC member hospitals with
//! their Malaysian state, membership tier, and website. robots.txt permits
//! crawling (only paginated news/press URLs are disallowed).

use crate::{
    collector::Collector,
    country,
    model::{CollectedClinic, CollectorError},
};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

pub const SOURCE: &str = "mhtc";
pub const LISTING_URL: &str = "https://www.malaysiahealthcare.org/find-hospital";

pub struct MhtcCollector;

#[async_trait]
impl Collector for MhtcCollector {
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

/// Map MHTC's state-level `data-location` slugs to (display state, city).
/// The directory only exposes state granularity; the state capital is used
/// as the city and the state is recorded in the description. Records land
/// in pending moderation regardless.
fn location_to_city(slug: &str) -> (&'static str, &'static str) {
    match slug {
        "wilayah-persekutuan-0d3rf" => ("Kuala Lumpur (Federal Territory)", "Kuala Lumpur"),
        "selangor" => ("Selangor", "Shah Alam"),
        "penang" => ("Penang", "George Town"),
        "johor" => ("Johor", "Johor Bahru"),
        "melaka" => ("Melaka", "Malacca City"),
        "negeri-sembilan" => ("Negeri Sembilan", "Seremban"),
        "perak" => ("Perak", "Ipoh"),
        "sabah" => ("Sabah", "Kota Kinabalu"),
        "sarawak" => ("Sarawak", "Kuching"),
        _ => ("Malaysia", "Kuala Lumpur"),
    }
}

pub fn parse(html: &str) -> Result<Vec<CollectedClinic>, CollectorError> {
    let doc = Html::parse_document(html);
    let item_sel = Selector::parse("div.hospital-item").unwrap();
    let name_sel = Selector::parse("h5.hospital-tile").unwrap();
    let link_sel = Selector::parse("a.hostpital-link").unwrap();

    let (country_code, country_name) =
        country::normalize("Malaysia").expect("Malaysia is a known country");

    let mut out = Vec::new();
    for item in doc.select(&item_sel) {
        let name = item
            .select(&name_sel)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .filter(|n| !n.is_empty());
        let Some(name) = name else { continue };

        let website = item
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|h| h.trim().to_string())
            .filter(|h| h.starts_with("http"));

        let location = item.value().attr("data-location").unwrap_or_default();
        let (state, city) = location_to_city(location);

        let accreditation = match item.value().attr("data-partner") {
            Some("elite-membership") => "MHTC Elite Partner",
            _ => "MHTC Member",
        };

        // The website URL plus state slug is the most stable per-clinic
        // identifier the source provides (one chain reuses a website across
        // states); fall back to a name/city slug.
        let external_ref = match (&website, location) {
            (Some(w), loc) if !loc.is_empty() => format!("{w}#{loc}"),
            (Some(w), _) => w.clone(),
            (None, _) => format!("{}/{}", super::slugify(&name), super::slugify(city)),
        };

        out.push(CollectedClinic {
            source: SOURCE,
            external_ref,
            name: name.clone(),
            country_code: country_code.into(),
            country_name: country_name.into(),
            city: city.into(),
            accreditations: vec![accreditation.into()],
            description: Some(format!(
                "{name} is an MHTC member healthcare facility in {state}, Malaysia, serving international medical travelers."
            )),
            source_url: LISTING_URL.into(),
        });
    }

    if out.is_empty() {
        return Err(CollectorError::parse(
            SOURCE,
            "no hospital-item entries found; page structure may have changed",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/mhtc_find_hospital.html");

    #[test]
    fn parses_member_hospitals_from_fixture() {
        let clinics = parse(FIXTURE).unwrap();
        assert_eq!(clinics.len(), 91);
        assert!(clinics.iter().all(|c| c.country_code == "MY"));
        assert!(clinics.iter().all(|c| c.source == "mhtc"));
        assert!(clinics
            .iter()
            .all(|c| !c.name.is_empty() && !c.city.is_empty()));

        let alpha = clinics
            .iter()
            .find(|c| c.name.contains("Alpha IVF"))
            .expect("Alpha IVF present");
        assert_eq!(alpha.city, "Shah Alam");
        assert_eq!(alpha.accreditations, vec!["MHTC Elite Partner"]);
        assert_eq!(
            alpha.external_ref,
            "https://www.alphafertilitycentre.com/#selangor"
        );

        let kl = clinics
            .iter()
            .find(|c| c.name.contains("ALPS Medical"))
            .expect("ALPS present");
        assert_eq!(kl.city, "Kuala Lumpur");
        assert_eq!(kl.accreditations, vec!["MHTC Member"]);

        // External refs must be unique for idempotent upserts.
        let mut refs: Vec<&str> = clinics.iter().map(|c| c.external_ref.as_str()).collect();
        refs.sort_unstable();
        refs.dedup();
        assert_eq!(refs.len(), clinics.len());
    }
}
