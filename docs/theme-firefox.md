# theme-cli: Firefox surface (deactivated)

Status: **deactivated 2026-09-08**. Code and themes stay in the repo; the
surface no-ops via empty IDs in `theme-cli/config.toml`.

## What it did

`theme light` / `theme dark` killed the running Firefox and relaunched it via
`npx web-ext run --source-dir firefox/everforest-{light,dark} --firefox-profile
default-release --keep-profile-changes`, loading the matching theme manifest as
a *temporary* add-on. Temporary was the only option: Firefox stable refuses to
permanently install unsigned themes, with no override outside
Nightly/Dev Edition/ESR.

## Why it's off

`--keep-profile-changes` points web-ext's automation harness at the real
profile, and web-ext writes its 72-line test-harness `user.js` there. Firefox
then persists those into `prefs.js` (64 of them, verified in
`vcjm4lr5.default-release` after the 2026-09-08 11:07 run). The profile came
back with:

- `browser.safebrowsing.enabled` / `.malware.enabled` = false, and the
  safebrowsing provider URLs repointed at `http://localhost/…-dummy/…`
- `signon.rememberSignons` = false — no password saving
- `browser.startup.page` = 0 and `browser.startup.homepage` = `about:blank`
- `browser.sessionstore.resume_from_crash` = false — tabs lost on crash
- `app.update.*`, `extensions.update.*`, `extensions.blocklist.enabled` = off
- `devtools.debugger.remote-enabled` = true with
  `devtools.debugger.prompt-connection` = false — silent remote debugging
- `xpinstall.signatures.required` = false, `extensions.autoDisableScopes` = 10
- the `security.warn_*` family and `dom.disable_open_during_load` = false

Mozilla documents `--keep-profile-changes` as destructive and insecure for
daily use. The earlier judgement (in the removed config comment) accepted that
to theme the real profile; the concrete blast radius above — a browser that no
longer saves passwords, restores tabs, updates, or checks malware — is not a
trade worth making for a color scheme.

Also relevant: every theme switch had to kill and relaunch the browser, so a
`theme toggle` cost the whole browsing session.

## Cleanup applied

- `user.js` deleted from `vcjm4lr5.default-release`
- the 64 persisted prefs stripped from `prefs.js` (backup:
  `prefs.js.pre-webext-cleanup`) — a copy of the removed `user.js` and the
  pref-name list are in the session scratchpad
- `flatpak override --user --reset org.mozilla.firefox` (dropped the
  `--filesystem=<repo>:ro` grant the surface added)

## Non-goals

- **Don't** sign the themes through addons.mozilla.org just to keep the
  surface. Two throwaway themes on AMO, per-machine API credentials, and a
  review turnaround per color tweak — too much machinery for this.
- **Don't** run the theme in a separate web-ext throwaway profile. A second
  Firefox with none of the real history, logins, or extensions isn't the
  browser being themed.
- **Don't** replace it with `userChrome.css`. That needs
  `toolkit.legacyUserProfileCustomizations.stylesheets`, breaks across Firefox
  releases, and is a second theme definition to keep in sync with
  `firefox/*/manifest.json`.

## If it's ever revisited

The blocker is Firefox's signing requirement, not the theme content. A real fix
would set the theme through an API Firefox exposes without a harness — e.g. one
signed switcher extension holding both palettes that flips on a message, or the
built-in light/dark themes if `browser.theme.toolbar-theme` following the GTK
scheme is ever good enough. Until one of those exists, `theme` leaves Firefox
alone; Firefox's own "System theme" setting tracks the GNOME color-scheme that
`theme` already sets.
