# telescope.nvim for Zed

Zed extensions cannot draw a picker, so telescope.nvim cannot be ported as an
extension. This folder is the closest working equivalent:

| Part | What it does | Where |
|---|---|---|
| Pickers | `telescope-zed`: fzf pickers with rg, fd and bat, run as Zed tasks in a center terminal tab. The pick opens in Zed through `zed <file>:<line>:<col>`. | `tasks/` |
| Keymap | `space p …` bindings: picker tasks, plus Zed's own pickers where Zed already has the builtin. | `tasks/keymap.json` |
| Agent tools | A Zed extension that registers an MCP (Model Context Protocol) server with `find_files`, `live_grep`, `grep_string`, `git_status`, `git_commits` and `todo` for the Agent panel. | `extension/`, `mcp-server/` |

## Install

Needs `fzf` 0.63+, `rg`, `fd`, `bat`, `git` and the `zed` CLI (command-line interface).

```sh
./install.sh
```

It links `telescope-zed` and `telescope-zed-mcp` into `~/.local/bin`, builds the MCP server, and lists what is missing. Run it again any time. Then, once:

1. Merge `tasks/tasks.json` into `~/.config/zed/tasks.json`.
2. Merge `tasks/keymap.json` into `~/.config/zed/keymap.json`.
3. For the agent tools: in Zed run `zed: install dev extension` and pick `extension/`. Zed builds it with rustup-managed Rust (`brew install rustup`).

The script is linked, not copied, so edits to `tasks/bin/telescope-zed` apply at once.

## Pickers

A picker opens as a terminal tab in the center pane. Enter opens the pick in the same Zed window and the tab closes; Esc closes it. Colors are Gruvbox, light or dark after the macOS appearance.

| Key | Action |
|---|---|
| `enter` | open the entry, or every marked entry |
| `tab` / `ctrl-a` | mark an entry / mark all |
| `ctrl-j` / `ctrl-k` | move down / up |
| `ctrl-u` / `ctrl-d` | scroll the preview |
| `ctrl-/` | show or hide the preview |
| `ctrl-h` | show or hide dotfiles (`find_files`, `oldfiles`, `live_grep`) |
| `ctrl-f` | `live_grep`: fuzzy-filter the current matches |
| `ctrl-s` | `git_status`: stage or unstage |
| `ctrl-y` | `git_commits`, `git_bcommits`: copy the hash |

The preview sits on the right and moves below the list in a narrow pane. `.gitignore` is always respected.

| Variable | Effect |
|---|---|
| `TELESCOPE_ZED_THEME=light\|dark` | fixed palette |
| `TELESCOPE_ZED_HIDDEN=1` | start with dotfiles shown |
| `TELESCOPE_ZED_DRY_RUN=1` | print `OPEN <target>` instead of calling `zed` |
| `TELESCOPE_ZED_BIN=<path>` | `zed` CLI to use |

## Telescope builtin → Zed

| telescope builtin | Zed native | Here (`space p` key) |
|---|---|---|
| `find_files` | `file_finder::Toggle` | task, multi-select (`space p f`); MCP `find_files` |
| `git_files` | `file_finder::Toggle` | task (`space p F`) |
| `oldfiles` | file finder with an empty query | task, sorted by modification time (`space p o`) |
| `live_grep` | `pane::DeploySearch` | task, rg on each keystroke (`space p g`); MCP `live_grep` |
| `grep_string` | `pane::DeploySearch` with the selection | task, selection or word under the cursor (`space p w`); MCP `grep_string` |
| `current_buffer_fuzzy_find` | `buffer_search::Deploy`, not fuzzy | task, starts on the cursor line (`space p /`) |
| `git_status` | git panel | task, diff preview (`space p s`); MCP `git_status` |
| `git_commits` | none | task, Enter lists the commit's files (`space p c`); MCP `git_commits` |
| `git_bcommits` | none (`git::Blame`) | task for the current file (`space p C`); MCP `git_commits` with `file` |
| `git_branches` | `git::Branch` | task, Enter switches (`space p B`) |
| `git_stash` | `git_picker::ActivateStashTab` | — |
| `buffers` | `tab_switcher::Toggle` | `space p b` → native |
| `diagnostics` | `diagnostics::Deploy` | `space p d` → native |
| `lsp_references` | `editor::FindAllReferences` | `space p r` → native |
| `lsp_document_symbols`, `treesitter` | `outline::Toggle` | `space p S` → native |
| `lsp_workspace_symbols` | `project_symbols::Toggle` | `space p W` → native |
| `lsp_definitions`, `lsp_implementations`, `lsp_type_definitions` | `editor::GoToDefinition` and siblings | — |
| `commands` | `command_palette::Toggle` | `space p :` → native |
| `keymaps` | `zed::OpenKeymap` | `space p k` → native |
| `colorscheme` | `theme_selector::Toggle` | — |
| `resume` | `task::Rerun`, without the typed query | `space p .` |
| `builtin` | `task::Spawn`, type "Telescope" | `space p T` |
| `help_tags`, `man_pages`, `vim_options`, `registers`, `marks`, `jumplist`, `quickfix`, `loclist`, `spell_suggest`, `autocommands`, `highlights`, `filetypes`, `reloader`, `pickers` | none | not ported |
| extra: todo-comments | — | task `todo` (`space p t`); MCP `todo` |

## Agent tools

The `telescope` context server starts `telescope-zed-mcp` from PATH. Each tool takes an optional `path` (project root); without it the server uses `root` from the settings, then its working directory.

```jsonc
"context_servers": {
  "telescope": {
    "command": { "path": "/Users/you/.local/bin/telescope-zed-mcp" },
    "settings": { "root": "/path/to/project" }
  }
}
```

## What a Zed extension can and cannot do

Checked against Zed 1.29.1 and `zed_extension_api` 0.7.0 (November 2026).

| | |
|---|---|
| Can | languages, grammars, language servers; themes, icon themes, snippets; MCP context servers; debug adapters; language model providers |
| Cannot | draw UI (pickers, panels, modals, previews); register actions, commands or keybindings; open a file, move the cursor, or read buffers, selection or diagnostics |
| Dead | slash commands: still in the API, never called since [zed#52757](https://github.com/zed-industries/zed/pull/52757) |

Extensions are wasm32-wasip2 modules limited to the host functions of the WIT (WebAssembly Interface Types) world: process exec, HTTP, downloads, settings, worktree file reads. Sources: [extension docs](https://zed.dev/docs/extensions), [capabilities](https://zed.dev/docs/extensions/capabilities), [tasks](https://zed.dev/docs/tasks), [extension.wit](https://github.com/zed-industries/zed/blob/main/crates/extension_api/wit/since_v0.8.0/extension.wit), discussions [#6679](https://github.com/zed-industries/zed/discussions/6679) and [#27163](https://github.com/zed-industries/zed/discussions/27163).

## Known gaps

- A picker is a terminal tab, not a floating window, and the preview is bat output, not a Zed buffer.
- The `zed` CLI cannot open a file in a split, so there is no `ctrl-v` / `ctrl-x`.
- Pickers read files from disk; unsaved buffer contents are not visible to a task.
- `oldfiles` sorts by modification time; Zed's open history is not available to scripts.
- A path containing `:` cannot be opened from the grep pickers.
- The tasks pass `${ZED_SELECTED_TEXT:}` with an empty default so they stay listed when nothing is selected.

## Development

```sh
shellcheck tasks/bin/telescope-zed install.sh
(cd mcp-server && cargo clippy --all-targets -- -D warnings && cargo test)
(cd extension && cargo build --release --target wasm32-wasip2)
TELESCOPE_ZED_DRY_RUN=1 tasks/bin/telescope-zed find_files   # try a picker outside Zed
```

`telescope-zed` must keep running under macOS `/bin/bash` 3.2.
