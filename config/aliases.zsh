# ~/.config/zsh/aliases.zsh — sourced by zshrc

# --- Modern CLI replacements ------------------------------------------------
alias ls='eza --group-directories-first --icons=auto'
alias ll='eza -lah --group-directories-first --icons=auto --git'
alias la='eza -a  --group-directories-first --icons=auto'
alias lt='eza --tree --level=2 --icons=auto'
alias cat='bat --paging=never'
alias catp='bat'                       # paged cat
alias top='btop'

# --- Git --------------------------------------------------------------------
alias g='git'
alias gs='git status -sb'
alias ga='git add'
alias gaa='git add -A'
alias gc='git commit'
alias gcm='git commit -m'
alias gp='git push'
alias gl='git pull'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
alias glog='git log --oneline --graph --decorate -20'

# --- Navigation -------------------------------------------------------------
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'

# --- Safety nets ------------------------------------------------------------
alias rm='rm -i'
alias cp='cp -i'
alias mv='mv -i'

# --- Battery / system monitoring -------------------------------------------
alias power='macmon'                   # live Apple-silicon power/temp (Ctrl-C to quit)
alias batt='pmset -g batt'             # quick battery % + charging state
alias temps='sudo powermetrics --samplers smc -i1 -n1 2>/dev/null | grep -i temp'

# --- Maintenance ------------------------------------------------------------
alias brewup='brew update && brew upgrade && brew cleanup'
alias status='bash "${NEWMAC:-$HOME/newmac}/scripts/status.sh"'   # what's installed + versions
alias macup='bash "${NEWMAC:-$HOME/newmac}/update.sh"'            # update everything
alias reload='exec zsh'
alias keys='bash "${NEWMAC:-$HOME/newmac}/scripts/keys.sh"'      # hotkey cheat-sheet (also ⌥⇧/)
alias myip='curl -s ifconfig.me; echo'
alias path='echo $PATH | tr ":" "\n"'
