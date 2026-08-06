# wt — git worktree CLI

A Rust CLI that packages the bare-repo worktree workflow used in
`~/workspace/sonax/sonax-apps` and `~/workspace/sonax/sonax-redesign` into five
commands. Replaces hand-typed `git worktree add` / `git branch -d` /
`git worktree prune` sequences.

Status: implemented in `wt/`. `./wt/install.sh` builds it and links
`~/.local/bin/wt`. See [Implementation notes](#implementation-notes) for the
decisions the design left open.

## The layout it assumes

```
sonax-apps/
  .git            -> file containing "gitdir: ./.bare"
  .bare/          bare clone, fetch refspec +refs/heads/*:refs/remotes/origin/*
  main/           worktree
  feat-consent/   worktree
  ...
```

Repo discovery: walk up from cwd until a directory contains `.bare/`. Works
from inside any worktree or from the repo root. No `.bare/` above cwd is a hard
error.

## Commands

```
wt new <words...> [--from <ref>]   create branch + worktree, scaffold, push, PR
wt get [branch]                    check out an existing remote branch
wt rm  [dir...]                    remove worktree(s) + local branch(es)
wt ls                              table of worktrees
wt clone <url> [name]              build the .bare layout from scratch
```

`wt get` and `wt rm` with no argument open a ratatui picker, both multi-select.
`wt get` lists remote branches that have no worktree and checks out every one
you pick; `wt rm` lists existing worktrees with dirty/ahead markers and asks for
a y/N confirmation before anything is deleted.

## Decisions

| Area | Decision |
| --- | --- |
| Naming | `slug` = lowercase, every run of non-alphanumerics collapsed to a single `-`, trimmed. Directory name is **always** `slug(branch)` |
| `wt new` branch name | The slug itself. `wt new "Feat: Cookie Banner!!"` → branch and dir `feat-cookie-banner`. No auto `feat-` prefix — you type the prefix you want |
| `wt get` branch name | Verbatim. A remote branch `feat/master-product-data-table` keeps its name; only the directory is slugged to `feat-master-product-data-table` |
| Existing worktrees | Never re-derived. `git worktree list --porcelain` is the authoritative dir↔branch map, which is why the legacy `feat-website-in-sign-up-mail` ↔ `feat-website-in-signup-mail/` mismatch is harmless |
| Collisions | Hard error, never an auto-suffix. Two branches slugging to the same directory is a mistake worth surfacing |
| Base ref | `git fetch origin`, then branch from `refs/remotes/origin/HEAD`. `--from <ref>` for stacked branches |
| `wt rm` | Remove worktree → delete local branch → `git worktree prune` → `tms refresh`. Refuses on dirty tree or unpushed commits; `--force` overrides. Prints what is about to go and waits for y/N; `--yes` skips the prompt |
| Remote branches | Never deleted. GitHub deletes on merge |
| tms | `tms refresh` after `new`, `get`, `rm`. Nothing else — no session creation, no window management |
| git access | Shell out to `git`. `git2`'s worktree API is thin and this is a wrapper, not a reimplementation |
| Install | `wt/` in this repo, `cargo build --release`, symlink `~/.local/bin/wt`, same as `qr-lan` |

### Why `origin/HEAD` and not the `main/` worktree

`main/` in `sonax-apps` currently has `feat-feature-kernel` checked out, not
`main`. Basing new branches on "whatever is in `main/`" would silently produce
wrong bases. `refs/remotes/origin/HEAD` is immune to that and to which directory
you invoke from.

### Why the scaffold matters

An empty branch identical to its base cannot have a PR — GitHub rejects it with
"no commits between". The scaffolded `docs/<slug>/` files are what make the
first commit, and therefore the draft PR, possible.

## Hook system

The CLI does worktree plumbing and nothing else. Everything with an opinion
lives in shell dropins in this repo, next to the CLI:

```
wt/hooks/post-create.d/
  05-seed-env.sh          copy gitignored .env* from main/
  10-scaffold-docs.sh     docs/<slug>/prd.md + knowledge.md
  30-commit-push-pr.sh    commit, push -u, gh pr create --draft
```

Contract:

- Files matching `*.sh`, run in lexical order, executable bit required, shebang
  honoured.
- cwd is the new worktree.
- Non-zero exit aborts the remaining dropins. The worktree is **left in place**
  so the failure can be fixed and the hook re-run by hand.
- Environment:

```
WT_EVENT=new|get
WT_REPO_ROOT=/home/ole/workspace/sonax/sonax-apps
WT_DIR=$WT_REPO_ROOT/feat-cookie-banner
WT_BRANCH=feat-cookie-banner
WT_SLUG=feat-cookie-banner
WT_BASE_REF=origin/main
WT_DEFAULT_BRANCH=main
```

`wt new` and `wt get` share one directory. Scripts that only apply to creation
open with `[ "$WT_EVENT" = new ] || exit 0` — one guard line beats a second
event directory plus symlinks.

The hooks are global (they live in dotfiles, not in each project). They gate
themselves on repo shape: `10-scaffold-docs.sh` exits 0 unless `docs/` exists at
the repo root. **There is no per-repo config file and no repo-local hook
override.**

### 05-seed-env.sh

A fresh `git worktree add` produces a repo that cannot run — every worktree
carries gitignored `brand-service-center/.env.{development,production,staging}`.
The hook finds every gitignored `.env*` under `$WT_REPO_ROOT/main` and copies it
to the same relative path in the new worktree. Matching only `.env*` avoids any
need for a node_modules/dist exclude list.

### 10-scaffold-docs.sh

Follows the convention already in `sonax-apps`: `docs/<slug>/prd.md`, with a
German template at `docs/templates/minimal-prd.md`.

- `feat-*` → `prd.md` (from the repo's template if present, else a stub) **and**
  `knowledge.md`.
- Anything else → `knowledge.md` only.

`knowledge.md` is new — no instance exists in the repo yet. It gets a stub
heading and empty sections.

### 30-commit-push-pr.sh

```sh
git add -A
git diff --cached --quiet && exit 0
git commit -m "chore($WT_SLUG): scaffold docs"
git push -u origin "$WT_BRANCH"
command -v gh >/dev/null && gh pr create --draft \
  --base "$WT_DEFAULT_BRANCH" --title "$WT_SLUG" --body "See docs/$WT_SLUG/prd.md"
```

Keeping push and PR in a hook rather than in Rust means no `gh` dependency in
the binary, and the commit message or PR template can change without a rebuild.

## Deliberate non-goals

- **No tmux/tms integration beyond `tms refresh`.** No session creation, no
  switching, no window cleanup. Consequence: `tms refresh` only *adds* windows
  for new worktrees — after `wt rm` the stale window in the mega session stays
  until you close it.
- **No `cd`.** A CLI cannot change the parent shell's directory, so `wt` never
  pretends to "open" anything. That is also why there is no TUI dashboard.
- **No dependency install.** `npm install` in a new worktree stays manual.
- **No per-repo configuration**, no `.wt.toml`, no repo-local hook overrides.
- **No remote branch deletion.**
- **No repair mode.** `wt` will not rename directories or branches to fix the
  legacy slash-branches (`feat/update-logos`) or the mismatched
  `feat-website-in-sign-up-mail`. It reads the real mapping and leaves them be.

## Implementation notes

Decisions taken while building it that the design above did not fix.

| Area | Decision | Why |
| --- | --- | --- |
| Layout | `wt/` — `src/{main,git,repo,slug,hooks,picker,tms}.rs` + `src/cmd/{new,get,rm,ls,clone}.rs`. Deps: clap, anyhow, ratatui | `git.rs` is the only place that shells out; every command reads the worktree map through `repo.rs` |
| Hook lookup | Resolved from the binary's canonicalised path (`<crate>/target/<profile>/wt` → `<crate>/hooks/post-create.d`), so the `~/.local/bin` symlink still finds them. `WT_HOOKS_DIR` overrides | No install step has to copy the dropins, and `cargo run` uses the same ones |
| "Unpushed" | `git rev-list --count <branch> --not --remotes`, asked of the branch from the repo root | Works whether or not the branch has an upstream (a fresh `wt new` worktree has none until the hook pushes), and a worktree whose directory is already gone still answers |
| `wt new` tracking | `git worktree add --no-track` | Branching off `origin/main` otherwise leaves `main` as the new branch's upstream until `30-commit-push-pr.sh` pushes. `git push` with no upstream is the safer state |
| Branch deletion | `git branch -D`, gated on wt's own unpushed check rather than on `-d` | `-d` only knows HEAD and the upstream, so it refuses branches that are fully on origin but lost their tracking config — which is exactly what an earlier `wt rm` leaves behind. Our check ("no commit outside the remotes") is the stronger one. A refusal is still reported as a warning, not an error: the worktree is already gone by then |
| Lost tracking | `add_worktree` re-points a branch at `origin/<branch>` when it exists locally with no upstream | Re-getting a branch that survived an earlier `wt rm` should not leave `git push` without a target |
| `wt rm` picker | Hides the default branch's directory (`main/`). Explicit `wt rm main` still works | It is the directory `05-seed-env.sh` seeds from; losing it by mis-click is worse than typing it out |
| `wt rm` validation | All targets are checked before any is removed, then listed for a y/N confirmation. `--yes` skips it; EOF on stdin (no answer possible) aborts | Multi-select must not half-apply, and the one irreversible command should say what it is about to do |
| Picker | ratatui, multi-select in both `get` and `rm`, type-to-filter substring match, `↑`/`↓` or `ctrl-p`/`ctrl-n`, `tab` toggles, `enter` confirms (the row under the cursor if nothing is toggled), `esc` cancels. Not a TTY → hard error telling you to pass the argument | A fuzzy matcher is a dependency for a list that is never longer than a screen. `wt get` picking several branches at once is the same "grab the day's work" gesture as `wt rm` clearing it |
| `tms refresh` | Skipped entirely when `$TMUX` is unset; a failure is a warning | `tms refresh` needs a current session; running `wt` outside tmux is normal and must not fail |
| `05-seed-env.sh` | Seeds from `$WT_REPO_ROOT/$WT_DEFAULT_BRANCH`, skips paths under `node_modules/`, never overwrites | `--others --ignored` does recurse into ignored directories, and packages do ship `.env` files. One `case` guard, still no exclude list |
| `wt clone` | Also runs `git remote set-head origin -a` and sets the first worktree's upstream | `wt new` depends on `origin/HEAD`; a bare clone does not set it |
| Verification | `cargo test` covers `slug()` and URL→name; the rest was exercised end to end against a throwaway local remote (clone → new → ls → get → collisions → dirty/unpushed refusals → confirmation y/n/EOF → `--force` → rm). Both pickers were driven through a pty to check rendering, filtering, toggling and cancel | |

## Known state to be aware of

- `sonax-apps` has a prunable worktree entry for `feat/cookie-information`.
  The first `wt rm` (or any `wt` run that prunes) clears it.
- `main/` has `feat-feature-kernel` checked out. Seeding from `main/` is
  unaffected — `.env*` files are untracked, so the checked-out branch is
  irrelevant.
