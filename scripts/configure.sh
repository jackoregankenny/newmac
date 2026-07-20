#!/usr/bin/env bash
# ============================================================
#  configure.sh — choose what to install; writes newmac.conf.
#
#    bash scripts/configure.sh                    # interactive picker
#    bash scripts/configure.sh --defaults         # catalog defaults, no TUI
#    bash scripts/configure.sh --preset webdev    # seed from a preset, then tweak
#    bash scripts/configure.sh --preset ai --defaults   # preset as-is, no TUI
#
#  Interactive runs open with a preset screen (Balanced / Minimal /
#  Web dev / AI power user / Full rice — or keep your current conf),
#  then walk the category checkboxes seeded from that choice.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
CONF="$REPO_DIR/newmac.conf"

source "$SCRIPTS_DIR/lib.sh"
source "$SCRIPTS_DIR/catalog.sh"
source "$SCRIPTS_DIR/presets.sh"
source "$SCRIPTS_DIR/tui.sh"

USE_DEFAULTS=0
PRESET=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --defaults) USE_DEFAULTS=1 ;;
    --preset)
      shift
      PRESET="${1:-}"
      case " $NEWMAC_PRESETS " in
        *" $PRESET "*) ;;
        *) err "Unknown preset '$PRESET'. Available: $NEWMAC_PRESETS"; exit 1 ;;
      esac ;;
    *) err "Unknown option '$1' (use --defaults / --preset <name>)"; exit 1 ;;
  esac
  shift
done
[[ ! -t 0 ]] && USE_DEFAULTS=1

HAVE_CONF=0
if [[ -f "$CONF" ]]; then
  # shellcheck disable=SC1090
  source "$CONF"
  HAVE_CONF=1
fi

# --- Decide the pre-selection basis ----------------------------
# PRESEL: conf | preset | defaults
if [[ -n "$PRESET" ]]; then
  PRESEL="preset"
elif [[ $HAVE_CONF -eq 1 ]]; then
  PRESEL="conf"
else
  PRESEL="defaults"
fi

_banner() {
  printf '%s' "$c_mauve"
  cat <<'BANNER'

  _ __   _____      ___ __ ___   __ _  ___
 | '_ \ / _ \ \ /\ / / '_ ` _ \ / _` |/ __|
 | | | |  __/\ V  V /| | | | | | (_| | (__
 |_| |_|\___| \_/\_/ |_| |_| |_|\__,_|\___|
BANNER
  printf '%s' "$c_reset"
  printf ' %syour Mac, your way — tick exactly what you want in each category%s\n' "$c_dim" "$c_reset"
  printf ' %sÀ la carte: everything starts at a sensible default; change anything.%s\n\n' "$c_dim" "$c_reset"
}

# No preset gate: interactive runs go straight to the category pickers,
# seeded from your existing conf (re-run) or the catalog defaults (first
# run). Presets remain available only as an explicit `--preset <name>`
# shortcut for the one-liner installs.
if [[ $USE_DEFAULTS -eq 0 ]]; then
  _banner
fi

PRESET_IDS=""
[[ -n "$PRESET" ]] && PRESET_IDS="$(newmac_preset_ids "$PRESET")"
# "default" preset id list is empty -> catalog defaults
[[ "$PRESEL" == "preset" && -z "$PRESET_IDS" ]] && PRESEL="defaults"

# Catch typos in preset id lists (ids must exist in the catalog).
ALL_IDS=" "
i=0
while [[ $i -lt ${#CAT_ID[@]} ]]; do ALL_IDS="$ALL_IDS${CAT_ID[$i]} "; i=$((i+1)); done
for pid in $PRESET_IDS; do
  case "$ALL_IDS" in
    *" $pid "*) ;;
    *) warn "Preset '$PRESET' references unknown id '$pid' — check scripts/presets.sh" ;;
  esac
done

_preselected() {  # _preselected <idx> -> 0 if item should start checked
  local i="$1"
  case "$PRESEL" in
    conf)   newmac_selected "${CAT_ID[$i]}" ;;
    preset) case " $PRESET_IDS " in *" ${CAT_ID[$i]} "*) return 0 ;; esac; return 1 ;;
    *)      [[ "${CAT_DEFAULT[$i]}" == on ]] ;;
  esac
}

SELECTED=""
add_selected() { SELECTED="$SELECTED $1"; }

# Step counter for the picker header (categories + theme screen).
TOTAL_STEPS=1
for category in $NEWMAC_CATEGORIES; do TOTAL_STEPS=$((TOTAL_STEPS + 1)); done
STEP=0

for category in $NEWMAC_CATEGORIES; do
  # Collect catalog indices for this category.
  idxs=""
  i=0
  while [[ $i -lt ${#CAT_ID[@]} ]]; do
    [[ "${CAT_CATEGORY[$i]}" == "$category" ]] && idxs="$idxs $i"
    i=$((i+1))
  done
  [[ -z "$idxs" ]] && continue

  STEP=$((STEP + 1))
  if [[ $USE_DEFAULTS -eq 1 ]]; then
    for i in $idxs; do
      _preselected "$i" && add_selected "${CAT_ID[$i]}"
    done
    continue
  fi

  TUI_STEP="$STEP/$TOTAL_STEPS"
  TUI_TITLE="$(newmac_category_title "$category")"
  TUI_ITEMS=(); TUI_DESCS=(); TUI_STATE=(); MAP=()
  for i in $idxs; do
    TUI_ITEMS[${#TUI_ITEMS[@]}]="${CAT_NAME[$i]}"
    TUI_DESCS[${#TUI_DESCS[@]}]="${CAT_DESC[$i]}"
    if _preselected "$i"; then TUI_STATE[${#TUI_STATE[@]}]=1; else TUI_STATE[${#TUI_STATE[@]}]=0; fi
    MAP[${#MAP[@]}]="$i"
  done
  tui_multiselect
  j=0
  while [[ $j -lt ${#MAP[@]} ]]; do
    [[ "${TUI_STATE[$j]}" == 1 ]] && add_selected "${CAT_ID[${MAP[$j]}]}"
    j=$((j+1))
  done
done

# --- Theme -----------------------------------------------------
THEMES_DIR="$REPO_DIR/config/themes"
THEME_SEL="${NEWMAC_THEME:-tokyonight}"
if [[ -d "$THEMES_DIR" ]]; then
  # tokyonight first, then the rest alphabetically
  THEME_LIST="tokyonight"
  for f in "$THEMES_DIR"/*.sh; do
    [[ -f "$f" ]] || continue
    tid="$(basename "$f" .sh)"
    [[ "$tid" == tokyonight ]] && continue
    THEME_LIST="$THEME_LIST $tid"
  done
  if [[ $USE_DEFAULTS -eq 0 ]]; then
    TUI_STEP="$TOTAL_STEPS/$TOTAL_STEPS"
    TUI_TITLE="Theme"
    TUI_ITEMS=(); TUI_DESCS=(); TUI_STATE=(); THEME_MAP=()
    TUI_CHOICE=0
    for tid in $THEME_LIST; do
      ttitle="$(. "$THEMES_DIR/$tid.sh"; echo "${THEME_TITLE:-$tid}")"
      [[ "$tid" == "$THEME_SEL" ]] && TUI_CHOICE=${#TUI_ITEMS[@]}
      TUI_ITEMS[${#TUI_ITEMS[@]}]="$ttitle"
      TUI_DESCS[${#TUI_DESCS[@]}]="terminal · bar · borders · prompt"
      THEME_MAP[${#THEME_MAP[@]}]="$tid"
    done
    tui_select
    THEME_SEL="${THEME_MAP[$TUI_CHOICE]}"
  fi
fi

# --- Toggles ---------------------------------------------------
if [[ "$PRESEL" == "preset" ]]; then
  # shellcheck disable=SC2046  # intentional split of "1 1 1 0 1" into args
  set -- $(newmac_preset_toggles "$PRESET")
  t_ricing="$1"; t_defaults="$2"; t_power="$3"; t_schedule="$4"; t_dock="${5:-1}"
else
  t_ricing="${NEWMAC_TOGGLE_RICING:-1}"
  t_defaults="${NEWMAC_TOGGLE_MACOS_DEFAULTS:-1}"
  t_power="${NEWMAC_TOGGLE_POWER:-1}"
  t_schedule="${NEWMAC_TOGGLE_SCHEDULE:-0}"
  t_dock="${NEWMAC_TOGGLE_DOCK:-1}"
fi

if [[ $USE_DEFAULTS -eq 0 ]]; then
  case " $SELECTED " in
    *" aerospace "*|*" sketchybar "*)
      if tui_yesno "Apply the tiling-desktop configs (AeroSpace/sketchybar/borders/Karabiner)?" "$([[ "$t_ricing" == 1 ]] && echo y || echo n)"; then t_ricing=1; else t_ricing=0; fi ;;
    *) t_ricing=0 ;;
  esac
  if tui_yesno "Apply opinionated macOS UX defaults (keyboard, Finder, Dock, screenshots)?" "$([[ "$t_defaults" == 1 ]] && echo y || echo n)"; then t_defaults=1; else t_defaults=0; fi
  if tui_yesno "Apply battery/power tuning via pmset (needs sudo)?" "$([[ "$t_power" == 1 ]] && echo y || echo n)"; then t_power=1; else t_power=0; fi
  if tui_yesno "Schedule weekly auto-updates (LaunchAgent, Mondays 10:00)?" "$([[ "$t_schedule" == 1 ]] && echo y || echo n)"; then t_schedule=1; else t_schedule=0; fi
  if tui_yesno "Arrange the Dock to match your selection (replaces the current Dock)?" "$([[ "$t_dock" == 1 ]] && echo y || echo n)"; then t_dock=1; else t_dock=0; fi
else
  # Non-interactive with tiling deselected: never run ricing.
  case " $SELECTED " in
    *" aerospace "*|*" sketchybar "*) ;;
    *) t_ricing=0 ;;
  esac
fi

# --- Summary + confirm -----------------------------------------
if [[ $USE_DEFAULTS -eq 0 ]]; then
  head1 "Summary"
  for category in $NEWMAC_CATEGORIES; do
    names=""; n=0; i=0
    while [[ $i -lt ${#CAT_ID[@]} ]]; do
      if [[ "${CAT_CATEGORY[$i]}" == "$category" ]]; then
        case " $SELECTED " in
          *" ${CAT_ID[$i]} "*) names="$names${names:+, }${CAT_NAME[$i]}"; n=$((n+1)) ;;
        esac
      fi
      i=$((i+1))
    done
    [[ $n -eq 0 ]] && names="${c_dim}(none)${c_reset}"
    printf '  %s%-24s%s %s\n' "$c_bold" "$(newmac_category_title "$category") ($n)" "$c_reset" "$names"
  done
  printf '  %s%-24s%s %s\n' "$c_bold" "Theme" "$c_reset" "$THEME_SEL"
  printf '  %s%-24s%s ricing=%s · macos-defaults=%s · power=%s · weekly-updates=%s · dock=%s\n' \
    "$c_bold" "Toggles" "$c_reset" "$t_ricing" "$t_defaults" "$t_power" "$t_schedule" "$t_dock"
  echo
  if ! tui_yesno "Save this selection?" y; then
    err "Aborted — nothing written."; exit 1
  fi
fi

# --- Write conf ------------------------------------------------
{
  echo "# newmac.conf — generated by scripts/configure.sh $(date '+%Y-%m-%d %H:%M')"
  [[ -n "$PRESET" ]] && echo "# preset: $PRESET"
  echo "# Edit by hand or re-run: bash scripts/configure.sh"
  echo "NEWMAC_SELECTED=\"$SELECTED \""
  echo "NEWMAC_THEME=$THEME_SEL"
  echo "NEWMAC_TOGGLE_RICING=$t_ricing"
  echo "NEWMAC_TOGGLE_MACOS_DEFAULTS=$t_defaults"
  echo "NEWMAC_TOGGLE_POWER=$t_power"
  echo "NEWMAC_TOGGLE_SCHEDULE=$t_schedule"
  echo "NEWMAC_TOGGLE_DOCK=$t_dock"
} > "$CONF"

ok "Saved selections to newmac.conf${PRESET:+ (preset: $PRESET)}"
if [[ $USE_DEFAULTS -eq 1 ]]; then
  case "$PRESEL" in
    conf)    info "Kept existing conf (non-interactive)." ;;
    preset)  info "Applied preset '$PRESET' (non-interactive)." ;;
    *)       info "Used catalog defaults (non-interactive)." ;;
  esac
fi
