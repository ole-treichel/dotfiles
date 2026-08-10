/// Lowercase, every run of non-alphanumerics collapsed to a single `-`, trimmed.
pub fn slug(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pending_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn collapses_and_trims() {
        assert_eq!(slug("Feat: Cookie Banner!!"), "feat-cookie-banner");
        assert_eq!(slug("feat/master-product-data-table"), "feat-master-product-data-table");
        assert_eq!(slug("  --Foo__Bar--  "), "foo-bar");
        assert_eq!(slug("feat-website-in-sign-up-mail"), "feat-website-in-sign-up-mail");
        assert_eq!(slug("!!!"), "");
        assert_eq!(slug("v2.1"), "v2-1");
    }
}
