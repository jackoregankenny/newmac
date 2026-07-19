#!/usr/bin/env bash
# Battery percentage + a Nerd Font icon, charging-aware.
PERCENT="$(pmset -g batt | grep -Eo '[0-9]+%' | head -n1 | tr -d '%')"
[ -z "$PERCENT" ] && exit 0
CHARGING="$(pmset -g batt | grep -c 'AC Power')"

case "$PERCENT" in
  100|9[0-9]) ICON="󰁹" ;;
  [6-8][0-9]) ICON="󰂀" ;;
  [3-5][0-9]) ICON="󰁾" ;;
  [1-2][0-9]) ICON="󰁻" ;;
  *)          ICON="󰁺" ;;
esac
[ "$CHARGING" -gt 0 ] && ICON="󰂄"

sketchybar --set "$NAME" icon="$ICON" label="${PERCENT}%"
