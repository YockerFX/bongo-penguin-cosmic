# Bongo Penguin — a COSMIC applet

Tux drums along with every keystroke.

![Status](https://img.shields.io/badge/status-alpha-orange)
![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)
![Rust](https://img.shields.io/badge/rust-stable%20(edition%202024)-informational)
![Desktop](https://img.shields.io/badge/desktop-COSMIC%20(Pop!__OS)-6a5acd)

## Idea

`cosmic-applet-bongo-penguin` is a Bongo-Cat-style applet for the
[COSMIC desktop](https://system76.com/cosmic/), starring Tux instead of a cat.
It lives in your panel or dock and flips its flippers in time with your
keyboard and mouse — system-wide, including in native Linux games, Wayland
apps, and under Proton/Wine.

A live keystroke counter sits next to the penguin. It survives reboots and
logins, and is **AES-256-GCM encrypted and bound to your `machine-id`**, so a
stored count can't be trivially tampered with or copied to another machine.

## Table of contents

- [Progress & features](#progress--features)
- [Next steps](#next-steps)
- [Installation / setup](#installation--setup)
- [How it works](#how-it-works)
- [Contributing](#contributing)
- [Runtime dependencies](#runtime-dependencies)

## Progress & features

| Phase                      | Status         | Notes                                                   |
| -------------------------- | -------------- | ------------------------------------------------------- |
| 1. Scaffold                | ✅ done        | Applet registers with the panel                         |
| 2. Input capture           | ✅ done        | evdev → tokio task per device → iced subscription       |
| 2b. Counter + persistence  | ✅ done        | AES-256-GCM, survives reboots                           |
| 3. Animation (Tux SVGs)    | 🚧 mostly done | Assets in place, state machine + decay live             |
| 4. Hotplug                 | ⏳ planned     | Re-scan on BT keyboard reconnect after suspend          |
| 5. Settings popup          | 🚧 partial     | Tabs exist; decay slider + counter-reset button pending |
| 6. Debian packaging        | ✅ ready       | `debian/` rules present; CI-built `.deb` pending        |
| 7. Polish / v0.1.0 release | ⏳ planned     | Screenshots, AppStream metadata, GitHub Actions         |

Shipped features:

- **Panel _and_ dock applet**, horizontal and vertical layouts.
- **Four penguin poses** (idle / left / right / both) rendered from SVG.
- **System-wide input capture** via `evdev` — works in Wayland, native Linux
  games, and Proton titles.
- **Encrypted keystroke counter** (AES-256-GCM, bound to `/etc/machine-id`).
- **Battery-friendly** — `epoll` event streams, no polling; disk I/O only
  every 5 s and only when the count changed.
- **Left/right heuristic** drives the matching flipper; parallel input →
  "both flippers".
- **Popup tabs**: Cosmetics, Achievements (100 / 1k / 10k / 100k milestones),
  About.
- **Race-safe persistence** — atomic `tmp.<pid>` + `rename` handles the two
  dock instances COSMIC spawns.
- **Dual-sink logging** — `stderr` _and_ `~/.cache/bongo-penguin.log`.

## Next steps

- **Hotplug** (phase 4): udev monitor so Bluetooth keyboards are picked up
  after suspend without restarting the applet.
- **Settings popup** (phase 5): decay slider, counter-reset button,
  `cosmic-config` integration.
- **Skin switching**: wire the Cosmetics tab to actually swap SVGs.
- **CI `.deb`** via GitHub Actions, published to Releases.
- **Unit tests** for `persistence::{load, save}` and `classify::Side`.

See [`PLAN.md`](./PLAN.md) for the detailed plan.

## Installation / setup

Requires **COSMIC desktop** (Pop!_OS 24.04 alpha+) and membership in the
`input` group.

### From a `.deb` (easiest)

```sh
# Prebuilt (once published to Releases):
curl -LO https://github.com/YockerFX/bongo-penguin-cosmic/releases/latest/download/cosmic-applet-bongo-penguin_VERSION_ARCH.deb
sudo apt install ./cosmic-applet-bongo-penguin_*.deb

# Or build it yourself:
sudo apt install build-essential devscripts debhelper pkg-config \
    libxkbcommon-dev libudev-dev libinput-dev \
    libssl-dev libfontconfig1-dev libwayland-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/YockerFX/bongo-penguin-cosmic.git
cd bongo-penguin-cosmic && just deb
sudo apt install ../cosmic-applet-bongo-penguin_*.deb
```

The `.deb`'s `postinst` adds `$SUDO_USER` to the `input` group automatically.

### From source

```sh
git clone https://github.com/YockerFX/bongo-penguin-cosmic.git
cd bongo-penguin-cosmic
just install                        # system-wide, needs sudo
# or dev install into ~/.local:
cargo build --release
install -Dm0755 target/release/cosmic-applet-bongo-penguin ~/.local/bin/
install -Dm0644 data/com.github.bongopenguin.CosmicAppletBongoPenguin.desktop \
    ~/.local/share/applications/

sudo usermod -aG input "$USER"      # manual installs only
# log out and back in
```

Finally, restart the panel and add the applet:

```sh
pkill -x cosmic-panel   # cosmic-session respawns it in ~10 s
```

Then go to **COSMIC Settings → Panel / Dock → Applets** and add
**Bongo Penguin**.

## How it works

```
[/dev/input/event*] ──evdev──▶ [tokio task per device (epoll)]
                                         │
                                         ▼  mpsc::Sender<InputEvent>
[udev monitor (phase 4)]  ──add/rm──▶ [device manager]
                                         │
                                         ▼  iced subscription
                            [App::update(Msg::Input)]
                              │            │
                              ▼            ▼
                     [count += 1]   [AnimationState]
                         │                  │
                         ▼                  ▼
              every 5s if dirty:    [App::view] → SVG + counter
              AES-GCM → count.dat
```

- **evdev works system-wide** because `/dev/input/event*` sits _below_
  Wayland, X11, SDL, and Proton — kernel events are captured before any
  compositor or game input grab. Anti-cheat (EAC / BattlEye / NetEase) is
  unaffected: reads are passive, no injection, no memory access.
- **One tokio task per device** (not one `select!`) isolates panics and
  makes hotplug-remove a no-op.
- **`Device::into_event_stream()`** uses kernel `epoll` — no busy loop.
- **Counter saves every 5 s and only if `count != last_saved`** → zero disk
  I/O when idle. Atomic `tmp.<pid>` + `rename` lets both dock instances write
  in parallel.
- **Counter crypto**: key = `SHA-256(APP_SECRET || /etc/machine-id)`,
  payload = 12 B nonce + 8 B count + 16 B GCM tag = 36 B on disk. Tamper or
  machine-id change → fall back to `0` with an `info` log, no panic.
- **SVGs embedded via `include_bytes!`** → no runtime path resolution.

Files on disk:

| Path                                            | Purpose                       |
| ----------------------------------------------- | ----------------------------- |
| `~/.cache/bongo-penguin.log`                    | Log (default level `info`)   |
| `~/.local/share/bongo-penguin-cosmic/count.dat` | 36 B encrypted counter        |
| `/etc/machine-id`                               | Part of the key derivation    |

Reset the counter: `rm ~/.local/share/bongo-penguin-cosmic/count.dat`.

## Contributing

Issues and PRs welcome at
[github.com/YockerFX/bongo-penguin-cosmic](https://github.com/YockerFX/bongo-penguin-cosmic).

Before opening a PR:

```sh
just fmt       # cargo fmt --all
just clippy    # cargo clippy --all-targets -- -D warnings
just check     # cargo check --all-targets
```

Dev loop: edit → `just build-release` → `just install` (or copy into
`~/.local/bin/`) → `pkill -x cosmic-panel` → `tail -f ~/.cache/bongo-penguin.log`.

New SVG skins are very welcome — please keep them original (not derived from
Bongo Cat), transparent-background, roughly matched to 768 × 1152.

## Runtime dependencies

Released under **GPL-3.0-only** (see [`debian/copyright`](./debian/copyright)).

- [`libcosmic`](https://github.com/pop-os/libcosmic) — MPL-2.0
- [`evdev`](https://github.com/emberian/evdev) — Apache-2.0 OR MIT
- [`aes-gcm`](https://github.com/RustCrypto/AEADs) — Apache-2.0 OR MIT
- [`tokio`](https://tokio.rs/), [`tracing`](https://github.com/tokio-rs/tracing),
  [`sha2`](https://github.com/RustCrypto/hashes) — Apache-2.0 OR MIT

All compatible with GPL-3.0-only linking.
