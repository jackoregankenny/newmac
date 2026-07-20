#!/usr/bin/env bash
# ============================================================
#  keys.sh — the hotkey cheat-sheet (Omarchy-style help).
#
#    keys                    # pretty print in the terminal (alias)
#    bash keys.sh --dialog   # native popup — bound to ⌥⇧/ in AeroSpace
#
#  ricing.sh links this to ~/.config/newmac/keys.sh so the
#  AeroSpace binding has a stable path.
# ============================================================
set -uo pipefail

# Plain text master copy — used by both output modes.
KEYS_TEXT='AEROSPACE — WINDOWS                 AEROSPACE — WORKSPACES
⌥h/j/k/l   focus (vim dirs)         ⌥1…⌥5     go to workspace
⌥⇧h/j/k/l  move window              ⌥⇧1…⌥⇧5   send window + follow
⌥- / ⌥=    resize                   ⌥b        previous workspace
⌥f         fullscreen               ⌥Tab      window switcher (AltTab)
⌥/         toggle split             ⌥,        accordion
⌥⇧space    float / un-float         ⌥⇧/       this cheat-sheet
⌥⇧;  service → r reset · esc reload config · ⌫ close other windows
Workspaces: 1 Agents · 2 Browser · 3 Editor · 4 Comms · 5 Spare

ZELLIJ — TERMINAL MULTIPLEXER  (Ctrl modes; ⌥ is taken by AeroSpace)
Ctrl p  PANE:  n new · x close · f fullscreen · h/j/k/l focus
Ctrl t  TAB:   n new · x close · r rename · ←/→ switch
Ctrl n  resize    Ctrl s  scroll / (/) search    Ctrl o  session (d detach)
Ctrl g  lock (toggle)    Ctrl q  quit    Enter/Esc  leave a mode

FLOATING: System Settings, Zoom, FaceTime, Calculator, 1Password, Calendar
float automatically — ⌥⇧space tiles them again.'

if [[ "${1:-}" == "--dialog" ]]; then
  # Native popup via osascript — works from an AeroSpace keybinding.
  # (Text goes in via argv: AppleScript literals can't hold newlines.)
  /usr/bin/osascript \
    -e 'on run argv' \
    -e 'display dialog (item 1 of argv) with title "newmac — keys (⌥ Option · Ctrl for Zellij)" buttons {"OK"} default button "OK"' \
    -e 'end run' \
    "$KEYS_TEXT" >/dev/null 2>&1
  exit 0
fi

# Terminal output with a little colour.
if [[ -t 1 ]]; then
  b=$'\033[1m'; d=$'\033[2m'; m=$'\033[35m'; r=$'\033[0m'
else
  b=''; d=''; m=''; r=''
fi
printf '%s%s newmac keys %s%s(⌥ = Option · Caps Lock = ⌘ · Ctrl-prefix = Zellij)%s\n\n' "$b" "$m" "$r" "$d" "$r"
printf '%s\n' "$KEYS_TEXT"
