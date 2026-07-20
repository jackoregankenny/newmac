#!/usr/bin/env bash
# ============================================================
#  get-ui.sh — download the prebuilt Rust picker (newmac-ui)
#  from GitHub Releases and install it to ~/.local/bin. No cargo
#  needed, so this can run on a fresh Mac before anything else.
#
#    bash scripts/get-ui.sh            # latest release
#    bash scripts/get-ui.sh v0.1.0     # a specific tag
#    bash scripts/get-ui.sh --quiet    # only speak up on failure
#
#  Best-effort: if there is no release yet (or no network), it
#  exits 0 quietly so bootstrap can fall back to the bash picker.
# ============================================================
set -uo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPTS_DIR/lib.sh"

REPO_SLUG="jackoregankenny/newmac"
ASSET="newmac-ui-macos-universal.tar.gz"
BIN_DIR="$HOME/.local/bin"

QUIET=0
TAG="latest"
for a in "$@"; do
  case "$a" in
    --quiet) QUIET=1 ;;
    v*)      TAG="$a" ;;
    *)       warn "get-ui.sh: ignoring unknown arg '$a'" ;;
  esac
done
say() { [[ $QUIET -eq 1 ]] || "$@"; }

# macOS only — the released binary is a macOS universal build.
if [[ "$(uname -s)" != "Darwin" ]]; then
  say warn "get-ui.sh only installs the macOS binary (build from source elsewhere: newmac ui --build)."
  exit 0
fi

if [[ "$TAG" == "latest" ]]; then
  URL="https://github.com/$REPO_SLUG/releases/latest/download/$ASSET"
else
  URL="https://github.com/$REPO_SLUG/releases/download/$TAG/$ASSET"
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

say info "Downloading newmac-ui ($TAG)…"
if ! curl -fsSL "$URL" -o "$TMP/$ASSET"; then
  say warn "No prebuilt newmac-ui available yet ($URL)."
  say info "Falling back to the bash picker. To build from source: newmac ui --build"
  exit 0
fi

# Verify the checksum when the .sha256 sidecar is published alongside it.
if curl -fsSL "$URL.sha256" -o "$TMP/$ASSET.sha256" 2>/dev/null; then
  if ( cd "$TMP" && shasum -a 256 -c "$ASSET.sha256" >/dev/null 2>&1 ); then
    say ok "Checksum verified."
  else
    err "Checksum mismatch for $ASSET — refusing to install."
    exit 1
  fi
else
  say warn "No checksum published — skipping verification."
fi

tar -xzf "$TMP/$ASSET" -C "$TMP" || { err "Failed to unpack $ASSET."; exit 1; }
[[ -f "$TMP/newmac-ui" ]] || { err "Archive did not contain newmac-ui."; exit 1; }

mkdir -p "$BIN_DIR"
# curl doesn't quarantine, but strip it defensively just in case.
xattr -d com.apple.quarantine "$TMP/newmac-ui" 2>/dev/null || true
chmod +x "$TMP/newmac-ui"
mv -f "$TMP/newmac-ui" "$BIN_DIR/newmac-ui"

if "$BIN_DIR/newmac-ui" --version >/dev/null 2>&1; then
  say ok "Installed $("$BIN_DIR/newmac-ui" --version) → $BIN_DIR/newmac-ui"
else
  err "Installed binary won't run (arch/signature?). Try: newmac ui --build"
  exit 1
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) say warn "$BIN_DIR is not on PATH — 'newmac doctor --fix' adds it." ;;
esac
