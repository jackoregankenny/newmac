#!/usr/bin/env bash
# ============================================================
#  build-ui.sh — build the Rust picker (newmac-ui) and link it
#  onto PATH so `newmac configure` uses it.
#
#    bash scripts/build-ui.sh            # build + install to ~/.local/bin
#    bash scripts/build-ui.sh --dry-run  # just report what it would do
#
#  Needs a Rust toolchain (cargo). It is in the catalog by default; if it
#  is missing this prints how to get it and exits cleanly.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPTS_DIR/.." && pwd)"
UI_DIR="$REPO_DIR/ui"
BIN_DIR="$HOME/.local/bin"

source "$SCRIPTS_DIR/lib.sh"

DRY=0
[[ "${1:-}" == "--dry-run" ]] && DRY=1

if ! have cargo; then
  warn "cargo not found — the Rust picker needs a Rust toolchain."
  info "Get it with:  newmac configure  (tick Rust)  then  newmac install"
  info "or directly:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 0
fi

if [[ $DRY -eq 1 ]]; then
  info "Would run: cargo build --release  (in $UI_DIR)"
  info "Would link: $BIN_DIR/newmac-ui -> $UI_DIR/target/release/newmac-ui"
  exit 0
fi

info "Building newmac-ui (release)…"
if ! ( cd "$UI_DIR" && cargo build --release ); then
  err "Build failed."; exit 1
fi

mkdir -p "$BIN_DIR"
ln -sf "$UI_DIR/target/release/newmac-ui" "$BIN_DIR/newmac-ui"
ok "Installed $BIN_DIR/newmac-ui"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) warn "$BIN_DIR is not on PATH — add it, or run 'newmac doctor --fix'." ;;
esac

ok "Done. 'newmac configure' will now open the Rust picker."
