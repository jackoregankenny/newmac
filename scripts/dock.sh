#!/usr/bin/env bash
# ============================================================
#  dock.sh — arrange the macOS Dock to match your selection.
#  Omarchy-style: the Dock is rebuilt from newmac.conf, in a
#  fixed sensible order (terminals → agents → browsers → …).
#
#    bash scripts/dock.sh              # rebuild the Dock
#    bash scripts/dock.sh --dry-run    # print the planned order only
#
#  Requires dockutil:  brew install dockutil
#  The previous Dock layout is replaced entirely — anything you
#  miss can be re-added via System Settings > Desktop & Dock,
#  or by dragging the app from /Applications back onto the Dock.
#
#  NOTE: must stay bash-3.2 compatible (stock macOS bash).
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
CONF="$REPO_DIR/newmac.conf"

source "$SCRIPTS_DIR/lib.sh"

DRY=0
[[ "${1:-}" == "--dry-run" ]] && DRY=1

IS_MAC=0
[[ "$(uname -s)" == "Darwin" ]] && IS_MAC=1

if [[ $DRY -eq 0 && $IS_MAC -eq 0 ]]; then
  err "dock.sh only runs on macOS (use --dry-run elsewhere)."; exit 1
fi

if [[ ! -f "$CONF" ]]; then
  err "No newmac.conf found at $CONF"
  info "Run ./bootstrap.sh (or: bash scripts/configure.sh) first to pick your apps."
  exit 1
fi
# shellcheck disable=SC1090
source "$CONF"

# --- id → .app name mapping ------------------------------------
# DOCK_ORDER is the Dock layout, left to right. To add an app:
# append its catalog id here and give it a case in dock_app_name.
DOCK_ORDER="
  ghostty rio
  orca cmux conductor nimbalyst
  dia arc chrome firefox brave zen edge
  vscode cursor-ide zed
  teams outlook slack discord
  obsidian notion figma linear
  1password
"
dock_app_name() {  # dock_app_name <id> -> "<Name>.app" (empty = unknown id)
  case "$1" in
    # Terminals
    ghostty)    echo "Ghostty.app" ;;
    rio)        echo "Rio.app" ;;
    # Agent workbenches
    orca)       echo "Orca.app" ;;
    cmux)       echo "cmux.app" ;;
    conductor)  echo "Conductor.app" ;;
    nimbalyst)  echo "Nimbalyst.app" ;;
    # Browsers
    dia)        echo "Dia.app" ;;
    arc)        echo "Arc.app" ;;
    chrome)     echo "Google Chrome.app" ;;
    firefox)    echo "Firefox.app" ;;
    brave)      echo "Brave Browser.app" ;;
    zen)        echo "Zen.app" ;;
    edge)       echo "Microsoft Edge.app" ;;
    # Editors
    vscode)     echo "Visual Studio Code.app" ;;
    cursor-ide) echo "Cursor.app" ;;
    zed)        echo "Zed.app" ;;
    # Work
    teams)      echo "Microsoft Teams.app" ;;
    outlook)    echo "Microsoft Outlook.app" ;;
    slack)      echo "Slack.app" ;;
    discord)    echo "Discord.app" ;;
    # Notes & design
    obsidian)   echo "Obsidian.app" ;;
    notion)     echo "Notion.app" ;;
    figma)      echo "Figma.app" ;;
    linear)     echo "Linear.app" ;;
    # Essentials
    1password)  echo "1Password.app" ;;
    *)          echo "" ;;
  esac
}

dock_find_app() {  # dock_find_app <Name.app> -> prints path, or fails
  local app="$1" dir
  for dir in "/Applications" "$HOME/Applications"; do
    if [[ -d "$dir/$app" ]]; then echo "$dir/$app"; return 0; fi
  done
  return 1
}

# --- Build the plan --------------------------------------------
head1 "Dock"

PLAN_PATHS=()   # resolved .app paths, in Dock order
PLAN_LABELS=()  # matching display labels
for id in $DOCK_ORDER; do
  newmac_selected "$id" || continue
  app="$(dock_app_name "$id")"
  if [[ -z "$app" ]]; then warn "No .app mapping for '$id' — skipped"; continue; fi
  if [[ $IS_MAC -eq 1 ]]; then
    if path="$(dock_find_app "$app")"; then
      PLAN_PATHS[${#PLAN_PATHS[@]}]="$path"
      PLAN_LABELS[${#PLAN_LABELS[@]}]="${app%.app}"
    else
      warn "$app not found in /Applications or ~/Applications — skipped (install it, then re-run)"
    fi
  else
    # Off-macOS (dry-run only): can't check the disk, preview selection order.
    PLAN_PATHS[${#PLAN_PATHS[@]}]="/Applications/$app"
    PLAN_LABELS[${#PLAN_LABELS[@]}]="${app%.app} (existence not checked off-macOS)"
  fi
done

if [[ ${#PLAN_PATHS[@]} -eq 0 ]]; then
  warn "Nothing to put in the Dock — no selected apps found on disk."
  [[ $DRY -eq 0 ]] && { info "Dock left untouched."; exit 0; }
fi

# --- Dry run: print and stop -----------------------------------
if [[ $DRY -eq 1 ]]; then
  info "Planned Dock order (left → right):"
  i=0
  while [[ $i -lt ${#PLAN_PATHS[@]} ]]; do
    printf '  %2d. %s\n' "$((i+1))" "${PLAN_LABELS[$i]}"
    i=$((i+1))
  done
  printf '  %2d. %s\n' "$(( ${#PLAN_PATHS[@]} + 1 ))" "Downloads (folder, fan view)"
  info "Dry run — Dock not touched."
  exit 0
fi

# --- Apply -----------------------------------------------------
if ! have dockutil; then
  err "dockutil is required to arrange the Dock."
  info "Install it with:  brew install dockutil   — then re-run: bash scripts/dock.sh"
  exit 1
fi

warn "Replacing the previous Dock layout entirely."
info "Anything you miss: System Settings > Desktop & Dock, or drag the app back from /Applications."

dockutil --remove all --no-restart >/dev/null 2>&1 || true
i=0
while [[ $i -lt ${#PLAN_PATHS[@]} ]]; do
  if dockutil --add "${PLAN_PATHS[$i]}" --no-restart >/dev/null 2>&1; then
    ok "Dock: ${PLAN_LABELS[$i]}"
  else
    warn "dockutil failed to add ${PLAN_PATHS[$i]}"
  fi
  i=$((i+1))
done
# Downloads stack after the apps (right side, other-items section).
if dockutil --add "$HOME/Downloads" --view fan --display folder --no-restart >/dev/null 2>&1; then
  ok "Dock: Downloads (folder, fan view)"
else
  warn "dockutil failed to add ~/Downloads"
fi

killall Dock 2>/dev/null || true
ok "Dock rebuilt — ${#PLAN_PATHS[@]} apps + Downloads."
