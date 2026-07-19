#!/usr/bin/env bash
# ============================================================
#  tui.sh — tiny dependency-free TUI widgets (checkbox picker,
#  yes/no) in pure bash. Runs on stock macOS bash 3.2, before
#  Homebrew or anything else exists.
#
#  Usage (multiselect):
#    TUI_TITLE="Pick things"
#    TUI_ITEMS=(one two);  TUI_DESCS=(first second);  TUI_STATE=(1 0)
#    tui_multiselect            # updates TUI_STATE in place
#
#  Keys: ↑/↓ or j/k move · space toggle · a all · n none · enter confirm
# ============================================================

_tui_cleanup() { printf '\033[?25h'; stty echo 2>/dev/null || true; }

_tui_win_size() {
  local lines
  lines=$(tput lines 2>/dev/null) || lines=24
  [[ -z "$lines" ]] && lines=24
  TUI_WIN=$((lines - 7))
  [[ $TUI_WIN -lt 4 ]] && TUI_WIN=4
  [[ $TUI_WIN -gt ${#TUI_ITEMS[@]} ]] && TUI_WIN=${#TUI_ITEMS[@]}
}

_tui_draw() {
  local i end mark cursor namecol name desc count=0 total=${#TUI_ITEMS[@]} sel=0 step=""
  for i in "${TUI_STATE[@]}"; do [[ "$i" == 1 ]] && sel=$((sel+1)); done
  [[ -n "${TUI_STEP:-}" ]] && step="  ${c_dim}·  step $TUI_STEP$c_reset"
  printf '\033[K %s◆%s %s%s%s%s  %s%d/%d selected%s\n' \
    "$c_mauve" "$c_reset" "$c_bold" "$TUI_TITLE" "$c_reset" "$step" "$c_dim" "$sel" "$total" "$c_reset"
  printf '\033[K   %s%s%s\n' "$c_dim" "${TUI_HINT:-↑/↓ move · space toggle · a all · n none · enter continue}" "$c_reset"
  end=$((TUI_TOP + TUI_WIN)); [[ $end -gt $total ]] && end=$total
  i=$TUI_TOP
  while [[ $i -lt $end ]]; do
    if [[ "${TUI_STATE[$i]}" == 1 ]]; then mark="${c_green}●${c_reset}"; else mark="${c_dim}○${c_reset}"; fi
    if [[ $i -eq $TUI_CUR ]]; then cursor="${c_mauve}❯${c_reset}"; namecol="$c_bold$c_mauve"; else cursor=" "; namecol="$c_bold"; fi
    name="${TUI_ITEMS[$i]}"; desc="${TUI_DESCS[$i]}"
    printf '\033[K %s %s %s%-18s%s %s%s%s\n' \
      "$cursor" "$mark" "$namecol" "$name" "$c_reset" "$c_dim" "$desc" "$c_reset"
    i=$((i+1)); count=$((count+1))
  done
  if [[ $TUI_WIN -lt $total ]]; then
    printf '\033[K%s   … %d more (scroll with ↑/↓)%s\n' "$c_dim" "$((total - TUI_WIN))" "$c_reset"
    count=$((count+1))
  fi
  TUI_DRAWN=$((count + 2))
}

tui_multiselect() {
  local key rest i total=${#TUI_ITEMS[@]}
  TUI_HINT=""
  TUI_CUR=0; TUI_TOP=0; TUI_DRAWN=0
  _tui_win_size
  # Non-interactive: keep defaults untouched.
  if [[ ! -t 0 || ! -t 1 ]]; then return 0; fi
  trap '_tui_cleanup; exit 130' INT
  printf '\033[?25l'
  _tui_draw
  while true; do
    IFS= read -rsn1 key || break
    if [[ "$key" == $'\x1b' ]]; then
      read -rsn2 -t 1 rest || rest=""
      key="$key$rest"
    fi
    case "$key" in
      $'\x1b[A'|k) TUI_CUR=$((TUI_CUR - 1)); [[ $TUI_CUR -lt 0 ]] && TUI_CUR=$((total - 1)) ;;
      $'\x1b[B'|j) TUI_CUR=$((TUI_CUR + 1)); [[ $TUI_CUR -ge $total ]] && TUI_CUR=0 ;;
      ' ') if [[ "${TUI_STATE[$TUI_CUR]}" == 1 ]]; then TUI_STATE[$TUI_CUR]=0; else TUI_STATE[$TUI_CUR]=1; fi ;;
      a|A) i=0; while [[ $i -lt $total ]]; do TUI_STATE[$i]=1; i=$((i+1)); done ;;
      n|N) i=0; while [[ $i -lt $total ]]; do TUI_STATE[$i]=0; i=$((i+1)); done ;;
      "") break ;;
    esac
    # keep cursor inside the visible window
    [[ $TUI_CUR -lt $TUI_TOP ]] && TUI_TOP=$TUI_CUR
    [[ $TUI_CUR -ge $((TUI_TOP + TUI_WIN)) ]] && TUI_TOP=$((TUI_CUR - TUI_WIN + 1))
    printf '\033[%dA' "$TUI_DRAWN"
    _tui_draw
  done
  printf '\033[?25h'
  trap - INT
  printf '\n'
}

# tui_select — single choice (radio). Same globals as multiselect;
# TUI_CHOICE gets the chosen index (starts at its initial value or 0).
tui_select() {
  local key rest i total=${#TUI_ITEMS[@]}
  [[ -z "${TUI_CHOICE:-}" ]] && TUI_CHOICE=0
  TUI_HINT="↑/↓ move · enter choose"
  TUI_CUR=$TUI_CHOICE; TUI_TOP=0; TUI_DRAWN=0
  _tui_win_size
  if [[ ! -t 0 || ! -t 1 ]]; then return 0; fi
  trap '_tui_cleanup; exit 130' INT
  printf '\033[?25l'
  # reuse the multiselect renderer: exactly one box checked
  i=0; while [[ $i -lt $total ]]; do TUI_STATE[$i]=0; i=$((i+1)); done
  TUI_STATE[$TUI_CUR]=1
  _tui_draw
  while true; do
    IFS= read -rsn1 key || break
    if [[ "$key" == $'\x1b' ]]; then
      read -rsn2 -t 1 rest || rest=""
      key="$key$rest"
    fi
    case "$key" in
      $'\x1b[A'|k) TUI_CUR=$((TUI_CUR - 1)); [[ $TUI_CUR -lt 0 ]] && TUI_CUR=$((total - 1)) ;;
      $'\x1b[B'|j) TUI_CUR=$((TUI_CUR + 1)); [[ $TUI_CUR -ge $total ]] && TUI_CUR=0 ;;
      ""|' ') break ;;
    esac
    i=0; while [[ $i -lt $total ]]; do TUI_STATE[$i]=0; i=$((i+1)); done
    TUI_STATE[$TUI_CUR]=1
    [[ $TUI_CUR -lt $TUI_TOP ]] && TUI_TOP=$TUI_CUR
    [[ $TUI_CUR -ge $((TUI_TOP + TUI_WIN)) ]] && TUI_TOP=$((TUI_CUR - TUI_WIN + 1))
    printf '\033[%dA' "$TUI_DRAWN"
    _tui_draw
  done
  TUI_CHOICE=$TUI_CUR
  printf '\033[?25h'
  trap - INT
  printf '\n'
}

# tui_yesno "Question" <default y|n>  -> return 0 for yes
tui_yesno() {
  local q="$1" def="${2:-y}" hint a
  if [[ ! -t 0 ]]; then [[ "$def" == y ]]; return; fi
  if [[ "$def" == y ]]; then hint="[Y/n]"; else hint="[y/N]"; fi
  read -r -p "$(printf '%s ?? %s%s %s ' "$c_yellow" "$q" "$c_reset" "$hint")" a
  [[ -z "$a" ]] && a="$def"
  [[ "$a" == [yY]* ]]
}
