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
| Popup | QR + rewritten URL as text + click-to-copy + `commands` keyboard shortcut (`_execute_action`, MV3-only in both browsers) |
| Permissions | `activeTab` only. `data_collection_permissions.required: ["none"]` for Firefox — the tab URL goes to a loopback companion on the same machine and is never transmitted off-device, so there is nothing to disclose |
| Extension API namespace | `globalThis.browser ?? globalThis.chrome`, not the `webextension-polyfill` dependency. Firefox keeps `chrome` callback-only and puts promises on `browser`; Chrome only added `browser` in 148. The popup makes exactly one API call, so a two-token fallback beats a vendored polyfill |
| Firefox add-on ID | Pinned as `qr-lan@pixelwerte.de`. Required for signing, and harmless otherwise |
| Firefox version floor | `strict_min_version` 140 (142 for `gecko_android`), set by `data_collection_permissions` rather than by anything the extension does — the code itself works from Firefox 109. Chosen so `addons-linter` is clean; nothing in reach runs a Firefox that old |
| Location | This repo, `qr-lan/` with `companion/` (Cargo) and `extension/` |
| Install | Single `install.sh`: `cargo build --release`, symlink binary and the per-OS unit file, enable it, print the per-browser load instructions |

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

What needed no change: the loopback `fetch` (the popup is a `moz-extension:`
page making an ordinary non-credentialed cross-origin request, and the companion
already answers `Access-Control-Allow-Origin: *`), `activeTab`, the `action`
popup, the `Alt+Shift+Q` command, and the CSS.

Verified with `addons-linter` (0 errors, 0 warnings) plus a Playwright harness
that renders `popup.html` against a stubbed `browser`/`chrome` pair in both
shapes and checks the QR, the copy label, and all three failure states.

### The install asymmetry

This is the one place Firefox is genuinely worse, and it is not fixable in the
manifest. Chrome's load-unpacked is permanent. Firefox's equivalent,
`about:debugging` → Load Temporary Add-on, is discarded on every browser
restart, and release and beta Firefox refuse unsigned add-ons permanently.
A restart-surviving install therefore needs either AMO signing or Developer
Edition / Nightly / ESR with `xpinstall.signatures.required = false`.
`install.sh` prints both options and does not choose for you.

## Layout

```
qr-lan/
  companion/src/main.rs     axum server, /qr handler, PORT constant
  companion/src/lan.rs      default-route source IP
  companion/src/rewrite.rs  loopback-host rewrite + its tests
  extension/                MV3 manifest (Chrome + Firefox), popup HTML/CSS/JS
  service/qr-lan.service    systemd --user unit (Linux)
  service/de.pixelwerte.qr-lan.plist  LaunchAgent (macOS)
  install.sh                build, symlink, enable, print per-browser load steps
```

## Install

```bash
qr-lan/install.sh
```

Then load the extension, once per machine per browser. Shortcut in both:
`Alt+Shift+Q`.

- **Chrome/Chromium:** `chrome://extensions` → Developer mode → Load unpacked →
  `qr-lan/extension`. Permanent.
- **Firefox:** `about:debugging#/runtime/this-firefox` → Load Temporary Add-on →
  `qr-lan/extension/manifest.json`. Gone on restart — see the install asymmetry
  above for the permanent alternatives.

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
- **No AMO submission, signing pipeline, or packaged release.** That would put
  a Mozilla review step and a re-sign on every edit between this repo and a
  working browser, which is the opposite of what a dotfiles checkout is for.
  The consequence is accepted: on release Firefox this is a temporary add-on.
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
