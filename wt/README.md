# wt

git worktree CLI for the bare-repo layout:

```
sonax-apps/
  .git            -> file containing "gitdir: ./.bare"
  .bare/          bare clone, fetch refspec +refs/heads/*:refs/remotes/origin/*
  main/           worktree
  feat-consent/   worktree
```

Design and rationale: [`../docs/wt.md`](../docs/wt.md).

## Install

```
./install.sh
```

Builds `target/release/wt` and symlinks it to `~/.local/bin/wt`. The hook
dropins are found relative to the binary's real path, so the symlink works.

## Use

```
wt new <words...> [--from <ref>]   create branch + worktree, scaffold, push, PR
wt get [branch]                    check out existing remote branch(es)
wt rm  [dir...] [--force] [--yes]  remove worktree(s) + local branch(es)
wt ls                              table of worktrees
wt clone <url> [name]              build the .bare layout from scratch
```

Run from anywhere inside the repo — `wt` walks up until it finds `.bare/`.

- `wt new "Feat: Cookie Banner!!"` → branch **and** directory `feat-cookie-banner`,
  based on `origin/HEAD`. No prefix is added; type the prefix you want.
- `wt get feat/master-product-data-table` → branch keeps its name, directory is
  `feat-master-product-data-table`.
- `wt get` / `wt rm` with no argument open a multi-select picker: type to
  filter, `↑`/`↓` or `ctrl-p`/`ctrl-n` to move, `tab` to toggle, `enter` to
  confirm (the row under the cursor if you toggled nothing), `esc` to cancel.
  `wt get` checks out every branch you pick, freshest branch first.

```
╭ remove worktrees ───────────────────────────────── 1 selected  4/4 ╮
│ ❯ ▏ type to filter                                                 │
│                                                                    │
│   ✓ feat-consent                    feat-consent  dirty            │
│ ❯ · feat-cookie-banner              feat-cookie-banner             │
│   · feat-master-product-data-table  feat/master-product-data-table │
│   · fix-login-redirect              fix-login-redirect  ↑1         │
╰───────────── tab select · enter confirm · esc cancel ──────────────╯
```

- `wt rm` lists what it is about to delete and waits for a y/N; `--yes` skips
  the prompt. It refuses on uncommitted changes or unpushed commits; `--force`
  overrides. Remote branches are never deleted.

## Hooks

`hooks/post-create.d/*.sh` run after `wt new` and `wt get`, in lexical order,
with cwd set to the new worktree. Executable bit required. A non-zero exit
aborts the rest and leaves the worktree in place.

```
05-seed-env.sh        copy gitignored .env* from the default-branch worktree
10-scaffold-docs.sh   docs/<slug>/prd.md + knowledge.md  (new only, needs docs/)
30-commit-push-pr.sh  commit "init", push -u, gh pr create  (new only)
```

Environment:

```
WT_EVENT=new|get
WT_REPO_ROOT=/home/ole/workspace/sonax/sonax-apps
WT_DIR=$WT_REPO_ROOT/feat-cookie-banner
WT_BRANCH=feat-cookie-banner
WT_SLUG=feat-cookie-banner
WT_BASE_REF=origin/main
WT_DEFAULT_BRANCH=main
```

Creation-only scripts open with `[ "$WT_EVENT" = new ] || exit 0`. The hooks are
global and gate themselves on repo shape — there is no per-repo config and no
repo-local override. `WT_HOOKS_DIR` points the runner at a different directory
(useful for testing).
