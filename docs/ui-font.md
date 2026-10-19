# UI font: Geist

Status: implemented 2026-10-16. Script: [`fonts/install_geist.sh`](../fonts/install_geist.sh).

## Decisions

- **Source: Fedora package `vercel-geist-vf-fonts`** (Fedora 44 `updates`, 1.8.0).
  Updates arrive with `dnf upgrade`; files land in `/usr/share/fonts`, which
  flatpak apps (Thunderbird) see under `/run/host/fonts`. Rejected: copying
  release files into `~/.local/share/fonts` (no updates, no sudo is the only gain).
- **Variable font, not static** (`vercel-geist-fonts`). One file per style, all
  weights; same approach as the previous UI font, Inter Variable.
- **Value: `Geist Medium 10 @wght=500`** for `font-name` and
  `document-font-name`. Same size and weight as the previous setting
  (`Inter Variable Medium 10 @wght=500`, chosen in Tweaks). Format is what the
  Tweaks font chooser writes for a variable font.
- **GNOME Shell** follows `font-name`: the Oxide shell theme (`gtk/`) sets no
  `font-family`.

## Non-goals

- Monospace font stays Berkeley Mono. Geist Mono is not installed.
- `org.gnome.desktop.wm.preferences titlebar-font` is untouched; it is unused
  while `titlebar-uses-system-font` is `true`.
- Inter Variable stays in `~/.local/share/fonts` (Obsidian vault uses it).
