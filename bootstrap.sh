#!/usr/bin/env bash
# ============================================================
#  bootstrap.sh — set up a fresh Apple Silicon Mac.
#
#    bash bootstrap.sh                  # interactive picker (TUI)
#    bash bootstrap.sh --defaults       # no questions, sensible defaults
#    bash bootstrap.sh --preset webdev  # a ready-made stack, no questions
#                                       # (default | minimal | webdev | ai | rice)
#    bash bootstrap.sh --reconfigure    # re-open the picker, then install
#
#  Selections live in newmac.conf — edit it or re-run the picker
#  any time. Everything is idempotent; re-running is safe.
# ============================================================
set -uo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$REPO_DIR/config"
SCRIPTS_DIR="$REPO_DIR/scripts"

source "$SCRIPTS_DIR/lib.sh"

MODE="${1:-}"
PRESET="${2:-}"
if [[ "$MODE" == "--preset" && -z "$PRESET" ]]; then
  err "Usage: bash bootstrap.sh --preset <default|minimal|webdev|ai|rice>"; exit 1
fi

# --- guard rails -----------------------------------------------
if [[ "$(uname -s)" != "Darwin" ]]; then
  err "This script is for macOS only."; exit 1
fi
if [[ "$(uname -m)" != "arm64" ]]; then
  warn "Not arm64 — this was written for Apple Silicon. Continuing anyway."
fi

# --- 1. Xcode Command Line Tools -------------------------------
if ! xcode-select -p >/dev/null 2>&1; then
  info "Installing Xcode Command Line Tools (a GUI dialog will appear)…"
  xcode-select --install || true
  info "Waiting for the CLT install to finish (Ctrl-C to abort and re-run later)…"
  until xcode-select -p >/dev/null 2>&1; do sleep 10; done
fi
ok "Xcode Command Line Tools present."

# --- 2. Homebrew -----------------------------------------------
if ! have brew; then
  info "Installing Homebrew…"
  NONINTERACTIVE=1 /bin/bash -c \
    "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi
if [[ -x /opt/homebrew/bin/brew ]]; then
  eval "$(/opt/homebrew/bin/brew shellenv)"
fi
if ! grep -q 'brew shellenv' "$HOME/.zprofile" 2>/dev/null; then
  echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zprofile"
fi
ok "Homebrew ready: $(brew --version | head -n1)"
brew update

# --- 3. Choose what to install ---------------------------------
if [[ "$MODE" == "--defaults" ]]; then
  bash "$SCRIPTS_DIR/configure.sh" --defaults || { err "Configuration failed."; exit 1; }
elif [[ "$MODE" == "--preset" ]]; then
  bash "$SCRIPTS_DIR/configure.sh" --preset "$PRESET" --defaults || { err "Configuration failed."; exit 1; }
elif [[ "$MODE" == "--reconfigure" || ! -f "$REPO_DIR/newmac.conf" ]]; then
  bash "$SCRIPTS_DIR/configure.sh" || { err "Configuration aborted."; exit 1; }
else
  ok "Using existing newmac.conf (run with --reconfigure to change it)."
fi
# shellcheck disable=SC1090
source "$REPO_DIR/newmac.conf"

# --- 4. Install everything selected ----------------------------
bash "$SCRIPTS_DIR/install.sh"

# --- 5. Dotfiles / config symlinks -----------------------------
info "Linking config files…"
newmac_link "$CONFIG_DIR/zshrc"          "$HOME/.zshrc"
newmac_link "$CONFIG_DIR/aliases.zsh"    "$HOME/.config/zsh/aliases.zsh"
newmac_link "$CONFIG_DIR/starship.toml"  "$HOME/.config/starship.toml"
newmac_link "$CONFIG_DIR/tmux.conf"      "$HOME/.tmux.conf"
newmac_selected ghostty && newmac_link "$CONFIG_DIR/ghostty/config" "$HOME/.config/ghostty/config"
newmac_selected rio     && newmac_link "$CONFIG_DIR/rio/config.toml" "$HOME/.config/rio/config.toml"

# Apply the chosen colour theme (terminal / bar / borders / prompt).
if [[ -d "$CONFIG_DIR/themes" ]]; then
  bash "$SCRIPTS_DIR/theme.sh" || warn "theme.sh reported issues."
fi

# Persist the repo location so the `status` / `macup` aliases work
# no matter where this repo was cloned.
if ! grep -q 'NEWMAC=' "$HOME/.zshrc.local" 2>/dev/null; then
  echo "export NEWMAC=\"$REPO_DIR\"" >> "$HOME/.zshrc.local"
  ok "Recorded repo location in ~/.zshrc.local"
fi

# Install the `newmac` command (repo path baked in, so it works
# from any shell even before ~/.zshrc.local is sourced).
mkdir -p "$HOME/.local/bin"
cat > "$HOME/.local/bin/newmac" <<LAUNCHER
#!/usr/bin/env bash
export NEWMAC="\${NEWMAC:-$REPO_DIR}"
exec bash "$REPO_DIR/bin/newmac" "\$@"
LAUNCHER
chmod +x "$HOME/.local/bin/newmac"
ok "Installed the 'newmac' command (try: newmac help)."

# --- 5b. Git identity & niceties -------------------------------
if have git; then
  git config --global init.defaultBranch main 2>/dev/null || true
  if have delta && [[ -z "$(git config --global core.pager 2>/dev/null || true)" ]]; then
    git config --global core.pager delta
    git config --global interactive.diffFilter 'delta --color-only'
    git config --global delta.navigate true
    ok "git: delta wired up as pager."
  fi
  if [[ -z "$(git config --global user.email 2>/dev/null || true)" && -t 0 ]]; then
    info "Git identity not set — enter it now (leave blank to skip)."
    read -r -p "  git user.name:  " _gname || _gname=""
    read -r -p "  git user.email: " _gmail || _gmail=""
    [[ -n "$_gname" ]] && git config --global user.name "$_gname"
    [[ -n "$_gmail" ]] && git config --global user.email "$_gmail"
    [[ -n "$_gmail" ]] && ok "git identity saved."
  fi
fi

# --- 6. fzf keybindings ----------------------------------------
if have fzf; then
  "$(brew --prefix)/opt/fzf/install" --key-bindings --completion --no-update-rc --no-bash --no-fish >/dev/null 2>&1 || true
fi

# --- 7. Optional steps (chosen in the picker) -------------------
if [[ "${NEWMAC_TOGGLE_RICING:-0}" == 1 ]]; then
  bash "$SCRIPTS_DIR/ricing.sh"
fi
if [[ "${NEWMAC_TOGGLE_MACOS_DEFAULTS:-0}" == 1 ]]; then
  bash "$SCRIPTS_DIR/macos-defaults.sh"
fi
if [[ "${NEWMAC_TOGGLE_POWER:-0}" == 1 ]]; then
  bash "$SCRIPTS_DIR/power.sh"
fi
if [[ "${NEWMAC_TOGGLE_SCHEDULE:-0}" == 1 ]]; then
  bash "$SCRIPTS_DIR/schedule-updates.sh"
fi
if [[ "${NEWMAC_TOGGLE_DOCK:-0}" == 1 ]] && have dockutil; then
  bash "$SCRIPTS_DIR/dock.sh" || warn "dock.sh reported issues."
fi

# --- 8. PATH health check --------------------------------------
# Verify a fresh shell will actually find everything we installed,
# and repair the PATH automatically if not.
bash "$SCRIPTS_DIR/doctor.sh" --fix || warn "doctor found problems — run 'newmac doctor' in a new terminal."

# --- done ------------------------------------------------------
printf "\n%s========================================%s\n" "$c_green" "$c_reset"
ok "Bootstrap complete."
cat <<'EOF'

Next steps:
  1. Restart your terminal (or run: exec zsh) to load the new shell.
  2. Sign in: 1Password app, then `op signin` for the CLI.
     Enable shell plugins:  op plugin init gh   (etc.)
  3. Authenticate any agents you installed on first run:
     claude / codex / droid / opencode / amp / kimi / …
  4. Battery: System Settings > Battery > Charging > Charge Limit = 80%.
  5. Open Cloudflare WARP and connect (if installed).
  6. Tiling desktop: finish the Accessibility permissions
     (System Settings > Privacy & Security > Accessibility) — see ricing.sh output.
  7. Superconductor (parallel agents) is a separate download:
     https://super.engineering (not in brew or the App Store).
  8. See what's installed:  bash scripts/status.sh   (alias: status)
     Change selections:     bash bootstrap.sh --reconfigure
     Keep it fresh:         bash update.sh
EOF
