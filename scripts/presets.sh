#!/usr/bin/env bash
# ============================================================
#  presets.sh — ready-made stacks. A preset just pre-selects
#  catalog ids (core is always included) + sets the toggles;
#  the picker still opens so you can tweak before installing.
#
#  Add your own: append a name to NEWMAC_PRESETS and extend the
#  three case statements. Ids must exist in catalog.sh.
#  Keep bash-3.2 compatible.
# ============================================================

NEWMAC_PRESETS="default minimal webdev ai rice"

newmac_preset_title() {
  case "$1" in
    default) echo "Balanced" ;;
    minimal) echo "Minimal" ;;
    webdev)  echo "Web dev" ;;
    ai)      echo "AI power user" ;;
    rice)    echo "Full rice" ;;
    *)       echo "$1" ;;
  esac
}

newmac_preset_desc() {
  case "$1" in
    default) echo "The author's daily setup — agents, tiling desktop, the works" ;;
    minimal) echo "Lean: Ghostty + Zellij + Claude + 1Password, no rice, no extras" ;;
    webdev)  echo "TypeScript/web: Bun, Node, containers, VS Code, Chrome, Figma" ;;
    ai)      echo "Every coding agent + agent terminals + full tiling desktop" ;;
    rice)    echo "Linux-style desktop first: tiling, bar, borders, all the fonts" ;;
    *)       echo "" ;;
  esac
}

# Selected catalog ids (core tools are always added automatically).
# "default" returns empty = use the catalog's own on/off defaults.
newmac_preset_ids() {
  case "$1" in
    minimal)
      echo "ghostty zellij claude 1password 1password-cli vscode bun fnm uv mole mas dust duf font-jetbrains font-symbols" ;;
    webdev)
      echo "ghostty rio cmux orca zellij tmux claude codex opencode bun fnm uv docker docker-compose colima vscode chrome figma dia 1password 1password-cli cloudflare-warp raycast stats mole mas dust duf font-jetbrains font-symbols" ;;
    ai)
      echo "ghostty rio cmux orca conductor nimbalyst zellij tmux claude codex droid opencode amp kimi crush goose cursor copilot aider qwen bun fnm uv go rust docker docker-compose colima vscode dia 1password 1password-cli cloudflare-warp raycast stats coconutbattery sketchybar aerospace borders alt-tab karabiner mole mas dust duf font-jetbrains font-symbols" ;;
    rice)
      echo "ghostty rio zellij tmux claude bun fnm uv sketchybar stats aerospace borders alt-tab dockdoor karabiner raycast 1password 1password-cli vscode dia mole mas dust duf font-jetbrains font-symbols font-fira font-meslo font-monaspace" ;;
    *) echo "" ;;
  esac
}

# Toggles: "ricing defaults power schedule dock" as 0/1, space-separated.
newmac_preset_toggles() {
  case "$1" in
    minimal) echo "0 1 1 0 1" ;;
    webdev)  echo "0 1 1 0 1" ;;
    ai)      echo "1 1 1 0 1" ;;
    rice)    echo "1 1 1 0 1" ;;
    *)       echo "1 1 1 0 1" ;;
  esac
}
