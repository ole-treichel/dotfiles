# nvim lag with rust-analyzer

## Symptom

Typing in Rust buffers felt laggy, and completion, highlighting and diagnostics
showed up late. The machine has 20 cores and was mostly idle.

## Why cores don't help

- nvim runs a single event loop. LSP message handling, `vim.diagnostic.set`,
  treesitter highlighting and statusline evaluation all run on that one thread.
- rust-analyzer handles queries in parallel, but edits go through its main loop
  one at a time. Each `didChange` cancels in-flight requests and applies the edit,
  and the main loop also runs GC. The log shows 70–135 ms of GC per edit
  ("overly long loop turn"). Completion and semantic tokens wait behind that.
- Lowering rust-analyzer's CPU priority, `--jobs` and `numThreads` (commit 317d247,
  reverted) changed nothing because CPU was never the limit. Measured: load average
  1.5; statusline cost the same with 0, 20 or 40 busy processes.

## Measurements

Tools: a probe running the real config in tmux against a copy of `todo/main`
(main-loop stall detector, per-handler timing, request latency), a passive live
probe (stall detector, LuaJIT sampling profiler, request latency) loaded into the
running session, and a 250 ms CPU sampler over nvim, tmux, ghostty and
rust-analyzer.

Live session, 3 windows, 25 min total, ~2,600 keystrokes, including the moment
the user reported lag:

| | |
|---|---|
| Main-loop stalls > 100 ms | 0 (max 88 ms) |
| Stall time in Lua GC | ~75% |
| Diagnostic updates | 13/s while editing |
| Diagnostic pulls, html, on Rust buffers | 3,935 in 20 min |
| Diagnostic pulls, rust-analyzer | 3,572 in 20 min |
| rust-analyzer completion | p50 57 ms, p95 161 ms |
| nvim main thread CPU when lag was reported (14:00) | 3% avg |

Conclusion: nvim does not freeze. The lag comes from async results arriving late
(completion, semantic tokens, diagnostics), many small main-thread work items,
GC pauses up to ~90 ms, and fixed input waits from mappings.

## Causes and fixes

1. **Diagnostic pull amplification.** nvim 0.12 handles rust-analyzer's
   `workspace/diagnostic/refresh` (sent about once per edit) by re-pulling
   `textDocument/diagnostic` from *every* client on *every* buffer attached to
   rust-analyzer (`vim/lsp/diagnostic.lua`, `on_refresh` → `_refresh(bufnr)` with no
   client filter). `html` is attached to Rust buffers for completion inside
   `view!`, so each refresh also made html re-validate every open Rust buffer. Each
   response → `vim.diagnostic.set` → `DiagnosticChanged` → the builtin autocmd forces
   a statusline + window redraw (treesitter highlighting).
   **Fix:** html `before_init` removes `textDocument.diagnostic` from the client
   capabilities. vscode-html-language-server then pushes diagnostics instead
   (`htmlServer.js`: pull only if the client advertises it; push is debounced
   500 ms per changed document). Completion inside `view!` is unchanged, and html
   files still get diagnostics.
2. **Completion payload → GC.** The top Lua stacks during stalls were nvim-cmp
   turning rust-analyzer completion items into entries (`entry.lua`, `str.lua`, snippet
   parsing). **Not fixed.** `rust-analyzer.completion.limit = 50` was tried and
   reverted: rust-analyzer truncates by its own relevance ranking *before* the
   client filters by the typed prefix, so matching items are cut. A/B against
   rust-analyzer in `todo/main`, item wanted after typing the prefix:

   | typed | wanted | limit 50 | unlimited |
   |---|---|---|---|
   | `users.ite` | `iter` | found | found |
   | `HashM` | `HashMap` | **missing** | found |
   | `load_us` | `load_users` | **missing** | found |
   | `Strin` | `String` | **missing** | found |
   | `TaskSt` | `TaskStatus` | found | found |
   | `tokio::sp` | `spawn` | found | found |

   Unlimited lists are 180–260 items per request.
3. **Statusline spawned git synchronously.** `io.popen('git rev-parse ...')` on
   every statusline evaluation: 6.7 ms each, on every cursor move and every forced
   redraw. **Fix:** cache the branch and refresh it asynchronously (`vim.system`) on
   `VimEnter`/`DirChanged`/`FocusGained`/`BufEnter`. `FocusGained` does not fire in
   tmux here (`focus-events off`), so a branch switched outside nvim shows after the
   next buffer switch.
4. **Prefix-mapping waits.** `d` + `dd`/`diw` (n), `d` + `dd` (v) and `c` + `cc` (n)
   made `d`/`c` wait `timeoutlen` (300 ms) for a possible second key. **Fix:** remove
   the longer maps. `"_d` followed by `d`/`iw` already gives `"_dd`/`"_diw`, so
   black-hole behaviour is unchanged (verified: unnamed register untouched after
   `diw`, `dd`, `cc`).

## Result

Same scripted scenario (copy of `todo/main`, real config, keystrokes via tmux),
baseline vs. fixes 1, 3, 4 (no completion limit):

| | before | after |
|---|---|---|
| `vim.diagnostic.set` calls / main-thread time | ~500 / 3.9–4.5 s | 190 / 0.14 s |
| `DiagnosticChanged` autocmd time | 3.8–4.3 s | 0.09 s |
| html diagnostic pulls | every edit + every refresh | 0 (5 pushes) |
| Keystroke latency p50 / p95 / max | 10–17 / 20–46 / 30–62 ms | 10–13 / 14–18 / 15–22 ms |
| Stalls > 50 ms while typing | 1–4 per phase | 0 (max 48 ms) |
| `status_line()` | 6.7 ms | 0.005 ms |

## Non-goals / rejected

- **nice / `--jobs` / `numThreads` / `cachePriming.numThreads`:** CPU is not the
  limit (see above). Reverted earlier; not reintroduced.
- **Separate rust-analyzer target dir (`cargo.targetDir`):** would stop
  rust-analyzer's `cargo check` waiting on cargo-watch's build lock after save, but
  costs disk space and only affects diagnostics after a save, not typing. Not
  done; revisit if save → diagnostics latency bothers.
- **Dropping `rust` from html filetypes:** would remove html completion inside
  `view!`. Kept by user decision.
- **Disabling semantic tokens, smear-cursor or tiny-inline-diagnostic:** they cost
  some main-thread time, but none dominated the measurements.
- **Patching nvim's refresh handler** to pull only the refreshing client: would
  fix the amplification for every server, but means maintaining an override of
  nvim internals. The capability change fixes the one server that mattered.
