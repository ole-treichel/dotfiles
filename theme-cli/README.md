# theme-cli

Single command to switch system-wide light/dark mode across GNOME, Neovim, tmux, and GNOME Terminal.

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

Edit `config.toml` (or copy to `~/.config/theme-cli/config.toml` for a user-local override) and fill in:

- `[gnome_terminal] profile_uuid` — see `gnome-terminal/README.md`
- `[gtk] light` / `dark` — GTK theme names shown in gnome-tweaks (defaults: `Everforest-Light-Medium` / `Everforest-Dark-Medium`)

## Use

```
theme status   # → light | dark
theme dark
theme light
theme toggle
```

Each surface reports its own outcome. Exit code is 1 if any surface failed, 0 otherwise.
