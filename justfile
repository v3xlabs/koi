koi-db := justfile_directory() / "crates" / "bin" / "koi.db"
export KOI_DATABASE_URL := "sqlite://" + koi-db

default:
    just --list

install:
    cd interfaces/web && pnpm install
    cargo fmt --all

ui:
    cd interfaces/web && pnpm dev
openapi:
    cd interfaces/web && pnpm openapi

[private]
run +args:
    cd crates/bin && cargo run --release -- {{args}}

[private]
build-web:
    cd interfaces/web && pnpm build

dev: (run "daemon")
tui: (run "tui")
gui: (build-web) (run "gui")
migrate: (run "migrate")
migrate-skip: (run "migrate" "--skip")

lint:
    cargo fmt --all
    cargo clippy --workspace
    cd interfaces/web && pnpm lint --fix
bacon:
    cd crates/bin && bacon

build:
    cd interfaces/web && pnpm build
    cd crates/bin && cargo build --release

[parallel]
both: ui dev

# --- mobile (run inside `nix develop .#mobile`) ---

# Regenerate flutter_rust_bridge bindings after changing crates/ffi/src/api
# (config lives in the pubspec's flutter_rust_bridge section)
bridge:
    cd interfaces/mobile && flutter_rust_bridge_codegen generate

# Host-side test of the full bridge: flutter test loads the real rust cdylib
# via the generated loader's ioDirectory (crates/ffi/target -> ../../target)
mobile-test:
    cargo build --release -p koi-ffi
    ln -sfn ../../target crates/ffi/target
    cd interfaces/mobile && flutter test

# Build the Rust core for Android and drop it into jniLibs.
# Run before `flutter build` / `flutter run` whenever Rust changed.
rust-android:
    cargo ndk -t arm64-v8a \
        -o {{justfile_directory()}}/interfaces/mobile/android/app/src/main/jniLibs \
        build --release -p koi-ffi

# iOS static libs (mac only); linked via interfaces/mobile/ios/Flutter/koi.xcconfig
rust-ios:
    cargo build --release --target aarch64-apple-ios -p koi-ffi

rust-ios-sim:
    cargo build --release --target aarch64-apple-ios-sim -p koi-ffi

# Full Android debug build
apk: rust-android
    cd interfaces/mobile && flutter build apk --debug

# Full iOS debug build (mac only)
ios: rust-ios
    cd interfaces/mobile && flutter build ios --debug --no-codesign
