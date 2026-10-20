# Oxide everywhere

Status: implemented 2026-10-16. Replaces Everforest on every `theme` surface,
plus Obsidian and Thunderbird (not `theme` surfaces).

Upstream: [oxidescheme](https://github.com/orgs/oxidescheme/repositories).
Palette for this repo: [`oxide/colors.txt`](../oxide/colors.txt).

## Decisions

### Light variant is derived here

Oxide is dark-only upstream (no light palette in `colors.txt`, `palette.json`,
or any port). `theme` exists to toggle light/dark, so a light palette is
derived in OKLCH with the rules upstream uses for dark:

| Group | Dark (upstream) | Light (derived) | Rule |
|---|---|---|---|
| base | L 0.20 | L 0.97 | |
| mantle, surface0-2 | base −0.02, +0.05 steps | base −0.02, −0.05 steps | same offsets, mirrored |
| text ramp | 0.95 / 0.85 / 0.75 / 0.60 / 0.45 | 0.22 / 0.32 / 0.42 / 0.57 / 0.72 | same distance from base |
| accents | L 0.70 C 0.15 | L 0.52 C 0.15 | ≥ 4.3:1 on base (teal 4.33, sky 4.45, rest ≥ 4.7) |
| bright accents | L 0.80 | L 0.45 | "bright" = more contrast against base, in both modes |
| diff backgrounds | L 0.32 C 0.04 | L 0.90 C 0.04 | |

Out-of-gamut values are clipped per sRGB channel; that reproduces upstream's
dark hex values exactly except teal/sky (±2 per channel).
ANSI mapping is the same role mapping as the upstream ghostty port in both
modes (0 = surface0, 7 = text, 8 = subtext1, 15 = bright_text).

Rejected: keeping Everforest Light for light mode (two scheme families);
dropping light mode (removes the reason `theme` exists).

### Monochrome UI

All new ports follow upstream `UI_COLOR_GUIDE.md`: chrome is surface/text
only, selection = surface1, borders = surface2, accents only for state
(error red, warning orange/yellow, success green, current mode / prefix
blue). No blue "brand" accent on focus rings, tabs, toggles, or headers.

### Upstream ports used as-is

- **Neovim**: `oxidescheme/nvim`. Light = same plugin, palette swapped through
  `on_colors` when `background == "light"`
  (`.config/nvim/lua/plugins/theme.lua`). `:set background=` reloads the
  colorscheme, so the existing `theme` → nvim push keeps working. The old
  Everforest diagnostic-undercurl override is gone: oxide already sets only
  `sp` on `DiagnosticUnderline*`.
- **Ghostty**: `ghostty/themes/Oxide Dark` is upstream `Oxide` verbatim;
  `Oxide Light` is the same file over the light palette.

### New ports (no upstream exists)

| Surface | Files | Notes |
|---|---|---|
| GTK 3/4, libadwaita, GNOME Shell | `gtk/` | see [GTK](#gtk) |
| tmux | `tmux/oxide-{dark,light}.conf` | status mantle/subtext0; active window bright_text on surface1; prefix-active session block blue; activity yellow; bell red |
| Chrome (ungoogled-chromium) | `chrome/oxide-{dark,light}/` | frame darker than toolbar; incognito frame surface1 / surface2 |
| Firefox | `firefox/oxide/` | one theme, `theme` + `dark_theme`, see [Firefox](#firefox) |
| Thunderbird (flatpak) | `thunderbird/install_theme.sh` | same manifest as Firefox, see [Thunderbird](#thunderbird) |
| hunk | `hunk/config.toml` | `[themes.oxide-dark]` / `[themes.oxide-light]`; `~/.config/hunk/config.toml` symlinks here. Untokenized text (plain files, first frame) comes from the base theme's foreground, not a config key: bases `dark-plus` / `vitesse-light` give `#d4d4d4` / `#393a34`, closest bundled match to oxide text |
| delta | `bat/themes/*.tmTheme`, `.gitconfig` `[delta "oxide-*"]` | syntax via bat cache; `theme` sets `delta.features` |
| GNOME Terminal (module unused) | `gnome-terminal/oxide-{dark,light}.dconf` | kept in sync for completeness |
| Obsidian | `obsidian/oxide/`, `obsidian/install_theme.sh` | see [Obsidian](#obsidian) |

Generated values (tmux, browser manifests, dconf) are written as literal hex.
No generator lives in the repo: the palette changes rarely and the files are
short enough to read.

## GTK

`gtk/build_oxide_gtk.sh` builds `~/.themes/Oxide-Dark` and `~/.themes/Oxide-Light`
(GTK3, GTK4/libadwaita, GNOME Shell) from
[Everforest-GTK-Theme](https://github.com/Fausto-Korpsvart/Everforest-GTK-Theme)
at pinned commit `9b8be4d` (Colloid-based). The repo holds only the delta:

- `_color-palette-oxide.scss`: oxide roles in upstream's variable names.
- `oxide.patch`: remaps upstream's color assignments (surfaces, selection,
  accents, text cutoff on colored backgrounds 156 → 125, checked switch knob
  contrast, GTK3 `border-spacing` warning).
- `render_png_assets.py`: GTK3 PNGs via librsvg (upstream needs inkscape).

Rerun the script after editing any of them. Decisions:

| Role | Oxide color |
|---|---|
| window, views, dialogs | base |
| headerbar, sidebars | mantle |
| cards | surface0 (dark) / mantle (light) |
| popovers, menus | surface0 (dark) / base (light) |
| accent (checkbox, switch, slider, progress, focus, suggested button) | text: monochrome |
| selection (text, rows, treeview) | surface1 |
| borders | text at 18% alpha ≈ surface2 |
| links / visited | sky / purple |
| warning / error / success / destructive | orange / red / green / red |
| Shell panel | base at 85% (dark), #f5f5f5 at 45% (light) |

Rejected: vendoring upstream (28 MB of sources for a palette change); the
Compact/hdpi/xhdpi, GTK2, Cinnamon, XFWM variants (unused on GNOME).
Shell widget set is upstream's newest (48-0) on GNOME Shell 50.

`theme` sets `gtk-theme`, the user-theme Shell name, and retargets
`~/.config/gtk-4.0/{assets,gtk.css,gtk-dark.css}` (libadwaita) as before.
`~/.config/gtk-{3,4}.0/colors.css` are KDE Breeze leftovers (`*_breeze`
names only) and do not affect Oxide.

`gsettings` calls in theme-cli, nvim `theme.lua` and `.tmux.conf` drop
`GSETTINGS_BACKEND`. A long-running tmux server had inherited
`GSETTINGS_BACKEND=keyfile` (from an older Ghostty AppImage), so `theme` run
inside tmux wrote to `~/.config/glib-2.0/settings/keyfile` and the Shell kept
the old theme while libadwaita apps (symlink-based) switched.

## Firefox

Facts checked on Fedora's `firefox-157.0.1-1.fc44` (`omni.ja`):

- `MOZ_REQUIRE_SIGNING: true` → `xpinstall.signatures.required=false` does
  nothing.
- `MOZ_UNSIGNED_SYSTEM_SCOPE: true`, `MOZ_ALLOW_ADDON_SIDELOAD: true`,
  `extensions.autoDisableScopes = 0` → an unsigned add-on placed in
  `/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/`
  loads and is not auto-disabled. A symlink named after the add-on ID that
  points at an unpacked directory is accepted (`DirectoryLocation._readLinkFile`).
- The static theme schema accepts `dark_theme`; Firefox uses it when the
  system prefers dark. One theme covers both modes, so `theme` has nothing to
  push and Firefox is never relaunched.
- `extensions.startupScanScopes = 0` → the system scope is only scanned when
  the Firefox build ID changes. `install_addons.sh` sets it to 8
  (`SCOPE_SYSTEM`) in each install's default profile so the theme is found on
  the next start and manifest edits apply after a restart.

Install once: close Firefox, run `firefox/install_addons.sh` (sudo for the
symlinks; also installs the new tab page, see
[firefox-newtab.md](firefox-newtab.md)), start Firefox; if Oxide is not active, enable it in
about:addons → Themes. The `theme` Firefox surface now only checks the
symlink.

Non-goals (unchanged from [theme-firefox.md](theme-firefox.md)): web-ext,
AMO signing, `userChrome.css`. This route depends on Fedora's build flags; a
Mozilla-built Firefox (tarball, Mozilla flatpak) would not load it.

macOS (Firefox Developer Edition): Mozilla builds require signing in every
scope; Developer Edition / Nightly honor `xpinstall.signatures.required=false`.
`install_addons.sh` on Darwin zips each add-on into
`<profile>/extensions/<add-on id>.xpi` for each install's default
profile, sets that pref in `prefs.js`, and deletes `addonStartup.json.lz4` so
the profile scope is rescanned. Trade-off accepted: the pref applies to all
add-ons in the profile. Sideloaded add-ons start disabled; enable once in
about:addons → Themes. Rerun after editing the manifest. Stable Firefox on
macOS cannot load it.

## macOS Ghostty

Ghostty on macOS reads `~/Library/Application Support/com.mitchellh.ghostty/config`
(not `~/.config/ghostty.config`). That file is a symlink to
`.config/ghostty.config`; `~/.config/ghostty/themes/Oxide {Dark,Light}` are
symlinks to `ghostty/themes/`. The macOS theme-cli config
(`~/Library/Application Support/theme-cli/config.toml`) names the Oxide
themes under `[ghostty]`.

## Thunderbird

Same theme format as Firefox, so `thunderbird/install_theme.sh` zips
`firefox/oxide/manifest.json` into `oxide@theme-cli.xpi` in each install's
default profile under `~/.var/app/net.thunderbird.Thunderbird/.thunderbird/`.
One manifest for both apps, no second copy to keep in sync.

Facts checked on the flatpak (156) `omni.ja`: `MOZ_REQUIRE_SIGNING: false`
and `xpinstall.signatures.required` defaults to `false`, so an unsigned XPI
in the profile loads without changing any pref; `extensions.autoDisableScopes
= 0`; the theme schema accepts `dark_theme`. The profile scope is scanned at
startup only when `addonStartup.json.lz4` is missing or the build changes,
so the script deletes that cache (Thunderbird rebuilds it). Thunderbird must
be closed. Rerun after editing the manifest. The previously active Everforest
theme from addons.thunderbird.net stays installed; enable Oxide once in
Add-ons and Themes → Themes.

## Obsidian

`theme.css` defines the palette per `.theme-dark` / `.theme-light` and maps
it onto Obsidian's own CSS variables; no DOM restyling, no fonts (the vaults
set Inter / Newsreader / Berkeley Mono). Vault appearance stays
`"theme": "system"`, so Obsidian follows the GNOME color-scheme; no `theme`
surface needed.

- Headings bright_text (size/weight carry the level), links blue, external
  links sky, tags pink, inline code orange, syntax per nvim roles, callouts by
  semantic type.
- `install_theme.sh` copies (not symlinks) the theme into
  `<vault>/.obsidian/themes/Oxide/` for `~/workspace/obsidian/{work,private}`
  and sets `cssTheme`. Copy because the vaults sync to GitHub/mobile through
  obsidian-git, where a symlink into this repo would not resolve. Rerun after
  editing `theme.css`.
- Replaces Minimal as the vault theme. Minimal's layout styling is gone;
  Minimal and its settings plugin stay installed.

## Non-goals

- Icon theme (Adwaita stays).
- Ableton skins (`ableton/`), Spotify: not `theme` surfaces.
- Lazygit / yazi: upstream ports exist but they are not `theme` surfaces.
- Upstreaming the light palette or the new ports to oxidescheme.
