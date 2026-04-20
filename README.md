# Bongo Penguin — a COSMIC applet

Tux drums along with every keystroke.

`cosmic-applet-bongo-penguin` is a Bongo-Cat-style applet for the
[COSMIC desktop](https://system76.com/cosmic/) (Pop!_OS), starring Tux the
Linux penguin instead of a cat. It lives in your panel or dock and flips its
flippers in time with your keyboard and mouse — system-wide, including in
native Linux games, in Wayland apps, and under Proton/Wine.

A live keystroke counter sits next to the penguin. It survives reboots,
panel restarts, and logout/login. The counter is **AES-256-GCM encrypted and
bound to your `machine-id`**, so a stored count can't be trivially tampered
with by a hex editor or copied to a different machine for bragging rights.

![Status](https://img.shields.io/badge/status-alpha-orange)
![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)
![Rust](https://img.shields.io/badge/rust-stable%20(edition%202024)-informational)
![Desktop](https://img.shields.io/badge/desktop-COSMIC%20(Pop!__OS)-6a5acd)

---

## Table of contents

- [Features](#features)
- [Status & roadmap](#status--roadmap)
- [Screenshots](#screenshots)
- [Installation](#installation)
  - [Option A — prebuilt `.deb` from GitHub Releases](#option-a--prebuilt-deb-from-github-releases)
  - [Option B — build the `.deb` yourself](#option-b--build-the-deb-yourself)
  - [Option C — build & install from source](#option-c--build--install-from-source)
  - [Option D — dev install into `~/.local`](#option-d--dev-install-into-local)
- [Post-install: join the `input` group](#post-install-join-the-input-group)
- [Using the applet](#using-the-applet)
- [Files on disk](#files-on-disk)
- [How it works](#how-it-works)
- [Why evdev works system-wide](#why-evdev-works-system-wide)
- [Counter threat model](#counter-threat-model)
- [Troubleshooting](#troubleshooting)
- [Known limitations](#known-limitations)
- [Contributing](#contributing)
- [Releasing (maintainers)](#releasing-maintainers)
- [License](#license)

---

## Features

- **Panel _and_ dock applet** — works in COSMIC's panel and in the dock,
  horizontal and vertical layouts supported.
- **Four penguin poses** — idle, left flipper, right flipper, both flippers —
  rendered from SVG, so they stay crisp at any panel size.
- **System-wide input capture** — reads `/dev/input/event*` via `evdev`, so
  keystrokes register in GNOME apps, native Linux games, and Proton titles.
- **Live keystroke counter** with AES-256-GCM encrypted persistence bound
  to `/etc/machine-id`.
- **Battery-friendly** — `epoll`-backed event streams, no polling loop, disk
  I/O only every 5 s and only when the count has actually changed.
- **Left / right heuristic** — keys on the left half of the keyboard drive
  the left flipper, right-half keys drive the right flipper, simultaneous
  inputs trigger the "both flippers" frame.
- **Popup with tabs** — Cosmetics (skin preview), Achievements (milestones at
  100 / 1 000 / 10 000 / 100 000 keystrokes), About (version, links).
- **Race-safe persistence** — COSMIC spawns one applet instance per dock
  output, so two instances run in parallel. They write atomically via
  `tmp.<pid>` + `rename`; no corruption, no races.
- **Dual-sink logging** — `stderr` _and_ `~/.cache/bongo-penguin.log`, so you
  still get logs even when COSMIC swallows panel stderr.

## Status & roadmap

Alpha. The core event pipeline, counter, persistence, and animation work.
Hotplug, a real settings popup, and CI-built `.deb` artifacts are next.

| Phase                      | Status     | Notes                                                      |
| -------------------------- | ---------- | ---------------------------------------------------------- |
| 1. Scaffold                | ✅ done    | Applet registers with the panel                            |
| 2. Input capture           | ✅ done    | evdev → tokio task per device → iced subscription          |
| 2b. Counter + persistence  | ✅ done    | AES-256-GCM, verified to survive reboots                   |
| 3. Animation (Tux SVGs)    | 🚧 mostly done | Assets in place, state machine + decay live             |
| 4. Hotplug                 | ⏳ planned | Re-scan on BT keyboard reconnect after suspend             |
| 5. Settings popup          | 🚧 partial | Tabs exist; decay slider + counter-reset button pending    |
| 6. Debian packaging        | ✅ ready   | `debian/` rules present; CI-built `.deb` pending           |
| 7. Polish / v0.1.0 release | ⏳ planned | Screenshots, AppStream metadata, GitHub Actions            |

See [`PLAN.md`](./PLAN.md) for the detailed plan.

## Screenshots

> _Placeholder — screenshots land in the v0.1.0 release._
>
> ```
> ┌──────────────────┐
> │  🐧 42 537       │  ← panel (horizontal)
> └──────────────────┘
> ```

## Installation

The applet needs:

- **COSMIC desktop** (ships with Pop!_OS 24.04 alpha and later). On other
  distros you'll need COSMIC installed from source or your distro's packages.
- **Membership in the `input` group** so the applet can read
  `/dev/input/event*`. The `.deb` handles this for you; the manual installs
  don't. See [Post-install](#post-install-join-the-input-group).

Pick whichever install method suits you.

### Option A — prebuilt `.deb` from GitHub Releases

> This is the easiest path once the first `.deb` has been published to
> GitHub Releases. Until then, use Option B or C.

```sh
# Replace VERSION and ARCH with the current release, e.g. 0.1.0-1 and amd64.
curl -LO https://github.com/YockerFX/bongo-penguin-cosmic/releases/latest/download/cosmic-applet-bongo-penguin_VERSION_ARCH.deb
sudo apt install ./cosmic-applet-bongo-penguin_VERSION_ARCH.deb
```

`apt install ./file.deb` resolves runtime dependencies automatically, which is
why it's preferred over `sudo dpkg -i …`.

### Option B — build the `.deb` yourself

Use this on Pop!_OS / Ubuntu / Debian when you want a proper installable
package. The packaging files live in [`debian/`](./debian).

```sh
# 1. Build-time dependencies (Debian / Ubuntu / Pop!_OS package names)
sudo apt install \
    build-essential devscripts debhelper \
    pkg-config \
    libxkbcommon-dev libudev-dev libinput-dev \
    libssl-dev libfontconfig1-dev libwayland-dev

# 2. A recent Rust toolchain (Debian's rustc is usually too old for
#    edition-2024 crates; use rustup instead).
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Clone & build the .deb
git clone https://github.com/YockerFX/bongo-penguin-cosmic.git
cd bongo-penguin-cosmic
just deb
# or, without just:
# dpkg-buildpackage -us -uc -b

# 4. Install the resulting .deb (it lands in the parent directory)
sudo apt install ../cosmic-applet-bongo-penguin_*.deb
```

`dpkg-buildpackage` produces an unsigned (`-us -uc`) binary-only (`-b`)
package. For signed / source packages see the Debian packaging docs.

### Option C — build & install from source

No `.deb`, just copy the binary and desktop file into `/usr/`.

```sh
# Build-time deps
sudo apt install \
    build-essential pkg-config \
    libxkbcommon-dev libudev-dev libinput-dev \
    libssl-dev libfontconfig1-dev libwayland-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Build + install
git clone https://github.com/YockerFX/bongo-penguin-cosmic.git
cd bongo-penguin-cosmic
just install        # or: cargo build --release && sudo just install
```

The `install` target writes:

| Path                                                                                 | What it is                              |
| ------------------------------------------------------------------------------------ | --------------------------------------- |
| `/usr/bin/cosmic-applet-bongo-penguin`                                               | the applet binary                       |
| `/usr/share/applications/com.github.bongopenguin.CosmicAppletBongoPenguin.desktop`   | `.desktop` entry (`X-CosmicApplet=true`) |

Uninstall with `just uninstall`.

### Option D — dev install into `~/.local`

Useful while hacking on the applet — no `sudo`, no system changes.

```sh
cargo build --release
install -Dm0755 target/release/cosmic-applet-bongo-penguin \
    ~/.local/bin/cosmic-applet-bongo-penguin
install -Dm0644 data/com.github.bongopenguin.CosmicAppletBongoPenguin.desktop \
    ~/.local/share/applications/com.github.bongopenguin.CosmicAppletBongoPenguin.desktop
pkill -x cosmic-panel     # respawned by cosmic-session in ~10s
```

Make sure `~/.local/bin` is on your `$PATH`.

## Post-install: join the `input` group

To read `/dev/input/event*` your user needs to be in the `input` group.

- **If you installed the `.deb`:** the `postinst` script adds `$SUDO_USER`
  automatically and prints a reminder. You still need to log out and log
  back in for the new group to take effect.
- **If you installed from source (Option C or D):** do it manually:

  ```sh
  sudo usermod -aG input "$USER"
  # log out and log back in; `groups` should now include `input`
  ```

Once the group is active, make the panel pick up the new applet:

```sh
pkill -x cosmic-panel
```

`cosmic-session` respawns the panel (and the dock) within ~10 s with the
applet catalogue reloaded. Then go to **COSMIC Settings → Panel / Dock →
Applets**, scroll to **Bongo Penguin**, and add it.

## Using the applet

- **Click the penguin** to open the popup with three tabs:
  - **Cosmetics** — pick a skin (placeholder UI; SVGs don't react to the
    choice yet, planned for a future release).
  - **Achievements** — live counter and milestone progress.
  - **About** — version, GitHub link, Discord link.
- **Flipper logic** — left-hand keys (`Q W E R T`-ish, left Shift / Ctrl /
  Alt, left mouse button) → left flipper. Right-hand keys → right flipper.
  Parallel events → both flippers.
- **Decay** — after 120 ms with no input the penguin returns to the idle
  pose.
- **Two penguins in the dock is expected** — COSMIC spawns one applet
  instance per dock output. Both write race-safely to the same counter file.

## Files on disk

| Path                                                 | Purpose                                                                |
| ---------------------------------------------------- | ---------------------------------------------------------------------- |
| `~/.cache/bongo-penguin.log`                         | Log file (default level `info`; per-event logs are `trace`)            |
| `~/.local/share/bongo-penguin-cosmic/count.dat`      | 36 bytes: 12 B nonce + 8 B counter + 16 B GCM tag, AES-256-GCM         |
| `/dev/input/event*`                                  | Kernel input sources (requires `input` group membership)               |
| `/etc/machine-id`                                    | Used as part of the counter key derivation                             |

Raise the log level (e.g. to see every event):

```sh
RUST_LOG=trace cosmic-applet-bongo-penguin 2>&1 | tee -a ~/.cache/bongo-penguin.log
```

Reset the counter (until the settings popup ships a button for it):

```sh
rm ~/.local/share/bongo-penguin-cosmic/count.dat
```

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
              AES-GCM → count.dat   [iced::time::every(5s)]  → Msg::SaveTick
                                    [iced::time::every(50ms)] → Msg::AnimTick
```

### Project layout

```
bongo-penguin-cosmic/
├── Cargo.toml                  # libcosmic (git), evdev, aes-gcm, sha2, tokio, tracing
├── rust-toolchain.toml         # stable + rustfmt + clippy
├── justfile                    # check / fmt / clippy / build / install / deb / clean
├── src/
│   ├── main.rs                 # entry + dual-sink tracing
│   ├── lib.rs                  # module graph + cosmic::applet::run
│   ├── app.rs                  # BongoPenguin: state, update, view, popup, subscription
│   ├── persistence.rs          # AES-256-GCM count.dat, atomic write
│   └── input/
│       ├── mod.rs
│       ├── watcher.rs          # evdev → epoll stream per device → mpsc
│       └── classify.rs         # keyboard vs. mouse, left/right side heuristic
├── assets/                     # Tux poses (none / left / right / both)
├── data/                       # .desktop entry
└── debian/                     # Debian packaging (control, rules, postinst, …)
```

### Design decisions

- **One tokio task per input device** (not a single `select!`) → panics are
  isolated, hotplug-remove is simply "task ends".
- **`Device::into_event_stream()`** uses kernel `epoll`. No busy loop, no
  battery drain.
- **Counter save every 5 s and only when `count != last_saved`** → zero disk
  I/O when idle.
- **Atomic write via `tmp.<pid>` + `rename`** → both dock instances can write
  in parallel without corrupting the file.
- **SVGs embedded via `include_bytes!`** → no runtime path resolution, no
  "missing assets" after install.

## Why evdev works system-wide

`/dev/input/event*` is a kernel interface — _below_ Wayland, X11, SDL, and
games. Events are captured there before the compositor or a game's input
grab sees them.

Confirmed working:

- **COSMIC / Wayland** — the compositor itself reads evdev; multiple readers
  are the normal case.
- **Proton games**, including Marvel Rivals launched with
  `PROTON_ENABLE_WAYLAND=1 gamemoderun PROTON_ENABLE_NVAPI=1 VKD3D_CONFIG=dxr11`.
  Proton translates DirectInput/XInput into SDL/Wayland, not raw evdev, so
  the kernel still sees the keys.
- **Exclusive fullscreen / SDL relative mouse** — kernel events are emitted
  regardless.
- **Anti-cheat (EAC / BattlEye / NetEase)** — passive reads, no injection, no
  memory access, so nothing to flag.

The only case that can block us is `EVIOCGRAB`, used by low-level tools like
`wev` or input remappers. In practice this is rare and out of scope.

## Counter threat model

The counter is designed to defeat _casual_ cheating (screenshot bragging,
"I just hex-edited it to a million", copying a file to another machine), not
determined reverse engineering.

- **AES-256-GCM** with a 12 B nonce, 8 B payload, 16 B tag → 36 B per file.
- **Key derivation**: `SHA-256(APP_SECRET || /etc/machine-id)`. The key is
  unique per install and non-portable.
- **Tamper evidence**: any byte change breaks the GCM tag; the applet falls
  back to `count = 0` with an `info` log line. No panic.
- **Not protected against**: reverse-engineering + memory debuggers, patched
  binaries, replaying an old `count.dat`. This is deliberate — the effort
  required far exceeds the payoff of a slightly higher keystroke number.

## Troubleshooting

**The applet doesn't appear in the panel configuration.**

- Check the `.desktop` entry is installed:
  `ls /usr/share/applications/ ~/.local/share/applications/ 2>/dev/null | grep -i bongo`.
- Restart the panel: `pkill -x cosmic-panel`.
- `NoDisplay=true` and `X-CosmicApplet=true` are required — both are set in
  the shipped `.desktop` file.

**The counter doesn't count.**

- `groups "$USER"` must contain `input`. If it doesn't:
  `sudo usermod -aG input "$USER"` then log out and log back in.
- `ls -l /dev/input/event*` — your user must have read access through the
  `input` group.
- Check the log: `tail -n 200 ~/.cache/bongo-penguin.log`. A healthy startup
  logs `evdev device` and `tokio task started` for each device.

**The counter suddenly went back to 0.**

- Either `count.dat` was modified (GCM tag invalid) or your `machine-id`
  changed (re-imaged VM, cloned filesystem). Both are intentional: remove
  `count.dat` and start fresh.

**Two penguins show up in the dock.**

- Expected. COSMIC runs one applet instance per dock output. Both share the
  same `count.dat` via atomic `tmp.<pid>` + `rename` writes.

**Empty button, no penguin drawn.**

- Build cached before the SVG assets landed. Force a fresh `include_bytes!`:
  `cargo clean && cargo build --release`.

## Known limitations

- **Hotplug isn't implemented yet** (phase 4). Bluetooth keyboards that
  reappear after suspend are only picked up after restarting the applet.
- **Skin selection** in the Cosmetics tab is a UI placeholder — doesn't
  change the SVGs yet.
- **No persisted settings** — `cosmic-config` integration is still to come
  (phase 5).
- **Discord link** in the About tab is a placeholder
  (`discord.gg/your-invite`).
- **Two dock instances** means every state change is _rendered_ twice, though
  the underlying counter is written from whichever process saves first — both
  processes see the same count.

## Contributing

Issues and pull requests are very welcome at
[github.com/YockerFX/bongo-penguin-cosmic](https://github.com/YockerFX/bongo-penguin-cosmic).

### Before opening a PR

```sh
just fmt       # cargo fmt --all
just clippy    # cargo clippy --all-targets -- -D warnings
just check     # cargo check --all-targets
```

### Dev loop

The panel embeds `stderr` into a pipe, so iterate through the log file:

1. Edit → `just build-release` (or `cargo watch -x 'build --release'`).
2. Reinstall: `just install` (system-wide) or copy the binary into
   `~/.local/bin/` (dev install).
3. `pkill -x cosmic-panel` — COSMIC respawns everything in ~10 s.
4. `tail -f ~/.cache/bongo-penguin.log`.

### Logging levels

| Level   | Contents                                                   |
| ------- | ---------------------------------------------------------- |
| `info`  | Startup, counter load, skin selection (**default**)        |
| `debug` | Persist success                                            |
| `trace` | Every single input event (silent unless `RUST_LOG` is set) |
| `warn`  | Persist failures, failed `xdg-open` calls                  |

Enable with:

```sh
RUST_LOG=cosmic_applet_bongo_penguin=debug,info
```

### Art contributions

SVGs in `assets/` were drawn specifically for this project — they are **not**
derived from Bongo Cat or any other copyrighted source. New skins are very
welcome; please keep them original, transparent-background, and roughly
matched to the existing 768 × 1152 aspect ratio so they drop in cleanly.

### Tests

No automated tests yet — the end-to-end pipeline is verified by log
inspection and live typing. Unit tests for `persistence::{load, save}` and
the `classify::Side` heuristic are on the to-do list; PRs appreciated.

## Releasing (maintainers)

1. Bump the version in `Cargo.toml` and add a `debian/changelog` entry
   (`dch -v <version>-1` if you have `devscripts`).
2. `just check && just clippy && just fmt`.
3. Commit, tag `vX.Y.Z`, push.
4. `just deb` locally _or_ let CI build the `.deb` (GitHub Actions workflow
   to be added in phase 7).
5. Attach the `.deb` to the GitHub Release.

## License

`cosmic-applet-bongo-penguin` is released under the **GPL-3.0-only** license.
See [`debian/copyright`](./debian/copyright) for details.

Runtime dependencies:

- [`libcosmic`](https://github.com/pop-os/libcosmic) — MPL-2.0
- [`evdev`](https://github.com/emberian/evdev) — Apache-2.0 OR MIT
- [`aes-gcm`](https://github.com/RustCrypto/AEADs) — Apache-2.0 OR MIT
- [`tokio`](https://tokio.rs/), [`tracing`](https://github.com/tokio-rs/tracing),
  [`sha2`](https://github.com/RustCrypto/hashes) — Apache-2.0 OR MIT

All are compatible with GPL-3.0-only linking.
