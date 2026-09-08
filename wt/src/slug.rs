/// Lowercase, umlauts spelled out, every run of non-alphanumerics collapsed to
/// a single `-`, trimmed. The output charset is exactly `[a-z0-9-]` with no
/// leading or trailing dash, which is also the branch-name charset.
pub fn slug(input: &str) -> String {
    let expanded = transliterate(input);
    let mut out = String::with_capacity(expanded.len());
    let mut pending_dash = false;
    for ch in expanded.chars() {
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

/// Runs before the slug so German text survives it: every non-ASCII char is a
/// separator to `slug`, so without this `Jubiläum` would come out `jubil-um`.
/// The MOCO project names are German, and so is half of what gets typed.
fn transliterate(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            'ä' | 'Ä' => out.push_str("ae"),
            'ö' | 'Ö' => out.push_str("oe"),
            'ü' | 'Ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c => out.push(c),
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

    #[test]
    fn spells_out_umlauts() {
        assert_eq!(slug("10-jähriges Jubiläum"), "10-jaehriges-jubilaeum");
        assert_eq!(slug("Größe"), "groesse");
        assert_eq!(slug("Änderung Öffnungszeiten"), "aenderung-oeffnungszeiten");
        assert_eq!(slug("Assets für Ceramic"), "assets-fuer-ceramic");
        // Only the German set is spelled out. Everything else stays a
        // separator, which is what the en dash in the MOCO names wants anyway.
        assert_eq!(slug("XTREME Heroes – EMS 2026"), "xtreme-heroes-ems-2026");
        assert_eq!(slug("Café"), "caf");
    }

    #[test]
    fn project_identifiers_pass_through() {
        assert_eq!(slug("P26059"), "p26059");
        assert_eq!(slug("P1903-003"), "p1903-003");
    }
}
