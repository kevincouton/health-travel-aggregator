//! Country-name → ISO 3166-1 alpha-2 normalization for source records.

/// Normalize a free-text country name to (alpha-2 code, canonical name).
/// Returns None for unrecognized names; callers skip those records and log.
pub fn normalize(raw: &str) -> Option<(&'static str, &'static str)> {
    // Sources use non-breaking spaces and assorted official names.
    let cleaned: String = raw
        .replace(['\u{202f}', '\u{00a0}'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    match cleaned.as_str() {
        "bulgaria" => Some(("BG", "Bulgaria")),
        "colombia" => Some(("CO", "Colombia")),
        "croatia" => Some(("HR", "Croatia")),
        "dominican republic" => Some(("DO", "Dominican Republic")),
        "egypt" => Some(("EG", "Egypt")),
        "georgia" => Some(("GE", "Georgia")),
        "germany" => Some(("DE", "Germany")),
        "greece" => Some(("GR", "Greece")),
        "iraq" => Some(("IQ", "Iraq")),
        "jamaica" => Some(("JM", "Jamaica")),
        "malaysia" => Some(("MY", "Malaysia")),
        "mexico" => Some(("MX", "Mexico")),
        "northern cyprus" => Some(("CY", "Cyprus")),
        "panama" => Some(("PA", "Panama")),
        "philippines" => Some(("PH", "Philippines")),
        "republic of korea" | "south korea" | "korea" => Some(("KR", "South Korea")),
        "republic of north macedonia" | "north macedonia" => Some(("MK", "North Macedonia")),
        "kingdom of saudi arabia" | "saudi arabia" => Some(("SA", "Saudi Arabia")),
        "seychelles" => Some(("SC", "Seychelles")),
        "switzerland" => Some(("CH", "Switzerland")),
        "thailand" => Some(("TH", "Thailand")),
        "trinidad and tobago" => Some(("TT", "Trinidad and Tobago")),
        "turkey" | "türkiye" => Some(("TR", "Turkey")),
        "ukraine" => Some(("UA", "Ukraine")),
        "united arab emirates" | "uae" => Some(("AE", "United Arab Emirates")),
        "usa" | "united states" | "united states of america" | "us" => {
            Some(("US", "United States"))
        }
        "uzbekistan" => Some(("UZ", "Uzbekistan")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_variants() {
        assert_eq!(normalize("Türkiye"), Some(("TR", "Turkey")));
        assert_eq!(normalize("Turkey"), Some(("TR", "Turkey")));
        assert_eq!(
            normalize("Kingdom of Saudi\u{202f}Arabia"),
            Some(("SA", "Saudi Arabia"))
        );
        assert_eq!(normalize("USA"), Some(("US", "United States")));
        assert_eq!(
            normalize("Republic of North Macedonia"),
            Some(("MK", "North Macedonia"))
        );
        assert_eq!(normalize("Atlantis"), None);
    }
}
