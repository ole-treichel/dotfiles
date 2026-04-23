# Everforest themes for Chrome

Two unpacked extensions — one per mode. The `theme` CLI swaps between them by editing Chrome's `Preferences` JSON.

## First-time setup

1. Open `chrome://extensions` and enable **Developer mode** (top-right toggle).
2. Click **Load unpacked** and select `everforest-dark/` from this repo.
3. Repeat for `everforest-light/`.
4. On the extensions page, copy the **ID** under each theme (a 32-char lowercase string).
5. Paste both IDs into `theme-cli/config.toml` under `[chrome]`:

   ```toml
   [chrome]
   dark_extension_id  = "<dark-id>"
   light_extension_id = "<light-id>"
   ```

Unpacked extension IDs are derived from the absolute directory path, so they stay stable as long as this repo stays at its current location.

## Source

- Dark: [talwat/dotfiles](https://github.com/talwat/dotfiles) — Everforest Dark Medium.
- Light: hand-authored from the upstream Everforest Light Medium palette.
