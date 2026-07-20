#!/usr/bin/env bash
# ============================================================
#  ricing.sh — wire up the Linux-style tiling desktop:
#  AeroSpace + sketchybar + borders + Karabiner + AltTab.
#  Assumes the Brewfile has been installed. Re-runnable.
# ============================================================
set -uo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="$REPO_DIR/config"

head1 "Linking ricing configs"
newmac_link "$CONFIG_DIR/aerospace/aerospace.toml" "$HOME/.config/aerospace/aerospace.toml"
newmac_link "$CONFIG_DIR/sketchybar"               "$HOME/.config/sketchybar"
newmac_link "$CONFIG_DIR/borders/bordersrc"        "$HOME/.config/borders/bordersrc"
newmac_link "$REPO_DIR/scripts/keys.sh"            "$HOME/.config/newmac/keys.sh"  # ⌥⇧/ cheat-sheet
# Karabiner rewrites its own file, so copy (don't symlink):
newmac_copy "$CONFIG_DIR/karabiner/karabiner.json" "$HOME/.config/karabiner/karabiner.json"

# Make sketchybar plugins executable
chmod +x "$HOME/.config/sketchybar/sketchybarrc" "$HOME/.config/sketchybar/plugins/"*.sh 2>/dev/null || true
chmod +x "$HOME/.config/borders/bordersrc" 2>/dev/null || true

head1 "macOS settings that fight tiling"
defaults write com.apple.dock mru-spaces -bool false          # don't auto-rearrange Spaces
defaults write com.apple.dock expose-group-apps -bool true    # group windows by app
defaults write com.apple.universalaccess reduceMotion -bool true 2>/dev/null || true
killall Dock 2>/dev/null || true
ok "Applied (Reduce Motion may also need a toggle in System Settings > Accessibility)."

head1 "Starting services"
if command -v sketchybar >/dev/null 2>&1; then
  brew services restart sketchybar >/dev/null 2>&1 && ok "sketchybar running"
fi
if command -v borders >/dev/null 2>&1; then
  brew services restart borders >/dev/null 2>&1 && ok "borders running"
fi
# Launch the GUI apps so they register (and prompt for permissions)
open -a AeroSpace 2>/dev/null || warn "Open AeroSpace manually."
open -a AltTab 2>/dev/null || warn "Open AltTab manually."
open -a "Karabiner-Elements" 2>/dev/null || warn "Open Karabiner-Elements manually."

cat <<'EOF'

────────────────────────────────────────────
 MANUAL STEPS (macOS won't let scripts do these)
────────────────────────────────────────────
 1. System Settings > Privacy & Security > Accessibility — enable:
      AeroSpace, AltTab, Karabiner-Elements, sketchybar
    (Karabiner also needs Input Monitoring — it will prompt.)
 2. AltTab: set its shortcut to ⌥-Tab (Option-Tab), "Show windows from all spaces".
 3. Caps Lock is now ⌘ (held) / Esc (tapped) via Karabiner. To use the simpler
    native remap instead: System Settings > Keyboard > Modifier Keys > Caps Lock → ⌘,
    then quit Karabiner.
 4. Reduce Motion: System Settings > Accessibility > Display > Reduce Motion (on).

 Keybindings:  press ⌥⇧/ any time for the cheat-sheet popup
               (or run `keys` in a terminal).
   ⌥1..5            switch workspace (1 Agents · 2 Browser · 3 Editor · 4 Comms · 5 Spare)
   ⌥hjkl            focus window      ⌥⇧hjkl    move window
   ⌥- / ⌥=          resize            ⌥f        fullscreen
   ⌥/ , ⌥,          split / accordion ⌥b        previous workspace
   ⌥⇧space          float/un-float    ⌥⇧;       service mode
   ⌥Tab             AltTab window switcher
 Settings, Zoom, FaceTime, Calculator, 1Password and Calendar float
 automatically — exec-friendly by default.
────────────────────────────────────────────
EOF
ok "Ricing setup complete."

# New to tiling? Offer the friendly walkthrough.
if [[ -t 0 ]] && ask "First time with a tiling desktop? Take the 2-minute tour?"; then
  bash "$REPO_DIR/scripts/tour.sh"
else
  info "Take it any time with:  newmac tour"
fi
