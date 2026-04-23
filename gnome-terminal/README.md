# Everforest for GNOME Terminal

Two dconf files, one per mode. The `theme` CLI creates the profile on first run
and loads the matching colors on each invocation — no manual setup in the
terminal preferences needed (useful, since recent GNOME Terminal removed most
of the profile UI).

## What the CLI does on first run

1. Registers a profile with a fixed UUID
   (`3b1a4e2f-8c7d-4b9e-a1f2-c3d4e5f6a7b8`) in
   `/org/gnome/terminal/legacy/profiles:/list`.
2. Names it `Everforest` and sets it as the default profile.
3. Loads the dark or light dconf file into that profile's subpath.

Subsequent runs just reload the colors.

## Manual load (if you ever need it)

```
UUID=3b1a4e2f-8c7d-4b9e-a1f2-c3d4e5f6a7b8
dconf load /org/gnome/terminal/legacy/profiles:/:$UUID/ < everforest-dark.dconf
```

## Source

- Dark: [em3n/Everforest-GnomeTerminal](https://github.com/em3n/Everforest-GnomeTerminal)
- Light: hand-authored from the upstream Everforest Light Medium palette
