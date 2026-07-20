//! Generate `scripts/catalog.sh` from the canonical `catalog.toml`.
//!
//! `catalog.toml` is the source of truth; the bash picker/installer still read
//! the old pipe-separated `catalog.sh`. This keeps them in lockstep: edit the
//! TOML, run `newmac-ui catalog gen-sh`, commit both. The generated file is
//! byte-for-byte deterministic so CI can assert it is up to date.

use newmac_core::Catalog;

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
