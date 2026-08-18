# wt vault — symlink branch docs into Obsidian

Status: implemented in `wt/`. Extends [`wt`](wt.md) with a new subcommand.
`./wt/install.sh` links `~/.config/wt/config.toml`. See
[Implementation notes](#implementation-notes) for the decisions the design
left open.

## Problem

`10-scaffold-docs.sh` (see [wt.md](wt.md#10-scaffold-docsh)) already writes
`docs/<slug>/{prd.md,knowledge.md}` into every worktree. Reading/editing that
prose in Obsidian instead of an editor has so far meant hand-making a symlink
into `~/workspace/obsidian/work`. The vault already has a dozen of these,
made ad hoc over time — inconsistent targets (whole `docs/` vs. a specific
`docs/<slug>/`), inconsistent names (`docs`, bare slug, `repo-slug`), nested
under hand-curated project folders (`Sonax/`, `Sonax/Website/`,
`Pixelwerte/Prism/`), and at least one (`Sonax/Website/a11y-contrast`) is
dangling — it still points at a `/Users/ole/...` path from before this
machine. `wt vault` automates the linking step with one fixed naming scheme,
without trying to clean up or reorganize what's already there.

## Decisions

| Area | Decision |
| --- | --- |
| Command | `wt vault [slugs...]` — a new, separate subcommand. Never runs implicitly from `new`/`get`/`rm` |
| Vault location config | `vault = "~/…"` in a new `wt/config.toml`, checked into this repo, symlinked to `~/.config/wt/config.toml` by `install.sh` — same pattern as `tms/config.toml` |
| Argument form | Zero or more worktree-directory names (slugs), like `rm`'s `Vec<String>` — not `get`'s single verbatim branch name, since linking several worktrees in one call is the point |
| No-args behaviour | Multi-select picker, styled like `get`/`rm`'s, listing only worktrees in the current repo that have a `docs/<slug>/` directory. Worktrees without one are left off the list, not offered-then-rejected |
| What gets linked | The whole `docs/<slug>/` directory, as-is — not individual `*.md` files |
| Symlink name | `<repo-name>-<slug>` — `<repo-name>` is the bare-repo root directory's name (e.g. `sonax-apps`), `<slug>` is the worktree directory/slug (e.g. `feat-cookie-banner`) |
| Symlink location | Flat, at the vault root (`<vault>/<repo-name>-<slug>`) — not nested under any project folder, not in a dedicated `wt/` subfolder |
| Overwrite behaviour | `ln -sfn`-style: relink if the destination is already a symlink (even a dangling one); hard error only if something that is *not* a symlink already occupies that name, so a real vault file/folder is never clobbered |
| Batch behaviour | Each selected slug is linked independently; one failure is reported and does not abort the rest |
| Cleanup | None. No `wt vault --unlink`, no hook into `wt rm`. Removing a link is a plain `rm` on the vault-side symlink |

## Rationale

**Why a config file, not an env var.** `wt` already has one env var,
`WT_HOOKS_DIR`, but it exists purely as a test escape hatch over a value that
otherwise has a sensible derived default (the binary's own path). The vault
path has no such default — it's arbitrary per-machine state — so it belongs
in a file, matching the `tms/config.toml` precedent already in this repo
rather than inventing a second config mechanism.

**Why a picker filtered to worktrees with `docs/<slug>/`.** Mirrors `wt
get`, whose picker only lists remote branches with no worktree yet rather
than listing everything and erroring on the ineligible ones. Keeps the list
short and every row actionable.

**Why the whole directory, not per-file symlinks.** Matches the phrasing
this was requested in, and matches the more careful examples already in the
vault (e.g. `international-bsc -> .../docs/international-bsc`) over the
sloppier whole-`docs/`-folder ones. Keeps `prd.md` and `knowledge.md` (and
anything else later dropped into that folder) together as one Obsidian
folder instead of scattering loose notes at the vault root.

**Why flat at the vault root, no subfolder.** Considered a dedicated `wt/`
subfolder to keep automated links visually separate from the hand-curated
`Sonax/`/`Pixelwerte/` structure. Rejected: the vault is explicitly a
disposable read/write convenience layer, not a system of record — branches
come and go, and the resulting dangling symlinks are expected and fine.
Optimizing the storage layout for that is not worth a second folder to look
in.

**Why `ln -sfn` semantics instead of always erroring on an existing
target.** Re-running `wt vault` on the same slug is the common case, not an
edge case — a worktree's docs get linked once and then the command is
mostly re-run for *other* slugs, but should stay idempotent rather than
requiring `--force`. The one guard (refuse to overwrite a non-symlink) exists
solely so a naming collision can't silently eat unrelated vault content.

## Deliberate non-goals

- **No unlink command, no `wt rm` integration.** Per the "vault is disposable"
  stance above: a dangling symlink after a worktree is removed is expected,
  and cleaning it up is a `rm` away. Wiring this into `wt rm` would also
  couple a worktree-plumbing command to an Obsidian-specific concern, which
  `wt`'s existing hook system deliberately keeps separate (see
  [wt.md](wt.md#hook-system)).
- **No migration/repair of the existing hand-made symlinks.** The dozen or so
  already in the vault (including the one dangling macOS-path link) are left
  untouched. `wt vault` only ever creates new links under its own fixed
  naming scheme.
- **No per-repo override of the vault path.** One vault, one machine-wide
  config file — consistent with `wt`'s existing "no per-repo config" stance.
- **No listing/status command** (e.g. "which worktrees are already linked").
  Out of scope until it's actually needed.

## Implementation notes

Decisions taken while building it that the design above did not fix.

| Area | Decision | Why |
| --- | --- | --- |
| Layout | `wt/config.toml` (new) linked by `install.sh`, mirroring `tms/install.sh`'s `~/.config/tms/config.toml` step. New `src/config.rs` and `src/cmd/vault.rs` | Same install pattern already used for `tms`; `vault.rs` reuses `picker.rs` and `repo::{Repo, Worktree}` like `rm.rs` does |
| Config parsing | Hand-written line parser (`vault_value` in `config.rs`) for the single `vault = "..."` key — no `toml`/`serde` dependency added | `wt`'s only other dependencies are clap/anyhow/ratatui; the config has exactly one key, so a parser is a few lines and a full TOML crate would be dead weight until the file actually grows a second key |
| `docs/<slug>/` location | `<worktree-path>/docs/<worktree-name>` — e.g. `sonax-apps/feat-cookie-banner/docs/feat-cookie-banner/` | Matches what `10-scaffold-docs.sh` actually writes: it runs with cwd set to the worktree and does `mkdir -p docs/$WT_SLUG`, and `$WT_SLUG` is the same slug as the worktree's own directory name |
| Argument resolution | `slugs` resolves against *all* checkouts by name or path, exactly like `rm.rs`'s `resolve` — an unknown slug is a hard error for the whole invocation, not a per-slug failure | Matches the existing `rm` precedent the design explicitly compares this argument form to. The "one failure doesn't abort the rest" guarantee applies once slugs are resolved to worktrees — to a missing `docs/<slug>/` or a name collision at the vault destination, not to typos in the arguments |
| Overwrite guard | `Path::is_symlink()` (lstat-based, true even for a dangling symlink) gates removal; `remove_file` then `symlink`, no atomic rename | `is_symlink()` is the one check that tells a real vault file/folder apart from even a broken link, which is exactly the guard the design calls for. Plain remove-then-create matches the rest of the codebase's preference for the simplest thing that is still correct |
| Verification | `cargo test` (the new `config::tests::reads_the_vault_line`, plus the existing `slug`/`clone` tests) and `cargo clippy --all-targets`, both clean. Exercised end to end against a throwaway bare-repo layout: picker-eligibility filtering (docs vs. no-docs worktrees), the per-slug batch failure with one success + one failure, idempotent relink, relinking over a dangling symlink, and refusal to clobber a real file/folder occupying the destination name | Mirrors `wt.md`'s own "throwaway local remote" verification approach |
