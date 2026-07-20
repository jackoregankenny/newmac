#!/usr/bin/env bash
# ============================================================
#  get.sh — one-liner bootstrap for a brand-new Mac.
#
#    bash -c "$(curl -fsSL https://raw.githubusercontent.com/jackoregankenny/newmac/main/get.sh)"
#
#  Installs Xcode CLT (for git), clones the repo to ~/newmac
#  (override with NEWMAC_DIR), and hands off to bootstrap.sh.
#  Pass bootstrap args through, e.g.:
#    bash -c "$(curl -fsSL .../get.sh)" -- --preset webdev
#
#  NOTE: intentionally does NOT use `set -u` — this script is fetched
#  and run in whatever environment the user has, so it defends its own
#  variables rather than trusting the shell.
# ============================================================
set -o pipefail

REPO_URL="${NEWMAC_REPO:-https://github.com/jackoregankenny/newmac.git}"
DEST="${NEWMAC_DIR:-${HOME:-/tmp}/newmac}"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "xx  newmac is for macOS only." >&2; exit 1
fi
case "$REPO_URL" in
  *OWNER*) echo "xx  get.sh has no repo URL yet — set NEWMAC_REPO or edit get.sh." >&2; exit 1 ;;
esac

# Xcode Command Line Tools (provides git)
if ! xcode-select -p >/dev/null 2>&1; then
  echo "==> Installing Xcode Command Line Tools (a dialog will appear)..."
  xcode-select --install || true
  echo "==> Waiting for the CLT install to finish (Ctrl-C to abort)..."
  until xcode-select -p >/dev/null 2>&1; do sleep 10; done
fi

if [ -d "$DEST/.git" ]; then
  echo "==> Updating existing checkout at $DEST..."
  git -C "$DEST" pull --ff-only || echo " !!  Could not fast-forward — continuing with what's there."
else
  echo "==> Cloning newmac to $DEST..."
  if ! git clone "$REPO_URL" "$DEST"; then
    echo "xx  Clone failed. If the repo is private, either make it public or run:" >&2
    echo "      gh auth login   # then re-run this" >&2
    echo "    Or clone it yourself and run bootstrap.sh directly." >&2
    exit 1
  fi
fi

if [ ! -f "$DEST/bootstrap.sh" ]; then
  echo "xx  $DEST/bootstrap.sh not found — clone looks incomplete." >&2
  exit 1
fi

# When piped from curl, stdin isn't a terminal — reattach it so the
# interactive picker works (Chris Titus style).
if [ -t 0 ]; then
  exec bash "$DEST/bootstrap.sh" "$@"
elif [ -r /dev/tty ]; then
  exec bash "$DEST/bootstrap.sh" "$@" < /dev/tty
else
  echo "==> No terminal available — running with defaults."
  exec bash "$DEST/bootstrap.sh" --defaults
fi
