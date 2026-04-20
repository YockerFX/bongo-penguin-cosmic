name := "cosmic-applet-bongo-penguin"
app-id := "com.github.bongopenguin.CosmicAppletBongoPenguin"
prefix := "/usr"
bindir := prefix / "bin"
sharedir := prefix / "share"
appdir := sharedir / "applications"
iconsdir := sharedir / "icons/hicolor"

default: build-release

check:
    cargo check --all-targets

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets -- -D warnings

build-debug:
    cargo build

build-release:
    cargo build --release

run: build-debug
    ./target/debug/{{name}}

install: build-release
    install -Dm0755 target/release/{{name}} {{bindir}}/{{name}}
    install -Dm0644 data/{{app-id}}.desktop {{appdir}}/{{app-id}}.desktop

uninstall:
    rm -f {{bindir}}/{{name}}
    rm -f {{appdir}}/{{app-id}}.desktop

deb:
    dpkg-buildpackage -us -uc -b
    @echo ""
    @echo "Built .deb is in the parent directory:"
    @ls -1 ../{{name}}_*.deb 2>/dev/null || true

clean:
    cargo clean
    rm -rf debian/.debhelper debian/{{name}} debian/files debian/debhelper-build-stamp debian/*.substvars .cargo-home
