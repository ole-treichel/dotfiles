# wt new — branch-name wizard

`wt new` with no arguments walks a three-step wizard and produces a branch name
that carries the MOCO project number. `wt new <words>` keeps its old behaviour
and is the escape hatch for scratch branches.

Status: implemented in `wt/`. Part of [wt](wt.md).

## The convention

```
<prefix>-<description>-p<number>
feat-import-button-p26059
```

- Charset is exactly `[a-z0-9-]`. No slashes.
- `prefix` is `feat`, `chore` or `fix`. Nothing else.
- The project number is **last**, separated by a single hyphen, lowercase. Last
  so shell tab-completion on the prefix and description still works.
- Directory name is `slug(branch)`, which for a conforming branch is the branch
  name itself.

## The two paths

```
wt new                    wizard  → feat-import-button-p26059
wt new cache poc          verbatim → cache-poc
```

`wt new <words>` adds nothing: no prefix, no project number. The words are
slugged and used as given. That is the whole point of it — experiments and
scratch branches that should not be forced through the convention. `slug`
already emits exactly `[a-z0-9-]` with no leading or trailing dash, so the
escape hatch needs no separate validation.

## Wizard steps

1. **prefix** — a three-row picker, `feat` / `chore` / `fix`.
2. **description** — typed, then slugged.
3. **project** — a picker over the active MOCO projects, fuzzy-filtered on
   customer, project name and number alike.
4. **confirm** — the assembled name, y/N.

Steps 1–3 all draw the same bordered panel, with the step's title in the top
border and the same `❯ …▏` input line:

```
╭ feat-<description>-p… ───────╮
│ ❯ import-button▏             │
╰─ enter confirm · esc cancel ─╯
```

Steps run in the order the name reads, so the title only ever grows to the
right: `feat-<description>-p…`, then `project for feat-import-button-p…`, then
the finished name in the confirmation.

Step 4 is a plain y/N on the terminal, shaped like `wt rm`'s confirmation:

```
about to create:
  feat-import-button-p26059  P26059 · VGH Versicherungen
create this branch? [y/N]
```

`Repo::discover()` runs *before* step 1. Walking three steps only to be told
there is no `.bare/` in this directory would waste all of them.

## MOCO access

`GET https://pixelwerte.mocoapp.com/api/v1/projects?per_page=1000`

- The subdomain is hardcoded in `src/moco.rs`. One account, and it is not a
  secret. `config.toml` is committed to this repo, so it is the wrong place for
  anything MOCO-related.
- The token comes from `$MOCO_API_KEY` — the same variable name
  `pixelwerte/cloud-moco-billing` uses. Not from `config.toml`: that file is
  committed and symlinked into `~/.config/wt/` by `install.sh`.
- Fetching shells out to `curl`, matching how the rest of `wt` treats `git` and
  `gh`. curl reads its options from **stdin** (`--config -`), not argv, so the
  token never appears in `ps` output.
- Fields used: `identifier`, `name`, `active`, `customer.name`. A project with
  no `identifier` is dropped — there would be no number to append. So is one
  with `active: false`, which `include_archived` being absent should already
  have prevented.
- `per_page=1000` is the documented maximum and covers the 30 active projects
  in one request. The page loop is still there, capped at 20 pages, in case the
  account outgrows it.

## Decisions

| Area | Decision |
| --- | --- |
| Escape hatch | `wt new <words>`, not a `--scratch` flag. The old invocation already *is* the escape hatch — nothing new to remember |
| Escape-hatch output | `slug(words)`, nothing added. No prefix, no number |
| Prefix entry | Picker, not typed. Three fixed values; a picker cannot be typo'd |
| Project number | `slug(identifier)`. `P26059` → `p26059`. Slugged rather than lowercased so an identifier like `P1903-003` still lands inside the charset |
| Projects listed | Active only. The 234 archived projects are finished work and never get a branch. No filtering by intern or retainer |
| Project order | Identifier descending, newest project first |
| Row layout | `<customer> · <project name>` in the padded name column, identifier dim beside it. Customer first because it is the coarser sort — the left edge of the column groups the list by client while scanning |
| Typed step | Drawn in the same bordered panel as the two pickers, not with `print!`. A bare two-line shell prompt appearing the instant a full-screen picker exits reads as a different program |
| Terminal takeover | One `picker::Session` held across all three steps. `init`/`restore` per step left and re-entered the alternate screen in between, which flashes the shell. The session is dropped before the confirmation, which is deliberately shell output |
| Retainers | Ordinary rows. MOCO's `isRetainer` flag disagrees with which projects are actually treated as retainers — `P26040 Website Maintenance` is `false`, and the flag marks three others that are not — so pinning by it would mislead |
| Caching | None. A project created ten minutes ago has to show up |
| MOCO unreachable, or no token | Hard failure. A branch name without a project number is not the convention, so there is nothing to fall back to. The error names `wt new <words>` as the way out |
| Picker matching | Fuzzy subsequence with scoring, replacing the old substring filter — in the *shared* picker, so `wt get`, `wt rm` and `wt vault` change too. Runs and word starts score above scattered hits. Greedy-leftmost, not optimal; a few hundred rows never show the difference |
| Umlauts | `slug` spells out ä ö ü ß before slugging. Without it `Jubiläum` became `jubil-um`, and the MOCO project names are German. Shared, so `wt new <words>` changes too |
| Deps | `serde_json` only. No HTTP client, no async runtime |

## Non-goals

- **`wt get` is untouched.** It still takes remote branch names verbatim,
  slashes and all. Branches that predate this convention keep working; the
  convention is enforced at creation, not at checkout.
- **No length cap** on the assembled name.
- **The wizard never asks for a base ref.** `--from` stays a flag, used for
  stacked branches only. A step skipped every time is noise.
- **No per-repo configuration.** The convention is global, like the hooks.
- **Only the German set is transliterated.** `é` and the en dash stay
  separators, which is what the en dash in the MOCO names wants anyway.

## Files

```
src/moco.rs        projects(), token(), curl + serde_json
src/cmd/new.rs     wizard(), the three steps, create()
src/picker.rs      Session::{pick,prompt} sharing chrome() + input_line();
                   score() + filter(), fuzzy, used by every picker
src/slug.rs        transliterate() ahead of the slug
```
