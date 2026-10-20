# Spotify window decorations

Status: applied 2026-10-19 via `spotify/install.sh`.

## Problem

Spotify flatpak (1.2.95, CEF 146) on GNOME 50 Wayland shows a blue title bar
with small min/max/close buttons.

## Cause

Spotify detects Wayland and runs with `--ozone-platform=wayland`. Mutter
implements no server-side decorations (`xdg-decoration`), so CEF draws its own
frame: Chromium's `views::CustomFrameView`, default color blue.

## Decision

Run Spotify under XWayland. Mutter then draws the frame (`mutter-x11-frames`
window, GNOME title bar).

    flatpak override --user --nosocket=wayland --socket=x11 com.spotify.Client

Both flags are required:

- `--nosocket=wayland`: without the Wayland socket, Spotify selects X11.
- `--socket=x11`: the manifest only has `fallback-x11`. Flatpak does not grant
  it on a Wayland session, even when the app's Wayland socket is removed
  (verified: `DISPLAY` empty inside the sandbox).

Verification: `xprop` on the Spotify window shows `_MOTIF_WM_HINTS` with
decorations = 1, and Mutter creates a `mutter-x11-frames` window for it.

## Rejected

- `--ozone-platform=x11` in `~/.var/app/com.spotify.Client/config/spotify-flags.conf`:
  no X11 socket in the sandbox, Spotify exits at startup.
- `--enable-features=WaylandWindowDecoration`: Electron feature, Spotify is CEF.
- Symlinking the override file from the repo: `flatpak override` rewrites
  `~/.local/share/flatpak/overrides/com.spotify.Client` and replaces the link.
  The script is idempotent instead.

## Trade-offs

- XWayland renders blurry under fractional scaling. Fractional scaling is off
  (`org.gnome.mutter experimental-features` = `[]`). Re-evaluate if it is
  enabled.
- Revert: `flatpak override --user --reset com.spotify.Client`.

## Non-goals

- Theming the title bar. It follows the GNOME/Mutter theme.
- Spicetify. It modifies web content only, not the native frame.
