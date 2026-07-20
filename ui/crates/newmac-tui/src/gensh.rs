//! Generate `scripts/catalog.sh` (from `catalog.toml`) and `scripts/presets.sh`
//! (from `flavours/*.toml`).
//!
//! The TOML files are the source of truth; the bash picker/installer still read
//! the generated `.sh`. This keeps them in lockstep: edit the TOML, run
//! `newmac-ui catalog gen-sh`, commit both. Output is byte-for-byte
//! deterministic so CI can assert it is up to date.

use newmac_core::{Catalog, Flavour};

fn b(v: bool) -> u8 {
    v as u8
}

/// Render `scripts/presets.sh` from the flavours (plus the special `default`
/// = catalog on/off defaults). Keeps the same function surface bash already
/// uses (`newmac_preset_{title,desc,ids,toggles}`) and adds `_theme`/`_glass`.
pub fn render_presets(flavours: &[Flavour]) -> String {
    let mut out = String::new();
    out.push_str(PRESETS_HEADER);

    // `default` first, then the flavours in Presets-screen order.
    let mut names = String::from("default");
    for f in flavours {
        names.push(' ');
        names.push_str(&f.id);
    }
    out.push_str(&format!("NEWMAC_PRESETS=\"{names}\"\n\n"));

    // title
    out.push_str("newmac_preset_title() {\n  case \"$1\" in\n");
    out.push_str("    default) echo \"Balanced\" ;;\n");
    for f in flavours {
        out.push_str(&format!("    {}) echo \"{}\" ;;\n", f.id, f.title));
    }
    out.push_str("    *) echo \"$1\" ;;\n  esac\n}\n\n");

    // desc
    out.push_str("newmac_preset_desc() {\n  case \"$1\" in\n");
    out.push_str(
        "    default) echo \"The catalog defaults — every category at its sensible default\" ;;\n",
    );
    for f in flavours {
        out.push_str(&format!("    {}) echo \"{}\" ;;\n", f.id, f.desc));
    }
    out.push_str("    *) echo \"\" ;;\n  esac\n}\n\n");

    // ids (default = empty -> catalog defaults)
    out.push_str("newmac_preset_ids() {\n  case \"$1\" in\n");
    for f in flavours {
        out.push_str(&format!("    {}) echo \"{}\" ;;\n", f.id, f.ids.join(" ")));
    }
    out.push_str("    *) echo \"\" ;;\n  esac\n}\n\n");

    // toggles: "ricing macos power schedule dock"
    out.push_str("newmac_preset_toggles() {\n  case \"$1\" in\n");
    for f in flavours {
        let t = f.toggles();
        out.push_str(&format!(
            "    {}) echo \"{} {} {} {} {}\" ;;\n",
            f.id,
            b(t.ricing),
            b(t.macos_defaults),
            b(t.power),
            b(t.schedule),
            b(t.dock)
        ));
    }
    out.push_str("    *) echo \"1 1 1 0 1\" ;;\n  esac\n}\n\n");

    // theme ("" = keep the picker/conf default)
    out.push_str("newmac_preset_theme() {\n  case \"$1\" in\n");
    for f in flavours {
        out.push_str(&format!("    {}) echo \"{}\" ;;\n", f.id, f.theme));
    }
    out.push_str("    *) echo \"\" ;;\n  esac\n}\n\n");

    // glass (0/1)
    out.push_str("newmac_preset_glass() {\n  case \"$1\" in\n");
    for f in flavours {
        out.push_str(&format!("    {}) echo \"{}\" ;;\n", f.id, b(f.glass)));
    }
    out.push_str("    *) echo \"0\" ;;\n  esac\n}\n");

    out
}

const PRESETS_HEADER: &str = r#"#!/usr/bin/env bash
# ============================================================
#  presets.sh — GENERATED from flavours/*.toml. Do not edit by hand.
#      Regenerate with:  newmac-ui catalog gen-sh
#  A preset pre-selects catalog ids + theme/glass/toggles; the picker still
#  opens so you can tweak. Add your own by dropping a flavours/<id>.toml
#  (see CONTRIBUTING.md). Keep this file bash-3.2 compatible.
# ============================================================

"#;

/// Render the full `scripts/catalog.sh` for a catalog.
pub fn render(catalog: &Catalog) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push_str("\nNEWMAC_CATALOG='\n");

    for cat in &catalog.categories {
        let items: Vec<_> = catalog.items_in(&cat.id).collect();
        if items.is_empty() {
            continue;
        }
        let tag = if cat.always {
            format!("{} (always installed)", cat.title)
        } else {
            cat.title.clone()
        };
        out.push_str(&format!("# --- {} ", tag));
        // pad the section rule to a consistent width
        let dashes = 62usize.saturating_sub(tag.len() + 6);
        out.push_str(&"-".repeat(dashes.max(3)));
        out.push('\n');
        for it in items {
            let def = if it.default { "on" } else { "off" };
            out.push_str(&format!(
                "{}|{}|{}|{}|{}|{}|{}\n",
                it.id,
                it.category,
                kind_str(it.kind),
                def,
                it.payload,
                it.name,
                it.description
            ));
        }
        out.push('\n');
    }
    out.push_str("'\n\n");

    // NEWMAC_CATEGORIES excludes the always-installed (core) ones.
    let cats: Vec<&str> = catalog
        .selectable_categories()
        .map(|c| c.id.as_str())
        .collect();
    out.push_str("# Category ids (display order) + human titles.\n");
    out.push_str(&format!("NEWMAC_CATEGORIES=\"{}\"\n", cats.join(" ")));
    out.push_str("newmac_category_title() {\n  case \"$1\" in\n");
    for cat in catalog.selectable_categories() {
        out.push_str(&format!("    {})    echo \"{}\" ;;\n", cat.id, cat.title));
    }
    out.push_str("    *)            echo \"$1\" ;;\n  esac\n}\n");
    out.push_str(FOOTER);
    out
}

fn kind_str(k: newmac_core::Kind) -> &'static str {
    use newmac_core::Kind::*;
    match k {
        Brew => "brew",
        Cask => "cask",
        Curl => "curl",
        Npm => "npm",
        Uv => "uv",
        Mas => "mas",
        Rustup => "rustup",
    }
}

const HEADER: &str = r#"#!/usr/bin/env bash
# ============================================================
#  catalog.sh — GENERATED from catalog.toml. Do not edit by hand.
#      Regenerate with:  newmac-ui catalog gen-sh
#  Format (pipe-separated): id|category|kind|default|payload|name|description
#  Keep this file bash-3.2 compatible. No apostrophes in the catalog string.
# ============================================================
"#;

const FOOTER: &str = r#"
# --- Parse the catalog into parallel arrays --------------------
CAT_ID=(); CAT_CATEGORY=(); CAT_KIND=(); CAT_DEFAULT=(); CAT_PAYLOAD=(); CAT_NAME=(); CAT_DESC=()
_newmac_parse_catalog() {
  local id cat kind def payload name desc n=0
  while IFS='|' read -r id cat kind def payload name desc; do
    [[ -z "$id" ]] && continue
    case "$id" in \#*) continue ;; esac
    CAT_ID[n]="$id"; CAT_CATEGORY[n]="$cat"; CAT_KIND[n]="$kind"
    CAT_DEFAULT[n]="$def"; CAT_PAYLOAD[n]="$payload"; CAT_NAME[n]="$name"; CAT_DESC[n]="$desc"
    n=$((n+1))
  done <<EOF
$NEWMAC_CATALOG
EOF
}
_newmac_parse_catalog
"#;
