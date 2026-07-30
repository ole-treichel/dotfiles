# QR-LAN — plan

Chrome extension + Rust companion that turns the current tab into a QR code a
phone on the same LAN can scan, rewriting loopback hosts to the machine's LAN IP.

Status: planned, not implemented.

## Why the companion is required

The original brief assumed the companion binary might be optional. It isn't:

- `chrome.system.network` is a **Platform Apps** API, not an extension API
  ([docs](https://developer.chrome.com/docs/apps/reference/system/network)), and
  Chrome Apps are dead.
- The remaining trick — harvesting WebRTC ICE host candidates — is deliberately
  defeated by Chrome's mDNS `.local` candidate obfuscation.

So an MV3 extension cannot learn its machine's LAN IP by itself. Rendering the
QR server-side (see below) makes the daemon a hard dependency for *every* QR,
including ones for public URLs.

## Decisions

| Area | Decision |
| --- | --- |
| Browser | Chrome/Chromium, MV3 only |
| IP source | Rust companion, auto-detected |
| IP selection | Default-route source IP: `UdpSocket::bind("0.0.0.0:0")` + `connect("1.1.1.1:80")` + `local_addr()`. Connectionless — sends no packets, needs no root, identical on Linux and macOS, and avoids an interface denylist that would otherwise pick `docker0`/`virbr0` |
| Address family | IPv4 only. Link-local IPv6 needs a zone index that isn't URL-expressible; global IPv6 doesn't route over the LAN. Explicit error if no IPv4 route |
| Transport | axum + tokio, bound to `127.0.0.1:48213` |
| API | `GET /qr?url=<tab url>` → `{ "url": "<rewritten>", "svg": "<qr svg>" }` |
| Rewrite rule | Rewrite host only when it is `localhost`, `127.0.0.0/8`, `0.0.0.0`, `[::1]` or `*.localhost`; preserve port, path, query, hash. Every other URL is QR'd verbatim |
| QR render | Rust `qrcode` crate → SVG string, inlined into the popup DOM |
| Access control | `Access-Control-Allow-Origin: *`, loopback bind, no token, no pinned extension ID |
| Lifecycle | Login service: systemd `--user` unit (Linux) + LaunchAgent (macOS), enabled once per machine |
| Popup | QR + rewritten URL as text + click-to-copy + `chrome.commands` keyboard shortcut |
| Permissions | `activeTab` only |
| Location | This repo, `qr-lan/` with `companion/` (Cargo) and `extension/` |
| Install | Single `install.sh`: `cargo build --release`, symlink binary and the per-OS unit file, enable it, print the load-unpacked path |

## Deliberate non-goals

- **No reachability probe.** If the dev server binds `127.0.0.1` instead of
  `0.0.0.0`, or `firewalld` blocks the port, the QR looks correct and the phone
  simply hangs. Diagnosing that is manual.
- **No rich error UI.** A dead daemon renders a terse one-line failure, not a
  start command.
- **No config file, flags, or env vars.** The port is a constant duplicated in
  the Rust source and the popup JS.
- **No tunnels, HTTPS, or auth.** Same-LAN plain HTTP only.

## Open details

- Non-`http(s)` tabs (`chrome://`, `file://`, `about:`) show a "can't share this"
  state instead of a QR.
- The extension stays unpacked and is loaded from the repo path, so its ID
  differs per machine — harmless given `ACAO: *`.
- A VPN owning the default route (Tailscale, WireGuard) makes the UDP trick
  return the VPN address rather than the LAN address.
