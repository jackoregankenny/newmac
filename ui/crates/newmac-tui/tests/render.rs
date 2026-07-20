//! Render each tab to a headless `TestBackend` and assert the important text
//! actually reaches the screen. Catches layout/rendering regressions without a
//! real terminal — the drawing-side complement to `app_logic.rs`.

use newmac_core::{Catalog, Selection};
use newmac_tui::app::{App, Key};
use newmac_tui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn screen(app: &mut App, w: u16, h: u16) -> String {
    let backend = TestBackend::new(w, h);
    let mut terminal = Terminal::new(backend).unwrap();
    app.normalize();
    terminal.draw(|f| ui::draw(f, app)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..h {
        for x in 0..w {
            out.push_str(buf.cell((x, y)).map(|c| c.symbol()).unwrap_or(" "));
        }
        out.push('\n');
    }
    out
}

fn fresh() -> App {
    let catalog = Catalog::embedded();
    let sel = Selection::from_defaults(&catalog);
    App::new(catalog, sel, std::env::temp_dir().join("newmac.conf"))
}

#[test]
fn packages_tab_shows_chrome_and_categories() {
    let mut app = fresh();
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("newmac"), "header");
    assert!(s.contains("Categories"), "sidebar");
    assert!(s.contains("Ghostty"), "a known item");
    assert!(s.contains("Terminals"), "a category title");
}

#[test]
fn warning_badges_render() {
    let mut app = fresh();
    // Navigate to the dev-apps category where Xcode (large + App Store) lives,
    // via global search so we don't have to count Down presses.
    app.on_key(Key::Char('/'));
    for c in "xcode".chars() {
        app.on_key(Key::Char(c));
    }
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("Xcode"));
    assert!(s.contains("[large]"), "large badge:\n{s}");
    assert!(s.contains("App Store"), "app store badge");
}

#[test]
fn browse_tab_lists_popular_packages() {
    let mut app = fresh();
    app.on_key(Key::Char('2'));
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("Popular Homebrew"));
    assert!(
        s.contains("htop") || s.contains("neovim"),
        "a popular formula"
    );
}

#[test]
fn theme_tab_shows_hex_swatches() {
    let mut app = fresh();
    app.on_key(Key::Char('3'));
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("live preview"));
    // tokyonight accent hex appears in the swatch labels.
    assert!(s.contains("#bb9af7"), "accent hex:\n{s}");
    assert!(s.contains("cargo build"), "mock prompt line");
}

#[test]
fn options_tab_shows_toggles() {
    let mut app = fresh();
    app.on_key(Key::Char('4'));
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("Tiling desktop configs"));
    assert!(s.contains("[x]") || s.contains("[ ]"), "a checkbox");
}

#[test]
fn save_tab_summarises_and_warns() {
    let mut app = fresh();
    // Select Xcode so the warnings block appears.
    app.on_key(Key::Char('/'));
    for c in "xcode".chars() {
        app.on_key(Key::Char(c));
    }
    app.on_key(Key::Char(' ')); // toggle it on
    app.on_key(Key::Esc); // leave search
    app.on_key(Key::Char('5')); // Save tab
    let s = screen(&mut app, 120, 44);
    assert!(s.contains("Selection summary"));
    assert!(s.contains("Heads up"), "warnings block:\n{s}");
    assert!(s.contains("Theme"));
}

#[test]
fn help_overlay_renders() {
    let mut app = fresh();
    app.on_key(Key::Char('?'));
    let s = screen(&mut app, 120, 40);
    assert!(s.contains("keys"));
    assert!(s.contains("switch tabs"));
}
