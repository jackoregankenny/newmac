#!/usr/bin/env bash
# ============================================================
#  macos-defaults.sh — opinionated, sane macOS UX defaults.
#  Everything here is reversible via System Settings.
# ============================================================
set -uo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

info "Closing System Settings to avoid overriding changes…"
osascript -e 'tell application "System Settings" to quit' >/dev/null 2>&1 || true

# --- Keyboard: fast key repeat (great for coding) --------------
defaults write NSGlobalDomain KeyRepeat -int 2
defaults write NSGlobalDomain InitialKeyRepeat -int 15
defaults write NSGlobalDomain ApplePressAndHoldEnabled -bool false   # repeat instead of accent popup
defaults write NSGlobalDomain AppleKeyboardUIMode -int 3             # full keyboard control

# --- Trackpad (tap to click — built-in + Bluetooth) ------------
defaults write com.apple.AppleMultitouchTrackpad Clicking -bool true
defaults write com.apple.driver.AppleBluetoothMultitouch.trackpad Clicking -bool true
defaults write NSGlobalDomain com.apple.mouse.tapBehavior -int 1

# --- Finder -----------------------------------------------------
defaults write NSGlobalDomain AppleShowAllExtensions -bool true
defaults write com.apple.finder AppleShowAllFiles -bool true          # show hidden files
defaults write com.apple.finder ShowPathbar -bool true
defaults write com.apple.finder ShowStatusBar -bool true
defaults write com.apple.finder FXPreferredViewStyle -string "Nlsv"   # list view
defaults write com.apple.finder _FXSortFoldersFirst -bool true
defaults write com.apple.finder FXDefaultSearchScope -string "SCcf"   # search current folder
defaults write com.apple.finder FXEnableExtensionChangeWarning -bool false
defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true  # no .DS_Store on network
chflags nohidden "$HOME/Library" 2>/dev/null || true

# --- Dock -------------------------------------------------------
defaults write com.apple.dock autohide -bool true
defaults write com.apple.dock autohide-delay -float 0
defaults write com.apple.dock autohide-time-modifier -float 0.3
defaults write com.apple.dock tilesize -int 44
defaults write com.apple.dock show-recents -bool false
defaults write com.apple.dock minimize-to-application -bool true
defaults write com.apple.dock mru-spaces -bool false                 # don't reorder spaces

# --- Screenshots -> ~/Screenshots, PNG, no shadow --------------
mkdir -p "$HOME/Screenshots"
defaults write com.apple.screencapture location -string "$HOME/Screenshots"
defaults write com.apple.screencapture type -string "png"
defaults write com.apple.screencapture disable-shadow -bool true

# --- Misc quality of life --------------------------------------
defaults write NSGlobalDomain NSNavPanelExpandedStateForSaveMode -bool true
defaults write NSGlobalDomain NSDocumentSaveNewDocumentsToCloud -bool false
defaults write NSGlobalDomain NSAutomaticCapitalizationEnabled -bool false
defaults write NSGlobalDomain NSAutomaticPeriodSubstitutionEnabled -bool false
defaults write com.apple.LaunchServices LSQuarantine -bool true      # keep the safety prompt

# --- Apply ------------------------------------------------------
info "Restarting Finder, Dock, SystemUIServer…"
killall Finder Dock SystemUIServer 2>/dev/null || true
printf "\033[32m ok\033[0m  macOS defaults applied. Some changes need a logout/restart.\n"
