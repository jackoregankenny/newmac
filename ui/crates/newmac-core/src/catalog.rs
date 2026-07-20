//! The canonical catalog — parsed from the embedded `catalog.toml`.
//!
//! `catalog.toml` at the repo root is the single source of truth. The bash
//! `scripts/catalog.sh` is *generated* from it (`newmac-ui catalog gen-sh`),
//! so both the Rust UI and the legacy bash picker stay in lockstep.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The `catalog.toml` shipped inside the binary. On a fresh Mac the single
/// downloaded binary is fully self-contained — no repo checkout required.
pub const EMBEDDED_CATALOG: &str = include_str!("../../../../catalog.toml");

/// How an item is installed. Mirrors the `kind` column bash understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Homebrew formula.
    Brew,
    /// Homebrew cask.
    Cask,
    /// Vendor install script piped to bash.
    Curl,
    /// Global JS package (bun/npm).
    Npm,
    /// Python CLI via `uv tool`.
    Uv,
    /// Mac App Store app (`<id>:<name>`).
    Mas,
    /// Special-cased rustup toolchain.
    Rustup,
}

impl Kind {
    /// Short human tag shown in the picker, e.g. `brew`, `cask`, `App Store`.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Brew => "brew",
            Kind::Cask => "cask",
            Kind::Curl => "script",
            Kind::Npm => "npm",
            Kind::Uv => "uv",
            Kind::Mas => "App Store",
            Kind::Rustup => "rustup",
        }
    }
}

/// A cost/friction warning surfaced as a badge in the picker (ROADMAP #4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Flag {
    /// Costs money / needs a paid plan.
    Paid,
    /// Needs a sign-in / account before it is useful.
    Account,
    /// A large download (multiple GB).
    Large,
    /// Installed from the Mac App Store (needs App Store sign-in).
    Appstore,
}

impl Flag {
    /// The compact badge text, e.g. `$`, `account`, `large`, `App Store`.
    pub fn badge(self) -> &'static str {
        match self {
            Flag::Paid => "paid",
            Flag::Account => "account",
            Flag::Large => "large",
            Flag::Appstore => "App Store",
        }
    }

    /// A one-line explanation for the confirm screen.
    pub fn note(self) -> &'static str {
        match self {
            Flag::Paid => "costs money or needs a paid plan",
            Flag::Account => "needs an account / sign-in",
            Flag::Large => "large download (can be several GB)",
            Flag::Appstore => "installs from the Mac App Store (sign-in required)",
        }
    }
}

/// One installable thing.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    pub category: String,
    pub kind: Kind,
    #[serde(default)]
    pub default: bool,
    pub payload: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub flags: Vec<Flag>,
}

impl Item {
    /// The command name we probe to see if it is already present. Usually the
    /// id, with the handful of exceptions the bash installer also special-cases.
    pub fn probe_command(&self) -> &str {
        match self.id.as_str() {
            "cursor" => "cursor-agent",
            _ => &self.id,
        }
    }
}

/// A display grouping. `core` is `always` — installed unconditionally and
/// never shown as a toggle in the picker.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub always: bool,
}

#[derive(Debug, Deserialize)]
struct RawCatalog {
    #[serde(default)]
    #[allow(dead_code)]
    schema: u32,
    #[serde(default)]
    category: Vec<Category>,
    #[serde(default)]
    item: Vec<Item>,
}

/// The whole catalog: ordered categories + items.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub categories: Vec<Category>,
    pub items: Vec<Item>,
}

impl Catalog {
    /// Parse the `catalog.toml` compiled into the binary.
    pub fn embedded() -> Self {
        Self::parse(EMBEDDED_CATALOG).expect("embedded catalog.toml must be valid")
    }

    /// Load a `catalog.toml` from disk — used so a prebuilt binary can read the
    /// freshly-cloned repo's catalog instead of the one baked in at release time.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading catalog {}", path.display()))?;
        Self::parse(&text)
    }

    /// Load from `path` if given and readable, otherwise fall back to embedded.
    pub fn from_path_or_embedded(path: Option<&std::path::Path>) -> Self {
        match path {
            Some(p) => Self::load(p).unwrap_or_else(|e| {
                eprintln!("warning: {e}; using the built-in catalog");
                Self::embedded()
            }),
            None => Self::embedded(),
        }
    }

    /// Parse a `catalog.toml` string, validating cross-references.
    pub fn parse(s: &str) -> Result<Self> {
        let raw: RawCatalog = toml::from_str(s).context("catalog.toml is not valid TOML")?;
        let cat = Catalog {
            categories: raw.category,
            items: raw.item,
        };
        cat.validate()?;
        Ok(cat)
    }

    fn validate(&self) -> Result<()> {
        // Every item must point at a known category.
        for item in &self.items {
            if !self.categories.iter().any(|c| c.id == item.category) {
                anyhow::bail!(
                    "item '{}' references unknown category '{}'",
                    item.id,
                    item.category
                );
            }
        }
        // Ids must be unique — dupes silently break selection tracking.
        let mut seen = std::collections::HashSet::new();
        for item in &self.items {
            if !seen.insert(item.id.as_str()) {
                anyhow::bail!("duplicate catalog id '{}'", item.id);
            }
        }
        Ok(())
    }

    /// Human title for a category id (falls back to the id itself).
    pub fn category_title(&self, id: &str) -> String {
        self.categories
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.title.clone())
            .unwrap_or_else(|| id.to_string())
    }

    /// Items in a category, in catalog order.
    pub fn items_in<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a Item> + 'a {
        self.items.iter().filter(move |i| i.category == category)
    }

    /// Look an item up by id.
    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// The `core` (always-installed) category ids — never shown in the picker.
    pub fn always_ids(&self) -> Vec<&str> {
        let always: Vec<&str> = self
            .categories
            .iter()
            .filter(|c| c.always)
            .map(|c| c.id.as_str())
            .collect();
        self.items
            .iter()
            .filter(|i| always.contains(&i.category.as_str()))
            .map(|i| i.id.as_str())
            .collect()
    }

    /// Categories a user actually picks in (everything but `always` ones).
    pub fn selectable_categories(&self) -> impl Iterator<Item = &Category> {
        self.categories.iter().filter(|c| !c.always)
    }
}
