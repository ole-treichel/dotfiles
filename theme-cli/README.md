# theme-cli

Single command to switch system-wide light/dark mode across GNOME (GTK + Shell), Neovim, tmux, Ghostty, Chrome, Hunk (git pager/difftool), and delta (git's `interactive.diffFilter`). All surfaces use the Oxide scheme: upstream dark, derived light (`../oxide/colors.txt`, `../docs/oxide-theme.md`).

`gsettings get org.gnome.desktop.interface color-scheme` is the source of truth. `theme light` / `theme dark` set it and push matching changes to the surfaces that don't auto-follow.

## Build

```
cargo build --release
```

The binary lands at `target/release/theme`. Symlink it into `~/.local/bin/`:

```
ln -sf "$PWD/target/release/theme" ~/.local/bin/theme
```

## One-time setup

Edit `config.toml` (or copy to `~/.config/theme-cli/config.toml` for a user-local override):

- `[gtk] light` / `dark` — GTK theme names shown in gnome-tweaks (defaults: `Oxide-Light` / `Oxide-Dark`, built by `../gtk/build_oxide_gtk.sh`)

The GNOME Terminal profile is created automatically on first run — no manual setup.

Firefox: run `../firefox/install_addons.sh` once. The theme holds both palettes (`dark_theme`) and follows the system color-scheme, so the Firefox surface only reports whether it is installed. See [docs/oxide-theme.md](../docs/oxide-theme.md).

## Use

```
theme status   # → light | dark
theme dark
theme light
theme toggle
```

Each surface reports its own outcome. Exit code is 1 if any surface failed, 0 otherwise.
