# Firefox: custom new tab page

Status: implemented 2026-10-19. Scope for now: blank page, Oxide `base`
background, light or dark per color-scheme.

Files: `firefox/newtab/` (`manifest.json`, `index.html`), installed by
`firefox/install_addons.sh` together with the Oxide theme.

## Decisions

### WebExtension `chrome_url_overrides.newtab`

The page is an unsigned add-on (`newtab@dotfiles`) that overrides the new tab
URL, symlinked into the system extension scope the same way as the Oxide
theme. Same build-flag dependency and facts as
[oxide-theme.md § Firefox](oxide-theme.md#firefox).

Separate add-on from the theme: a manifest with a `theme` key is a static
theme and cannot carry `chrome_url_overrides`.

Facts checked on Fedora's `firefox-157.0.1-1.fc44` (`browser/omni.ja`):

- `parent/ext-url-overrides.js` sets `AboutNewTab.newTabURL` to the
  extension's `moz-extension://…/index.html`.
- `browser.js` `isInitialPage()` treats `BROWSER_NEW_TAB_URL` (that URL) as an
  initial page: the address bar stays empty and focused, as with
  `about:newtab`.
- `ExtensionControlledPopup` shows a one-time notice on the first new tab
  ("an extension changed your new tab page"). Keep the change; the
  confirmation is stored per add-on.

Rejected:

- `browser.newtab.url` pref: removed in Firefox 41.
- Autoconfig (`firefox.cfg` + `defaults/pref/autoconfig.js` in
  `/usr/lib64/firefox`, setting `AboutNewTab.newTabURL` from privileged JS):
  files inside the RPM install dir, unsandboxed chrome JS, a `file://` URL in
  the address bar.
- Enterprise policies: `NewTabPage` only enables/disables Firefox's page; no
  policy sets a URL.

### Light/dark via `prefers-color-scheme`

`index.html` sets `#f5f5f5` (light base) and `#161616` (dark base) under
`@media (prefers-color-scheme: dark)`, plus `color-scheme: light dark` so the
canvas before first paint already matches. Same values as `ntp_background` in
`firefox/oxide/manifest.json`. Content `prefers-color-scheme` follows the
browser theme (Website appearance: Automatic), and the Oxide theme follows the
GNOME color-scheme through `dark_theme`, so `theme light|dark` switches the
page with no extra surface. Literal hex, no generator (as in
oxide-theme.md).

No JavaScript: nothing to compute.

## Non-goals

- Content (clock, search box, shortcuts, bookmarks). Add to `index.html`
  later; extension CSP allows only scripts from the add-on itself.
- Home page / `about:home` (`browser.startup.homepage`): unchanged.
- Private windows: extensions do not run there by default, so private
  windows keep `about:privatebrowsing`. Enable "Run in Private Windows" for
  New Tab Page in about:addons if wanted.
- A `theme` CLI surface for it: nothing to switch at runtime.
