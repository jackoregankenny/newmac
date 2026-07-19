#!/usr/bin/env bash
# ============================================================
#  get.sh — one-liner bootstrap for a brand-new Mac.
#
#    bash -c "$(curl -fsSL https://raw.githubusercontent.com/OWNER/newmac/main/get.sh)"
#
#  Installs Xcode CLT (for git), clones the repo to ~/newmac
#  (override with NEWMAC_DIR), and hands off to bootstrap.sh.
#  Pass bootstrap args through, e.g.:
#    bash -c "$(curl -fsSL …/get.sh)" -- --preset webdev
#
#  NOTE: update OWNER above / NEWMAC_REPO below after pushing
#  this repo to GitHub.
# ============================================================
set -uo pipefail

REPO_URL="${NEWMAC_REPO:-https://github.com/OWNER/newmac.git}"
DEST="${NEWMAC_DIR:-$HOME/newmac}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "xx  newmac is for macOS only." >&2; exit 1
fi
case "$REPO_URL" in
  *OWNER*) echo "xx  get.sh has no repo URL yet — set NEWMAC_REPO or edit get.sh." >&2; exit 1 ;;
esac

# Xcode Command Line Tools (provides git)
if ! xcode-select -p >/dev/null 2>&1; then
  echo "==> Installing Xcode Command Line Tools (a dialog will appear)…"
  xcode-select --install || true
  echo "==> Waiting for the CLT install to finish (Ctrl-C to abort)…"
  until xcode-select -p >/dev/null 2>&1; do sleep 10; done
fi

if [[ -d "$DEST/.git" ]]; then
  echo "==> Updating existing checkout at $DEST…"
  git -C "$DEST" pull --ff-only || echo " !!  Could not fast-forward — continuing with what's there."
else
  echo "==> Cloning newmac to $DEST…"
  git clone "$REPO_URL" "$DEST"
fi

# When piped from curl, stdin isn't a terminal — reattach it so the
# interactive picker works (Chris Titus style).
if [[ -t 0 ]]; then
  exec bash "$DEST/bootstrap.sh" "$@"
elif [[ -r /dev/tty ]]; then
  exec bash "$DEST/bootstrap.sh" "$@" < /dev/tty
else
  echo "==> No terminal available — running with defaults."
  exec bash "$DEST/bootstrap.sh" --defaults
fi
