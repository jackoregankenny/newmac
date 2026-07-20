//! Application state + input handling for the newmac picker.
//!
//! Drawing lives in [`crate::ui`]; this module owns *what* is shown and how
//! keys mutate it. Kept UI-framework-light so it can be unit-tested without a
//! terminal (see the tests at the bottom and the snapshot tests in `tests/`).

use newmac_core::brew::{self, BrewKind, Package};
use newmac_core::search::{self, Searcher};
use newmac_core::selection::Custom;
use newmac_core::theme::{self, Theme};
use newmac_core::{Catalog, Selection};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::path::PathBuf;

/// A drawn list: its inner (clickable) area + the scroll state ratatui owns.
/// [`crate::ui`] fills these in every frame so mouse clicks can hit-test.
#[derive(Default)]
pub struct Slot {
    pub rect: Rect,
    pub state: ListState,
}

impl Slot {
    /// Which row index a click at `row` maps to, accounting for scroll offset.
    fn index_at(&self, row: u16) -> Option<usize> {
        if row < self.rect.y || row >= self.rect.y + self.rect.height {
            return None;
        }
        Some(self.state.offset() + (row - self.rect.y) as usize)
    }
}

/// Geometry recorded during draw, consumed by mouse hit-testing.
#[derive(Default)]
pub struct UiState {
    pub tabs_rect: Rect,
    pub tab_spans: Vec<(u16, u16)>, // x0..x1 per Tab::ALL index
    pub cat: Slot,
    pub items: Slot,
    pub brew: Slot,
    pub theme: Slot,
    pub options: Slot,
}

fn rect_contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}

/// A framework-agnostic mouse event, so mouse handling stays testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mouse {
    Down(u16, u16),         // left click at (col, row)
    Scroll(bool, u16, u16), // (down?, col, row)
}

/// The top-level tabs, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Packages,
    Browse,
    Theme,
    Options,
    Save,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Packages,
        Tab::Browse,
        Tab::Theme,
        Tab::Options,
        Tab::Save,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Packages => "Packages",
            Tab::Browse => "Browse Homebrew",
            Tab::Theme => "Theme",
            Tab::Options => "Options",
            Tab::Save => "Save",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap()
    }

    fn from_index(i: usize) -> Tab {
        Self::ALL[i % Self::ALL.len()]
    }
}

/// Which pane has keyboard focus on the Packages tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Categories,
    Items,
}

/// A short-lived text prompt (e.g. "add custom formula").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub label: String,
    pub input: String,
    pub kind: PromptKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    CustomFormula,
    CustomCask,
}

/// How the last in-TUI install run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOutcome {
    Ok,
    Failed(i32),
    DryRun,
}

/// The whole picker.
pub struct App {
    pub catalog: Catalog,
    pub sel: Selection,
    pub themes: Vec<Theme>,
    pub conf_path: PathBuf,
    /// Repo root (for `scripts/install.sh`), from $NEWMAC or the conf's parent.
    pub repo_dir: Option<PathBuf>,
    searcher: Searcher,

    pub tab: Tab,
    pub status: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub saved: bool,

    /// Recorded draw geometry, for mouse hit-testing.
    pub ui: UiState,
    /// Set on the Save screen; the event loop runs the install, then clears it.
    pub install_requested: bool,
    pub dryrun_requested: bool,
    pub last_install: Option<InstallOutcome>,

    // Packages tab
    pub pane: Pane,
    pub cat_idx: usize,  // 0 = "All", then selectable categories
    pub item_idx: usize, // index into the currently visible item list
    pub query: String,
    pub searching: bool,

    // Browse tab
    pub brew: Vec<Package>,
    pub brew_idx: usize,
    pub brew_query: String,
    pub brew_searching: bool,
    pub prompt: Option<Prompt>,

    // Theme tab
    pub theme_idx: usize,

    // Options tab
    pub option_idx: usize,
}

/// A category choice in the left pane: `None` is the "All" pseudo-entry.
pub type CatChoice = Option<String>;

impl App {
    /// Build with the embedded themes (used by tests and the default path).
    pub fn new(catalog: Catalog, sel: Selection, conf_path: PathBuf) -> Self {
        Self::with_themes(catalog, sel, conf_path, theme::all())
    }

    /// Build with an explicit theme list (the binary passes repo-loaded themes).
    pub fn with_themes(
        catalog: Catalog,
        sel: Selection,
        conf_path: PathBuf,
        themes: Vec<Theme>,
    ) -> Self {
        let theme_idx = themes.iter().position(|t| t.id == sel.theme).unwrap_or(0);
        let repo_dir = std::env::var("NEWMAC")
            .ok()
            .map(PathBuf::from)
            .or_else(|| conf_path.parent().map(|p| p.to_path_buf()));
        App {
            catalog,
            sel,
            themes,
            conf_path,
            repo_dir,
            searcher: Searcher::new(),
            tab: Tab::Packages,
            status: "Space/click toggles · / searches · mouse works · ? for help".into(),
            show_help: false,
            should_quit: false,
            saved: false,
            ui: UiState::default(),
            install_requested: false,
            dryrun_requested: false,
            last_install: None,
            pane: Pane::Categories,
            cat_idx: 0,
            item_idx: 0,
            query: String::new(),
            searching: false,
            brew: brew::bundled(),
            brew_idx: 0,
            brew_query: String::new(),
            brew_searching: false,
            prompt: None,
            theme_idx,
            option_idx: 0,
        }
    }

    // ---- Packages: category + item derivation ---------------------------

    /// The left-pane category choices: `All` first, then selectable categories.
    pub fn categories(&self) -> Vec<CatChoice> {
        let mut v: Vec<CatChoice> = vec![None];
        v.extend(
            self.catalog
                .selectable_categories()
                .map(|c| Some(c.id.clone())),
        );
        v
    }

    /// Catalog item indices currently visible in the right pane, honouring the
    /// search query (global) or the selected category.
    pub fn visible_items(&mut self) -> Vec<usize> {
        let selectable: Vec<&str> = self
            .catalog
            .selectable_categories()
            .map(|c| c.id.as_str())
            .collect();

        if !self.query.is_empty() {
            // Global fuzzy search across every selectable item.
            let haystacks: Vec<(usize, String)> = self
                .catalog
                .items
                .iter()
                .enumerate()
                .filter(|(_, it)| selectable.contains(&it.category.as_str()))
                .map(|(i, it)| (i, search::haystack(it)))
                .collect();
            let refs: Vec<(usize, &str)> =
                haystacks.iter().map(|(i, h)| (*i, h.as_str())).collect();
            return self
                .searcher
                .rank(&self.query, refs)
                .into_iter()
                .map(|(i, _)| i)
                .collect();
        }

        // No search: items in the selected category (or all, catalog order).
        let cats = self.categories();
        let chosen = cats.get(self.cat_idx).cloned().flatten();
        self.catalog
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| selectable.contains(&it.category.as_str()))
            .filter(|(_, it)| match &chosen {
                Some(c) => &it.category == c,
                None => true,
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Count of selected items in a category (for the sidebar badge).
    pub fn selected_in(&self, cat: &CatChoice) -> (usize, usize) {
        let selectable: Vec<&str> = self
            .catalog
            .selectable_categories()
            .map(|c| c.id.as_str())
            .collect();
        let mut total = 0;
        let mut on = 0;
        for it in &self.catalog.items {
            let in_scope = match cat {
                Some(c) => &it.category == c,
                None => selectable.contains(&it.category.as_str()),
            };
            if in_scope {
                total += 1;
                if self.sel.is_selected(&it.id) {
                    on += 1;
                }
            }
        }
        (on, total)
    }

    fn clamp_item(&mut self) {
        let n = self.visible_items().len();
        if n == 0 {
            self.item_idx = 0;
        } else if self.item_idx >= n {
            self.item_idx = n - 1;
        }
    }

    // ---- Browse derivation ---------------------------------------------

    pub fn visible_brew(&mut self) -> Vec<usize> {
        if self.brew_query.is_empty() {
            return (0..self.brew.len()).collect();
        }
        let hs: Vec<(usize, String)> = self
            .brew
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.haystack()))
            .collect();
        let refs: Vec<(usize, &str)> = hs.iter().map(|(i, h)| (*i, h.as_str())).collect();
        self.searcher
            .rank(&self.brew_query, refs)
            .into_iter()
            .map(|(i, _)| i)
            .collect()
    }

    fn brew_is_added(&self, p: &Package) -> bool {
        match p.kind {
            BrewKind::Formula => self.sel.extra_brew.iter().any(|n| n == &p.name),
            BrewKind::Cask => self.sel.extra_cask.iter().any(|n| n == &p.name),
        }
    }

    pub fn brew_added(&self, i: usize) -> bool {
        self.brew
            .get(i)
            .map(|p| self.brew_is_added(p))
            .unwrap_or(false)
    }

    // ---- Save -----------------------------------------------------------

    /// Selected items carrying warning flags — surfaced on the Save screen.
    pub fn flagged_selected(&self) -> Vec<&newmac_core::Item> {
        self.catalog
            .items
            .iter()
            .filter(|it| !it.flags.is_empty() && self.sel.is_selected(&it.id))
            .collect()
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.sel.theme = self.themes[self.theme_idx].id.clone();
        let note = format!("newmac-ui {}", env!("CARGO_PKG_VERSION"));
        let body = self.sel.render_conf(&note);
        std::fs::write(&self.conf_path, body)?;
        self.saved = true;
        self.status = format!("Saved {} — run `newmac install`", self.conf_path.display());
        Ok(())
    }

    // ---- Input ----------------------------------------------------------

    /// Handle a key. Returns nothing; sets `should_quit` when done.
    pub fn on_key(&mut self, key: Key) {
        if self.show_help {
            // Any key dismisses help.
            self.show_help = false;
            return;
        }
        if self.prompt.is_some() {
            self.on_prompt_key(key);
            return;
        }
        // Search modes capture text first.
        if self.tab == Tab::Packages && self.searching && self.on_search_key(key) {
            return;
        }
        if self.tab == Tab::Browse && self.brew_searching && self.on_brew_search_key(key) {
            return;
        }

        match key {
            Key::Char('q') | Key::Esc => self.should_quit = true,
            Key::Char('?') => self.show_help = true,
            Key::Tab | Key::Char(']') => self.set_tab(Tab::from_index(self.tab.index() + 1)),
            Key::BackTab | Key::Char('[') => {
                let i = self.tab.index();
                let prev = if i == 0 { Tab::ALL.len() - 1 } else { i - 1 };
                self.set_tab(Tab::from_index(prev));
            }
            Key::Char(c @ '1'..='5') => {
                let i = c as usize - '1' as usize;
                self.set_tab(Tab::from_index(i));
            }
            Key::CtrlS => {
                let _ = self.save();
                self.tab = Tab::Save;
            }
            _ => match self.tab {
                Tab::Packages => self.on_packages_key(key),
                Tab::Browse => self.on_browse_key(key),
                Tab::Theme => self.on_theme_key(key),
                Tab::Options => self.on_options_key(key),
                Tab::Save => self.on_save_key(key),
            },
        }
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn on_packages_key(&mut self, key: Key) {
        match key {
            Key::Char('/') => {
                self.searching = true;
                self.pane = Pane::Items;
                self.status = "Search: type to filter · Esc clears · Enter keeps".into();
            }
            Key::Left | Key::Char('h') => self.pane = Pane::Categories,
            Key::Right | Key::Char('l') => self.pane = Pane::Items,
            Key::Up | Key::Char('k') => self.move_cursor(-1),
            Key::Down | Key::Char('j') => self.move_cursor(1),
            Key::Char(' ') | Key::Enter => {
                if self.pane == Pane::Items {
                    self.toggle_current_item();
                } else {
                    self.pane = Pane::Items;
                }
            }
            Key::Char('a') => self.set_visible(true),
            Key::Char('n') => self.set_visible(false),
            _ => {}
        }
    }

    fn move_cursor(&mut self, delta: i64) {
        match (self.tab, self.pane) {
            (Tab::Packages, Pane::Categories) if self.query.is_empty() => {
                let n = self.categories().len() as i64;
                self.cat_idx = wrap(self.cat_idx as i64 + delta, n);
                self.item_idx = 0;
            }
            (Tab::Packages, _) => {
                let n = self.visible_items().len() as i64;
                if n > 0 {
                    self.item_idx = wrap(self.item_idx as i64 + delta, n);
                }
            }
            _ => {}
        }
    }

    fn toggle_current_item(&mut self) {
        let vis = self.visible_items();
        if let Some(&idx) = vis.get(self.item_idx) {
            let id = self.catalog.items[idx].id.clone();
            self.sel.toggle(&id);
            let on = self.sel.is_selected(&id);
            self.status = format!(
                "{} {}",
                if on { "Selected" } else { "Removed" },
                self.catalog.items[idx].name
            );
        }
    }

    fn set_visible(&mut self, on: bool) {
        for idx in self.visible_items() {
            let id = self.catalog.items[idx].id.clone();
            self.sel.set(&id, on);
        }
        self.status = if on {
            "Selected all shown".into()
        } else {
            "Cleared all shown".into()
        };
    }

    /// Returns true if the key was consumed by search editing.
    fn on_search_key(&mut self, key: Key) -> bool {
        match key {
            Key::Esc => {
                self.query.clear();
                self.searching = false;
                self.item_idx = 0;
                self.status = "Search cleared".into();
                true
            }
            Key::Enter => {
                self.searching = false;
                self.status = if self.query.is_empty() {
                    "".into()
                } else {
                    format!("Filtered by \"{}\" · Esc clears", self.query)
                };
                true
            }
            Key::Backspace => {
                self.query.pop();
                self.item_idx = 0;
                true
            }
            Key::Up => {
                self.move_cursor(-1);
                true
            }
            Key::Down => {
                self.move_cursor(1);
                true
            }
            Key::Char(' ') => {
                // Space toggles the highlighted item even mid-search.
                self.toggle_current_item();
                true
            }
            Key::Char(c) => {
                self.query.push(c);
                self.item_idx = 0;
                true
            }
            _ => false,
        }
    }

    fn on_browse_key(&mut self, key: Key) {
        match key {
            Key::Char('/') => {
                self.brew_searching = true;
                self.status = "Search Homebrew: type to filter · Esc clears".into();
            }
            Key::Up | Key::Char('k') => self.move_brew(-1),
            Key::Down | Key::Char('j') => self.move_brew(1),
            Key::Char(' ') | Key::Enter | Key::Char('a') => self.toggle_current_brew(),
            Key::Char('c') => {
                self.prompt = Some(Prompt {
                    label: "Add custom formula (brew install)".into(),
                    input: String::new(),
                    kind: PromptKind::CustomFormula,
                });
            }
            Key::Char('C') => {
                self.prompt = Some(Prompt {
                    label: "Add custom cask (brew install --cask)".into(),
                    input: String::new(),
                    kind: PromptKind::CustomCask,
                });
            }
            Key::Char('r') => self.refresh_brew(),
            _ => {}
        }
    }

    fn move_brew(&mut self, delta: i64) {
        let n = self.visible_brew().len() as i64;
        if n > 0 {
            self.brew_idx = wrap(self.brew_idx as i64 + delta, n);
        }
    }

    fn toggle_current_brew(&mut self) {
        let vis = self.visible_brew();
        if let Some(&i) = vis.get(self.brew_idx) {
            let p = self.brew[i].clone();
            let custom = Custom {
                name: p.name.clone(),
                cask: p.kind == BrewKind::Cask,
            };
            if self.brew_is_added(&p) {
                self.sel.remove_custom(&p.name, custom.cask);
                self.status = format!("Removed {}", p.name);
            } else {
                self.sel.add_custom(&custom);
                self.status = format!("Added {} ({})", p.name, p.kind.label());
            }
        }
    }

    fn refresh_brew(&mut self) {
        self.status = "Refreshing from formulae.brew.sh…".into();
        match brew::refresh(200) {
            Ok(list) if !list.is_empty() => {
                self.brew = list;
                self.brew_idx = 0;
                self.status = format!("Refreshed — {} popular packages (live)", self.brew.len());
            }
            Ok(_) => self.status = "Refresh returned nothing — keeping bundled list".into(),
            Err(e) => self.status = format!("Refresh failed ({e}) — keeping bundled list"),
        }
    }

    fn on_brew_search_key(&mut self, key: Key) -> bool {
        match key {
            Key::Esc => {
                self.brew_query.clear();
                self.brew_searching = false;
                self.brew_idx = 0;
                true
            }
            Key::Enter => {
                self.brew_searching = false;
                true
            }
            Key::Backspace => {
                self.brew_query.pop();
                self.brew_idx = 0;
                true
            }
            Key::Up => {
                self.move_brew(-1);
                true
            }
            Key::Down => {
                self.move_brew(1);
                true
            }
            Key::Char(c) => {
                self.brew_query.push(c);
                self.brew_idx = 0;
                true
            }
            _ => false,
        }
    }

    fn on_prompt_key(&mut self, key: Key) {
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        match key {
            Key::Esc => self.prompt = None,
            Key::Enter => {
                let name = prompt.input.trim().to_string();
                let kind = prompt.kind;
                self.prompt = None;
                if name.is_empty() {
                    return;
                }
                let cask = kind == PromptKind::CustomCask;
                let added = self.sel.add_custom(&Custom {
                    name: name.clone(),
                    cask,
                });
                self.status = if added {
                    format!(
                        "Added custom {} {}",
                        if cask { "cask" } else { "formula" },
                        name
                    )
                } else {
                    format!("{name} already added")
                };
            }
            Key::Backspace => {
                prompt.input.pop();
            }
            Key::Char(c) => prompt.input.push(c),
            _ => {}
        }
    }

    fn on_theme_key(&mut self, key: Key) {
        match key {
            Key::Up | Key::Char('k') => {
                self.theme_idx = wrap(self.theme_idx as i64 - 1, self.themes.len() as i64)
            }
            Key::Down | Key::Char('j') => {
                self.theme_idx = wrap(self.theme_idx as i64 + 1, self.themes.len() as i64)
            }
            Key::Char(' ') | Key::Enter => {
                self.sel.theme = self.themes[self.theme_idx].id.clone();
                self.status = format!("Theme: {}", self.themes[self.theme_idx].title);
            }
            _ => {}
        }
    }

    fn on_options_key(&mut self, key: Key) {
        match key {
            Key::Up | Key::Char('k') => self.option_idx = wrap(self.option_idx as i64 - 1, 5),
            Key::Down | Key::Char('j') => self.option_idx = wrap(self.option_idx as i64 + 1, 5),
            Key::Char(' ') | Key::Enter => {
                let t = &mut self.sel.toggles;
                match self.option_idx {
                    0 => t.ricing = !t.ricing,
                    1 => t.macos_defaults = !t.macos_defaults,
                    2 => t.power = !t.power,
                    3 => t.schedule = !t.schedule,
                    4 => t.dock = !t.dock,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn on_save_key(&mut self, key: Key) {
        match key {
            Key::Char('s') | Key::Enter => {
                let _ = self.save();
            }
            // Run the bash installer from the picker. The event loop notices the
            // flag, suspends the TUI, runs it with live output, then returns.
            Key::Char('i') => {
                if !self.saved {
                    let _ = self.save();
                }
                self.install_requested = true;
            }
            Key::Char('d') => {
                if !self.saved {
                    let _ = self.save();
                }
                self.dryrun_requested = true;
            }
            _ => {}
        }
    }

    /// Whether an in-TUI install is even possible (we know the repo + a script).
    pub fn can_install(&self) -> bool {
        self.repo_dir
            .as_ref()
            .map(|d| d.join("scripts/install.sh").exists())
            .unwrap_or(false)
    }

    // ---- Mouse ----------------------------------------------------------

    /// Handle a mouse event using the geometry recorded during the last draw.
    pub fn on_mouse(&mut self, m: Mouse) {
        // Overlays swallow mouse input just like keys.
        if self.show_help {
            if let Mouse::Down(..) = m {
                self.show_help = false;
            }
            return;
        }
        if self.prompt.is_some() {
            return;
        }
        match m {
            Mouse::Scroll(down, col, row) => self.mouse_scroll(down, col, row),
            Mouse::Down(col, row) => self.mouse_click(col, row),
        }
    }

    fn mouse_scroll(&mut self, down: bool, col: u16, row: u16) {
        let delta = if down { 1 } else { -1 };
        // Route the wheel to whichever list the pointer is over.
        if self.tab == Tab::Packages
            && rect_contains(self.ui.cat.rect, col, row)
            && self.query.is_empty()
        {
            self.pane = Pane::Categories;
            self.move_cursor(delta);
            return;
        }
        match self.tab {
            Tab::Packages => {
                self.pane = Pane::Items;
                self.move_cursor(delta);
            }
            Tab::Browse => self.move_brew(delta),
            Tab::Theme => {
                self.theme_idx = wrap(self.theme_idx as i64 + delta, self.themes.len() as i64)
            }
            Tab::Options => self.option_idx = wrap(self.option_idx as i64 + delta, 5),
            Tab::Save => {}
        }
    }

    fn mouse_click(&mut self, col: u16, row: u16) {
        // Tab bar: click a tab title.
        if rect_contains(self.ui.tabs_rect, col, row) {
            for (i, (x0, x1)) in self.ui.tab_spans.iter().enumerate() {
                if col >= *x0 && col < *x1 {
                    self.set_tab(Tab::from_index(i));
                    return;
                }
            }
            return;
        }
        match self.tab {
            Tab::Packages => {
                if self.query.is_empty() {
                    if let Some(idx) = self.ui.cat.index_at(row) {
                        if rect_contains(self.ui.cat.rect, col, row)
                            && idx < self.categories().len()
                        {
                            self.cat_idx = idx;
                            self.item_idx = 0;
                            self.pane = Pane::Categories;
                            return;
                        }
                    }
                }
                if let Some(idx) = self.ui.items.index_at(row) {
                    if rect_contains(self.ui.items.rect, col, row)
                        && idx < self.visible_items().len()
                    {
                        self.pane = Pane::Items;
                        self.item_idx = idx;
                        self.toggle_current_item();
                    }
                }
            }
            Tab::Browse => {
                if let Some(idx) = self.ui.brew.index_at(row) {
                    if rect_contains(self.ui.brew.rect, col, row) && idx < self.visible_brew().len()
                    {
                        self.brew_idx = idx;
                        self.toggle_current_brew();
                    }
                }
            }
            Tab::Theme => {
                if let Some(idx) = self.ui.theme.index_at(row) {
                    if rect_contains(self.ui.theme.rect, col, row) && idx < self.themes.len() {
                        self.theme_idx = idx;
                        self.sel.theme = self.themes[idx].id.clone();
                        self.status = format!("Theme: {}", self.themes[idx].title);
                    }
                }
            }
            Tab::Options => {
                if let Some(idx) = self.ui.options.index_at(row) {
                    if rect_contains(self.ui.options.rect, col, row) && idx < 5 {
                        self.option_idx = idx;
                        self.on_options_key(Key::Char(' '));
                    }
                }
            }
            Tab::Save => {}
        }
    }

    /// Keep list cursors in range after any state change.
    pub fn normalize(&mut self) {
        self.clamp_item();
        let n = self.visible_brew().len();
        if n == 0 {
            self.brew_idx = 0;
        } else if self.brew_idx >= n {
            self.brew_idx = n - 1;
        }
    }
}

fn wrap(v: i64, n: i64) -> usize {
    if n <= 0 {
        return 0;
    }
    (((v % n) + n) % n) as usize
}

/// A framework-agnostic key, so `on_key` is testable without crossterm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    CtrlS,
}
