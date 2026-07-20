# Jack's flavour on Nord — what a working install looks like

This is the reference for a correctly set-up **Jack's flavour** with the **Nord**
theme: what should be installed, what should be running, and how the desktop
should look and behave. Use it to spot what's missing on a half-finished install.

> **The #1 cause of "the rice didn't work" is that the apps never installed.**
> `ricing.sh` only *links configs and starts services* — it assumes Homebrew
> already installed AeroSpace, sketchybar, borders, AltTab. If `brew bundle`
> didn't run, the configs are in place but there's nothing to run them. See
> **Recovery** at the bottom.

---

## 1. What should be installed

**Rice apps (Homebrew casks/formulae):**

| Tool | Kind | What it does | Check |
|---|---|---|---|
| AeroSpace | cask | i3-style tiling window manager | `aerospace --version` |
| SketchyBar | brew (service) | the top menu bar | `brew services list \| grep sketchybar` → `started` |
| JankyBorders (`borders`) | brew (service) | outline around the focused window | `brew services list \| grep borders` → `started` |
| AltTab | cask | Windows-style ⌥-Tab window switcher | app in `/Applications` |
| Raycast | cask | launcher (⌘-Space) + clipboard history | app in `/Applications` |
| Stats | cask | menu-bar system monitor | app in `/Applications` |
| MonitorControl | cask | external-display brightness/volume | app in `/Applications` |

**Terminal & shell:** Rio, Zellij, plus the core CLI (starship, zoxide, fzf, eza,
bat, ripgrep, fd, delta, lazygit, atuin, direnv, …).

**Fonts:** JetBrains Mono Nerd Font + Symbols Nerd Font (the bar and terminal
need the Nerd Font glyphs, or icons render as tofu boxes).

Quick sanity check that the rice apps landed:
```sh
for a in aerospace sketchybar borders; do command -v $a >/dev/null && echo "✓ $a" || echo "✗ $a MISSING"; done
ls -d /Applications/AeroSpace.app /Applications/AltTab.app 2>/dev/null
```

---

## 2. The terminal — Rio, Nord, glassy

- **Rio**, ~**94% opacity with background blur** — the "glass" look: you can see
  the wallpaper faintly through it, with Nord's blue-grey tint.
- Font **JetBrains Mono Nerd Font, 13pt**; block cursor; 8px padding.
- **Nord colours** (`theme = "newmac"`, written to
  `~/.config/rio/themes/newmac.toml`): background `#2e3440` (nord0), text
  `#eceff4`, accent/cyan `#88c0d0` (frost).
- **Starship prompt**: a clean two-line prompt — directory in Nord blue, git
  branch/status, language versions, a colored `❯`.
- Inside Rio run **Zellij** (multiplexer): a thin status bar at the bottom
  showing the mode + tabs. Keys are **Ctrl-prefixed** (`Ctrl p` pane, `Ctrl t`
  tab, `Ctrl n` resize, …) because AeroSpace owns the ⌥ keys.

Typing `z <dir>` jumps around (zoxide); `Ctrl-r` opens atuin history search;
`lazygit` opens the git TUI.

---

## 3. The menu bar — SketchyBar

A **36px translucent bar** at the top with a 30px blur (so it's glassy over the
wallpaper), Nord-tinted. Layout:

```
┌───────────────────────────────────────────────────────────────────────────┐
│  1  2  3  4  5    Rio                                    14:32   󰥔  85% 󰁹    │
└───────────────────────────────────────────────────────────────────────────┘
   └ workspaces ┘   └ focused app ┘                       └ clock ┘ └ battery ┘
```

- **Left:** workspace indicators **1–5**. The **focused workspace is
  highlighted** in Nord frost (`#88c0d0`); click a number to jump there.
- **Left, after them:** the **focused app's name** in the accent colour.
- **Right:** a **clock** (HH:MM) and a **battery** percentage + glyph.
- Icons use JetBrains Mono / Symbols Nerd Font — if you see boxes, the Nerd
  Fonts didn't install.

Stats adds its own CPU/GPU/RAM readouts to the native menu bar (separate from
SketchyBar).

---

## 4. The desktop — AeroSpace tiling

- **Windows tile automatically** — no overlapping. Open two terminals and they
  split the screen; a third splits again. **8px gaps** between windows and to the
  screen edge (the Linux-WM look).
- **The focused window has a border**: a **6px rounded Nord-frost (`#88c0d0`)
  outline** (JankyBorders). Unfocused windows get a dim `#3b4252` outline.
- **Five workspaces**, switched with `⌥1`–`⌥5`:
  **1 Agents · 2 Browser · 3 Editor · 4 Comms · 5 Spare**.
- **Apps auto-assign** on launch: Rio → 1, Dia → 2, VS Code → 3, Slack → 4.
- **Some apps float** (never tiled): System Settings, Calculator, FaceTime, Zoom,
  1Password, Calendar, Activity Monitor. `⌥⇧space` floats/re-tiles anything else.
- **Caps Lock acts as ⌘** (set in System Settings → Keyboard → Modifier Keys).
  The AeroSpace modifier is **⌥ (Option)**.

### Keys (⌥ = Option) — `⌥⇧/` shows this popup any time
```
⌥1…⌥5      switch workspace         ⌥h/j/k/l    focus window (vim dirs)
⌥⇧1…⌥⇧5    send window + follow      ⌥⇧h/j/k/l   move window
⌥b         previous workspace        ⌥- / ⌥=     resize
⌥Tab       AltTab window switcher    ⌥f          fullscreen
⌥/         toggle split              ⌥,          accordion
⌥⇧space    float / un-float          ⌥⇧;         service mode (r reset · esc reload)
```

---

## 5. Nord palette (reference)

| Role | Hex | Nord |
|---|---|---|
| Background | `#2e3440` | nord0 |
| Surface | `#3b4252` | nord1 |
| Text | `#eceff4` | nord6 |
| Subtext | `#d8dee9` | nord4 |
| Accent (frost) | `#88c0d0` | nord8 |
| Accent 2 | `#81a1c1` | nord9 |
| Red | `#bf616a` | nord11 |
| Green | `#a3be8c` | nord14 |
| Yellow | `#ebcb8b` | nord13 |

Applied to: Rio, SketchyBar, JankyBorders, and the starship prompt — one
`newmac theme nord` re-paints all of them.

---

## 6. One-time manual steps (macOS blocks scripting these)

The install can't grant these — do them once or the rice looks dead:

1. **System Settings → Privacy & Security → Accessibility** — enable **AeroSpace**,
   **AltTab**, **SketchyBar** (and Karabiner if you installed it). *Without this,
   AeroSpace's keys and window moves silently do nothing.*
2. **AltTab**: set its shortcut to **⌥-Tab**, "Show windows from all Spaces".
3. **Caps Lock → ⌘**: System Settings → Keyboard → Modifier Keys.
4. **Reduce Motion**: System Settings → Accessibility → Display (kills Spaces
   animation so workspace switches are instant).

---

## 7. Recovery — if the rice "didn't set up"

Almost always: Homebrew didn't install the apps. Fix it, then re-run:

```sh
# Is brew even there?
command -v brew || /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
eval "$(/opt/homebrew/bin/brew shellenv)"

cd ~/newmac
git pull
bash scripts/install.sh          # installs everything in newmac.conf (idempotent)
bash scripts/ricing.sh           # links configs + starts sketchybar/borders/AeroSpace
newmac doctor                    # checks PATH + that binaries resolve
```

Then grant the Accessibility permissions (§6), log out/in once (AeroSpace starts
at login), and you should see the bar, borders, and tiling. `newmac theme nord`
re-applies Nord if anything looks off-colour.
```
