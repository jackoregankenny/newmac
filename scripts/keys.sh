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
KEYS_TEXT='WORKSPACES                          WINDOWS
⌥1…⌥5     go to workspace           ⌥h/j/k/l   focus (vim dirs)
⌥⇧1…⌥⇧5   send window + follow      ⌥⇧h/j/k/l  move window
⌥b        previous workspace        ⌥- / ⌥=    resize
⌥Tab      window switcher (AltTab)  ⌥f         fullscreen

LAYOUTS                             HELP & SERVICE
⌥/        toggle split direction    ⌥⇧/        this cheat-sheet
⌥,        accordion                 ⌥⇧;        service mode:
⌥⇧space   float / un-float             r  reset layout
                                       esc  reload config
FLOATING                               ⌫  close other windows
System Settings, Zoom, FaceTime,
Calculator, 1Password, Calendar     Workspaces: 1 Agents · 2 Browser
float automatically — ⌥⇧space          3 Editor · 4 Comms · 5 Spare
tiles them again.'

if [[ "${1:-}" == "--dialog" ]]; then
  # Native popup via osascript — works from an AeroSpace keybinding.
  # (Text goes in via argv: AppleScript literals can't hold newlines.)
  /usr/bin/osascript \
    -e 'on run argv' \
    -e 'display dialog (item 1 of argv) with title "newmac — keys (⌥ = Option)" buttons {"OK"} default button "OK"' \
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
printf '%s%s newmac keys %s%s(⌥ = Option — Caps Lock is ⌘)%s\n\n' "$b" "$m" "$r" "$d" "$r"
printf '%s\n' "$KEYS_TEXT"
