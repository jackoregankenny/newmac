//! Drive the picker through `on_key` with no terminal attached, asserting on
//! the resulting state and the conf it writes. This is the Rust equivalent of
//! the `expect`-driven TUI tests the roadmap asks for (#9) — deterministic and
//! CI-friendly. (Mouse coverage lives in `mouse.rs`, which needs a render.)

use newmac_core::{flavour, theme, Catalog, Selection};
use newmac_tui::app::{App, Key, Pane, Screen, StartEntry, Tab};

fn app_with_conf(dir: &std::path::Path) -> App {
    let catalog = Catalog::embedded();
    let sel = Selection::from_defaults(&catalog);
    App::new(catalog, sel, dir.join("newmac.conf"))
}

fn app_start(dir: &std::path::Path, had_conf: bool, sel: Selection) -> App {
    let catalog = Catalog::embedded();
    App::new_full(
        catalog,
        sel,
        dir.join("newmac.conf"),
        theme::all(),
        flavour::all(),
        had_conf,
    )
}

fn feed(app: &mut App, keys: &[Key]) {
    for k in keys {
        app.on_key(*k);
        app.normalize();
    }
}

#[test]
fn toggling_an_item_updates_selection() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    // Move into the items pane, pick a category, toggle the first item.
    app.on_key(Key::Right); // focus items
    let vis = app.visible_items();
    let first = app.catalog.items[vis[0]].id.clone();
    let before = app.sel.is_selected(&first);
    app.on_key(Key::Char(' '));
    assert_ne!(before, app.sel.is_selected(&first));
}

#[test]
fn slash_enters_global_search_and_filters() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    feed(&mut app, &[Key::Char('/')]);
    assert!(app.searching);
    for c in "docker".chars() {
        app.on_key(Key::Char(c));
    }
    app.normalize();
    let vis = app.visible_items();
    assert!(!vis.is_empty());
    // Every visible item should be a fuzzy match; docker itself must be there.
    assert!(vis.iter().any(|&i| app.catalog.items[i].id == "docker"));
    // Esc clears the query.
    app.on_key(Key::Esc);
    assert!(!app.searching);
    assert!(app.query.is_empty());
}

#[test]
fn number_keys_switch_tabs() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    app.on_key(Key::Char('2'));
    assert_eq!(app.tab, Tab::Browse);
    app.on_key(Key::Char('3'));
    assert_eq!(app.tab, Tab::Theme);
    app.on_key(Key::Char('5'));
    assert_eq!(app.tab, Tab::Save);
}

#[test]
fn select_all_and_none_in_a_category() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    // Jump to a concrete category (skip "All"): Down once, focus items.
    app.on_key(Key::Down); // cat_idx = 1 (first real category)
    app.on_key(Key::Char('n')); // clear all shown
    let vis = app.visible_items();
    assert!(vis
        .iter()
        .all(|&i| !app.sel.is_selected(&app.catalog.items[i].id)));
    app.on_key(Key::Char('a')); // select all shown
    assert!(vis
        .iter()
        .all(|&i| app.sel.is_selected(&app.catalog.items[i].id)));
}

#[test]
fn browse_add_and_custom_flow_writes_extras() {
    let dir = std::env::temp_dir().join("newmac_test_browse");
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = app_with_conf(&dir);

    // Go to Browse, add the highlighted popular package.
    app.on_key(Key::Char('2'));
    assert_eq!(app.tab, Tab::Browse);
    let vis = app.visible_brew();
    let first = app.brew[vis[0]].clone();
    app.on_key(Key::Char('a'));
    let in_extras =
        app.sel.extra_brew.contains(&first.name) || app.sel.extra_cask.contains(&first.name);
    assert!(in_extras, "expected {} added to extras", first.name);

    // Add a custom formula via the prompt.
    app.on_key(Key::Char('c'));
    assert!(app.prompt.is_some());
    for ch in "neofetch".chars() {
        app.on_key(Key::Char(ch));
    }
    app.on_key(Key::Enter);
    assert!(app.prompt.is_none());
    assert!(app.sel.extra_brew.contains(&"neofetch".to_string()));

    // Save and read the conf back.
    app.on_key(Key::CtrlS);
    let conf = std::fs::read_to_string(dir.join("newmac.conf")).unwrap();
    assert!(conf.contains("NEWMAC_EXTRA_BREW="));
    assert!(conf.contains("neofetch"));
    let reparsed = Selection::parse_conf(&conf);
    assert!(reparsed.extra_brew.contains(&"neofetch".to_string()));
}

#[test]
fn theme_selection_persists_to_conf() {
    let dir = std::env::temp_dir().join("newmac_test_theme");
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = app_with_conf(&dir);
    app.on_key(Key::Char('3')); // Theme tab
    app.on_key(Key::Down); // move to 2nd theme
    app.on_key(Key::Char(' ')); // apply
    let chosen = app.themes[app.theme_idx].id.clone();
    app.on_key(Key::CtrlS);
    let conf = std::fs::read_to_string(dir.join("newmac.conf")).unwrap();
    assert!(conf.contains(&format!("NEWMAC_THEME={chosen}")));
}

#[test]
fn options_toggle_flips_conf_bits() {
    let dir = std::env::temp_dir().join("newmac_test_opts");
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = app_with_conf(&dir);
    app.on_key(Key::Char('4')); // Options
    let before = app.sel.toggles.ricing;
    app.on_key(Key::Char(' ')); // toggle row 0 (ricing)
    assert_ne!(before, app.sel.toggles.ricing);
}

#[test]
fn help_overlay_swallows_one_key() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    app.on_key(Key::Char('?'));
    assert!(app.show_help);
    app.on_key(Key::Char('j')); // dismisses, does not move
    assert!(!app.show_help);
}

#[test]
fn save_screen_install_keys_save_then_request() {
    let dir = std::env::temp_dir().join("newmac_test_install");
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = app_with_conf(&dir);
    // `i` on the Save screen saves first, then flags an install for the loop.
    app.on_key(Key::Char('5'));
    app.on_key(Key::Char('i'));
    assert!(app.saved, "install should save the conf first");
    assert!(app.install_requested);
    assert!(std::fs::read_to_string(dir.join("newmac.conf")).is_ok());

    app.install_requested = false;
    app.on_key(Key::Char('d')); // dry-run
    assert!(app.dryrun_requested);
}

#[test]
fn start_screen_seeds_jack_and_enters_picker() {
    let tmp = std::env::temp_dir();
    let catalog = Catalog::embedded();
    let mut app = app_start(&tmp, false, Selection::from_defaults(&catalog));
    assert_eq!(app.screen, Screen::Start);
    assert_eq!(
        app.start_idx, 0,
        "Jack's flavour highlighted on a fresh run"
    );

    app.on_key(Key::Enter); // choose Jack's flavour
    assert_eq!(app.screen, Screen::Picker);
    assert_eq!(app.tab, Tab::Packages);
    assert!(app.sel.is_selected("rio"));
    assert!(app.sel.is_selected("aerospace"));
    assert!(!app.sel.is_selected("ghostty"));
    assert_eq!(app.sel.theme, "nord");
    assert!(app.sel.glass);
    assert_eq!(app.themes[app.theme_idx].id, "nord");
}

#[test]
fn start_custom_lands_on_defaults() {
    let tmp = std::env::temp_dir();
    let catalog = Catalog::embedded();
    let mut app = app_start(&tmp, false, Selection::from_defaults(&catalog));
    let custom_idx = app.flavours.len(); // Custom sits right after the flavours
    assert_eq!(app.start_entries()[custom_idx], StartEntry::Custom);
    app.start_idx = custom_idx;
    app.on_key(Key::Enter);
    assert_eq!(app.screen, Screen::Picker);
    // Custom == catalog defaults, so a default-on item like ghostty is selected.
    assert!(app.sel.is_selected("ghostty"));
    assert!(!app.sel.glass);
}

#[test]
fn keep_current_preserves_loaded_selection() {
    let tmp = std::env::temp_dir();
    let mut sel = Selection::default();
    sel.selected.insert("vlc".to_string());
    let mut app = app_start(&tmp, true, sel);
    let entries = app.start_entries();
    assert_eq!(entries.last(), Some(&StartEntry::KeepCurrent));
    assert_eq!(
        app.start_idx,
        entries.len() - 1,
        "Keep current is default with a conf"
    );
    app.on_key(Key::Enter);
    assert_eq!(app.screen, Screen::Picker);
    assert!(app.sel.is_selected("vlc"));
}

#[test]
fn b_returns_to_the_presets_screen() {
    let tmp = std::env::temp_dir();
    let catalog = Catalog::embedded();
    let mut app = app_start(&tmp, false, Selection::from_defaults(&catalog));
    app.on_key(Key::Enter); // into the picker
    assert_eq!(app.screen, Screen::Picker);
    app.on_key(Key::Char('b'));
    assert_eq!(app.screen, Screen::Start);
}

#[test]
fn pane_navigation() {
    let tmp = std::env::temp_dir();
    let mut app = app_with_conf(&tmp);
    assert_eq!(app.pane, Pane::Categories);
    app.on_key(Key::Right);
    assert_eq!(app.pane, Pane::Items);
    app.on_key(Key::Left);
    assert_eq!(app.pane, Pane::Categories);
}
