//! Module containing all aliases and functions used for shell integration

/// Shell aliases to load and unload environment variables directly in the current shell
pub const ALIASES: &str = r##"keycli-load() { if [[ " $* " == *" -h "* ]] || [[ " $* " == *" --help "* ]]; then keycli load "$@"; else eval $(keycli load "$@"); fi }
keycli-unload() { if [[ " $* " == *" -h "* ]] || [[ " $* " == *" --help "* ]]; then keycli unload "$@"; else eval $(keycli unload "$@"); fi }"##;

/// Bash completion for keycli-load
pub const BASH_LOAD: &str = r##"_keycli_load() {
  COMP_WORDS=(keycli load "${COMP_WORDS[@]:1}")
  COMP_CWORD=$((COMP_CWORD+1))
  _keycli keycli "${COMP_WORDS[COMP_CWORD]}" "${COMP_WORDS[COMP_CWORD-1]}"
}
if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _keycli_load -o nosort -o bashdefault -o default keycli-load
else
    complete -F _keycli_load -o bashdefault -o default keycli-load
fi"##;

/// Bash completion for keycli-unload
pub const BASH_UNLOAD: &str = r##"_keycli_unload() {
  COMP_WORDS=(keycli unload "${COMP_WORDS[@]:1}")
  COMP_CWORD=$((COMP_CWORD+1))
  _keycli keycli "${COMP_WORDS[COMP_CWORD]}" "${COMP_WORDS[COMP_CWORD-1]}"
}
if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _keycli_unload -o nosort -o bashdefault -o default keycli-unload
else
    complete -F _keycli_unload -o bashdefault -o default keycli-unload
fi"##;

/// Zsh completion for keycli-load
pub const ZSH_LOAD: &str = r##"#compdef keycli-load
_keycli_load() { words=(keycli load ${words[2,-1]}) CURRENT=$((CURRENT+1)); _keycli; }
_keycli_load"##;

/// Zsh completion for keycli-unload
pub const ZSH_UNLOAD: &str = r##"#compdef keycli-unload
_keycli_unload() { words=(keycli unload ${words[2,-1]}) CURRENT=$((CURRENT+1)); _keycli; }
_keycli_unload"##;
