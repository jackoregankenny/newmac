//! The user's picks, read from and written to `newmac.conf`.
//!
//! The file is plain shell `KEY=value` sourced by every bash script, so we
//! reproduce its shape exactly. The Rust UI is a drop-in for the bash
//! `configure.sh` — it writes the same conf `install.sh` already reads, plus
//! two new keys (`NEWMAC_EXTRA_BREW` / `NEWMAC_EXTRA_CASK`) for the packages
//! the user adds via the Homebrew browse/add screen.

use std::collections::BTreeSet;

/// The five opt-in behaviours, mirroring `NEWMAC_TOGGLE_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toggles {
    /// Apply the tiling-desktop configs (AeroSpace/sketchybar/borders/Karabiner).
    pub ricing: bool,
    /// Apply opinionated macOS UX defaults.
    pub macos_defaults: bool,
    /// Apply battery/power tuning via pmset.
    pub power: bool,
    /// Schedule weekly auto-updates.
    pub schedule: bool,
    /// Arrange the Dock to match the selection.
    pub dock: bool,
}

impl Default for Toggles {
    fn default() -> Self {
        // Matches the bash defaults in configure.sh.
        Self {
            ricing: true,
            macos_defaults: true,
            power: true,
            schedule: false,
            dock: true,
        }
    }
}

/// A custom Homebrew package the user added by hand or from the browse list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Custom {
    pub name: String,
    pub cask: bool,
}

/// Everything the picker collects, ready to serialise to `newmac.conf`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub selected: BTreeSet<String>,
    pub theme: String,
    pub toggles: Toggles,
    pub extra_brew: Vec<String>,
    pub extra_cask: Vec<String>,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            selected: BTreeSet::new(),
            theme: "tokyonight".to_string(),
            toggles: Toggles::default(),
            extra_brew: Vec::new(),
            extra_cask: Vec::new(),
        }
    }
}

impl Selection {
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.contains(id)
    }

    pub fn toggle(&mut self, id: &str) {
        if !self.selected.remove(id) {
            self.selected.insert(id.to_string());
        }
    }

    pub fn set(&mut self, id: &str, on: bool) {
        if on {
            self.selected.insert(id.to_string());
        } else {
            self.selected.remove(id);
        }
    }

    /// Add a custom package (deduped). Returns false if it was already there.
    pub fn add_custom(&mut self, c: &Custom) -> bool {
        let list = if c.cask {
            &mut self.extra_cask
        } else {
            &mut self.extra_brew
        };
        if list.iter().any(|n| n == &c.name) {
            return false;
        }
        list.push(c.name.clone());
        true
    }

    pub fn remove_custom(&mut self, name: &str, cask: bool) {
        let list = if cask {
            &mut self.extra_cask
        } else {
            &mut self.extra_brew
        };
        list.retain(|n| n != name);
    }

    /// Seed a selection from the catalog's on/off defaults.
    pub fn from_defaults(catalog: &crate::Catalog) -> Self {
        let mut s = Self::default();
        for item in &catalog.items {
            if item.default {
                s.selected.insert(item.id.clone());
            }
        }
        s
    }

    /// Parse an existing `newmac.conf`. Unknown keys are ignored so the file
    /// can gain fields without breaking older readers.
    pub fn parse_conf(text: &str) -> Self {
        let mut s = Self::default();
        // Keep catalog defaults only if the file didn't mention selection.
        let mut saw_selected = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once('=') else {
                continue;
            };
            let val = unquote(val.trim());
            match key.trim() {
                "NEWMAC_SELECTED" => {
                    saw_selected = true;
                    s.selected = val.split_whitespace().map(str::to_string).collect();
                }
                "NEWMAC_THEME" => s.theme = val,
                "NEWMAC_TOGGLE_RICING" => s.toggles.ricing = as_bool(&val),
                "NEWMAC_TOGGLE_MACOS_DEFAULTS" => s.toggles.macos_defaults = as_bool(&val),
                "NEWMAC_TOGGLE_POWER" => s.toggles.power = as_bool(&val),
                "NEWMAC_TOGGLE_SCHEDULE" => s.toggles.schedule = as_bool(&val),
                "NEWMAC_TOGGLE_DOCK" => s.toggles.dock = as_bool(&val),
                "NEWMAC_EXTRA_BREW" => {
                    s.extra_brew = val.split_whitespace().map(str::to_string).collect()
                }
                "NEWMAC_EXTRA_CASK" => {
                    s.extra_cask = val.split_whitespace().map(str::to_string).collect()
                }
                _ => {}
            }
        }
        let _ = saw_selected;
        s
    }

    /// Render `newmac.conf` exactly as bash expects to source it.
    pub fn render_conf(&self, generated_note: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("# newmac.conf — generated by {generated_note}\n"));
        out.push_str("# Edit by hand or re-run: newmac configure\n");
        // Leading+trailing spaces mirror the bash writer so the substring
        // membership test `case " $NEWMAC_SELECTED " in *" $id "*)` stays true.
        let ids: Vec<&str> = self.selected.iter().map(String::as_str).collect();
        out.push_str(&format!("NEWMAC_SELECTED=\" {} \"\n", ids.join(" ")));
        out.push_str(&format!("NEWMAC_THEME={}\n", self.theme));
        out.push_str(&format!(
            "NEWMAC_TOGGLE_RICING={}\n",
            b(self.toggles.ricing)
        ));
        out.push_str(&format!(
            "NEWMAC_TOGGLE_MACOS_DEFAULTS={}\n",
            b(self.toggles.macos_defaults)
        ));
        out.push_str(&format!("NEWMAC_TOGGLE_POWER={}\n", b(self.toggles.power)));
        out.push_str(&format!(
            "NEWMAC_TOGGLE_SCHEDULE={}\n",
            b(self.toggles.schedule)
        ));
        out.push_str(&format!("NEWMAC_TOGGLE_DOCK={}\n", b(self.toggles.dock)));
        if !self.extra_brew.is_empty() {
            out.push_str(&format!(
                "NEWMAC_EXTRA_BREW=\"{}\"\n",
                self.extra_brew.join(" ")
            ));
        }
        if !self.extra_cask.is_empty() {
            out.push_str(&format!(
                "NEWMAC_EXTRA_CASK=\"{}\"\n",
                self.extra_cask.join(" ")
            ));
        }
        out
    }
}

fn b(v: bool) -> u8 {
    v as u8
}

fn as_bool(v: &str) -> bool {
    matches!(v.trim(), "1" | "true" | "yes" | "on")
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    let bytes = v.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}
