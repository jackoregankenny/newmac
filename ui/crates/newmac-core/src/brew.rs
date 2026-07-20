//! Browse popular Homebrew packages and add custom ones.
//!
//! Two data sources:
//! * **Bundled** — a curated snapshot baked into the binary ([`bundled`]) so
//!   the browse screen works offline on a freshly-imaged Mac.
//! * **Live** — [`refresh`] shells out to `curl` (always present on macOS) to
//!   pull the current install-count leaderboard from `formulae.brew.sh`. We
//!   shell out rather than pull in a TLS stack, matching how the rest of
//!   newmac already installs things.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

const BUNDLED: &str = include_str!("../brew_popular.json");

/// Whether a package is a formula (CLI) or a cask (GUI app).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrewKind {
    Formula,
    Cask,
}

impl BrewKind {
    pub fn label(self) -> &'static str {
        match self {
            BrewKind::Formula => "formula",
            BrewKind::Cask => "cask",
        }
    }
}

/// A browsable Homebrew package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub desc: String,
    pub kind: BrewKind,
    /// 90-day install count when known (live refresh only).
    pub installs: Option<u64>,
}

impl Package {
    /// Composite haystack for fuzzy search (name + description).
    pub fn haystack(&self) -> String {
        format!("{} {}", self.name, self.desc)
    }
}

#[derive(Debug, Deserialize)]
struct RawPkg {
    name: String,
    #[serde(default)]
    desc: String,
}

#[derive(Debug, Deserialize)]
struct RawSnapshot {
    #[serde(default)]
    formula: Vec<RawPkg>,
    #[serde(default)]
    cask: Vec<RawPkg>,
}

/// The bundled snapshot, formulae then casks, in curated (roughly popular) order.
pub fn bundled() -> Vec<Package> {
    let raw: RawSnapshot = serde_json::from_str(BUNDLED).expect("bundled brew snapshot must parse");
    let mut out = Vec::with_capacity(raw.formula.len() + raw.cask.len());
    out.extend(raw.formula.into_iter().map(|p| Package {
        name: p.name,
        desc: p.desc,
        kind: BrewKind::Formula,
        installs: None,
    }));
    out.extend(raw.cask.into_iter().map(|p| Package {
        name: p.name,
        desc: p.desc,
        kind: BrewKind::Cask,
        installs: None,
    }));
    // Bundled snapshot occasionally repeats a name (curation drift); keep first.
    dedupe(&mut out);
    out
}

// ---- Live refresh -----------------------------------------------------------

/// Homebrew analytics endpoints (name + install count, already ranked).
const FORMULA_ANALYTICS: &str = "https://formulae.brew.sh/api/analytics/install/90d.json";
const CASK_ANALYTICS: &str = "https://formulae.brew.sh/api/analytics/cask-install/90d.json";

#[derive(Debug, Deserialize)]
struct AnalyticsItem {
    #[serde(default)]
    formula: Option<String>,
    #[serde(default)]
    cask: Option<String>,
    #[serde(default)]
    count: String,
}

#[derive(Debug, Deserialize)]
struct Analytics {
    #[serde(default)]
    items: Vec<AnalyticsItem>,
}

/// Fetch the current top-`limit` formulae and casks by 90-day installs.
///
/// Descriptions are merged in from the bundled snapshot where available (the
/// analytics feed carries counts, not descriptions). Requires `curl` + network.
pub fn refresh(limit: usize) -> Result<Vec<Package>> {
    let descs: HashMap<String, String> = bundled()
        .into_iter()
        .map(|p| (format!("{}:{}", p.kind.label(), p.name), p.desc))
        .collect();

    let mut out = Vec::new();
    out.extend(fetch_ranked(
        FORMULA_ANALYTICS,
        BrewKind::Formula,
        limit,
        &descs,
    )?);
    out.extend(fetch_ranked(CASK_ANALYTICS, BrewKind::Cask, limit, &descs)?);
    dedupe(&mut out);
    Ok(out)
}

fn fetch_ranked(
    url: &str,
    kind: BrewKind,
    limit: usize,
    descs: &HashMap<String, String>,
) -> Result<Vec<Package>> {
    let body = curl(url)?;
    let parsed: Analytics =
        serde_json::from_str(&body).with_context(|| format!("unexpected JSON from {url}"))?;
    Ok(parsed
        .items
        .into_iter()
        .filter_map(|it| {
            let name = match kind {
                BrewKind::Formula => it.formula,
                BrewKind::Cask => it.cask,
            }?;
            // Analytics reports the fully-qualified formula (e.g. `openssl@3`);
            // the leading token is the installable name.
            let name = name.split(' ').next().unwrap_or(&name).to_string();
            Some(Package {
                desc: descs
                    .get(&format!("{}:{}", kind.label(), name))
                    .cloned()
                    .unwrap_or_default(),
                installs: parse_count(&it.count),
                name,
                kind,
            })
        })
        .take(limit)
        .collect())
}

fn parse_count(s: &str) -> Option<u64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn curl(url: &str) -> Result<String> {
    let out = std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .context("failed to spawn curl (is it on PATH?)")?;
    if !out.status.success() {
        anyhow::bail!(
            "curl {url} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    String::from_utf8(out.stdout).context("curl returned non-UTF-8")
}

fn dedupe(pkgs: &mut Vec<Package>) {
    let mut seen = std::collections::HashSet::new();
    pkgs.retain(|p| seen.insert((p.kind, p.name.clone())));
}
