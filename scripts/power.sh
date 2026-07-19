#!/usr/bin/env bash
# ============================================================
#  power.sh — battery / power optimisation for Apple Silicon.
#  Uses pmset (needs sudo). All settings are reversible.
#  Battery longevity is handled by the native Charge Limit
#  (System Settings > Battery > Charging, Tahoe 26.4+) +
#  Optimized Charging — this just tunes sleep/power behaviour.
# ============================================================
set -uo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

info "Current power settings:"
pmset -g custom || true
echo

info "Applying battery-friendly power profile (sudo required)…"

# --- On BATTERY -------------------------------------------------
sudo pmset -b lowpowermode 1        # enable Low Power Mode on battery
sudo pmset -b displaysleep 3        # screen off after 3 min idle
sudo pmset -b sleep 10              # system sleep after 10 min
sudo pmset -b powernap 0            # no Power Nap on battery (saves drain)
sudo pmset -b ttyskeepawake 1       # stay awake while an ssh/tty session is active

# --- On AC / CHARGER -------------------------------------------
sudo pmset -c lowpowermode 0
sudo pmset -c displaysleep 15
sudo pmset -c sleep 0               # don't sleep while plugged in (good for builds)
sudo pmset -c powernap 1

# --- Both -------------------------------------------------------
sudo pmset -a hibernatemode 3       # default safe-sleep (RAM + disk image)
sudo pmset -a proximitywake 0       # don't wake when nearby Apple devices wake

ok "Power profile applied."
echo
info "New settings:"
pmset -g custom || true

cat <<'EOF'

Battery longevity tips for this M4 Pro:
  • Native charge limit (macOS Tahoe 26.4+, replaces AlDente):
      System Settings > Battery > Charging > Charge Limit = 80%.
  • Keep "Optimized Battery Charging" ON in the same panel.
  • Use `macmon` for a live power/temp/efficiency view in the terminal.
  • Use `coconutBattery` to track cycle count and battery health over time.
  • `stats` (or your sketchybar battery item) shows charge at a glance.
EOF
