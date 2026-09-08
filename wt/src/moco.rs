use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde_json::Value;

/// Hardcoded rather than configurable: one account, and the subdomain is not a
/// secret. `config.toml` is committed to the dotfiles repo, so it is the wrong
/// place for anything MOCO anyway — see `token`.
const ENDPOINT: &str = "https://pixelwerte.mocoapp.com/api/v1/projects";
/// The documented maximum. 30 active projects today, so this is one request.
const PER_PAGE: usize = 1000;
/// Only reached if MOCO keeps handing back full pages. Stops a runaway loop.
const MAX_PAGES: usize = 20;

pub struct Project {
    pub identifier: String,
    pub name: String,
    pub customer: String,
}

/// The active projects, newest number first. No cache: a project created ten
/// minutes ago has to show up.
pub fn projects() -> Result<Vec<Project>> {
    let token = token()?;
    let mut all: Vec<Project> = Vec::new();
    for page in 1..=MAX_PAGES {
        let (batch, entries) = parse(&fetch(&token, page)?)?;
        all.extend(batch);
        if entries < PER_PAGE {
            break;
        }
    }
    if all.is_empty() {
        bail!("MOCO returned no active projects");
    }
    all.sort_by(|a, b| b.identifier.cmp(&a.identifier));
    Ok(all)
}

/// Hard failure by design: a branch name without a project number is not the
/// convention, so there is nothing sensible to fall back to. The scratch path
/// needs no token.
fn token() -> Result<String> {
    match std::env::var("MOCO_API_KEY") {
        Ok(t) if !t.trim().is_empty() => Ok(t.trim().to_string()),
        _ => bail!("MOCO_API_KEY is not set — export it, or `wt new <words>` for a scratch branch"),
    }
}

fn fetch(token: &str, page: usize) -> Result<Vec<u8>> {
    // curl reads its options from stdin instead of argv, so the token never
    // lands in `ps` output. Option names in a config file carry no `--`.
    let config = format!(
        "url = \"{ENDPOINT}?per_page={PER_PAGE}&page={page}\"\n\
         header = \"Authorization: Token token={token}\"\n\
         fail\n\
         silent\n\
         show-error\n"
    );
    let mut child = Command::new("curl")
        .args(["--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running curl — is it on PATH?")?;
    child
        .stdin
        .take()
        .context("curl closed its stdin")?
        .write_all(config.as_bytes())
        .context("writing the curl config")?;

    let out = child.wait_with_output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("MOCO request failed: {}", stderr.trim());
    }
    Ok(out.stdout)
}

/// Returns the usable projects plus the number of entries the page held — the
/// raw count, not the kept count, since it is what decides whether to ask for
/// another page.
fn parse(body: &[u8]) -> Result<(Vec<Project>, usize)> {
    let json: Value = serde_json::from_slice(body).context("MOCO returned invalid JSON")?;
    let entries = json
        .as_array()
        .context("expected a JSON array of projects from MOCO")?;
    let projects = entries
        .iter()
        .filter_map(|p| {
            // Belt and braces: the request already leaves `include_archived`
            // off, so MOCO should not be sending closed projects at all.
            if !p.get("active").and_then(Value::as_bool).unwrap_or(true) {
                return None;
            }
            // The branch name is built around the project number, so a project
            // without an identifier gives the picker nothing to offer.
            Some(Project {
                identifier: p.get("identifier")?.as_str()?.to_string(),
                name: text(p.get("name")).unwrap_or_else(|| "(unnamed)".into()),
                customer: text(p.pointer("/customer/name")).unwrap_or_else(|| "—".into()),
            })
        })
        .collect();
    Ok((projects, entries.len()))
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::parse;

    const PAGE: &[u8] = br#"[
      {"id": 1, "identifier": "P26059", "name": "Import-Button", "active": true,
       "customer": {"id": 9, "name": "VGH Versicherungen"}},
      {"id": 2, "identifier": "P24052", "name": "75 Jahre SONAX Film", "active": false,
       "customer": {"id": 8, "name": "SONAX GmbH"}},
      {"id": 3, "name": "no identifier, unusable", "active": true,
       "customer": {"id": 8, "name": "SONAX GmbH"}}
    ]"#;

    #[test]
    fn reads_the_fields_the_branch_name_needs() {
        let (projects, entries) = parse(PAGE).unwrap();
        assert_eq!(entries, 3, "the raw count drives pagination");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].identifier, "P26059");
        assert_eq!(projects[0].name, "Import-Button");
        assert_eq!(projects[0].customer, "VGH Versicherungen");
    }

    #[test]
    fn survives_a_project_with_no_customer() {
        let (projects, _) =
            parse(br#"[{"identifier": "P1", "name": "x", "customer": null}]"#).unwrap();
        assert_eq!(projects[0].customer, "—");
        assert_eq!(projects.len(), 1, "absent `active` reads as active");
    }

    #[test]
    fn rejects_a_non_array_body() {
        assert!(parse(br#"{"error": "unauthorized"}"#).is_err());
        assert!(parse(b"<html>").is_err());
    }
}
