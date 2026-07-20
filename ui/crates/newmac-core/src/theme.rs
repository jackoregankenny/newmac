//! Colour palettes, for the live swatch preview on the theme screen.
//!
//! `themes.toml` is generated from `config/themes/*.sh` (the bash source of
//! truth) so the previews always match what `newmac theme` actually applies.
//! Regenerate with the `xtask`/just recipe when a theme changes.

use serde::Deserialize;

const EMBEDDED_THEMES: &str = include_str!("../themes.toml");

/// A 24-bit RGB colour parsed from a 6-digit hex string (no leading `#`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    fn parse(hex: &str) -> Self {
        let h = hex.trim_start_matches('#');
        let n = u32::from_str_radix(h, 16).unwrap_or(0);
        Rgb {
            r: ((n >> 16) & 0xff) as u8,
            g: ((n >> 8) & 0xff) as u8,
            b: (n & 0xff) as u8,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawTheme {
    id: String,
    title: String,
    ghostty: String,
    base: String,
    surface: String,
    text: String,
    subtext: String,
    accent: String,
    accent2: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
}

#[derive(Debug, Deserialize)]
struct RawThemes {
    #[serde(default)]
    theme: Vec<RawTheme>,
}

/// A named palette. Colours are pre-parsed to RGB for drawing swatches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub id: String,
    pub title: String,
    pub ghostty: String,
    pub base: Rgb,
    pub surface: Rgb,
    pub text: Rgb,
    pub subtext: Rgb,
    pub accent: Rgb,
    pub accent2: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
}

impl Theme {
    /// The swatches shown in the preview strip, in draw order.
    pub fn swatches(&self) -> [Rgb; 6] {
        [
            self.accent,
            self.accent2,
            self.red,
            self.green,
            self.yellow,
            self.blue,
        ]
    }
}

fn build(raw: RawTheme) -> Theme {
    Theme {
        id: raw.id,
        title: raw.title,
        ghostty: raw.ghostty,
        base: Rgb::parse(&raw.base),
        surface: Rgb::parse(&raw.surface),
        text: Rgb::parse(&raw.text),
        subtext: Rgb::parse(&raw.subtext),
        accent: Rgb::parse(&raw.accent),
        accent2: Rgb::parse(&raw.accent2),
        red: Rgb::parse(&raw.red),
        green: Rgb::parse(&raw.green),
        yellow: Rgb::parse(&raw.yellow),
        blue: Rgb::parse(&raw.blue),
    }
}

fn sort_tokyonight_first(themes: &mut [Theme]) {
    themes.sort_by_key(|t| (t.id != "tokyonight", t.id.clone()));
}

/// Load the embedded themes, `tokyonight` first (the default).
pub fn all() -> Vec<Theme> {
    let raw: RawThemes = toml::from_str(EMBEDDED_THEMES).expect("embedded themes.toml must parse");
    let mut themes: Vec<Theme> = raw.theme.into_iter().map(build).collect();
    sort_tokyonight_first(&mut themes);
    themes
}

/// Load themes from a `config/themes/*.sh` directory, so a prebuilt binary can
/// show exactly the themes the cloned repo will apply. Falls back to the
/// embedded set if the directory is missing or yields nothing.
pub fn from_dir_or_embedded(dir: Option<&std::path::Path>) -> Vec<Theme> {
    let Some(dir) = dir else {
        return all();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return all();
    };
    let mut themes: Vec<Theme> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sh") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Some(t) = parse_theme_sh(&id, &text) {
                themes.push(build(t));
            }
        }
    }
    if themes.is_empty() {
        return all();
    }
    sort_tokyonight_first(&mut themes);
    themes
}

/// Pull the `THEME_*` / `T_*` assignments out of a bash theme file.
fn parse_theme_sh(id: &str, text: &str) -> Option<RawTheme> {
    let get = |key: &str| -> Option<String> {
        text.lines().find_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix(key)?.strip_prefix('=')?;
            // Drop any trailing `# comment` (the hex values carry one), then
            // trim whitespace and surrounding quotes. None of the values here
            // contain a literal '#', so this is safe.
            let val = rest.split('#').next().unwrap_or(rest).trim();
            Some(val.trim_matches('"').trim_matches('\'').to_string())
        })
    };
    Some(RawTheme {
        title: get("THEME_TITLE").unwrap_or_else(|| id.to_string()),
        ghostty: get("THEME_GHOSTTY").unwrap_or_else(|| id.to_string()),
        base: get("T_BASE")?,
        surface: get("T_SURFACE")?,
        text: get("T_TEXT")?,
        subtext: get("T_SUBTEXT")?,
        accent: get("T_ACCENT")?,
        accent2: get("T_ACCENT2")?,
        red: get("T_RED")?,
        green: get("T_GREEN")?,
        yellow: get("T_YELLOW")?,
        blue: get("T_BLUE")?,
        id: id.to_string(),
    })
}
