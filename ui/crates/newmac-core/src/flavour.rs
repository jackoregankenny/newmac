//! Flavours — named premade setups shown on the picker's "Presets" screen.
//!
//! Each lives in its own `flavours/<id>.toml` (repo root). They're collected at
//! build time into an embedded blob (see `build.rs`) so a prebuilt binary ships
//! them all, and can also be read from a `flavours/` dir at runtime
//! (`--flavours-dir`) so a freshly-cloned flavour shows against an older binary.
//! Adding one is a one-file PR — see CONTRIBUTING.md.

use crate::selection::Toggles;
use serde::Deserialize;
use std::path::Path;

const EMBEDDED: &str = include_str!(concat!(env!("OUT_DIR"), "/flavours.toml"));

/// A premade setup: a seed of catalog ids + theme + toggles.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Flavour {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub glass: bool,
    #[serde(default)]
    pub ricing: bool,
    #[serde(default)]
    pub macos_defaults: bool,
    #[serde(default)]
    pub power: bool,
    #[serde(default)]
    pub schedule: bool,
    #[serde(default)]
    pub dock: bool,
    #[serde(default)]
    pub ids: Vec<String>,
}

fn default_theme() -> String {
    "tokyonight".to_string()
}

impl Flavour {
    pub fn toggles(&self) -> Toggles {
        Toggles {
            ricing: self.ricing,
            macos_defaults: self.macos_defaults,
            power: self.power,
            schedule: self.schedule,
            dock: self.dock,
        }
    }

    /// Ids referenced by this flavour that don't exist in the catalog.
    pub fn unknown_ids<'a>(&'a self, catalog: &crate::Catalog) -> Vec<&'a str> {
        self.ids
            .iter()
            .filter(|id| catalog.get(id).is_none())
            .map(String::as_str)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct Embedded {
    #[serde(default)]
    flavour: Vec<Flavour>,
}

/// The flavours compiled into the binary, ordered for the Presets screen:
/// `jack` first, then the rest alphabetically by id.
pub fn all() -> Vec<Flavour> {
    let parsed: Embedded = toml::from_str(EMBEDDED).expect("embedded flavours must parse");
    order(parsed.flavour)
}

/// Load flavours from a `flavours/` dir (one flat table per file), falling back
/// to the embedded set if the dir is missing or empty.
pub fn from_dir_or_embedded(dir: Option<&Path>) -> Vec<Flavour> {
    let Some(dir) = dir else {
        return all();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return all();
    };
    let mut flavours = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            match toml::from_str::<Flavour>(&text) {
                Ok(f) => flavours.push(f),
                Err(e) => eprintln!("warning: skipping {}: {e}", path.display()),
            }
        }
    }
    if flavours.is_empty() {
        return all();
    }
    order(flavours)
}

fn order(mut flavours: Vec<Flavour>) -> Vec<Flavour> {
    // Jack's first, Basic second, then the rest alphabetically.
    let rank = |id: &str| match id {
        "jack" => 0,
        "basic" => 1,
        _ => 2,
    };
    flavours.sort_by(|a, b| (rank(&a.id), a.id.clone()).cmp(&(rank(&b.id), b.id.clone())));
    flavours
}
