# theme-cli

Single command to switch system-wide light/dark mode across GNOME, Neovim, tmux, GNOME Terminal, Chrome, Firefox, Hunk (git pager/difftool), and delta (git's `interactive.diffFilter`).

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

- `[gtk] light` / `dark` — GTK theme names shown in gnome-tweaks (defaults: `Everforest-Light` / `Everforest-Dark`)
- `[firefox] profile` — Firefox profile name (as shown in `about:profiles`) to load the theme into (default: `default-release`)

The GNOME Terminal profile is created automatically on first run — no manual setup.

Firefox is applied via [`web-ext run`](https://github.com/mozilla/web-ext) loading the theme as a temporary add-on (Firefox stable refuses to permanently install unsigned themes, no override outside Nightly/Dev Edition/ESR). This requires `npx`/`web-ext` on `PATH` and runs with `--keep-profile-changes`, which Mozilla's docs note makes the target profile insecure for daily use (enables silent remote debugging connections, etc) — an accepted trade-off for theming the real profile instead of a throwaway one.

## Use

```
theme status   # → light | dark
theme dark
theme light
theme toggle
```

Each surface reports its own outcome. Exit code is 1 if any surface failed, 0 otherwise.
