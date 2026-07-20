# The newmac guide

How to go from a box-fresh Mac to a fully riced, agent-ready machine — and how to
live with it afterwards. (What each tool *is* lives in the [catalog index](CATALOG.md);
this is about how to *use* the thing.)

## 1. Install

On the fresh Mac, one line:

```sh
bash -c "$(curl -fsSL https://raw.githubusercontent.com/jackoregankenny/newmac/main/get.sh)"
```

That installs the Xcode Command Line Tools, clones this repo to `~/newmac`, and starts
the picker. No Homebrew, no git, no prerequisites — the script handles all of it.

Prefer to see what you're running first? Clone and run it yourself:

```sh
git clone https://github.com/jackoregankenny/newmac.git ~/newmac
cd ~/newmac && bash bootstrap.sh
```

Useful variants:

| Command | What it does |
|---|---|
| `bash bootstrap.sh --defaults` | No questions — installs the author's defaults |
| `bash bootstrap.sh --preset webdev` | A ready-made stack (`minimal` / `webdev` / `ai` / `rice` / `default`) |
| `bash bootstrap.sh --reconfigure` | Reopen the picker on a machine that already ran it |
| `bash scripts/install.sh --dry-run` | Print exactly what would be installed, change nothing |

## 2. Your first run — what the picker asks

It's à la carte — no bundle to choose, just tick what you want per category.

1. **Category screens** — Terminals, Shells, AI coding agents, Agent workbenches,
   Menu bar, Tiling, Runtimes, Dev apps, Browsers, Office, VPNs, Maintenance, Fonts…
   Each item starts at a sensible default (● on / ○ off); change any of them.
   Arrow keys move, **space** toggles, **a**/**n** select all/none, **enter** continues.
2. **Theme** — six full-rice colour schemes (see §5). You can switch any time later.
3. **Toggles** — tiling desktop configs, macOS UX defaults, battery/power tuning,
   weekly auto-updates, Dock arrangement.
4. **Summary** — everything you picked, one confirm before anything is written.

On a re-run the boxes pre-fill from your existing `newmac.conf`, so it doubles as an
editor. Prefer to skip the questions? `bash bootstrap.sh --preset webdev` (or
`minimal` / `ai` / `rice`) applies a ready-made selection non-interactively — handy
for the one-liner, never forced on you in the picker.

Selections save to `newmac.conf`. The install then runs: Homebrew packages, vendor
installers (Claude Code, Amp, …), Rust/Node/Python toolchains, config symlinks, git
identity (it prompts if unset), theme application, and — if you enabled them — the
tiling desktop, macOS defaults, power profile, and Dock layout.

**Re-running is always safe.** Everything is idempotent.

## 3. After the install — 10 minutes of manual steps

macOS won't let scripts do these:

- **Accessibility permissions** — System Settings → Privacy & Security →
  Accessibility: enable AeroSpace, AltTab, Karabiner, sketchybar. (Karabiner also
  asks for Input Monitoring.)
- **AltTab** — set its shortcut to `⌥Tab`, "show windows from all Spaces".
- **Sign in** — 1Password (then `op signin`), App Store (if you picked mas apps),
  Cloudflare WARP / Tailscale.
- **Agents** — run each once to authenticate: `claude`, `codex`, `droid`, `amp`…
- **Battery** — System Settings → Battery → Charging → Charge Limit **80%**.
- Restart the terminal (`exec zsh`) so the new shell config loads.

## 4. The desktop — learning the tiling workflow

**New to tiling? Run `newmac tour`** — a paced, 2-minute walkthrough that teaches the
whole model one idea at a time and offers to open the panels it mentions. The ricing
step offers it automatically on first setup. Everything below is the same material as
reference.

The modifier is **⌥ (Option)**; Caps Lock became **⌘ held / Esc tapped**.

**Press `⌥⇧/` any time for the hotkey cheat-sheet popup.** That's the one binding to
memorise; it teaches the rest (`keys` in a terminal does the same).

The mental model:

- **Workspaces 1–5** (`⌥1`–`⌥5`): 1 Agents · 2 Browser · 3 Editor · 4 Comms ·
  5 Spare. Apps auto-assign on launch; `⌥⇧1`–`⌥⇧5` sends a window and follows it;
  `⌥b` bounces to the previous workspace.
- **Windows tile automatically.** `⌥h/j/k/l` moves focus (vim directions), `⌥⇧h/j/k/l`
  moves the window, `⌥-`/`⌥=` resizes, `⌥f` fullscreens.
- **Floating is first-class.** System Settings, Zoom, FaceTime, Calculator, 1Password
  and Calendar float automatically. `⌥⇧space` floats/re-tiles anything else. Tiling
  for deep work; floating for meetings and quick panels — both are the intended use.
- Something looks wrong? `⌥⇧;` then `r` resets the layout; `esc` reloads the config.
- `⌥Tab` is the AltTab window switcher (all windows, all Spaces).

## 5. Themes — deciding and switching

See them all with live colour swatches:

```sh
newmac theme            # lists all six with palette swatches + which is active
newmac theme kanagawa   # applies one everywhere, live
```

One command re-themes both terminals (Ghostty *and* Rio), the bar, the window
borders, and the prompt together. The current lineup:

| Theme | Feel | Shape |
|---|---|---|
| `tokyonight` | The modern default — cool blues, purple accent | rounded |
| `kanagawa` | Muted, earthy, ink-wash — the r/unixporn darling | **sharp corners** |
| `rosepine` | Soft, warm, low-contrast | rounded |
| `nord` | Cold, calm, blue-grey | rounded |
| `gruvbox` | Retro warm, high character | rounded |
| `catppuccin` | Pastel, widely themed everywhere | rounded |

Themes are ~20-line palette files in `config/themes/` — copy one, tweak the ten
colours (plus optional `T_RADIUS=0` / `T_BORDER_STYLE=square` for the sharp look),
and it appears in `newmac theme` and the picker automatically.

## 6. Daily driving — the `newmac` command

```sh
newmac status          # every tool + version, green ● / red ○
newmac update          # update brew, casks, runtimes, and all the agents
newmac configure       # change what's selected (picker pre-fills from your conf)
newmac install         # install anything newly selected
newmac theme <id>      # switch theme
newmac dock            # re-arrange the Dock to match the selection
newmac keys            # hotkey cheat-sheet in the terminal
newmac doctor          # check the PATH + that every tool resolves in a fresh shell
newmac doctor --fix    # repair the PATH automatically (idempotent)
```

`doctor` probes the PATH **a brand-new terminal would get** (via a login zsh), not
the current one — so it catches the classic "works in this tab, broken in the next"
problem. Bootstrap runs `doctor --fix` automatically at the end of every run.

Weekly auto-update (if you enabled the toggle) runs Mondays 10:00 and logs to
`~/Library/Logs/newmac-update.log`.

## 7. Removing things — and the test loop

newmac tracks **what it actually installed** (never what was already there) in a
manifest, which makes clean removal possible:

```sh
newmac list                    # everything newmac installed on this machine
newmac remove crush qwen       # uninstall specific tools, same way they went in
newmac remove --unselected     # prune whatever you deselected in the picker
newmac nuke                    # bulk-remove EVERYTHING newmac installed (confirms first)
newmac nuke --dry-run          # preview that
```

Testing the tool itself? The loop is: `bash bootstrap.sh` → poke around →
`newmac nuke` → tweak → repeat.

## 8. Customising

- **Add a tool for everyone**: one line in `scripts/catalog.sh`
  (`id|category|kind|default|payload|name|description` — no apostrophes in the text,
  the catalog is one single-quoted string). Run `bash scripts/docs.sh` and commit the
  regenerated catalog index; CI fails if you forget.
- **Add a preset**: three case entries in `scripts/presets.sh`.
- **Add a theme**: one palette file in `config/themes/`.
- **Machine-local shell tweaks**: `~/.zshrc.local` (git-ignored, auto-sourced).
- **Different WM philosophy**: the picker offers yabai (max power, SIP disable, pair
  with skhd), Amethyst (auto-tiling, native feel), or Rectangle/Loop (plain snapping)
  — pick one, not several.

## 9. When something goes wrong

| Symptom | Fix |
|---|---|
| A brew line failed mid-install | The run continues and reports at the end. `brew search <name>`, fix `scripts/catalog.sh`, re-run `newmac install`. |
| Ghostty looks unthemed | The bundled theme name may differ on your version: `ghostty +list-themes`, then edit `THEME_GHOSTTY` in `config/themes/<id>.sh` and re-apply. |
| App Store items failed | Sign in to the App Store first; `newmac install` again. |
| An app tiles when it shouldn't | `⌥⇧space` floats it now; make it permanent with an `on-window-detected` rule in `config/aerospace/aerospace.toml` (get its id via `aerospace list-apps`). |
| Wrong app on the wrong workspace | Same file, same command — the bundle ids shipped are best guesses. |
| Bar/borders didn't restyle | `bash scripts/theme.sh <id>` again — it reloads both; check `brew services list`. |
| Keybindings dead after login | AeroSpace probably lacks Accessibility permission (§3). |
| `newmac` command not found | `exec zsh`, or check `~/.local/bin` is on PATH; bootstrap re-creates the launcher. |
| A tool installed but the shell cannot find it | `newmac doctor` — it names the missing PATH line; `newmac doctor --fix` adds it. |
