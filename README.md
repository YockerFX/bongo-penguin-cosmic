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
- [Roadmap](#roadmap)
- [Installation / setup](#installation--setup)
- [Publishing to the COSMIC Store](#publishing-to-the-cosmic-store)
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
| 7. Polish / v0.1.0 release | 🚧 partial     | CI runs fmt/clippy/test/build; screenshots + AppStream pending |

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
- **CI on every PR** — GitHub Actions runs `rustfmt`, `clippy -D warnings`,
  the unit-test suite, and a release build before anything gets merged.

## Roadmap

Contributors welcome — here is what is still open. Pick an item, open an
issue, or send a PR.

### Features

- [ ] **Hotplug** (phase 4) — udev monitor so Bluetooth keyboards and other
      hot-plugged input devices get picked up after suspend / reconnect
      without restarting the applet.
- [ ] **Settings popup** (phase 5) — decay-duration slider, counter-reset
      button, per-device enable/disable, `cosmic-config` integration so
      settings persist across sessions.
- [ ] **Skin switching** — the `Cosmetics` tab already exists and lets you
      pick `Classic` / `Cosmic` / `Retro`, but the selection is a no-op.
      Wire it up to actually swap the rendered SVGs at runtime.
- [ ] **Counter display toggle** — option to hide the number and only show
      the penguin.
- [ ] **More achievement tiers** + nicer unlock UI (currently a static list
      at 100 / 1 000 / 10 000 / 100 000 keystrokes).

### Art / cosmetics

- [ ] **Pingu redesign** — the current poses in `assets/` (`none.svg`,
      `left.svg`, `right.svg`, `both.svg`) are placeholders. Polished,
      on-brand Tux artwork very welcome.
- [ ] **"Cosmic" skin** — space / nebula-themed variant of the four poses
      (idle, left flipper, right flipper, both flippers).
- [ ] **"Retro" skin** — pixel-art / CRT-themed variant of the four poses.
- [ ] **Community skins** — see [Contributing](#contributing) for the
      target size and style guidance. Extra skin packs are a nice way to
      contribute without touching Rust.

### Packaging & release

- [ ] **CI-built `.deb`** — extend the GitHub Actions workflow to produce a
      `.deb` on every tag and upload it to GitHub Releases.
- [x] **AppStream metadata** for COSMIC Store inclusion (`data/*.metainfo.xml`).
- [x] **Flatpak manifest** (`flatpak/` — drop-in for a PR against `pop-os/cosmic-flatpak`).
- [x] **Launcher icon** (`data/icons/hicolor/scalable/apps/*.svg`).
- [ ] **Regenerate `flatpak/cargo-sources.json`** from `Cargo.lock` before PR.
- [ ] **Screenshots** in the README (panel + dock, all four poses).
- [ ] **v0.1.0 tag** once the above lands.
- [ ] **Open PR against `pop-os/cosmic-flatpak`** to publish to the
      COSMIC Store — see [`flatpak/README.md`](./flatpak/README.md).

## Installation / setup

Requires **COSMIC desktop** (Pop!_OS 24.04 alpha+) and membership in the
`input` group.

> **Missing `input` group?** The applet detects this at launch and shows an
> in-popup banner with the exact `usermod` command. After fixing, log out
> and log back in — the banner disappears automatically.

### From the COSMIC Store (once published)

```sh
# Add the Pop!_OS COSMIC remote once:
flatpak remote-add --if-not-exists --user cosmic \
    https://apt.pop-os.org/cosmic/cosmic.flatpakrepo

# Then install via the COSMIC Store GUI, or:
flatpak install --user cosmic io.github.yockerfx.CosmicAppletBongoPenguin
```

After install, run once on the host (Flatpak can't grant kernel groups):

```sh
sudo usermod -aG input "$USER"
# log out and log back in
```

### From a `.deb` (easiest on Pop!_OS before Store publication)

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
install -Dm0644 data/io.github.yockerfx.CosmicAppletBongoPenguin.desktop \
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

## Publishing to the COSMIC Store

The applet ships with all metadata needed for the official COSMIC Store
(AppStream metainfo, icon, `.desktop`, Flatpak manifest under `flatpak/`).
Full submission checklist lives in [`flatpak/README.md`](./flatpak/README.md).

TL;DR for a new release:

1. Bump `version` in `Cargo.toml` and add a `<release>` entry to
   `data/io.github.yockerfx.CosmicAppletBongoPenguin.metainfo.xml`.
2. `appstreamcli validate data/*.metainfo.xml` and
   `desktop-file-validate data/*.desktop` — must be clean (the
   `screenshot-image-not-found` warning resolves once the tag is pushed).
3. Tag + push (`git tag v0.1.0 && git push --tags`).
4. Regenerate `flatpak/cargo-sources.json` from the current `Cargo.lock`:
   ```sh
   pip install --user aiohttp toml
   curl -fLO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
   python3 flatpak-cargo-generator.py -o flatpak/cargo-sources.json Cargo.lock
   ```
5. In `flatpak/io.github.yockerfx.CosmicAppletBongoPenguin.json`, replace
   `REPLACE_WITH_RELEASE_COMMIT_SHA` with the tag's commit SHA.
6. Fork [`pop-os/cosmic-flatpak`](https://github.com/pop-os/cosmic-flatpak),
   copy both files into
   `app/io.github.yockerfx.CosmicAppletBongoPenguin/`, and open a PR.

System76 reviews the manifest, the CI builds it, and on merge the applet
shows up in the COSMIC Store under "COSMIC Applets".

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
cargo test     # run the unit test suite
```

CI runs the same checks on every push and PR — see
[`.github/workflows/ci.yml`](./.github/workflows/ci.yml).

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
