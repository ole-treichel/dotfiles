# Everforest for GNOME Terminal

Both variants load into a single profile UUID so switching swaps colors in place.

## First-time setup

1. In GNOME Terminal → Preferences, create a new profile named "Everforest" and set it as default.
2. Find its UUID:

   ```
   dconf list /org/gnome/terminal/legacy/profiles:/
   ```

3. Put that UUID into `theme-cli/config.toml` under `[gnome_terminal] profile_uuid`.

## Manual load

```
UUID=<your-profile-uuid>
dconf load /org/gnome/terminal/legacy/profiles:/:$UUID/ < everforest-dark.dconf
dconf load /org/gnome/terminal/legacy/profiles:/:$UUID/ < everforest-light.dconf
```

The `theme` CLI automates this — you only use these commands if you're bootstrapping.

## Source

- Dark: [em3n/Everforest-GnomeTerminal](https://github.com/em3n/Everforest-GnomeTerminal)
- Light: hand-authored from the upstream Everforest Light Medium palette
