#!/usr/bin/env bash
# ============================================================
#  lib.sh — shared styling + PATH for every newmac script.
#  Source it at the top of a script:
#     source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"
#  NOTE: must stay bash-3.2 compatible (stock macOS bash) —
#  no associative arrays, no mapfile, no ${var,,}.
# ============================================================

# --- Colours / logging -----------------------------------------
# Only emit colour when stdout is a terminal (keeps logs clean).
if [[ -t 1 ]]; then
  c_reset=$'\033[0m';  c_bold=$'\033[1m';   c_dim=$'\033[2m'
  c_blue=$'\033[34m';  c_green=$'\033[32m'; c_yellow=$'\033[33m'
  c_red=$'\033[31m';   c_cyan=$'\033[36m';  c_mauve=$'\033[35m'
else
  c_reset=''; c_bold=''; c_dim=''
  c_blue=''; c_green=''; c_yellow=''
  c_red=''; c_cyan=''; c_mauve=''
fi

info()  { printf "%s==>%s %s\n"  "$c_blue"   "$c_reset" "$*"; }
ok()    { printf "%s ok%s  %s\n" "$c_green"  "$c_reset" "$*"; }
warn()  { printf "%s !!%s  %s\n" "$c_yellow" "$c_reset" "$*"; }
err()   { printf "%s xx%s  %s\n" "$c_red"    "$c_reset" "$*" >&2; }
head1() { printf "\n%s%s%s%s\n" "$c_bold" "$c_mauve" "$*" "$c_reset"; }
hr()    { printf "%s────────────────────────────────────────────%s\n" "$c_dim" "$c_reset"; }
ask()   { local p="$1" a; read -r -p "$(printf '%s ?? %s%s [y/N] ' "$c_yellow" "$p" "$c_reset")" a; [[ "$a" == [yY]* ]]; }
have()  { command -v "$1" >/dev/null 2>&1; }

# --- PATH: make sure brew + curl-installed tools are visible ----
# claude/droid/amp/kimi etc. land in ~/.local/bin; rust in ~/.cargo/bin; bun in ~/.bun/bin
newmac_load_paths() {
  [[ -x /opt/homebrew/bin/brew ]] && eval "$(/opt/homebrew/bin/brew shellenv)"
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$PATH"
  [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
}
newmac_load_paths

# --- Config helpers --------------------------------------------
# Symlink a config file, backing up anything already there.
newmac_link() {  # newmac_link <src> <dest>
  local src="$1" dest="$2"
  mkdir -p "$(dirname "$dest")"
  if [[ -L "$dest" && "$(readlink "$dest")" == "$src" ]]; then return; fi
  if [[ -e "$dest" || -L "$dest" ]]; then
    mv "$dest" "${dest}.backup.$(date +%s)"; warn "Backed up existing $dest"
  fi
  ln -s "$src" "$dest"; ok "Linked $dest"
}
# Copy (not symlink) — for files an app rewrites itself, e.g. karabiner.json.
newmac_copy() {  # newmac_copy <src> <dest>
  local src="$1" dest="$2"
  mkdir -p "$(dirname "$dest")"
  if [[ -e "$dest" && ! -L "$dest" ]]; then mv "$dest" "${dest}.backup.$(date +%s)"; warn "Backed up existing $dest"; fi
  rm -f "$dest"; cp "$src" "$dest"; ok "Copied $dest"
}

# Is a catalog item present on this machine? (light check by kind)
newmac_is_installed() {  # newmac_is_installed <kind> <payload> <id>
  case "$1" in
    brew)   have brew && brew list --formula --versions "${2##*/}" >/dev/null 2>&1 ;;
    cask)   have brew && brew list --cask --versions "${2##*/}" >/dev/null 2>&1 ;;
    npm|uv|curl)
      case "$3" in cursor) have cursor-agent ;; *) have "$3" ;; esac ;;
    rustup) have rustup ;;
    mas)    [[ -d "/Applications/${2#*:}.app" ]] ;;
    *)      return 1 ;;
  esac
}

# --- Selection helpers (newmac.conf) ---------------------------
# newmac.conf defines NEWMAC_SELECTED=" id1 id2 … " plus toggle vars.
newmac_selected() {  # newmac_selected <id>  -> 0 if selected
  case " ${NEWMAC_SELECTED:-} " in *" $1 "*) return 0 ;; esac
  return 1
}
