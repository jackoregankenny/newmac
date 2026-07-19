#!/usr/bin/env bash
# ============================================================
#  status.sh — list everything installed + versions.
#  Run:  bash scripts/status.sh   (or just `status` after setup)
# ============================================================
set -uo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

# row <label> <cmd> [args…]  -> green ● + version, or red ○ + "not installed"
row() {
  local name="$1"; shift
  if command -v "$1" >/dev/null 2>&1; then
    local v; v="$("$@" 2>/dev/null | head -n1)"
    printf "  %s●%s %-16s %s%s%s\n" "$c_green" "$c_reset" "$name" "$c_dim" "$v" "$c_reset"
  else
    printf "  %s○%s %-16s %snot installed%s\n" "$c_red" "$c_reset" "$name" "$c_dim" "$c_reset"
  fi
}

printf "%s%s newmac — installed tooling %s\n" "$c_bold" "$c_mauve" "$c_reset"

head1 "Shell & terminal"
row "starship"  starship --version
row "zoxide"    zoxide --version
row "fzf"       fzf --version
row "eza"       eza --version
row "bat"       bat --version
row "ripgrep"   rg --version
row "fd"        fd --version
row "git"       git --version
row "gh"        gh --version
row "delta"     delta --version
row "zellij"    zellij --version
row "tmux"      tmux -V

head1 "Languages & runtimes"
row "bun"       bun --version
row "rustc"     rustc --version
row "cargo"     cargo --version
row "node"      node --version
row "fnm"       fnm --version
row "go"        go version
row "uv"        uv --version
row "python3"   python3 --version

head1 "Containers"
row "docker"    docker --version
row "colima"    colima version
row "compose"   docker-compose version

head1 "AI agents"
row "claude"    claude --version
row "codex"     codex --version
row "droid"     droid --version
row "opencode"  opencode --version
row "amp"       amp --version
row "kimi"      kimi --version
row "crush"     crush --version
row "goose"     goose --version
row "cursor"    cursor-agent --version
row "copilot"   copilot --version
row "aider"     aider --version
row "qwen"      qwen --version
row "gemini"    gemini --version

head1 "Agent terminals"
row "cmux"      cmux --version

head1 "Monitoring & maintenance"
row "btop"      btop --version
row "macmon"    macmon --version
row "mole"      mole --version

if command -v brew >/dev/null 2>&1; then
  head1 "Homebrew"
  printf "  %s●%s formulae:   %s%s%s\n" "$c_green" "$c_reset" "$c_dim" "$(brew list --formula 2>/dev/null | wc -l | tr -d ' ')" "$c_reset"
  printf "  %s●%s casks:      %s%s%s\n" "$c_green" "$c_reset" "$c_dim" "$(brew list --cask 2>/dev/null | wc -l | tr -d ' ')" "$c_reset"
  outdated="$(brew outdated --quiet 2>/dev/null | wc -l | tr -d ' ')"
  printf "  %s●%s outdated:   %s%s%s  %s\n" "$c_green" "$c_reset" "$c_dim" "$outdated" "$c_reset" "$([[ "$outdated" != 0 ]] && echo '→ run: bash update.sh')"

  head1 "GUI apps (casks)"
  brew list --cask 2>/dev/null | sed 's/^/  • /'
fi
echo
