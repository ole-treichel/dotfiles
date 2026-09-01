# QR-LAN — plan

Browser extension + Rust companion that turns the current tab into a QR code a
phone on the same LAN can scan, rewriting loopback hosts to the machine's LAN IP.

Status: implemented in `qr-lan/`. Chrome/Chromium and Firefox.

## Why the companion is required

The original brief assumed the companion binary might be optional. It isn't:

- `chrome.system.network` is a **Platform Apps** API, not an extension API
  ([docs](https://developer.chrome.com/docs/apps/reference/system/network)), and
  Chrome Apps are dead.
- The remaining trick — harvesting WebRTC ICE host candidates — is deliberately
  defeated by Chrome's mDNS `.local` candidate obfuscation.

Firefox is no better: it has no LAN-IP API either, and it obfuscates WebRTC ICE
host candidates behind random `.local` mDNS names by default too
(`media.peerconnection.ice.obfuscate_host_addresses`, default `true` on desktop).

So an MV3 extension cannot learn its machine's LAN IP by itself, in either
browser. Rendering the QR server-side (see below) makes the daemon a hard
dependency for *every* QR, including ones for public URLs.

## Decisions

| Area | Decision |
| --- | --- |
| Browser | Chrome/Chromium and Firefox, MV3 only. One unpacked extension, no build step and no per-browser manifest |
| IP source | Rust companion, auto-detected |
| IP selection | Default-route source IP: `UdpSocket::bind("0.0.0.0:0")` + `connect("1.1.1.1:80")` + `local_addr()`. Connectionless — sends no packets, needs no root, identical on Linux and macOS, and avoids an interface denylist that would otherwise pick `docker0`/`virbr0` |
| Address family | IPv4 only. Link-local IPv6 needs a zone index that isn't URL-expressible; global IPv6 doesn't route over the LAN. Explicit error if no IPv4 route |
| Transport | axum + tokio, bound to `127.0.0.1:48213` |
| API | `GET /qr?url=<tab url>` → `{ "url": "<rewritten>", "svg": "<qr svg>" }` |
| Rewrite rule | Rewrite host only when it is `localhost`, `127.0.0.0/8`, `0.0.0.0`, `[::1]` or `*.localhost`; preserve port, path, query, hash. Every other URL is QR'd verbatim |
| QR render | Rust `qrcode` crate → SVG string, inlined into the popup DOM |
| Access control | `Access-Control-Allow-Origin: *`, loopback bind, no token. The companion authorises no origin in particular, so the extension's differing ID per browser and per machine does not matter |
| Lifecycle | Login service: systemd `--user` unit (Linux) + LaunchAgent (macOS), enabled once per machine |
| Popup | QR + rewritten URL as text + click-to-copy + `commands` keyboard shortcut (`_execute_action`, MV3-only in both browsers, no default key — see Firefox support) |
| Permissions | `activeTab` only. `data_collection_permissions.required: ["none"]` for Firefox — the tab URL goes to a loopback companion on the same machine and is never transmitted off-device, so there is nothing to disclose |
| Extension API namespace | `globalThis.browser ?? globalThis.chrome`, not the `webextension-polyfill` dependency. Firefox keeps `chrome` callback-only and puts promises on `browser`; Chrome only added `browser` in 148. The popup makes exactly one API call, so a two-token fallback beats a vendored polyfill |
| Firefox add-on ID | Pinned as `qr-lan@pixelwerte.de`. Required for signing, and harmless otherwise |
| Firefox version floor | `strict_min_version` 140 (142 for `gecko_android`), set by `data_collection_permissions` rather than by anything the extension does — the code itself works from Firefox 109. Chosen so `addons-linter` is clean; nothing in reach runs a Firefox that old |
| Location | This repo, `qr-lan/` with `companion/` (Cargo) and `extension/` |
| Install | Single `install.sh`: `cargo build --release`, symlink binary and the per-OS unit file, enable it, build the `.xpi`, print the per-browser load instructions |
| Packaging | `package.sh` zips `extension/` to `dist/qr-lan.xpi`. Not a build step — the extension stays plain unpacked files that Chrome loads directly. The zip exists solely because a Flatpak Firefox can only be handed one file through the portal |
| Signing | `sign.sh`, opt-in, `unlisted` channel. Only route to a restart-surviving install on release Firefox, which hard-codes signature enforcement. Credentials via `AMO_JWT_ISSUER`/`AMO_JWT_SECRET`, passed to web-ext through `WEB_EXT_*` env vars rather than `--api-secret` so the secret never lands in argv where `ps` can read it |

## Firefox support

Added after the fact; the original plan was Chrome-only. Three things were in
the way. Chrome ignores every manifest key involved, so there is still one
extension directory rather than two.

1. **`await chrome.tabs.query(...)` returned `undefined`.** Firefox exposes a
   `chrome` namespace for compatibility but keeps it *callback-only*; promises
   live on `browser`. So the destructuring in `popup.js` threw
   `TypeError: (intermediate value) is not iterable` before any `showStatus`
   call could run — the popup sat on its `…` placeholder with no error shown.
   Fixed by resolving the namespace once at the top of `popup.js`.
2. **No add-on ID.** Now pinned under `browser_specific_settings.gecko`.
3. **Data collection disclosure.** Mozilla has required
   `data_collection_permissions` for new extensions
   [since 2025-11-03](https://blog.mozilla.org/addons/2025/10/23/data-collection-consent-changes-for-new-firefox-extensions/);
   declared as `["none"]`.
4. **Popup rendered as a tiny empty box — the Flatpak document portal.** The
   inspector showed literally `<html><head></head><body></body></html>` and no
   errors. Not a CSS bug: Firefox never read `popup.html` at all.

   Firefox here is a Flatpak (`org.mozilla.firefox`). `about:debugging` ->
   Load Temporary Add-on opens the XDG desktop portal file picker, and the
   portal exports **only the single file selected**, at a synthetic path
   `/run/user/$UID/doc/<id>/manifest.json`. Firefox takes that directory as
   the extension root. It contains nothing but `manifest.json`, so the
   manifest parses and the extension installs with a working toolbar button,
   while `popup.html`, `popup.js` and `icons/` are simply absent. A missing
   popup document renders as an empty one, with nothing to log — and an empty
   document has no layout size, hence the ~30px panel.

   Confirmed directly, without attaching to the running browser:

   ```console
   $ flatpak documents --columns=all org.mozilla.firefox
   90ed68dd  /run/user/1000/doc/90ed68dd/manifest.json  /home/ole/workspace/dotfiles/qr-lan/extension/manifest.json  ...
   $ ls /run/user/1000/doc/90ed68dd/
   manifest.json
   ```

   Chrome is unaffected because *Load unpacked* uses a **directory** picker,
   so the portal exports the whole tree. Granting the sandbox the repo
   (`--filesystem=<repo>:ro`, which `theme-cli`'s Firefox surface already
   does) does not help: the problem is not permission on the real path, it is
   that Firefox was handed the portal path instead of the real one.

   Fix: `package.sh` zips `extension/` into `dist/qr-lan.xpi`, and Firefox
   loads *that*. One file, so the portal export is self-contained.
   `about:debugging` accepts an `.xpi`/`.zip` for temporary loading with no
   signature check. `web-ext run --source-dir` is the other way out, since it
   hands Firefox the real path and never touches the portal.

   Everything tried before this diagnosis was aimed at a phantom and none of
   it mattered: inlining `popup.css` into a `<style>` block, reserving the
   footprint with `min-height`, declaring all three states as static markup
   toggled by `data-state`, and adding `icons/icon-*.png`. Those changes are
   kept — they are defensible on their own and the popup works with them —
   with one exception. The redundant inline `style="background: #fffaf3"` on
   `<body>` was **removed**: inline styles outrank the stylesheet, so it
   pinned the popup to the light background and broke dark mode. The lesson
   worth keeping: when a browser shows an empty document and logs nothing,
   suspect the file it loaded, not the file you wrote.

What needed no change: the loopback `fetch` (the popup is a `moz-extension:`
page making an ordinary non-credentialed cross-origin request, and the companion
already answers `Access-Control-Allow-Origin: *`), `activeTab`, and the
`action` popup mechanism itself.

Verified with `addons-linter` (0 errors, 0 warnings) plus a Playwright harness
that renders `popup.html` against a stubbed `browser`/`chrome` pair in both
shapes and checks the QR, the copy label, and all three failure states. Note
this harness renders `popup.html` directly as a page — it does not exercise
the real WebExtension popup-panel sizing path, which is exactly what missed
issue 4 above; that one needed an actual `about:debugging` load to surface.

### The install asymmetry

This is the one place Firefox is genuinely worse, and it is not fixable in the
manifest. Chrome's load-unpacked is permanent. Firefox's equivalent,
`about:debugging` → Load Temporary Add-on, is discarded on every browser
restart, and release and beta Firefox refuse unsigned add-ons permanently.
A restart-surviving install therefore needs either AMO signing or Developer
Edition / Nightly / ESR with `xpinstall.signatures.required = false`.

On this machine only the first is actually available. Firefox here is 154
stable from the Fedora flatpak remote, and release builds ignore
`xpinstall.signatures.required` entirely — the pref is honoured only by
Developer Edition, Nightly and ESR. Neither Flathub nor Fedora ships a
Developer Edition or Nightly flatpak, so that branch means a tarball install
outside Flatpak, which is a bigger change than the problem justifies. Hence
`sign.sh`.

A Flatpak Firefox adds a second cost on top: the temporary add-on has to be
loaded as a packaged `.xpi` rather than as a loose directory, because the
portal file picker only ever exports one file (Firefox support item 4). So
every edit needs a `package.sh` re-run and a re-load, where Chrome needs only
a re-load — unless you iterate through `web-ext run`, which sidesteps both.

## Layout

```
qr-lan/
  companion/src/main.rs     axum server, /qr handler, PORT constant
  companion/src/lan.rs      default-route source IP
  companion/src/rewrite.rs  loopback-host rewrite + its tests
  extension/                MV3 manifest (Chrome + Firefox), popup HTML (CSS inlined) + JS, icons/
  service/qr-lan.service    systemd --user unit (Linux)
  service/de.pixelwerte.qr-lan.plist  LaunchAgent (macOS)
  package.sh                zip extension/ -> dist/qr-lan.xpi (for Flatpak Firefox)
  sign.sh                   opt-in: AMO-sign extension/ -> dist/*.xpi (permanent install)
  install.sh                build, symlink, enable, package, print per-browser load steps
  dist/                     build output, gitignored
```

## Install

```bash
qr-lan/install.sh
```

Then load the extension, once per machine per browser. No keyboard shortcut is
bound by default — assign one under `about:addons` → Manage Extension Shortcuts,
or `chrome://extensions/shortcuts`.

- **Chrome/Chromium:** `chrome://extensions` → Developer mode → Load unpacked →
  `qr-lan/extension`. Permanent.
- **Firefox:** `about:debugging#/runtime/this-firefox` → Load Temporary Add-on →
  `qr-lan/dist/qr-lan.xpi` (built by `install.sh`, or `qr-lan/package.sh` on its
  own after an edit). **Pick the `.xpi`, not `extension/manifest.json`** — see
  Firefox support item 4. Gone on restart; see the install asymmetry above for
  the permanent alternatives.
- **Firefox, while iterating:** `web-ext run --source-dir qr-lan/extension`
  reloads on save and skips the portal and the repackaging step entirely.

### Permanent Firefox install (signing)

Opt-in, and not part of `install.sh`. Get a JWT issuer and secret once from
<https://addons.mozilla.org/developers/addon/api/key/>, then:

```bash
export AMO_JWT_ISSUER=user:12345678:123
export AMO_JWT_SECRET=...
qr-lan/sign.sh
```

Then `about:addons` → gear → Install Add-on From File → the `dist/*.xpi` it
wrote. Survives restarts; remove the temporary add-on from `about:debugging`
first.

`sign.sh` refuses to run if `dist/` already holds a signed `.xpi` for the
current `manifest.json` version, because AMO rejects a re-used version several
seconds into the upload. So each update is: bump `version`, `sign.sh`,
re-install. That bump-and-re-sign cycle is the whole reason this is not the
default path.

Two implementation notes worth remembering:

- The LAN IP is resolved **per request**, not at startup, so moving between
  networks needs no restart.
- The LaunchAgent runs `/bin/sh -c 'exec "$HOME/.local/bin/qr-lan"'`. launchd
  does not expand `$HOME` in `ProgramArguments`, and the plist is symlinked from
  the repo rather than generated, so the absolute path cannot be baked in.

## Deliberate non-goals

- **No reachability probe.** If the dev server binds `127.0.0.1` instead of
  `0.0.0.0`, or `firewalld` blocks the port, the QR looks correct and the phone
  simply hangs. Diagnosing that is manual.
- **No rich error UI.** A dead daemon renders a terse one-line failure, not a
  start command.
- **No config file, flags, or env vars.** The port is a constant duplicated in
  the Rust source and the popup JS.
- **No signing on the default path.** `sign.sh` exists (see Signing below) but
  nothing calls it: `install.sh` does not sign, and the documented Firefox
  install is still the temporary `.xpi`. Putting a Mozilla round trip and a
  version bump between an edit and a working browser is the opposite of what a
  dotfiles checkout is for. Signing is the opt-in escape hatch for when the
  per-restart reload becomes more annoying than the re-sign, not the norm.
- **No AMO listing.** Signing uses the `unlisted` channel — self-distribution,
  no public add-on page, no human review. This extension is useless without
  the companion daemon on the same machine, so there is nobody to list it for.
- **No `update_url`.** A self-distributed add-on can advertise an update
  manifest and auto-update itself; that would mean hosting one. Re-running
  `sign.sh` and re-installing is fine at this frequency.
- **No `webextension-polyfill`.** One API call does not justify vendoring a
  dependency into a repo with no build step.
- **No tunnels, HTTPS, or auth.** Same-LAN plain HTTP only.

## Open details

- Non-`http(s)` tabs (`chrome://`, `file://`, `about:`) show a "can't share this"
  state instead of a QR.
- The extension stays unpacked and is loaded from the repo path, so its runtime
  ID differs per machine (and Firefox assigns a fresh temporary one per load) —
  harmless given `ACAO: *`. The pinned `gecko.id` is for signing, not identity
  at request time.
- `browser_specific_settings` and `gecko_android` are unknown keys to Chrome.
  Verified that Chromium loads the extension with no manifest warning.
- A VPN owning the default route (Tailscale, WireGuard) makes the UDP trick
  return the VPN address rather than the LAN address.
