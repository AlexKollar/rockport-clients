# Rockport — the standalone clients

The city, without the browser chrome.

Rockport is a text-based multiplayer world — equal parts cyberpunk life sim
and accurate hacking simulator — live in 2475 at
[rockport.nocturnerow.com](https://rockport.nocturnerow.com:4043/). This repo
holds its standalone desktop clients: one native window that opens the city's
homepage and web player together, as its own dockable app with its own icon.
No tabs, no address bar, no browser frame.

## Downloads

Grab the current release from
[Releases](https://github.com/AlexKollar/rockport-clients/releases):

- **Linux** — `Rockport_1.0.0_amd64.deb` (Debian/Ubuntu/Pop!_OS) or
  `Rockport_1.0.0_amd64.AppImage` (any distro; make it executable and run).
- **Windows** — `Rockport_1.0.0_x64-setup.exe` (per-user install, no admin;
  installs the WebView2 runtime if your machine doesn't already carry it).

The mobile door is the installable web app on the
[game's install page](https://rockport.nocturnerow.com:4043/install/).

## What the client is — and isn't

The client is a **door**, not a copy of the city. It opens the live site
first-party in a native window (Tauri + the operating system's own webview —
no browser engine is bundled). The game stays single-sourced on the server:
every update to the city is this client's content the moment it ships, with
no client release to wait for and no version drift to chase.

That means the client is **live-only by design** — Rockport is a multiplayer
world; there is nothing to play offline.

## Building

Needs a Rust toolchain and Tauri's platform prerequisites
(https://tauri.app/start/prerequisites/). On Linux:

```sh
cargo install tauri-cli --version ^2
cargo tauri icon icons/source-512.png   # regenerate the icon set
cargo tauri build --bundles deb,appimage
```

Windows installer, cross-built from Linux (mingw-w64 + NSIS installed):

```sh
rustup target add x86_64-pc-windows-gnu
cargo tauri build --target x86_64-pc-windows-gnu --bundles nsis
```

The window's destination is set in `tauri.conf.json`
(`app.windows[0].url`) — the production city. Point it at a staging door to
test against one.

## License

AGPL-3.0, like the rest of the Row's open work.
