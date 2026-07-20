# Contributing

## Add your own flavour (one-file PR)

A **flavour** is a named premade setup — a seed of catalog ids + theme + toggles
— that shows up on the picker's **Presets** screen (and as `newmac
--preset <id>` on the CLI). Contributing one is a single new file: no shared
list to merge, no code.

1. Copy [`flavours/jack.toml`](flavours/jack.toml) to `flavours/<your-id>.toml`.
2. Edit the fields:

   ```toml
   id = "myflavour"                 # unique, kebab-case; matches the filename
   title = "My flavour"             # shown on the Presets screen
   desc = "one line about it"
   theme = "nord"                   # a theme id from config/themes/*.sh
   glass = false                    # glassy terminal (transparency + blur)
   ricing = true                    # apply the AeroSpace tiling configs
   macos_defaults = true            # opinionated macOS UX defaults
   power = true                     # battery/power tuning
   schedule = false                 # weekly auto-updates
   dock = true                      # arrange the Dock to match
   ids = ["rio", "zellij", "claude", "..."]   # catalog ids to pre-select
   ```

   Every id in `ids` must exist in [`catalog.toml`](catalog.toml) — the tests
   and CI check this. `core` tools are always installed, so don't list them.

3. Regenerate the bash side and run the checks:

   ```sh
   cd ui
   cargo run -- catalog gen-sh --out-dir ../scripts   # updates scripts/{catalog,presets}.sh
   cargo test --workspace
   cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
   ```

4. Commit both your `flavours/<id>.toml` **and** the regenerated
   `scripts/presets.sh`, then open a PR. It'll appear on the Presets screen
   automatically.

## Add a tool to the catalog

Add one `[[item]]` block to [`catalog.toml`](catalog.toml) (id, category, kind,
default, payload, name, description, optional `flags`), then run the same
`gen-sh` + tests above and commit `catalog.toml` + `scripts/catalog.sh`.
`catalog.toml` is the single source of truth; `scripts/catalog.sh` is generated
from it and CI fails if they drift.

## Working on the Rust picker

See [`ui/README.md`](ui/README.md) for the architecture, `just` recipes, and how
releases are cut. In short: `cd ui && just ci` runs everything CI does.
