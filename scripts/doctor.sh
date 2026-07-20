#!/usr/bin/env bash
# ============================================================
#  doctor.sh — verify the PATH and that every installed tool
#  actually resolves in a fresh login shell; print (or apply)
#  the exact fix for anything broken.
#
#    newmac doctor          # report problems + the commands to fix them
#    newmac doctor --fix    # apply the fixes (idempotent)
#
#  What it checks:
#    1. Every tool directory is on the PATH a *new* zsh would get
#       (/opt/homebrew/bin, ~/.local/bin, ~/.cargo/bin, ~/.bun/bin,
#        ~/go/bin) — probed via `zsh -lic`, not this script's PATH.
#    2. Every selected non-brew tool resolves; if its binary exists
#       in a known dir but does not resolve, that is a PATH problem
#       and doctor says exactly which line is missing.
#    3. The `newmac` launcher itself.
#
#  Fixes go to ~/.zprofile (brew shellenv) and ~/.zshrc.local
#  (PATH exports) — both guarded, both idempotent.
#  Must stay bash-3.2 compatible.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
CONF="$REPO_DIR/newmac.conf"

source "$SCRIPTS_DIR/lib.sh"
source "$SCRIPTS_DIR/catalog.sh"
[[ -f "$CONF" ]] && source "$CONF"

FIX=0
[[ "${1:-}" == "--fix" ]] && FIX=1

PROBLEMS=0
FIXES=""    # newline-separated "target|line" pairs to apply/print

note_problem() { PROBLEMS=$((PROBLEMS + 1)); }
queue_fix() {  # queue_fix <target-file> <line-to-append>
  FIXES="$FIXES$1|$2
"
}

# --- Probe the PATH of a fresh login+interactive zsh ------------
# This is what matters: not this script's PATH (lib.sh fixed that),
# but what the user's next terminal tab will actually see.
if have zsh; then
  PROBE_PATH="$(zsh -lic 'printf %s "$PATH"' 2>/dev/null)"
  [[ -z "$PROBE_PATH" ]] && PROBE_PATH="$PATH"
else
  PROBE_PATH="$PATH"
fi

on_probe_path() {  # on_probe_path <dir>
  case ":$PROBE_PATH:" in *":$1:"*) return 0 ;; esac
  return 1
}
resolves() {  # resolves <cmd> — in the probed PATH, not ours
  PATH="$PROBE_PATH" command -v "$1" >/dev/null 2>&1
}

head1 "newmac doctor"

# --- 1. Tool directories on the fresh-shell PATH ----------------
# dir|relevant-when|fix-target|fix-line
DIR_CHECKS='/opt/homebrew/bin|brew|zprofile|eval "$(/opt/homebrew/bin/brew shellenv)"
'"$HOME"'/.local/bin|always|zshrc.local|export PATH="$HOME/.local/bin:$PATH"
'"$HOME"'/.cargo/bin|rust|zshrc.local|export PATH="$HOME/.cargo/bin:$PATH"
'"$HOME"'/.bun/bin|bun|zshrc.local|export PATH="$HOME/.bun/bin:$PATH"
'"$HOME"'/go/bin|go|zshrc.local|export PATH="$HOME/go/bin:$PATH"'

while IFS='|' read -r dir when target line; do
  [[ -z "$dir" ]] && continue
  # Only nag about dirs that are relevant to this machine/selection.
  case "$when" in
    always) ;;
    brew)   [[ "$(uname -s)" == "Darwin" ]] || continue ;;
    *)      newmac_selected "$when" || [[ -d "$dir" ]] || continue ;;
  esac
  if on_probe_path "$dir"; then
    ok "on PATH: $dir"
  else
    err "NOT on fresh-shell PATH: $dir"
    queue_fix "$target" "$line"
    note_problem
  fi
done <<EOF
$DIR_CHECKS
EOF

# --- 2. Selected tools resolve ---------------------------------
KNOWN_DIRS="/opt/homebrew/bin $HOME/.local/bin $HOME/.cargo/bin $HOME/.bun/bin $HOME/go/bin"
i=0
while [[ $i -lt ${#CAT_ID[@]} ]]; do
  id="${CAT_ID[$i]}"
  kind="${CAT_KIND[$i]}"
  if { [[ "${CAT_CATEGORY[$i]}" == core ]] || newmac_selected "$id"; } \
     && [[ "$kind" == npm || "$kind" == uv || "$kind" == curl || "$kind" == rustup ]]; then
    cmd="$id"
    [[ "$id" == cursor ]] && cmd="cursor-agent"
    [[ "$kind" == rustup ]] && cmd="rustc"
    if resolves "$cmd"; then
      ok "resolves: $cmd"
    else
      found=""
      for d in $KNOWN_DIRS; do
        [[ -x "$d/$cmd" ]] && { found="$d"; break; }
      done
      if [[ -n "$found" ]]; then
        err "installed but NOT resolving: $cmd (exists in $found — PATH problem)"
        note_problem
      else
        warn "not installed: $cmd (run: newmac install)"
      fi
    fi
  fi
  i=$((i+1))
done

# --- 3. The newmac launcher ------------------------------------
if resolves newmac; then
  ok "resolves: newmac"
else
  err "the newmac command does not resolve in a fresh shell"
  if [[ ! -x "$HOME/.local/bin/newmac" ]]; then
    info "  launcher missing — re-run: bash $REPO_DIR/bootstrap.sh (it re-creates it)"
  fi
  note_problem
fi

# --- Apply or print the fixes ----------------------------------
apply_fix() {  # apply_fix <target> <line>
  local file
  case "$1" in
    zprofile)    file="$HOME/.zprofile" ;;
    zshrc.local) file="$HOME/.zshrc.local" ;;
    *)           return 1 ;;
  esac
  grep -qF "$2" "$file" 2>/dev/null && { ok "already in $file: $2"; return 0; }
  printf '%s\n' "$2" >> "$file"
  ok "added to $file: $2"
}

if [[ -n "$FIXES" ]]; then
  echo
  if [[ $FIX -eq 1 ]]; then
    info "Applying PATH fixes…"
    while IFS='|' read -r target line; do
      [[ -z "$target" ]] && continue
      apply_fix "$target" "$line"
    done <<EOF
$FIXES
EOF
    info "Open a new terminal (or: exec zsh) and run 'newmac doctor' again to confirm."
  else
    info "Fix everything above with:  newmac doctor --fix"
    info "(or add the lines yourself:)"
    while IFS='|' read -r target line; do
      [[ -z "$target" ]] && continue
      case "$target" in
        zprofile)    printf '  ~/.zprofile     <-  %s\n' "$line" ;;
        zshrc.local) printf '  ~/.zshrc.local  <-  %s\n' "$line" ;;
      esac
    done <<EOF
$FIXES
EOF
  fi
fi

echo
if [[ $PROBLEMS -eq 0 ]]; then
  ok "PATH is healthy — everything resolves."
  exit 0
else
  warn "$PROBLEMS problem(s) found."
  exit 1
fi
