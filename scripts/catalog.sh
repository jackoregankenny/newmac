#!/usr/bin/env bash
# ============================================================
#  catalog.sh — the single source of truth for everything the
#  installer can set up. Sourced by configure/install/status.
#
#  Format (pipe-separated, one item per line):
#    id|category|kind|default|payload|name|description
#
#  kind:
#    brew  — Homebrew formula   (payload = formula, taps auto-derived)
#    cask  — Homebrew cask      (payload = cask,    taps auto-derived)
#    curl  — vendor install script piped to bash (payload = URL)
#    npm   — global JS package  (payload = package; installed via bun/npm)
#    uv    — Python CLI tool    (payload = package; installed via `uv tool`)
#    mas   — Mac App Store app  (payload = <id>:<name>; needs App Store sign-in)
#    rustup— special-cased Rust toolchain install
#
#  category `core` is always installed and never shown in the picker.
#  Keep this file bash-3.2 compatible.
#  IMPORTANT: the catalog below is one single-quoted string — no
#  apostrophes (') anywhere in names/descriptions, or it breaks.
# ============================================================

NEWMAC_CATALOG='
# --- Core shell & CLI (always installed) ------------------------
starship|core|brew|on|starship|starship|Fast cross-shell prompt
zoxide|core|brew|on|zoxide|zoxide|Smarter cd — `z <dir>`
fzf|core|brew|on|fzf|fzf|Fuzzy finder
eza|core|brew|on|eza|eza|Modern ls
bat|core|brew|on|bat|bat|Modern cat with syntax highlighting
ripgrep|core|brew|on|ripgrep|ripgrep|Fast grep (rg)
fd|core|brew|on|fd|fd|Fast find
git-delta|core|brew|on|git-delta|delta|Beautiful git diffs
zsh-autosuggestions|core|brew|on|zsh-autosuggestions|zsh-autosuggestions|Fish-style suggestions
zsh-syntax-highlighting|core|brew|on|zsh-syntax-highlighting|zsh-syntax-highlighting|Command highlighting
git|core|brew|on|git|git|Version control
gh|core|brew|on|gh|gh|GitHub CLI
jq|core|brew|on|jq|jq|JSON processor
yq|core|brew|on|yq|yq|YAML processor
wget|core|brew|on|wget|wget|Downloader
tree|core|brew|on|tree|tree|Directory trees
btop|core|brew|on|btop|btop|Resource monitor
macmon|core|brew|on|macmon|macmon|Apple-silicon power/temp monitor

# --- Terminals --------------------------------------------------
ghostty|terminals|cask|on|ghostty|Ghostty|Fast native GPU terminal — themed config included
rio|terminals|cask|on|rio|Rio|Light GPU terminal, RetroArch shaders
wezterm|terminals|cask|off|wezterm|WezTerm|GPU terminal configured in Lua
alacritty|terminals|cask|off|alacritty|Alacritty|Minimal fast GPU terminal
kitty|terminals|cask|off|kitty|kitty|Feature-rich GPU terminal
warp|terminals|cask|off|warp|Warp|AI-native terminal (account required)
iterm2|terminals|cask|off|iterm2|iTerm2|The classic macOS terminal

# --- Multiplexers ----------------------------------------------
zellij|multiplexers|brew|on|zellij|Zellij|Modern multiplexer — sessions/splits/layouts
tmux|multiplexers|brew|on|tmux|tmux|Classic multiplexer (SSH ubiquity)

# --- Shells (zsh + plugins is the wired-up default) ------------
fish|shells|brew|off|fish|fish|Great defaults, but not POSIX — the newmac rice is zsh-wired
nushell|shells|brew|off|nushell|Nushell|Structured-data shell — pipelines of tables, not text

# --- AI coding agents ------------------------------------------
claude|agents|curl|on|https://claude.ai/install.sh|Claude Code|Anthropic agentic CLI
codex|agents|cask|on|codex|Codex|OpenAI Codex CLI
droid|agents|curl|on|https://app.factory.ai/cli|Factory Droid|Factory agentic CLI
opencode|agents|brew|on|sst/tap/opencode|opencode|Open-source agentic CLI (SST)
amp|agents|curl|off|https://ampcode.com/install.sh|Amp|Sourcegraph agentic CLI
kimi|agents|curl|off|https://code.kimi.com/kimi-code/install.sh|Kimi Code|Moonshot agentic CLI (K-series models)
crush|agents|brew|off|charmbracelet/tap/crush|Crush|Charm glamorous TUI coding agent
goose|agents|brew|off|block-goose-cli|Goose|Block/Linux Foundation open agent
cursor|agents|curl|off|https://cursor.com/install|Cursor CLI|Cursor agent in the terminal
copilot|agents|npm|off|@github/copilot|Copilot CLI|GitHub Copilot coding agent
aider|agents|uv|off|aider-chat|Aider|Open-source pair programmer
qwen|agents|npm|off|@qwen-code/qwen-code|Qwen Code|Alibaba Qwen coding CLI
gemini|agents|brew|off|gemini-cli|Gemini CLI|Google (deprecated in brew — successor: antigravity-cli)

# --- Agent workbenches ------------------------------------------
orca|workbench|cask|on|stablyai/orca/orca|Orca|ADE — fleet of parallel agents in worktrees, browser + editor built in
cmux|workbench|cask|on|cmux|cmux|Agent terminal for multi-agent work (libghostty)
conductor|workbench|cask|off|conductor|Conductor|Parallel Claude/Codex/Cursor agents in worktrees (Melty Labs)
nimbalyst|workbench|cask|off|nimbalyst|Nimbalyst|Visual workspace for Claude Code + Codex (was Crystal)

# --- Menu bar ---------------------------------------------------
sketchybar|menubar|brew|on|FelixKratz/formulae/sketchybar|SketchyBar|Scriptable status bar (configured by this repo)
barik|menubar|cask|off|mocki-toki/formulae/barik|barik|Modern SwiftUI menu bar — TOML config, AeroSpace-aware
ice|menubar|cask|off|jordanbaird-ice|Ice|Menu bar manager — hide/rearrange icons (Bartender alternative)
stats|menubar|cask|on|stats|Stats|Menu-bar system monitor

# --- Tiling & window management --------------------------------
aerospace|tiling|cask|on|nikitabobko/tap/aerospace|AeroSpace|i3-like tiling WM — no SIP disable (the unixporn favourite)
yabai|tiling|brew|off|koekeishiya/formulae/yabai|yabai|Most powerful tiling WM (full features need SIP disable)
amethyst|tiling|cask|off|amethyst|Amethyst|Auto-tiling with a native feel, zero config (no SIP disable)
rectangle|tiling|cask|off|rectangle|Rectangle|Keyboard window snapping — light-touch, exec friendly
loop|tiling|cask|off|loop|Loop|Radial window snapping — pretty, exec friendly
borders|tiling|brew|on|FelixKratz/formulae/borders|JankyBorders|Focused-window border
alt-tab|tiling|cask|on|alt-tab|AltTab|Windows-style window-level alt-tab
dockdoor|tiling|cask|off|dockdoor|DockDoor|Dock hover previews + another alt-tab flavour
karabiner|tiling|cask|on|karabiner-elements|Karabiner|Keyboard remap (Caps → ⌘ held / Esc tapped)
raycast|tiling|cask|on|raycast|Raycast|Launcher / window management

# --- Dev runtimes & containers ---------------------------------
bun|runtimes|brew|on|oven-sh/bun/bun|Bun|JS/TS runtime + package manager
fnm|runtimes|brew|on|fnm|fnm|Fast Node version manager (installs Node LTS)
uv|runtimes|brew|on|uv|uv|Python package/venv manager
go|runtimes|brew|on|go|Go|Go toolchain
rust|runtimes|rustup|on|rustup|Rust|rustup + stable toolchain (the canonical way)
docker|runtimes|brew|on|docker|docker CLI|Docker CLI (engine via Colima/OrbStack)
docker-compose|runtimes|brew|on|docker-compose|docker-compose|Compose v2
colima|runtimes|brew|on|colima|Colima|Free container runtime — no Docker Desktop licence
orbstack|runtimes|cask|off|orbstack|OrbStack|Fast Docker Desktop alternative (free for personal use)

# --- Dev apps (GUI) --------------------------------------------
vscode|devgui|cask|on|visual-studio-code|VS Code|Editor
cursor-ide|devgui|cask|off|cursor|Cursor|AI editor (the app; CLI agent is separate)
zed|devgui|cask|off|zed|Zed|Fast collaborative editor
github-desktop|devgui|cask|off|github|GitHub Desktop|Git GUI
tableplus|devgui|cask|off|tableplus|TablePlus|Database GUI
bruno|devgui|cask|off|bruno|Bruno|Open-source API client
postman|devgui|cask|off|postman|Postman|API client
utm|devgui|cask|off|utm|UTM|Virtual machines on Apple Silicon
xcode|devgui|mas|off|497799835:Xcode|Xcode|Full IDE + simulators (App Store, huge)

# --- Browsers ---------------------------------------------------
dia|browsers|cask|on|thebrowsercompany-dia|Dia|The Browser Company
arc|browsers|cask|off|arc|Arc|The Browser Company (classic)
chrome|browsers|cask|off|google-chrome|Chrome|Google
firefox|browsers|cask|off|firefox|Firefox|Mozilla
brave|browsers|cask|off|brave-browser|Brave|Chromium + adblock
zen|browsers|cask|off|zen|Zen|Firefox-based, Arc-style UI
edge|browsers|cask|off|microsoft-edge|Edge|Microsoft (work-policy friendly)

# --- Essentials & productivity ---------------------------------
1password|productivity|cask|on|1password|1Password|Password manager (app)
1password-cli|productivity|cask|on|1password-cli|1Password CLI|`op` — terminal + SSH agent + shell plugins
coconutbattery|productivity|cask|on|coconutbattery|coconutBattery|Battery health / cycle count
obsidian|productivity|cask|off|obsidian|Obsidian|Markdown notes
notion|productivity|cask|off|notion|Notion|Docs / wiki
figma|productivity|cask|off|figma|Figma|Design
linear|productivity|cask|off|linear-linear|Linear|Issue tracking
todoist|productivity|cask|off|todoist|Todoist|Tasks
keka|productivity|cask|off|keka|Keka|Archiver
shottr|productivity|cask|off|shottr|Shottr|Screenshot annotation
amphetamine|productivity|mas|off|937984704:Amphetamine|Amphetamine|Keep the Mac awake (App Store)

# --- Comms & media ----------------------------------------------
slack|comms|cask|off|slack|Slack|Team chat
discord|comms|cask|off|discord|Discord|Chat
zoom|comms|cask|off|zoom|Zoom|Video calls
whatsapp|comms|cask|off|whatsapp|WhatsApp|Messaging
telegram|comms|cask|off|telegram|Telegram|Messaging
spotify|comms|cask|off|spotify|Spotify|Music
spotify-player|comms|brew|off|spotify_player|spotify_player|TUI Spotify client — rice-friendly (Premium required)
iina|comms|cask|off|iina|IINA|Video player
vlc|comms|cask|off|vlc|VLC|Plays anything

# --- Office & work ----------------------------------------------
teams|work|cask|on|microsoft-teams|Microsoft Teams|Work chat & calls
outlook|work|cask|on|microsoft-outlook|Outlook|Work email & calendar
word|work|cask|on|microsoft-word|Word|Documents
excel|work|cask|on|microsoft-excel|Excel|Spreadsheets
powerpoint|work|cask|on|microsoft-powerpoint|PowerPoint|Slides
onedrive|work|cask|off|onedrive|OneDrive|Microsoft cloud storage
libreoffice|work|cask|off|libreoffice|LibreOffice|Free & open Office suite
onlyoffice|work|cask|off|onlyoffice|OnlyOffice|Best Office-format compatibility (open source)

# --- Network & VPN ----------------------------------------------
cloudflare-warp|network|cask|on|cloudflare-warp|Cloudflare WARP|Encrypted DNS / VPN
tailscale|network|cask|on|tailscale-app|Tailscale|Zero-config WireGuard mesh — your devices, one network
mullvad|network|cask|off|mullvad-vpn|Mullvad|Privacy VPN (no account details)
protonvpn|network|cask|off|protonvpn|Proton VPN|Privacy VPN
wireguard|network|brew|off|wireguard-tools|WireGuard tools|wg / wg-quick CLI

# --- Local AI ---------------------------------------------------
ollama|localai|brew|off|ollama|Ollama|Run local LLMs from the terminal
lm-studio|localai|cask|off|lm-studio|LM Studio|Local LLM GUI — download & chat with open models

# --- Maintenance & cleanup -------------------------------------
mole|maintenance|brew|on|mole|Mole|Clean/uninstall/analyze/monitor from the terminal (run: mo)
dockutil|maintenance|brew|on|dockutil|dockutil|Dock layout CLI — powers the Dock arranger
mas|maintenance|brew|on|mas|mas|Mac App Store CLI — script App Store installs
dust|maintenance|brew|on|dust|dust|Visual disk usage (better du)
duf|maintenance|brew|on|duf|duf|Pretty disk free (better df)
topgrade|maintenance|brew|off|topgrade|Topgrade|Upgrade everything in one command (overlaps update.sh)
pearcleaner|maintenance|cask|off|pearcleaner|Pearcleaner|Open-source app uninstaller (GUI)
onyx|maintenance|cask|off|onyx|OnyX|Veteran macOS maintenance & tweaks GUI

# --- Fonts ------------------------------------------------------
font-jetbrains|fonts|cask|on|font-jetbrains-mono-nerd-font|JetBrains Mono NF|Default terminal font
font-symbols|fonts|cask|on|font-symbols-only-nerd-font|Symbols NF|Icons for sketchybar etc.
font-fira|fonts|cask|off|font-fira-code-nerd-font|Fira Code NF|Ligature classic
font-meslo|fonts|cask|off|font-meslo-lg-nerd-font|Meslo LG NF|Powerlevel-style font
font-monaspace|fonts|cask|off|font-monaspace|Monaspace|GitHub superfamily
'

# Category ids (display order) + human titles — parallel "arrays"
# kept as simple space/newline lists for bash 3.2.
NEWMAC_CATEGORIES="terminals multiplexers shells agents workbench localai menubar tiling runtimes devgui browsers productivity work comms network maintenance fonts"
newmac_category_title() {
  case "$1" in
    terminals)    echo "Terminals" ;;
    multiplexers) echo "Multiplexers" ;;
    shells)       echo "Shells (zsh is the default)" ;;
    agents)       echo "AI coding agents" ;;
    workbench)    echo "Agent workbenches" ;;
    localai)      echo "Local AI" ;;
    work)         echo "Office & work" ;;
    network)      echo "Network & VPN" ;;
    menubar)      echo "Menu bar" ;;
    tiling)       echo "Tiling & window management" ;;
    runtimes)     echo "Dev runtimes & containers" ;;
    devgui)       echo "Dev apps" ;;
    browsers)     echo "Browsers" ;;
    productivity) echo "Essentials & productivity" ;;
    comms)        echo "Comms & media" ;;
    maintenance)  echo "Maintenance & cleanup" ;;
    fonts)        echo "Fonts" ;;
    *)            echo "$1" ;;
  esac
}

# --- Parse the catalog into parallel arrays --------------------
CAT_ID=(); CAT_CATEGORY=(); CAT_KIND=(); CAT_DEFAULT=(); CAT_PAYLOAD=(); CAT_NAME=(); CAT_DESC=()
_newmac_parse_catalog() {
  local id cat kind def payload name desc n=0
  while IFS='|' read -r id cat kind def payload name desc; do
    [[ -z "$id" ]] && continue
    case "$id" in \#*) continue ;; esac
    CAT_ID[n]="$id"; CAT_CATEGORY[n]="$cat"; CAT_KIND[n]="$kind"
    CAT_DEFAULT[n]="$def"; CAT_PAYLOAD[n]="$payload"; CAT_NAME[n]="$name"; CAT_DESC[n]="$desc"
    n=$((n+1))
  done <<EOF
$NEWMAC_CATALOG
EOF
}
_newmac_parse_catalog
