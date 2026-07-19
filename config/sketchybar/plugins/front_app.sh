#!/usr/bin/env bash
# Shows the name of the focused application.
if [ "$SENDER" = "front_app_switched" ]; then
  sketchybar --set "$NAME" label="$INFO"
fi
