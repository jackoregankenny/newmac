#!/usr/bin/env bash
# ============================================================
#  presets.sh — GENERATED from flavours/*.toml. Do not edit by hand.
#      Regenerate with:  newmac-ui catalog gen-sh
#  A preset pre-selects catalog ids + theme/glass/toggles; the picker still
#  opens so you can tweak. Add your own by dropping a flavours/<id>.toml
#  (see CONTRIBUTING.md). Keep this file bash-3.2 compatible.
# ============================================================

NEWMAC_PRESETS="default jack basic ai rice webdev"

newmac_preset_title() {
  case "$1" in
    default) echo "Balanced" ;;
    jack) echo "Jack's flavour" ;;
    basic) echo "Basic" ;;
    ai) echo "AI power user" ;;
    rice) echo "Full rice" ;;
    webdev) echo "Web dev" ;;
    *) echo "$1" ;;
  esac
}

newmac_preset_desc() {
  case "$1" in
    default) echo "The catalog defaults — every category at its sensible default" ;;
    jack) echo "AeroSpace rice · agents · Rio+cmux · Nord glass — my daily driver" ;;
    basic) echo "Lean: Rio + Zellij + Claude + 1Password — no rice, no extras" ;;
    ai) echo "Every coding agent + agent terminals + full tiling desktop" ;;
    rice) echo "Linux-style desktop first: tiling, bar, borders, all the fonts" ;;
    webdev) echo "TypeScript/web: Bun, Node, containers, VS Code, Chrome, Figma" ;;
    *) echo "" ;;
  esac
}

newmac_preset_ids() {
  case "$1" in
    jack) echo "rio cmux zellij lazygit atuin direnv claude codex amp opencode orca wispr-flow aerospace borders alt-tab raycast sketchybar stats monitorcontrol bun fnm uv go rust docker docker-compose colima vscode dia 1password 1password-cli coconutbattery cloudflare-warp tailscale lulu spotify-player mole mas dust duf dockutil font-jetbrains font-symbols" ;;
    basic) echo "rio zellij claude 1password 1password-cli vscode bun fnm uv mole mas dust duf font-jetbrains font-symbols" ;;
    ai) echo "rio cmux orca conductor nimbalyst zellij tmux claude codex droid opencode amp kimi crush goose cursor copilot aider qwen bun fnm uv go rust docker docker-compose colima vscode dia 1password 1password-cli cloudflare-warp raycast stats coconutbattery sketchybar aerospace borders alt-tab karabiner mole mas dust duf font-jetbrains font-symbols" ;;
    rice) echo "rio zellij tmux claude bun fnm uv sketchybar stats aerospace borders alt-tab dockdoor karabiner raycast 1password 1password-cli vscode dia mole mas dust duf font-jetbrains font-symbols font-fira font-meslo font-monaspace" ;;
    webdev) echo "rio cmux orca zellij tmux claude codex opencode bun fnm uv docker docker-compose colima vscode chrome figma dia 1password 1password-cli cloudflare-warp raycast stats mole mas dust duf font-jetbrains font-symbols" ;;
    *) echo "" ;;
  esac
}

newmac_preset_toggles() {
  case "$1" in
    jack) echo "1 1 1 0 1" ;;
    basic) echo "0 1 1 0 1" ;;
    ai) echo "1 1 1 0 1" ;;
    rice) echo "1 1 1 0 1" ;;
    webdev) echo "0 1 1 0 1" ;;
    *) echo "1 1 1 0 1" ;;
  esac
}

newmac_preset_theme() {
  case "$1" in
    jack) echo "nord" ;;
    basic) echo "tokyonight" ;;
    ai) echo "tokyonight" ;;
    rice) echo "tokyonight" ;;
    webdev) echo "tokyonight" ;;
    *) echo "" ;;
  esac
}

newmac_preset_glass() {
  case "$1" in
    jack) echo "1" ;;
    basic) echo "0" ;;
    ai) echo "0" ;;
    rice) echo "0" ;;
    webdev) echo "0" ;;
    *) echo "0" ;;
  esac
}
