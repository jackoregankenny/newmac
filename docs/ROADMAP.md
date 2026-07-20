# Things that would make newmac better

Honest, prioritised. Each is a real gap, not filler — problem, why it matters, and a
sketch of the fix. (#1–10 are the core list; #11 — a compiled Rust UI — is the
newest idea and arguably reframes the whole project.)

> **Status (2026-07):** #11 phase 1 shipped — a Rust ratatui picker
> (`ui/`, binary `newmac-ui`) with fuzzy search (**#3 ✅**), warning badges
> (**#4 ✅**), a live theme preview, and a *browse / add popular Homebrew*
> screen. It writes the same `newmac.conf` bash consumes, and the catalog is now
> a canonical `catalog.toml` with `scripts/catalog.sh` generated from it. The
> riskiest surface (**#9**) now has automated coverage on the Rust side
> (`on_key`-driven logic tests, headless render tests, and mouse hit-testing).
> The picker has **full mouse support** (click tabs/rows/categories, scroll) and
> can **install from the Save screen** (`i` / `d`) — it suspends, runs
> `install.sh` with live output, and returns (**#5**, the in-TUI half).
> **Distribution is wired**: `release.yml` builds a universal, ad-hoc-signed
> macOS binary on tag and `bootstrap.sh`/`get-ui.sh` download it, so the Rust
> picker is the first-run experience on a fresh Mac (pending the first
> `git tag v*`). Still open on the bash side: **#1** real macOS install CI,
> **#2** lockfile, **#5** *resumable/transactional* installs, **#7** `unrice`,
> **#8** Neovim starter, **#10** deeper doctor.

## 1. Real macOS CI, not just syntax-on-Ubuntu

**Now:** CI runs `bash -n`, ShellCheck, and `--dry-run` on `ubuntu-latest`. Nothing has
ever actually installed a package, applied a theme for real, or run twice to prove
idempotency. The whole thing has only been *executed* on your Mac, by hand.

**Fix:** add a `macos-latest` job that installs a small real subset (say `jq`, `eza`,
one cask, one font), runs `theme.sh` against a real Ghostty/Rio config, runs bootstrap
**twice** and asserts the second run is a no-op, then `newmac nuke`s it. Catches
cask-name drift, Ghostty theme-name mismatches, and idempotency regressions before you
hit them.

## 2. Reproducibility — pin what a run installs

**Now:** the Brewfile is generated fresh each run, so two people (or you, months apart)
get whatever `brew` serves that day. No way to reproduce a known-good machine.

**Fix:** after a successful run, `brew bundle dump` a `newmac.lock` next to the conf;
offer `newmac install --locked` to reproduce exact versions. Optional, but it turns
"works on my machine" into "works on any machine."

## 3. Search / filter in the picker

**Now:** 150+ items across 18 categories. Finding one tool means scrolling every screen.

**Fix:** a `/` key in `tui.sh` that filters the current category (or a global "jump to
tool" screen). Small addition to the TUI loop; huge quality-of-life win as the catalog
grows.

## 4. Warning badges in the picker

**Now:** you can tick **Xcode** (~40 GB), **Warp** (needs an account), or an App Store
app (needs sign-in) with no hint about the cost. You find out mid-install.

**Fix:** an optional 8th catalog field for flags (`paid`, `account`, `large`,
`appstore`), rendered as a dim tag in the picker and summarised on the confirm screen.
No surprises.

## 5. Resumable, transactional installs

**Now:** if a run dies halfway (network drop, one bad cask), you re-run the whole thing.
The manifest tracks what landed, but there's no "continue where it stopped."

**Fix:** write progress to the manifest as each item completes; on re-run, skip what's
already done and offer to retry only the failures. Turns a 40-minute redo into a
10-second resume.

## 6. Back up / sync the selection

**Now:** `newmac.conf` is local and git-ignored. If the Mac dies, your exact selection
dies with it — the thing that makes this *yours* is the least durable part.

**Fix:** `newmac export` / `newmac import` (a portable conf blob), and optionally push it
to a private gist or a `profiles/` branch. Restore your whole setup on a new machine in
one command.

## 7. `newmac unrice` — a clean bail-out

**Now:** the tiling desktop is a one-way door in practice. Someone who tries it and
bounces has to manually unwind AeroSpace, SketchyBar, borders, and the Karabiner remap.

**Fix:** a script that stops the services, unlinks the configs (restoring the `.backup.*`
files bootstrap already makes), and reverts the Caps-Lock remap — the inverse of
`ricing.sh`. Lowers the risk of trying the rice at all.

## 8. Curated Neovim + shell starter configs

**Now:** we install Neovim and punt ("add LazyVim yourself"). The catalog's own origin
story is "unorganised dotfiles I'll clean up someday" — and we reproduce it.

**Fix:** ship an opt-in, theme-matched Neovim starter (LazyVim or a lean custom set) and
wire it to the active newmac theme, the same way the terminals are. Finish the editor
story instead of handing it back to the user.

## 9. Automated TUI input tests

**Now:** `tui.sh` is the riskiest surface — arrow keys, scrolling, toggle state — and it
has **zero** automated tests. Every change is verified by eye on one machine.

**Fix:** drive it with `expect` (or piped escape sequences) in CI: send ↓↓ space a
enter, assert the resulting `newmac.conf`. Locks in the picker's behaviour so refactors
can't silently break navigation.

## 10. A deeper `newmac doctor`

**Now:** doctor verifies the PATH and that binaries resolve — genuinely useful, but it
stops there. The most common *real* breakage is elsewhere: Accessibility not granted
(so AeroSpace/AltTab silently do nothing), a brew service not running, fonts not
registered, an agent never authenticated.

**Fix:** extend doctor to check Accessibility grants (scriptable via `sqlite3` on the
TCC db or a permissions probe), `brew services` state for sketchybar/borders, font
registration, and agent auth status — with the one-line fix for each. Make it the single
"is my machine healthy?" command.

## 11. Ship it as a compiled Rust app — TUI first, native GUI later

**Now:** the UI is a pure-bash checkbox picker (`tui.sh`). It works and it's readable,
but it's the ceiling of what bash can do — no mouse, no search, no progress bars, no
live theme preview, and every change is verified by eye (see #9).

**Fix (phased):**

- **Phase 1 — Rust TUI with [ratatui](https://ratatui.rs).** A single compiled binary
  that keeps the whole ethos: curl-install, terminal-native, SSH-able. Immediate wins
  over the bash picker: fuzzy search across all 150+ items, mouse support, live
  colour-swatch theme previews, and a real progress view during install. The install
  logic still shells out to `brew`/`curl`/`uv`, so it's a UI swap, not a rewrite of what
  works. Distribute via a Homebrew tap + `cargo dist` release binaries.
- **Phase 2 — native GUI, if wanted.** For an actual window: **Iced** (pure Rust,
  Elm-architecture, GPU-native, no webview — the polished "serious app" pick) or
  **Tauri v2** (Rust core + an HTML/CSS/JS frontend in the system WebView — lets us
  reuse the landing page's aesthetic directly). **egui** is the fastest way to
  prototype; **Slint**/**GPUI** are gorgeous but heavier bets (licensing / unstable API).

**Enabler:** promote the catalog to a `catalog.toml` read by a small Rust **core crate**
that both a CLI and the UI consume. That decouples the "which UI" decision from the
install logic — the bash CLI and a Rust UI could even coexist during the transition.

**Tradeoffs, honestly:** you lose "just readable bash you can inspect before running,"
and take on a build/release/**code-signing + macOS notarization** pipeline (real work
for a distributed `.app`). The bash tool ships today — treat this as an evolution, and
do Phase 1 before committing to Phase 2.

**Rust UI options at a glance:**

| Crate | Kind | Feel | Notes |
|---|---|---|---|
| **ratatui** | TUI | terminal | The standard. Single binary, SSH-friendly — best fit for phase 1. |
| **Iced** | GUI | native | Pure Rust, GPU, Elm-style. Powers COSMIC. Polished. |
| **Tauri v2** | GUI | web-in-native | Tiny binaries; reuse HTML/CSS frontend; mature. |
| **egui/eframe** | GUI | immediate-mode | Easiest to prototype; less native-looking. |
| **Slint** | GUI | native | Declarative markup, very polished; GPL/commercial. |
| **GPUI** | GUI | native (macOS) | Zed's framework, gorgeous, but no stable public API yet. |
| **cacao / objc2** | GUI | true native | AppKit bindings — most native, most work. |

---

### Runners-up (didn't make the 10)

- **Named profiles** (work vs personal Mac) layered over one conf.
- **`newmac update` changelog** — show version before/after, not just "done."
- **macOS-version awareness** — some `defaults`/casks differ on Tahoe vs Sequoia.
- **CONTRIBUTING.md + issue templates** — make the one-line catalog PR obvious to others.
- **`--purge` uninstall** — also remove leftover `~/.claude`, `~/.factory`, etc. config dirs.
