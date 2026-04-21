# Flatpak submission for cosmic-flatpak

This folder is the drop-in for a pull request against
[`pop-os/cosmic-flatpak`](https://github.com/pop-os/cosmic-flatpak).

## Files

- `io.github.yockerfx.CosmicAppletBongoPenguin.json` — the Flatpak manifest.
- `cargo-sources.json` — generated list of Rust crate sources for the offline
  `cargo build` inside the sandbox. **Regenerate on every Cargo.lock change.**

## Submitting to cosmic-flatpak

1. Tag a release of this repo (e.g. `v0.1.0`) and note the commit SHA.
2. Edit `io.github.yockerfx.CosmicAppletBongoPenguin.json`: replace
   `REPLACE_WITH_RELEASE_COMMIT_SHA` with the tag's commit SHA (or add a
   `"tag": "v0.1.0"` sibling). Flatpak-builder requires a pinned commit.
3. Regenerate `cargo-sources.json` (see below).
4. Fork `pop-os/cosmic-flatpak`, copy both files into
   `app/io.github.yockerfx.CosmicAppletBongoPenguin/`.
5. Open a PR. The System76 CI builds the manifest and reviewers check
   finish-args, manifest hygiene, and that the metainfo validates.
6. On merge, `cosmic-store` users with the `cosmic` remote enabled will see
   the applet in the "COSMIC Applets" category.

## Regenerate cargo-sources.json

```sh
# one-time tooling
pip install --user aiohttp toml

# fetch the generator
curl -fLO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py

# run it on this repo's Cargo.lock (from the repo root)
python3 flatpak-cargo-generator.py -o flatpak/cargo-sources.json Cargo.lock
```

## Test the build locally

```sh
sudo apt install flatpak flatpak-builder just
flatpak remote-add --if-not-exists --user flathub \
  https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.freedesktop.Platform//25.08 \
  org.freedesktop.Sdk//25.08 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08

# In a fresh clone of cosmic-flatpak, with this dir copied into app/<id>/:
just build io.github.yockerfx.CosmicAppletBongoPenguin
```

## Runtime notes / sandbox constraints

- **evdev needs `/dev/input/event*`.** We expose this via `--device=all`.
  The host user must still be a member of the `input` group — Flatpak
  cannot grant kernel groups. Without membership the applet starts but
  the counter stays at 0 (input streams error out).
- **`/etc/machine-id` is replaced by Flatpak** with a per-app fake. The
  AES key derived from it is therefore different from a `.deb` install,
  so previously-saved `count.dat` files won't decrypt across install
  methods. The code falls back to `0` cleanly — no crash, just a fresh
  counter.
- **Persistence path** inside the sandbox resolves to
  `~/.var/app/io.github.yockerfx.CosmicAppletBongoPenguin/data/bongo-penguin-cosmic/count.dat`.
