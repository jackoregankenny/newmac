#!/usr/bin/env bash
# ============================================================
#  update.sh — keep everything current. Run anytime:  bash update.sh
#  Safe under the weekly LaunchAgent too (skips anything that
#  could prompt for a password when not attached to a terminal).
# ============================================================
set -uo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$REPO_DIR/scripts/lib.sh"

# --- Homebrew ---------------------------------------------------
info "Homebrew: update + upgrade…"
brew update
brew upgrade
if [[ -t 0 ]]; then
  # Cask upgrades can prompt for an admin password and restart running
  # apps — only do the greedy pass when a human is watching.
  brew upgrade --cask --greedy
else
  info "Non-interactive run — skipping greedy cask upgrades (apps self-update)."
fi

info "Installing anything newly selected in newmac.conf…"
bash "$REPO_DIR/scripts/install.sh" || warn "install.sh reported issues."

info "Homebrew: cleanup…"
brew cleanup -s
brew autoremove
ok "Homebrew up to date."

# --- Rust -------------------------------------------------------
if have rustup; then
  info "Rust: rustup update…"; rustup update && ok "Rust up to date."
fi

# --- Bun --------------------------------------------------------
if have bun; then
  info "Bun: upgrade…"; bun upgrade && ok "Bun up to date."
fi

# --- Node (fnm) -------------------------------------------------
if have fnm; then
  info "Node: install latest LTS…"
  fnm install --lts >/dev/null 2>&1 && fnm default lts-latest >/dev/null 2>&1 || true
  ok "Node LTS current."
fi

# --- Agent CLIs with their own updaters ------------------------
# (brew-installed agents were covered by `brew upgrade` above;
#  Amp self-updates; npm-installed ones are refreshed by install.sh.)
info "Agents: updating…"
have claude       && { claude update           || warn "claude update failed"; }
have droid        && { droid update            || warn "droid update failed"; }
have cursor-agent && { cursor-agent update     || warn "cursor-agent update failed"; }
have opencode     && { opencode upgrade 2>/dev/null || true; }
ok "Agents updated."

# --- gh extensions ---------------------------------------------
have gh && gh extension upgrade --all >/dev/null 2>&1 || true

printf "\n"; ok "Everything is up to date."
