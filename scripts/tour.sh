#!/usr/bin/env bash
# ============================================================
#  tour.sh — "helpful rice": a friendly, paced walkthrough that
#  eases a Mac person into the tiling / Linux-flavoured desktop
#  without losing the niceties of macOS.
#
#    newmac tour        # the full walkthrough
#
#  It explains one idea per screen, waits for you, and (on macOS)
#  offers to open the panels / trigger the things it describes.
#  Read-only and safe — the only actions are opening System
#  Settings panes and showing the cheat-sheet, always on a y/N.
#  Must stay bash-3.2 compatible.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
source "$SCRIPTS_DIR/lib.sh"

IS_MAC=0
[[ "$(uname -s)" == "Darwin" ]] && IS_MAC=1

_pause() { [[ -t 0 ]] && { printf '\n%s   ↵ enter to continue…%s' "$c_dim" "$c_reset"; read -r _ || true; }; }
_k()     { printf '%s%s%s' "$c_bold$c_mauve" "$1" "$c_reset"; }   # a keybinding
_open()  { [[ $IS_MAC -eq 1 ]] && open "$1" >/dev/null 2>&1 || true; }

clear 2>/dev/null || true
cat <<EOF

$(printf '%s' "$c_bold$c_mauve")  Welcome to your riced Mac.$(printf '%s' "$c_reset")

  You kept macOS — Spotlight, Handoff, the App Store, the polish — and
  layered a Linux-style tiling desktop on top. This 2-minute tour shows
  you how to live in it. Nothing here changes anything; it just teaches.

  The one thing to remember: press $(_k '⌥⇧/') any time for the full
  cheat-sheet. Everything else you can forget and look up.
EOF
_pause

# --- 1. The modifier -------------------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '1. Your new modifier is ⌥ (Option)')

  Caps Lock was remapped: $(_k 'tap') it for Esc, $(_k 'hold') it for ⌘.
  So your left pinky now does Command — and Option drives the desktop.

  Why: it frees a big comfortable key and keeps ⌘-C / ⌘-V exactly where
  they were. Nothing about copy/paste/Spotlight changed.
EOF
_pause

# --- 2. Workspaces ---------------------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '2. Five workspaces, one per job')

    $(_k '⌥1') Agents   $(_k '⌥2') Browser   $(_k '⌥3') Editor   $(_k '⌥4') Comms   $(_k '⌥5') Spare

  Apps fly to the right workspace when they launch. Switching is instant —
  no swoosh animation, no trackpad. $(_k '⌥⇧2') sends the current window to
  Browser and follows it; $(_k '⌥b') bounces back to where you were.

  Think of it as five monitors you flip between with your left hand.
EOF
_pause

# --- 3. Tiling --------------------------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '3. Windows arrange themselves')

  Open two things in a workspace and they split the screen automatically —
  no dragging, no snapping to edges. Move around with vim keys:

    $(_k '⌥h ⌥j ⌥k ⌥l')  focus left/down/up/right
    $(_k '⌥⇧h/j/k/l')     move the window itself
    $(_k '⌥-  ⌥=')        shrink / grow      $(_k '⌥f')  fullscreen

  Messed up the layout? $(_k '⌥⇧;') then $(_k 'r') resets it.
EOF
_pause

# --- 4. Floating (the exec bit) --------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '4. Floating is fine — encouraged, even')

  Tiling is for deep work. But a Zoom call, a Settings panel, a calculator
  or a calendar should just $(_k 'float') — so they do, automatically.

  Any window: $(_k '⌥⇧space') toggles it between floating and tiled.

  This is the "executive" escape hatch: tile when you're building, float
  when you're in meetings. You are never trapped in a grid.
EOF
_pause

# --- 5. The bar & the look -------------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '5. The top bar and the colours')

  The bar up top is $(_k 'SketchyBar') — workspaces on the left, focused app,
  clock and battery on the right. The focus ring around the active window
  is $(_k 'JankyBorders').

  All of it — bar, borders, terminal, prompt — shares one colour theme.
  See them and switch live:

    $(_k 'newmac theme')            preview all six with swatches
    $(_k 'newmac theme kanagawa')   apply one everywhere at once
EOF
_pause

# --- 6. Offer the real things ----------------------------------
clear 2>/dev/null || true
cat <<EOF

$(_k '6. Try it')

  That's the whole model. A few real actions to finish — all optional.
EOF
echo
if [[ $IS_MAC -eq 1 && -t 0 ]]; then
  if newmac_selected sketchybar || true; then
    if ask "Show the hotkey cheat-sheet now (⌥⇧/ does this any time)?"; then
      bash "$SCRIPTS_DIR/keys.sh" --dialog 2>/dev/null || bash "$SCRIPTS_DIR/keys.sh"
    fi
  fi
  if ask "Open Accessibility settings? (AeroSpace/AltTab/Karabiner/sketchybar need it)"; then
    _open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
  fi
  if ask "Preview the colour themes?"; then
    bash "$SCRIPTS_DIR/theme.sh" --list
  fi
else
  info "On a Mac this step opens Accessibility settings and the cheat-sheet."
  info "Preview themes any time with:  newmac theme"
fi

cat <<EOF

$(printf '%s' "$c_green")  You're set.$(printf '%s' "$c_reset")  Reminders:
    $(_k '⌥⇧/')  cheat-sheet        $(_k 'newmac keys')   same, in the terminal
    $(_k 'newmac theme')  colours    $(_k 'newmac doctor') health check

  Re-run this any time:  $(_k 'newmac tour')
EOF
