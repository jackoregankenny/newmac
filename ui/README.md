# newmac-ui — the Rust picker

A single compiled binary that replaces the bash checkbox picker with a
searchable, filterable [ratatui](https://ratatui.rs) TUI. It writes the same
`newmac.conf` the bash `install.sh` already reads — so it's a **UI swap, not a
rewrite** (ROADMAP #11, phase 1). The install logic stays in bash.

```
┌ newmac — your Mac, your way ─────────────────────────────────────────────┐
│  1·Packages   2·Browse Homebrew   3·Theme   4·Options   5·Save            │
└──────────────────────────────────────────────────────────────────────────┘
┌ Categories ──────────┐┌ All packages ──────────────────────────────────┐
│All  44/119           ││● Ghostty      cask   Fast native GPU terminal   │
│Terminals  2/7        ││○ Warp         cask  [account]  AI-native term…  │
│AI coding agents 4/13 ││● Zellij       brew   Modern multiplexer…        │
└──────────────────────┘└─────────────────────────────────────────────────┘
```

## What it does

- **Fuzzy search / filter** (`/`) across all 130+ items — by name, id, or what
  a tool *does* (`password`, `vpn`, `diff`). (ROADMAP #3)
- **Warning badges** — `paid`, `account`, `large`, `App Store` render inline and
  are summarised on the Save screen so there are no mid-install surprises.
  (ROADMAP #4)
- **Browse popular Homebrew** — a bundled snapshot of ~160 popular
  formulae/casks that works offline on a fresh Mac, with a live `r`efresh from
  `formulae.brew.sh` when you have network. Add anything with `a`.
- **Add custom Homebrew** — `c` (formula) / `C` (cask) to type any package.
  These are written to `NEWMAC_EXTRA_BREW` / `NEWMAC_EXTRA_CASK` in the conf and
  installed by the bash `install.sh`.
- **Live theme preview** — colour swatches + a mock prompt line for each of the
  6 themes, applied to your conf on `space`.
- **Options** — the five toggles (tiling, macOS defaults, power, weekly updates,
  Dock).
- **Mouse** — click a tab, click a row to toggle it, click a category/theme, and
  scroll the wheel over any list. Keyboard still does everything.
- **Install from the picker** — on the Save screen, `i` installs (or `d` for a
  dry-run): it drops out of the TUI, runs `scripts/install.sh` with brew's own
  live output, then returns with the result. (ROADMAP #5)

## Keys

| Key | Action |
|---|---|
| `1`–`5`, `[` `]`, `Tab` | switch tabs |
| `↑`/`↓`, `j`/`k` | move |
| `←`/`→`, `h`/`l` | Packages: switch pane |
| `space` / `enter` | toggle highlighted item |
| `/` | fuzzy search (global) |
| `a` / `n` | select / clear all shown |
| `c` / `C` (Browse) | add custom formula / cask |
| `r` (Browse) | live-refresh popular list |
| `s` / `i` / `d` (Save) | save · install now · dry-run |
| `^S` | save from anywhere |
| mouse | click tabs/rows/categories, scroll lists |
| `?` | help · `q` quit |

## Architecture

```
ui/
├── catalog.toml is at repo root — the CANONICAL catalog (single source of truth)
├── crates/
│   ├── newmac-core/     model, fuzzy search, conf I/O, Homebrew data, themes
│   │   ├── catalog.toml        embedded via include_str! (repo root)
│   │   ├── themes.toml         generated from ../../config/themes/*.sh
│   │   └── brew_popular.json   bundled offline snapshot
│   └── newmac-tui/      the ratatui binary `newmac-ui` (+ lib for tests)
```

`catalog.toml` at the repo root is canonical. The bash `scripts/catalog.sh` is
**generated** from it and CI fails if the two drift:

```sh
cargo run -- catalog gen-sh --out ../scripts/catalog.sh
```

## Develop

Needs a Rust toolchain (it's in the catalog by default). With [`just`](https://just.systems):

```sh
cd ui
just test          # all unit + logic + render tests
just lint          # fmt --check + clippy -D warnings
just build         # release binary
just gen-sh        # regenerate scripts/catalog.sh from catalog.toml
just check-catalog # fail if catalog.sh is stale
just run           # open the picker against /tmp/newmac.demo.conf
```

Eyeball a screen without a terminal:

```sh
cargo run --example preview -- packages    # browse | theme | options | save | search
```

### Tests

- `newmac-core/tests/core.rs` — catalog parse/validate, fuzzy search, conf
  round-trip (exact bash shape), themes, Homebrew snapshot.
- `newmac-tui/tests/app_logic.rs` — drives the picker through `on_key` with no
  terminal and asserts the conf it writes (the Rust take on ROADMAP #9).
- `newmac-tui/tests/render.rs` — renders each tab to a headless `TestBackend`
  and asserts the important text reaches the screen.

## Install it

```sh
newmac ui              # download the prebuilt macOS binary → ~/.local/bin/newmac-ui
newmac ui --build      # or compile from source (needs cargo)
newmac configure       # opens the Rust picker (bash is the fallback)
```

`bootstrap.sh` already runs `get-ui.sh` before the picker, so on a fresh Mac the
Rust UI is the **first-run** experience — no cargo required. It's passed
`--catalog`/`--themes-dir` pointing at the clone, so a prebuilt binary always
reflects *this* checkout's catalog and themes (no embedded-vs-clone drift).

`newmac configure --defaults` and `--preset <name>` stay on bash (headless /
preset paths).

## Cutting a release

Prebuilt binaries come from [`.github/workflows/release.yml`](../.github/workflows/release.yml):
tag `v*` → build a **universal** (arm64 + x86_64) macOS binary, ad-hoc sign it
(no Apple Developer cert needed for a CLI), and publish it + a SHA-256 to the
GitHub Release. `scripts/get-ui.sh` downloads
`newmac-ui-macos-universal.tar.gz` from `releases/latest` and verifies the
checksum.

```sh
# from the repo root, once the tree is committed:
git tag v0.1.0 && git push origin v0.1.0
# → the workflow publishes the binary; `newmac ui` and bootstrap can now fetch it.
```

A Homebrew tap (`brew install …/newmac-ui`) is a later add-on.
