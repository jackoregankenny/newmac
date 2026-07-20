//! Integration tests for the newmac core: catalog, search, selection I/O,
//! themes and the Homebrew snapshot. These run anywhere (no macOS needed).

use newmac_core::brew::{self, BrewKind};
use newmac_core::search::{haystack, Searcher};
use newmac_core::selection::{Custom, Selection};
use newmac_core::{theme, Catalog, Flag, Kind};

#[test]
fn embedded_catalog_parses_and_validates() {
    let cat = Catalog::embedded();
    assert!(cat.items.len() >= 100, "expected the full catalog");
    // Every item's category exists (validate() already enforced this).
    for it in &cat.items {
        assert!(
            cat.categories.iter().any(|c| c.id == it.category),
            "{} -> {}",
            it.id,
            it.category
        );
    }
    // core is the only "always" category and is excluded from the picker.
    let selectable: Vec<&str> = cat.selectable_categories().map(|c| c.id.as_str()).collect();
    assert!(!selectable.contains(&"core"));
    assert!(!cat.always_ids().is_empty());
}

#[test]
fn duplicate_ids_are_rejected() {
    let toml = r#"
        [[category]]
        id = "a"
        title = "A"
        [[item]]
        id = "x"
        category = "a"
        kind = "brew"
        payload = "x"
        name = "X"
        [[item]]
        id = "x"
        category = "a"
        kind = "brew"
        payload = "x2"
        name = "X2"
    "#;
    assert!(Catalog::parse(toml).is_err());
}

#[test]
fn unknown_category_is_rejected() {
    let toml = r#"
        [[category]]
        id = "a"
        title = "A"
        [[item]]
        id = "x"
        category = "nope"
        kind = "brew"
        payload = "x"
        name = "X"
    "#;
    assert!(Catalog::parse(toml).is_err());
}

#[test]
fn flags_and_kinds_deserialize() {
    let cat = Catalog::embedded();
    let xcode = cat.get("xcode").expect("xcode present");
    assert_eq!(xcode.kind, Kind::Mas);
    assert!(xcode.flags.contains(&Flag::Appstore));
    assert!(xcode.flags.contains(&Flag::Large));
    let warp = cat.get("warp").unwrap();
    assert!(warp.flags.contains(&Flag::Account));
}

#[test]
fn fuzzy_search_is_forgiving() {
    let cat = Catalog::embedded();
    let mut s = Searcher::new();
    let hs: Vec<(usize, String)> = cat
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| (i, haystack(it)))
        .collect();
    let refs: Vec<(usize, &str)> = hs.iter().map(|(i, h)| (*i, h.as_str())).collect();

    // Typo-ish query still finds docker.
    let ranked = s.rank("dockr", refs.clone());
    let top = &cat.items[ranked[0].0];
    assert!(
        top.id.contains("docker"),
        "expected a docker item on top, got {}",
        top.id
    );

    // Searching by what a tool does (description) surfaces it.
    let ranked = s.rank("password manager", refs.clone());
    assert!(ranked
        .iter()
        .take(3)
        .any(|(i, _)| cat.items[*i].id.starts_with("1password")));

    // Empty query keeps every candidate, original order.
    let ranked = s.rank("", refs.clone());
    assert_eq!(ranked.len(), refs.len());
    assert_eq!(ranked[0].0, 0);
}

#[test]
fn conf_roundtrips_through_bash_shape() {
    let cat = Catalog::embedded();
    let mut sel = Selection::from_defaults(&cat);
    sel.theme = "gruvbox".into();
    sel.toggles.schedule = true;
    sel.add_custom(&Custom {
        name: "htop".into(),
        cask: false,
    });
    sel.add_custom(&Custom {
        name: "google-chrome".into(),
        cask: true,
    });

    let rendered = sel.render_conf("test");
    // The bits bash relies on.
    assert!(rendered.contains("NEWMAC_SELECTED=\" "));
    assert!(rendered.contains(" \"\n") || rendered.contains(" \"\r\n"));
    assert!(rendered.contains("NEWMAC_THEME=gruvbox"));
    assert!(rendered.contains("NEWMAC_TOGGLE_SCHEDULE=1"));
    assert!(rendered.contains("NEWMAC_EXTRA_BREW=\"htop\""));
    assert!(rendered.contains("NEWMAC_EXTRA_CASK=\"google-chrome\""));

    let parsed = Selection::parse_conf(&rendered);
    assert_eq!(parsed.selected, sel.selected);
    assert_eq!(parsed.theme, "gruvbox");
    assert!(parsed.toggles.schedule);
    assert_eq!(parsed.extra_brew, vec!["htop"]);
    assert_eq!(parsed.extra_cask, vec!["google-chrome"]);
}

#[test]
fn selected_membership_uses_space_padding() {
    // The bash test is `case " $NEWMAC_SELECTED " in *" $id "*)` — so a bare
    // id must be surrounded by spaces, and an id that is a prefix of another
    // must not spuriously match.
    let mut sel = Selection::default();
    sel.set("bat", true);
    let rendered = sel.render_conf("t");
    let line = rendered
        .lines()
        .find(|l| l.starts_with("NEWMAC_SELECTED="))
        .unwrap();
    let value = line
        .trim_start_matches("NEWMAC_SELECTED=")
        .trim_matches('"');
    assert!(value.contains(" bat "));
}

#[test]
fn custom_dedup() {
    let mut sel = Selection::default();
    assert!(sel.add_custom(&Custom {
        name: "htop".into(),
        cask: false
    }));
    assert!(!sel.add_custom(&Custom {
        name: "htop".into(),
        cask: false
    }));
    assert_eq!(sel.extra_brew.len(), 1);
    sel.remove_custom("htop", false);
    assert!(sel.extra_brew.is_empty());
}

#[test]
fn themes_load_with_tokyonight_first() {
    let themes = theme::all();
    assert_eq!(themes.len(), 6);
    assert_eq!(themes[0].id, "tokyonight");
    // Hex parsed to RGB (tokyonight accent #bb9af7).
    let tn = &themes[0];
    assert_eq!((tn.accent.r, tn.accent.g, tn.accent.b), (0xbb, 0x9a, 0xf7));
    assert_eq!(tn.swatches().len(), 6);
}

#[test]
fn catalog_loads_from_disk() {
    // The repo-root catalog.toml (canonical) parses via the disk loader and
    // matches the embedded copy item-for-item.
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let from_disk = Catalog::load(&repo_root.join("catalog.toml")).expect("disk catalog");
    let embedded = Catalog::embedded();
    assert_eq!(from_disk.items.len(), embedded.items.len());
    // Bad path falls back to embedded, doesn't panic.
    let fallback = Catalog::from_path_or_embedded(Some(std::path::Path::new("nope.toml")));
    assert_eq!(fallback.items.len(), embedded.items.len());
}

#[test]
fn themes_load_from_config_dir() {
    let themes_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("config")
        .join("themes");
    let from_dir = theme::from_dir_or_embedded(Some(&themes_dir));
    assert_eq!(from_dir.len(), 6);
    assert_eq!(from_dir[0].id, "tokyonight");
    // Parsed hex matches the embedded snapshot.
    assert_eq!(
        from_dir[0].accent,
        theme::all()[0].accent,
        "disk-parsed accent should match embedded"
    );
    // Missing dir falls back to embedded.
    let fallback = theme::from_dir_or_embedded(Some(std::path::Path::new("nope")));
    assert_eq!(fallback.len(), 6);
}

#[test]
fn brew_snapshot_is_usable_offline() {
    let pkgs = brew::bundled();
    assert!(pkgs.len() > 100, "curated snapshot should be sizeable");
    assert!(pkgs
        .iter()
        .any(|p| p.name == "htop" && p.kind == BrewKind::Formula));
    assert!(pkgs
        .iter()
        .any(|p| p.name == "google-chrome" && p.kind == BrewKind::Cask));
    // No dupes of the same (kind, name).
    let mut seen = std::collections::HashSet::new();
    for p in &pkgs {
        assert!(seen.insert((p.kind, p.name.clone())), "dupe: {}", p.name);
    }
}
