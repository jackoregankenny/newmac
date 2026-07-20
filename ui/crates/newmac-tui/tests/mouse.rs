//! Mouse tests: render once so `app.ui` geometry is populated, then feed clicks
//! and scrolls at the recorded coordinates and assert the state change.

use newmac_core::{Catalog, Selection};
use newmac_tui::app::{App, Key, Mouse, Pane, Tab};
use newmac_tui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn fresh() -> App {
    let catalog = Catalog::embedded();
    let sel = Selection::from_defaults(&catalog);
    App::new(catalog, sel, std::env::temp_dir().join("newmac.conf"))
}

/// Draw once at a fixed size so `app.ui` rects/spans are filled in.
fn render(app: &mut App) {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    app.normalize();
    terminal.draw(|f| ui::draw(f, app)).unwrap();
}

#[test]
fn click_a_tab_switches_to_it() {
    let mut app = fresh();
    render(&mut app);
    let (x0, x1) = app.ui.tab_spans[1]; // Browse
    let col = (x0 + x1) / 2;
    app.on_mouse(Mouse::Down(col, app.ui.tabs_rect.y));
    assert_eq!(app.tab, Tab::Browse);

    render(&mut app);
    let (x0, x1) = app.ui.tab_spans[3]; // Options
    app.on_mouse(Mouse::Down((x0 + x1) / 2, app.ui.tabs_rect.y));
    assert_eq!(app.tab, Tab::Options);
}

#[test]
fn click_a_category_selects_it() {
    let mut app = fresh();
    render(&mut app);
    let r = app.ui.cat.rect;
    // Third row (index 2) — offset is 0 on first render.
    app.on_mouse(Mouse::Down(r.x + 1, r.y + 2));
    assert_eq!(app.cat_idx, 2);
    assert_eq!(app.pane, Pane::Categories);
}

#[test]
fn click_an_item_toggles_it() {
    let mut app = fresh();
    render(&mut app);
    let vis = app.visible_items();
    let target = app.catalog.items[vis[1]].id.clone();
    let before = app.sel.is_selected(&target);
    let r = app.ui.items.rect;
    app.on_mouse(Mouse::Down(r.x + 2, r.y + 1)); // second visible row
    assert_ne!(before, app.sel.is_selected(&target));
    assert_eq!(app.pane, Pane::Items);
}

#[test]
fn scroll_moves_the_item_cursor() {
    let mut app = fresh();
    render(&mut app);
    let r = app.ui.items.rect;
    let before = app.item_idx;
    app.on_mouse(Mouse::Scroll(true, r.x + 2, r.y + 2)); // wheel down over items
    assert_eq!(app.item_idx, before + 1);
    app.on_mouse(Mouse::Scroll(false, r.x + 2, r.y + 2)); // wheel up
    assert_eq!(app.item_idx, before);
}

#[test]
fn click_an_option_toggles_it() {
    let mut app = fresh();
    app.on_key(Key::Char('4')); // Options tab
    render(&mut app);
    let r = app.ui.options.rect;
    let before = app.sel.toggles.macos_defaults; // second row (index 1)
    app.on_mouse(Mouse::Down(r.x + 1, r.y + 1));
    assert_ne!(before, app.sel.toggles.macos_defaults);
}

#[test]
fn click_a_theme_applies_it() {
    let mut app = fresh();
    app.on_key(Key::Char('3')); // Theme tab
    render(&mut app);
    let r = app.ui.theme.rect;
    app.on_mouse(Mouse::Down(r.x + 1, r.y + 2)); // third theme
    assert_eq!(app.sel.theme, app.themes[2].id);
}

#[test]
fn click_a_preset_on_the_start_screen_selects_it() {
    let catalog = Catalog::embedded();
    let sel = Selection::from_defaults(&catalog);
    let mut app = App::new_full(
        catalog,
        sel,
        std::env::temp_dir().join("newmac.conf"),
        newmac_core::theme::all(),
        newmac_core::flavour::all(),
        false,
    );
    render(&mut app);
    let r = app.ui.start.rect;
    // Click the first row — Jack's flavour.
    app.on_mouse(Mouse::Down(r.x + 2, r.y));
    assert_eq!(app.screen, newmac_tui::app::Screen::Picker);
    assert!(app.sel.is_selected("rio"));
    assert_eq!(app.sel.theme, "nord");
}

#[test]
fn click_a_popular_package_adds_it() {
    let mut app = fresh();
    app.on_key(Key::Char('2')); // Browse
    render(&mut app);
    let idx0 = app.visible_brew()[0];
    let first = app.brew[idx0].clone();
    let r = app.ui.brew.rect;
    app.on_mouse(Mouse::Down(r.x + 2, r.y)); // first row
    let added =
        app.sel.extra_brew.contains(&first.name) || app.sel.extra_cask.contains(&first.name);
    assert!(added, "clicking a popular package should add it");
}
