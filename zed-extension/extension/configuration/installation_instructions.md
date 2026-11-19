Build and link the `telescope-zed-mcp` server (needs `rg` and `git` on PATH):

```sh
cd <telescope.nvim>/zed-extension && ./install.sh
```

It exposes `find_files`, `live_grep`, `grep_string`, `git_status`, `git_commits` and `todo` tools to the agent.
Set `root` if the server should search a fixed directory instead of the path the agent passes.
