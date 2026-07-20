#!/usr/bin/env bash
# ============================================================
#  nuke.sh — remove everything newmac installed on this Mac.
#
#    bash -c "$(curl -fsSL https://raw.githubusercontent.com/jackoregankenny/newmac/main/nuke.sh)"
#
#  Reads the install manifest (~/.config/newmac/installed.list) and
#  bulk-uninstalls only what newmac itself added — your pre-existing
#  tools are left alone. Asks before removing anything.
#
#    ... /nuke.sh)" -- --dry-run    # preview what would be removed
#
#  Fresh start after this: re-run the installer (get.sh).
#  NOTE: no `set -u` — this is fetched and run in the user's own shell.
# ============================================================
set -o pipefail

REPO_URL="${NEWMAC_REPO:-https://github.com/jackoregankenny/newmac.git}"
DEST="${NEWMAC_DIR:-${HOME:-/tmp}/newmac}"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "xx  newmac is for macOS only." >&2; exit 1
fi

# We need the repo for the uninstaller + catalog. Clone it if it's gone
# (the manifest lives in ~/.config/newmac, so it survives even if ~/newmac
# was deleted).
if [ ! -f "$DEST/bin/newmac" ]; then
  if ! command -v git >/dev/null 2>&1; then
    echo "xx  git not found and $DEST is missing — can't run the uninstaller." >&2
    echo "    Install Xcode CLT (xcode-select --install) or clone the repo, then retry." >&2
    exit 1
  fi
  echo "==> Fetching newmac to $DEST (needed for the uninstaller)..."
  git clone "$REPO_URL" "$DEST" || { echo "xx  Clone failed." >&2; exit 1; }
fi

MANIFEST="${HOME:-/tmp}/.config/newmac/installed.list"
if [ ! -s "$MANIFEST" ]; then
  echo "==> Nothing tracked in $MANIFEST — newmac hasn't recorded any installs to remove."
  echo "    (If you installed before manifest tracking existed, remove tools with 'brew uninstall'.)"
  exit 0
fi

# `newmac nuke` reads the manifest, shows the count, and asks before removing.
# Reattach the terminal so that confirmation works even via curl | bash.
run() { NEWMAC="$DEST" bash "$DEST/bin/newmac" nuke "$@"; }
if [ -t 0 ]; then
  run "$@"
elif [ -r /dev/tty ]; then
  run "$@" < /dev/tty
else
  echo "xx  No terminal to confirm a destructive uninstall. Re-run in a real terminal," >&2
  echo "    or preview first:  ... /nuke.sh)\" -- --dry-run" >&2
  exit 1
fi
