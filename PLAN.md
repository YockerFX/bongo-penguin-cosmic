# Projektplan: `bongo-penguin-cosmic`

Bongo Penguin (Linux-Tux-Reference) als COSMIC-Applet für Pop!_OS — platzierbar in Panel **und** Dock, reagiert auf alle Tastatur- und Maus-Eingaben systemweit (auch in Spielen).

## Status

| Phase | Status | Notiz |
|---|---|---|
| 1. Scaffold | ✅ done | Cargo-Projekt kompiliert, Applet erscheint in Panel-Config als „Bongo Penguin" |
| 2. Input-Capture | ✅ done | evdev-Pipeline end-to-end verifiziert, Events erreichen `App::update(Msg::Input)` |
| 2b. Live-Counter + Persistenz | ✅ done | Zahl im Dock, AES-256-GCM-encrypted Persistenz, überlebt Reboots |
| 3. Animation (Tux-SVGs) | ⏳ next | Idle / LeftFlipper / RightFlipper / BothFlippers als Pinguin |
| 4. Hotplug | ⏳ pending | |
| 5. Settings-Popup | ⏳ pending | |
| 6. Packaging | ⏳ pending | |
| 7. Polish/Release | ⏳ pending | |

### Dev-Umgebung

- Rust 1.95 via `rustup` (auf Host, aufgerufen aus Flatpak-Sandbox via `flatpak-spawn --host`)
- libcosmic git dep (`pop-os/libcosmic`), features: `applet`, `applet-token`, `multi-window`, `tokio`, `wayland`, `winit`
- Host-Dev-Libs installiert: `libxkbcommon-dev`, `libudev-dev`, `libinput-dev`, `libssl-dev`, `libfontconfig1-dev`, `libwayland-dev`, `pkg-config`
- Build-Zeit erster `cargo check`: ~6 min (Fetch + Compile von ~1000 Crates), inkrementell: ~15–25 s
- Install-Ziel (Dev): `~/.local/bin/cosmic-applet-bongo-penguin` + `~/.local/share/applications/com.github.bongopenguin.CosmicAppletBongoPenguin.desktop`
- Laufzeit-Dateien:
  - Log: `~/.cache/bongo-penguin.log` (Default-Level `info`; per-Event = `trace`, silent außer `RUST_LOG=trace`)
  - Persistenz: `~/.local/share/bongo-penguin-cosmic/count.dat` (AES-256-GCM, 36 Bytes)
- Panel-Respawn: `kill $(pgrep -x cosmic-panel)` — systemd/cosmic-session bringt es inkl. Applets in ~10 s hoch; Applet läuft **zweimal parallel** (eine Instanz pro Dock-Output).

### Nächste Schritte

1. Proton-Spiel-Test (Marvel Rivals mit `PROTON_ENABLE_WAYLAND=1 gamemoderun …`) → Counter muss im Spiel weiterzählen
2. → Phase 3: Tux-SVGs zeichnen + State-Machine + Decay-Timer; Counter bleibt als optionales Overlay



## 1. Tech-Stack

| Komponente | Wahl | Begründung |
|---|---|---|
| Sprache | Rust (stable) | libcosmic ist Rust-nativ |
| UI | `libcosmic` + `cosmic-applet` | Direkte Integration ins Panel/Dock |
| Input | `evdev` crate | Kernel-Level, fängt alles (auch in Spielen) |
| Hotplug | `udev` crate | Bluetooth-Tastaturen nach Sleep wieder erkennen |
| Async | `tokio` | libcosmic nutzt's intern, Iced-Subscriptions kompatibel |
| Grafik | SVG via `resvg`/Iced `Svg` | Skalierbar für 32–64px Panel/Dock-Höhen |
| Config | `cosmic-config` | Native COSMIC-Settings-Integration |
| Counter-Persistenz | `aes-gcm` + `sha2` | Tamper-evidence gegen naive Edits; Key aus `/etc/machine-id` |
| Logging | `tracing` + `tracing-subscriber` | Dual-Sink (stderr + `~/.cache/bongo-penguin.log`) — Panel-embedded-safe |

## 2. Projektstruktur

```
bongo-penguin-cosmic/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry + dual-sink logging (stderr + ~/.cache/bongo-penguin.log)   ✅
│   ├── lib.rs               # Module-Graph + `cosmic::applet::run::<BongoPenguin>`             ✅
│   ├── app.rs               # Application-Trait: update/view/subscription + Counter-State     ✅
│   ├── persistence.rs       # AES-256-GCM encrypted count.dat, atomic write, machine-id-bound  ✅
│   ├── input/
│   │   ├── mod.rs                                                                              ✅
│   │   ├── watcher.rs       # evdev → epoll-Event-Stream pro Device → mpsc                    ✅
│   │   ├── classify.rs      # Keyboard vs. Maus via capabilities                              ✅
│   │   └── hotplug.rs       # udev monitor für add/remove                                      ⏳ Phase 4
│   ├── animation/                                                                              ⏳ Phase 3
│   │   ├── state.rs         # Idle / LeftFlipper / RightFlipper / BothFlippers
│   │   └── sprite.rs        # SVG-Auswahl + Decay-Timer
│   ├── settings/                                                                               ⏳ Phase 5
│   │   ├── mod.rs           # cosmic-config schema
│   │   └── popup.rs         # Applet-Popup-UI (Einstellungen)
│   └── theme.rs             # Panel-Höhen-Detection                                            ⏳ Phase 6
├── assets/                                                                                      ⏳ Phase 3
│   ├── tux-idle.svg
│   ├── tux-left.svg
│   ├── tux-right.svg
│   └── tux-both.svg
├── data/
│   └── com.github.bongopenguin.CosmicAppletBongoPenguin.desktop                                 ✅
├── debian/                                                                                      ⏳ Phase 6
│   ├── control
│   ├── postinst              # usermod -aG input $SUDO_USER
│   ├── postrm
│   └── rules
├── justfile                  # build / install / package                                        ✅
└── README.md                                                                                    ⏳ Phase 7
```

## 3. Architektur & Datenfluss

```
[/dev/input/event*] ──evdev──▶ [tokio task pro device (epoll)]        ✅ done
                                        │
                                        ▼ mpsc::Sender<InputEvent>
[udev monitor]     ──add/remove──▶ [device manager]                   ⏳ Phase 4
                                        │
                                        ▼ Iced subscription (stream)
                          [App::update(Msg::Input)]
                            │            │                            ✅ done
                            ▼            ▼
                   [count += 1]   [AnimationState]                    count ✅ / anim ⏳ Phase 3
                       │                  │
                       ▼                  ▼
          every 5s if dirty:      [App::view] → SVG / Counter im Dock
          AES-GCM → count.dat
                                  [iced::time::every(5s)] ──▶ Msg::SaveTick   ✅ done
```

**Key-Design-Entscheidungen:**
- **Ein Tokio-Task pro Device** (nicht ein Select-Loop) → simpler, Panics isoliert, Hotplug-Remove = Task-Exit
- **epoll via `Device::into_event_stream()`** → kein Busy-Loop, kein Battery-Drain
- **Decay-Timer** (Phase 3) über `iced::time::every(50ms)` Subscription → prüft „letzter Input > decay_ms" → zurück zu Idle
- **Flipper-Logik** (Phase 3): alternierend (klassisch), jede Eingabe (Key oder Mausklick) flippt; schnelle Inputs → BothFlippers
- **Counter-Persist nur alle 5 s** und **nur wenn dirty** → 0 I/O bei Inaktivität
- **Atomic Writes** via `count.tmp.<pid>` + `rename` → kein Corrupt-State, kein Race zwischen den 2 parallelen Dock-Instanzen

## 4. Animation State Machine (⏳ Phase 3)

```
       any input
Idle ──────────────▶ LeftFlipper ──input──▶ RightFlipper ──input──▶ LeftFlipper ...
  ▲                     │                       │
  └──decay (120ms)──────┴───────────────────────┘
```

Zusatzregel: schnelle Inputs (<40ms Abstand) → `BothFlippers`-Frame für 60ms.

## 5. Entwicklungs-Phasen

### Phase 1 — Scaffold (MVP-Skelett) ✅
- Cargo-Projekt + libcosmic-Applet-Template
- Statisches Emoji/Placeholder im Panel anzeigen
- **Success:** `cosmic-applet-bongo-penguin` erscheint in COSMIC Panel-Config

### Phase 2 — Input-Capture ✅
- Evdev-Device-Enumeration + Classify (Keyboard/Mouse via `EV_KEY`/`EV_REL`)
- Einzel-Tokio-Task pro Device, Events via `mpsc` → Iced-Subscription → `App::update(Msg::Input)`
- Dual-Sink-Logging (stderr + File), da Panel-embedded stderr in Pipe landet
- **Success:** Tastendruck erzeugt `INFO ... input ev=Key` im Log *und* erreicht den App-Message-Handler

### Phase 2b — Live-Counter + Persistenz ✅
- Live-Counter als Text-Button im Dock (via `core.applet.text_button`)
- AES-256-GCM encrypted Persistenz in `~/.local/share/bongo-penguin-cosmic/count.dat`
- Key-Derivation: `SHA-256(app-secret ‖ /etc/machine-id)` → nicht übertragbar auf anderen Rechner
- Save-Strategie: alle 5 s wenn dirty, atomic via `tmp.<pid>` + rename → race-safe zwischen den 2 Dock-Instanzen
- Tamper-evidence: GCM-Tag fängt jede Modifikation ab → Fallback auf `count=0`
- Realismus: schützt gegen naive Edits (Hexeditor, Zahl reinschreiben); nicht gegen RE + Debugger
- **Success:** Counter überlebt `kill cosmic-panel` + Restart mit dem alten Wert (bewiesen: 5 → 5)

### Phase 3 — Grafik + Animation ⏳ next
- Tux-SVG-Assets in 4 Posen neu zeichnen (idle / left-flipper / right-flipper / both-flippers)
- State-Machine (`animation/state.rs`) + Decay-Timer (50 ms Tick) + Alternating-Flipper-Logik
- SVG-Rendering via `cosmic::widget::icon` / `iced::widget::svg` statt `text_button`
- Entscheidung noch offen: Counter behalten als Overlay / Tooltip / separater Toggle?
- **Success:** Flossen animieren sichtbar bei Tipp-/Klick-Aktivität, Decay zurück zu Idle nach 120 ms Ruhe

### Phase 4 — Hotplug + Multi-Device ⏳
- udev-Monitor für `add`/`remove`
- Dynamisches Starten/Stoppen der Device-Tasks
- **Success:** Bluetooth-Tastatur disconnect/reconnect unterbricht nichts

### Phase 5 — Settings-Popup ⏳
- Applet-Click öffnet Popup mit:
  - Decay-Dauer (Slider)
  - Alternating vs. Key-Position-Modus
  - Einzelne Geräte ein-/ausschalten
  - Skalierung (optional)
  - **Counter-Reset-Button** + Counter-Anzeige ein/aus
- Persistenz via `cosmic-config` (Settings) + separat `count.dat` (Counter-Wert)

### Phase 6 — Dock-Kompatibilität + Packaging ⏳
- Panel-Höhen-Detection (kleiner Panel vs. großes Dock)
- `.deb` mit `postinst`:
  ```sh
  usermod -aG input "$SUDO_USER"
  echo "Logout/login required for input group membership"
  ```
- `.desktop`-Eintrag für Dock-Pin-Fallback (falls jemand es als App statt Applet will)
- AppStream-Metadata für COSMIC Store (Zukunft)

### Phase 7 — Polish & Release ⏳
- README mit Screenshots, Install-Anleitung, Pop!_OS-spezifische Hinweise
- CI-Build via GitHub Actions → `.deb`-Artefakt
- Tag v0.1.0

## 6. Risiken & Offene Punkte

| Risiko | Mitigation |
|---|---|
| libcosmic-API pre-1.0, Breaking Changes | Version in Cargo.lock pinnen, monatliches Bump-Review |
| SVG bei 24px unscharf | Raster-Fallback-Assets @32/48/64px vorsehen |
| User vergisst Re-Login nach `input`-Gruppe | Popup beim ersten Start: „keine Devices? → logout" |
| Battery-Drain durch Polling | ✅ gelöst: epoll via `Device::into_event_stream()` — kein Busy-Loop |
| Kernel-Anticheat (falls zukünftig relevant) | README-Disclaimer, passives Lesen dokumentieren |
| COSMIC-Panel-Höhen-Varianz | Dynamisch via `PanelAnchor` + `size` aus Applet-Context |
| Zwei Applet-Instanzen pro Dock schreiben parallel | ✅ gelöst: atomic tmp+rename mit `pid`-Suffix, beide zählen identisch |
| User manipuliert Counter-Datei | ✅ gelöst (für naive Edits): AES-GCM Tag-Check; File an machine-id gebunden |
| I/O-Druck bei schnellem Tippen | ✅ gelöst: Save nur alle 5 s + nur wenn dirty; per-Event-Log auf `trace` (silent by default) |

## 7. Grafik-Assets

**Empfehlung:** SVG-Neuzeichnung in 4 Posen (idle / left / right / both), Style: Tux-Penguin im Bongo-Cat-Meme-Gestus (trommelnde Flossen statt Pfoten). Originell gezeichnet — keine Urheberrechts-Themen. Einfarbiger Transparent-Hintergrund, damit Panel-Hintergrund durchscheint.

Alternative: Original-Sprites wären public-domain-fragwürdig — lieber neu zeichnen.

## 8. Aufwandsschätzung

| Phase | Status | Dauer (Vollzeit) |
|---|---|---|
| 1. Scaffold | ✅ | 0.5 Tage |
| 2. Input | ✅ | 1 Tag |
| 2b. Counter + Persistenz | ✅ | 0.5 Tage |
| 3. Animation | ⏳ next | 1 Tag |
| 4. Hotplug | ⏳ | 0.5 Tage |
| 5. Settings | ⏳ | 1 Tag |
| 6. Packaging | ⏳ | 1 Tag |
| 7. Polish | ⏳ | 0.5 Tage |
| **Summe** | | **~6 Tage** |

## 9. Warum evdev systemweit funktioniert

`/dev/input/event*` liegt auf Kernel-Ebene — *unter* Wayland/X11/Spielen. Events werden dort abgegriffen, bevor Compositor oder Game-Input-Grab sie sehen.

Bestätigte Kompatibilität:
- **COSMIC/Wayland**: ✅ Compositor liest selbst auch evdev, mehrere Reader sind Normalfall
- **Proton-Spiele** (inkl. Marvel Rivals mit `PROTON_ENABLE_WAYLAND=1 gamemoderun PROTON_ENABLE_NVAPI=1 VKD3D_CONFIG=dxr11`): ✅ Proton übersetzt DirectInput/XInput → SDL/Wayland, nicht raw evdev
- **Exclusive Fullscreen / SDL Relative Mouse**: ✅ Kernel-Events werden trotzdem erzeugt
- **Anti-Cheat (EAC/BattlEye/NetEase)**: ✅ Passives Lesen ohne Injection/Memory-Touch ist kein Flag

Einzige Ausnahme: `EVIOCGRAB` durch Low-Level-Tools (`wev`, Input-Remapper) — betrifft uns praktisch nie.
