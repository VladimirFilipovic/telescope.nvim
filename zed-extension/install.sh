#!/usr/bin/env bash
# Links telescope-zed and the MCP server into ~/.local/bin and reports what is still missing.
# Safe to run again: it only refreshes the links and rebuilds.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin="${TELESCOPE_ZED_PREFIX:-$HOME/.local/bin}"
zed_config="${XDG_CONFIG_HOME:-$HOME/.config}/zed"
mkdir -p "$bin"

link() {
  ln -sfn "$1" "$bin/$2"
  echo "linked   $bin/$2"
}

chmod +x "$here/tasks/bin/telescope-zed"
link "$here/tasks/bin/telescope-zed" telescope-zed

if command -v cargo >/dev/null 2>&1; then
  cargo build --quiet --release --manifest-path "$here/mcp-server/Cargo.toml"
  link "$here/mcp-server/target/release/telescope-zed-mcp" telescope-zed-mcp
else
  echo "skipped  telescope-zed-mcp (cargo not found)"
fi

case ":$PATH:" in
  *":$bin:"*) ;;
  *) echo "warning  $bin is not on PATH; Zed tasks will not find telescope-zed" ;;
esac

for tool in fzf rg fd bat git zed; do
  if command -v "$tool" >/dev/null 2>&1; then
    echo "ok       $tool"
  elif [[ "$tool" == zed ]]; then
    echo "missing  zed CLI (in Zed run \"cli: install\")"
  else
    echo "missing  $tool (brew install $tool)"
  fi
done

# The picker layout uses fzf options that arrived in 0.63.
if command -v fzf >/dev/null 2>&1; then
  version="$(fzf --version | awk '{ print $1 }')"
  minor="${version#*.}"
  minor="${minor%%.*}"
  if [[ "${version%%.*}" == 0 && "$minor" =~ ^[0-9]+$ && "$minor" -lt 63 ]]; then
    echo "warning  fzf $version is too old, 0.63 or newer is needed (brew upgrade fzf)"
  fi
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "note     Zed builds dev extensions only with rustup-managed Rust (brew install rustup)"
fi

# Says whether a config file already carries the telescope entries.
merged() {
  if [[ -f "$zed_config/$1" ]] && grep -q "Telescope: find_files" "$zed_config/$1"; then
    echo "present  Telescope entries in $zed_config/$1"
  else
    echo "todo     merge $here/tasks/$1 into $zed_config/$1"
  fi
}
merged tasks.json
merged keymap.json
echo "todo     in Zed, once: \"zed: install dev extension\" -> $here/extension"
