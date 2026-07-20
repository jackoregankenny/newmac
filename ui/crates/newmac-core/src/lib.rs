//! newmac-core — the shared brains behind the newmac TUI.
//!
//! Everything the UI needs that isn't drawing:
//! * [`catalog`] — the canonical package catalog (`catalog.toml`, embedded).
//! * [`search`] — fuzzy search / filter across the catalog.
//! * [`selection`] — the user's picks + toggles, read from / written to
//!   `newmac.conf` in the exact shape the bash `install.sh` consumes.
//! * [`brew`] — browse popular Homebrew formulae/casks and add custom ones.
//! * [`theme`] — the colour palettes, for live swatch previews.
//!
//! The install logic itself stays in bash — this crate only decides *what*
//! to install and records it. That keeps the Rust layer a UI swap, not a
//! rewrite (see docs/ROADMAP.md #11, phase 1).

pub mod brew;
pub mod catalog;
pub mod search;
pub mod selection;
pub mod theme;

pub use catalog::{Catalog, Category, Flag, Item, Kind};
pub use selection::{Selection, Toggles};
