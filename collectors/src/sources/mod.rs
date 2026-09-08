pub mod gha;
pub mod mhtc;
pub mod temos;

/// Lowercase ASCII slug: alphanumerics kept, everything else collapses to '-'.
pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut dash = false;
    for c in s.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_names() {
        assert_eq!(
            slugify("Bumrungrad International Hospital"),
            "bumrungrad-international-hospital"
        );
        assert_eq!(
            slugify("Clínica Santa Clarita Sc"),
            "cl-nica-santa-clarita-sc"
        );
        assert_eq!(slugify("  Alpha IVF & Women's  "), "alpha-ivf-women-s");
    }
}
