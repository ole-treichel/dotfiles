# nvim: auto-reload externally changed files

Code: `.config/nvim/lua/config/autoread.lua`

## Problem

git, Claude, formatters change files that are open in nvim. Two failures:

1. Unmodified buffer: nvim only notices on `:checktime` triggers (focus, shell
   command, some buffer switches). tmux has `focus-events` off, so in practice
   you start editing the stale buffer, which turns into case 2.
2. Modified buffer: W12 prompt "file has changed and the buffer was changed as
   well". Choices are lose your edits (Load) or overwrite the external change (OK).
   Then `:w` asks again.

## Decisions

- **Directory watcher (libuv `fs_event`) triggers `:checktime`**, debounced 100 ms.
  Rejected: CursorHold (fires once per idle period, so changes made while idle
  are seen only after you have already started typing) and timer polling (latency
  vs. constant stat calls). Watch the directory, not the file: atomic writes
  (tmp + rename, used by many tools) end an inotify watch on the file's inode.
  Paths are resolved with `realpath`, so symlinked files (`~/.bashrc`) are watched
  in the target's directory. One watcher per directory with loaded buffers.
- **BufEnter `:checktime` as fallback.** `:checktime` only checks buffers that are
  shown in a window, so hidden buffers are handled when you switch to them.
  Also covers files whose directory did not exist when they were opened.
- **Unmodified buffers: nvim's `autoread`** (default on, set explicitly). No
  custom code.
- **Modified buffers: 3-way merge** via `git merge-file`. base = buffer content at
  last read/write (stored per buffer), ours = buffer, theirs = disk.
  Non-overlapping changes merge silently (with a notification). Overlapping or
  adjacent edits get git conflict markers (`merge.conflictStyle` from git config
  applies). Nothing is lost either way.
- **Reload, then re-apply, instead of patching the buffer in place.**
  `FileChangedShell` sets `v:fcs_choice = 'reload'`, `FileChangedShellPost` applies
  the merge result as diff hunks. Reason: `:w` compares the file's mtime with
  the mtime at the last read/write and asks "changed since reading it". Only a
  real read or write updates that value. Patching in place made every save after
  a merge prompt.
- **Undo:** the reload (`'undoreload'`, 10000 lines by default) and the merge
  hunks are joined, so one `u` restores the buffer from before the merge. Above
  10000 lines the reload clears undo history; the merge is still applied.
- **Cursor:** reload keeps line numbers. Cursors in windows showing the buffer
  are shifted by the lines added/removed above them.
- **tmux `focus-events` stays off.** The watcher makes focus events unnecessary.

## Non-goals

- No plugin. ~180 lines, no dependency.
- No auto-save.
- Deleted files, permission changes: default nvim behaviour (`v:fcs_choice = 'ask'`).
- No recursive watching; only directories that contain loaded buffers.
- No protection for a `:w` that races an external write within the 100 ms
  debounce window.

## Verified

Headless tests (nvim 0.12.5): in-place write, tmp+rename write, merge
non-overlapping edits, conflict markers, identical change on both sides
(buffer becomes unmodified), cursor shift, single-step undo, `:w` after merge
without prompt, own writes silent, hidden buffer merged on enter, symlinked
file, `git checkout` from a job with the full config.
