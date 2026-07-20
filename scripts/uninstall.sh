#!/usr/bin/env bash
# ============================================================
#  uninstall.sh — remove catalog items from this machine.
#  The reverse of install.sh: brew/cask/npm/uv/curl/rustup
#  removals, then prunes NEWMAC_SELECTED in newmac.conf so the
#  config keeps matching reality.
#
#    bash scripts/uninstall.sh <id> [<id>…]   # remove the given items
#    bash scripts/uninstall.sh --unselected   # remove installed items that
#                                             # are no longer selected
#    bash scripts/uninstall.sh --dry-run …    # preview, change nothing
#
#  mas apps can't be removed from the CLI — guidance is printed.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
CONF="$REPO_DIR/newmac.conf"

source "$SCRIPTS_DIR/lib.sh"
source "$SCRIPTS_DIR/catalog.sh"

usage() {
  cat <<EOF
Usage:
  bash scripts/uninstall.sh [--dry-run] <id> [<id>…]
  bash scripts/uninstall.sh [--dry-run] --unselected

Modes:
  <id> [<id>…]    Uninstall the given catalog items.
  --unselected    Uninstall every catalog item that is installed but not
                  in NEWMAC_SELECTED (newmac.conf) — prunes the machine
                  after deselecting things in the picker. Items in the
                  core category are never touched in this mode.

Flags:
  --dry-run       Print what would happen without changing anything.
                  (Also the only mode that runs on non-macOS.)

Example ids: crush copilot aider ollama rectangle spotify
Full list:   scripts/catalog.sh
EOF
}

# --- Args -------------------------------------------------------
DRY=0
UNSELECTED=0
IDS=""
for arg in "$@"; do
  case "$arg" in
    --dry-run)    DRY=1 ;;
    --unselected) UNSELECTED=1 ;;
    -h|--help)    usage; exit 0 ;;
    --*)          err "Unknown flag: $arg"; usage; exit 1 ;;
    *)            IDS="$IDS $arg" ;;
  esac
done

if [[ $UNSELECTED -eq 0 && -z "${IDS// /}" ]]; then
  usage; exit 1
fi
if [[ $UNSELECTED -eq 1 && -n "${IDS// /}" ]]; then
  err "Pass either ids or --unselected, not both."; exit 1
fi
if [[ $DRY -eq 0 && "$(uname -s)" != "Darwin" ]]; then
  err "uninstall.sh only runs on macOS (use --dry-run elsewhere)."; exit 1
fi

# --- Helpers ----------------------------------------------------
cat_index() {  # cat_index <id>  -> echoes catalog index, rc 1 if unknown
  local i=0
  while [[ $i -lt ${#CAT_ID[@]} ]]; do
    if [[ "${CAT_ID[$i]}" == "$1" ]]; then echo "$i"; return 0; fi
    i=$((i+1))
  done
  return 1
}

# Command name probed for npm/uv/curl items (mirrors install.sh).
cmd_for_id() {
  case "$1" in
    cursor) echo "cursor-agent" ;;
    aider)  echo "aider" ;;
    *)      echo "$1" ;;
  esac
}

# Config/cache dir a curl-installed tool leaves behind (informational only).
curl_config_note() {
  case "$1" in
    claude) echo "$HOME/.claude" ;;
    droid)  echo "$HOME/.factory" ;;
    amp)    echo "$HOME/.config/amp" ;;
    kimi)   echo "$HOME/.kimi" ;;
    cursor) echo "$HOME/.cursor" ;;
  esac
}

runcmd() {  # print in dry-run, execute otherwise
  if [[ $DRY -eq 1 ]]; then
    printf "  %sdry-run:%s %s\n" "$c_dim" "$c_reset" "$*"
    return 0
  fi
  "$@"
}

REMOVED=""  # space-separated ids actually removed (never filled in dry-run)
mark_removed() { REMOVED="$REMOVED $1"; }

is_installed() {  # is_installed <catalog index>
  local i="$1"
  local payload="${CAT_PAYLOAD[$i]}"
  case "${CAT_KIND[$i]}" in
    brew)        have brew && brew list --formula --versions "${payload##*/}" >/dev/null 2>&1 ;;
    cask)        have brew && brew list --cask --versions "${payload##*/}" >/dev/null 2>&1 ;;
    npm|uv|curl) have "$(cmd_for_id "${CAT_ID[$i]}")" ;;
    rustup)      have rustup ;;
    mas)         [[ -d "/Applications/${payload#*:}.app" ]] ;;
    *)           return 1 ;;
  esac
}

remove_item() {  # remove_item <catalog index>
  local i="$1"
  local id="${CAT_ID[$i]}" kind="${CAT_KIND[$i]}" payload="${CAT_PAYLOAD[$i]}" name="${CAT_NAME[$i]}"
  info "$name ($id, $kind)"
  case "$kind" in
    brew)
      local short="${payload##*/}"   # tap-qualified payloads: brew accepts the short name
      if runcmd brew uninstall "$short"; then
        [[ $DRY -eq 0 ]] && { ok "Removed $name."; mark_removed "$id"; }
      else
        warn "brew uninstall $short failed — probably not installed; continuing."
      fi ;;
    cask)
      local short="${payload##*/}"
      if runcmd brew uninstall --cask "$short"; then
        [[ $DRY -eq 0 ]] && { ok "Removed $name."; mark_removed "$id"; }
      else
        warn "brew uninstall --cask $short failed — probably not installed; continuing."
      fi ;;
    npm)
      if have bun; then
        if runcmd bun remove -g "$payload"; then
          [[ $DRY -eq 0 ]] && { ok "Removed $name."; mark_removed "$id"; }
        else
          warn "bun remove -g $payload failed — probably not installed; continuing."
        fi
      elif have npm; then
        if runcmd npm uninstall -g "$payload"; then
          [[ $DRY -eq 0 ]] && { ok "Removed $name."; mark_removed "$id"; }
        else
          warn "npm uninstall -g $payload failed — probably not installed; continuing."
        fi
      else
        warn "Neither bun nor npm found — cannot remove $name ($payload)."
      fi ;;
    uv)
      if have uv; then
        if runcmd uv tool uninstall "$payload"; then
          [[ $DRY -eq 0 ]] && { ok "Removed $name."; mark_removed "$id"; }
        else
          warn "uv tool uninstall $payload failed — probably not installed; continuing."
        fi
      else
        warn "uv not found — cannot remove $name ($payload)."
      fi ;;
    mas)
      local app="${payload#*:}"
      warn "mas has no reliable uninstall — remove '$app' by dragging /Applications/$app.app to the Trash (or use Pearcleaner / mole)." ;;
    curl)
      local cmd; cmd="$(cmd_for_id "$id")"
      if ! have "$cmd"; then
        warn "$name ($cmd) not found on PATH — skipping."
        return 0
      fi
      local cfg; cfg="$(curl_config_note "$id")"
      case "$id" in
        claude)
          if [[ $DRY -eq 1 ]]; then
            printf "  %sdry-run:%s claude uninstall  (falling back to: rm -f %s)\n" \
              "$c_dim" "$c_reset" "$HOME/.local/bin/claude"
          elif claude --help 2>/dev/null | grep -qw "uninstall"; then
            if claude uninstall; then ok "Removed $name."; mark_removed "$id"
            else warn "'claude uninstall' failed — remove manually."; fi
          else
            rm -f "$HOME/.local/bin/claude"
            warn "No 'claude uninstall' subcommand — removed binary ~/.local/bin/claude only."
            mark_removed "$id"
          fi
          [[ -n "$cfg" ]] && info "Leftover config kept: $cfg (delete yourself if wanted)." ;;
        droid|amp|kimi|cursor)
          local bin="$HOME/.local/bin/$cmd"
          if [[ $DRY -eq 1 ]]; then
            runcmd rm -f "$bin"
          elif [[ -e "$bin" ]]; then
            rm -f "$bin"; ok "Removed $bin."; mark_removed "$id"
          else
            warn "$bin not found ($cmd installed elsewhere?) — remove manually."
          fi
          [[ -n "$cfg" ]] && info "Leftover config kept: $cfg (delete yourself if wanted)." ;;
        *)
          warn "No uninstall recipe for curl-installed $name — remove its binary from ~/.local/bin manually." ;;
      esac ;;
    rustup)
      if [[ $DRY -eq 1 ]]; then
        warn "dry-run: skipping 'rustup self uninstall -y' (would prompt for confirmation)."
      elif ! have rustup; then
        warn "rustup not found — skipping."
      elif [[ ! -t 0 ]]; then
        warn "stdin is not a terminal — skipping 'rustup self uninstall' (needs confirmation)."
      elif ask "Run 'rustup self uninstall -y' (removes ~/.cargo and ~/.rustup)?"; then
        if rustup self uninstall -y; then ok "Rust toolchain removed."; mark_removed "$id"
        else warn "rustup self uninstall failed."; fi
      else
        warn "Skipped rustup self uninstall."
      fi ;;
    *)
      warn "Unknown kind '$kind' for $id — nothing done." ;;
  esac
}

# --- Build the target list -------------------------------------
# shellcheck disable=SC1090
[[ -f "$CONF" ]] && source "$CONF"

TARGETS=""  # catalog indexes to remove
if [[ $UNSELECTED -eq 1 ]]; then
  if [[ ! -f "$CONF" ]]; then
    err "No newmac.conf — run 'bash scripts/configure.sh' first."; exit 1
  fi
  have brew || warn "brew not found — brew/cask items can't be detected as installed."
  i=0
  while [[ $i -lt ${#CAT_ID[@]} ]]; do
    if [[ "${CAT_CATEGORY[$i]}" != core ]] \
       && ! newmac_selected "${CAT_ID[$i]}" \
       && is_installed "$i"; then
      TARGETS="$TARGETS $i"
    fi
    i=$((i+1))
  done
  if [[ -z "${TARGETS// /}" ]]; then
    ok "Nothing to remove — everything installed is still selected."; exit 0
  fi
else
  BAD=0
  for id in $IDS; do
    if ! cat_index "$id" >/dev/null; then err "Unknown id: $id"; BAD=1; fi
  done
  if [[ $BAD -eq 1 ]]; then
    info "Valid ids:"
    echo "${CAT_ID[*]}" | fold -s -w 76 | sed 's/^/  /'
    exit 1
  fi
  for id in $IDS; do
    TARGETS="$TARGETS $(cat_index "$id")"
  done
fi

# --- Remove -----------------------------------------------------
n=0; for i in $TARGETS; do n=$((n+1)); done
if [[ $DRY -eq 1 ]]; then
  head1 "Would remove $n item(s) (dry run)"
else
  head1 "Removing $n item(s)"
fi
for i in $TARGETS; do
  remove_item "$i"
done

# --- Prune NEWMAC_SELECTED in newmac.conf ----------------------
if [[ $DRY -eq 0 && -n "$REMOVED" && -f "$CONF" ]]; then
  NEWSEL=""
  for id in ${NEWMAC_SELECTED:-}; do
    case " $REMOVED " in
      *" $id "*) ;;
      *) NEWSEL="$NEWSEL$id " ;;
    esac
  done
  TMPCONF="${CONF}.tmp.$$"
  while IFS= read -r line; do
    case "$line" in
      NEWMAC_SELECTED=*) printf 'NEWMAC_SELECTED=" %s"\n' "$NEWSEL" ;;
      *)                 printf '%s\n' "$line" ;;
    esac
  done < "$CONF" > "$TMPCONF"
  mv "$TMPCONF" "$CONF"
  ok "newmac.conf updated — dropped:$REMOVED"
fi

# --- Prune the install manifest (what `newmac list` shows) ------
MANIFEST="$HOME/.config/newmac/installed.list"
if [[ $DRY -eq 0 && -n "$REMOVED" && -f "$MANIFEST" ]]; then
  TMPMAN="${MANIFEST}.tmp.$$"
  while IFS= read -r line; do
    case " $REMOVED " in
      *" $line "*) ;;
      *) printf '%s\n' "$line" ;;
    esac
  done < "$MANIFEST" > "$TMPMAN"
  mv "$TMPMAN" "$MANIFEST"
fi

if [[ $DRY -eq 1 ]]; then
  info "Dry run — nothing was changed."
else
  ok "Uninstall pass complete."
fi
