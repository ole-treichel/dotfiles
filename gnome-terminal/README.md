# Oxide for GNOME Terminal

Two dconf files, one per mode. The `theme` CLI module for GNOME Terminal is
currently not called (ghostty superseded it); the files are kept in sync with
`oxide/colors.txt` in case it returns.

## What the module does on first run

1. Registers a profile with a fixed UUID
   (`3b1a4e2f-8c7d-4b9e-a1f2-c3d4e5f6a7b8`) in
   `/org/gnome/terminal/legacy/profiles:/list`.
2. Names it `Oxide` and sets it as the default profile.
3. Loads the dark or light dconf file into that profile's subpath.

Subsequent runs just reload the colors.

## Manual load

```
UUID=3b1a4e2f-8c7d-4b9e-a1f2-c3d4e5f6a7b8
dconf load /org/gnome/terminal/legacy/profiles:/:$UUID/ < oxide-dark.dconf
```

## Source

- Dark: ANSI mapping of [oxidescheme/ghostty](https://github.com/oxidescheme/ghostty)
- Light: same mapping over the derived light palette in `oxide/colors.txt`
