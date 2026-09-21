# nvim-treesitter master on Neovim 0.12

## Symptom

LSP hover in Rust threw, repeatedly:

```
Decoration provider "conceal_line" (ns=nvim.treesitter.highlighter):
Lua: /usr/share/nvim/runtime/lua/vim/treesitter.lua:197: attempt to call method 'range' (a nil value)
  ...nvim-treesitter/lua/nvim-treesitter/query_predicates.lua:141: in function 'handler'
```

Same trace from `render-markdown.nvim`. Not Rust-specific: any markdown code fence
triggers it, and hover renders markdown.

## Cause

Neovim 0.12.5 here; nvim-treesitter is pinned to `master`
(`cf12346a`, 2026-03-23), whose README states 0.12 is not supported. Master is archived.

Until 0.11, `vim.treesitter.query.add_predicate/add_directive` accepted `all = false`
(the default), and Neovim wrapped the handler so `match[capture_id]` was a single
`TSNode`. 0.12 dropped the option and the wrapper: handlers always receive
`table<integer, TSNode[]>`.

Six master handlers still index `match[id]` as a node — `nth?`, `is?`, `kind-eq?`,
`set-lang-from-mimetype!`, `set-lang-from-info-string!`, `downcase!`. They get a list,
call `:range()` on it, and throw. `set-lang-from-info-string!` is the one on the
markdown injection path.

## Decision

`.config/nvim/lua/config/ts-compat.lua` reinstates the 0.11 wrapper, for those six
names only, by patching `add_predicate`/`add_directive` before lazy.nvim runs.
Loaded first from `lua/config/init.lua`.

Scoped by name on purpose: a blanket wrapper would also hit plugins written for the
0.12 list signature (none installed today, but autotag/context-commentstring could
add one).

## Alternative rejected for now

Migrating to the nvim-treesitter `main` branch is the real fix — master gets no
further updates. Deferred because it is a rewrite, not a patch:

- `nvim-treesitter.configs.setup{}` is gone; highlight/indent move to
  `vim.treesitter.start()` and an `indentexpr` per FileType autocmd.
- `auto_install` is gone.
- the custom `datastar` parser registration changes shape.
- `nvim-treesitter/playground` is archived and master-only (`:Inspect`/`:EditQuery`
  replace it).
- `rayliwell/tree-sitter-rstml` calls the master API in its `setup()`.

Toolchain for that migration is already present: `tree-sitter` 0.26.11, gcc, git.

## Non-goals

- Downgrading Neovim.
- Vendoring a patched `query_predicates.lua`.
- Touching `render-markdown.nvim`; it was a victim, not a cause.
