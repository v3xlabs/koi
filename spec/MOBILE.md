# Mobile integration — Flutter UI on the koi core

_Written 2026-07-13. Source prototype: `koi-mobile` (sibling repo). Goal: this repo
ships the daemon + web + TUI + GUI it ships today, **plus** an Android/iOS Flutter
app running the same Rust core in-process via `flutter_rust_bridge` (FRB)._

**Ground rule: `crates/app` (koi-app) is the canonical core.** It already does
networks/endpoint pools, RPC health + rotation + throttling, accounts (read-only),
assets-per-account, quoting (uniswap v2/v3, erc4626 discovery, ECB fiat via
`eth-prices`), vendors, calldata decoding, and SQLite storage — all better than the
prototype's equivalents. The prototype's job was to prove Flutter-over-FRB works
(it does, device-verified on Android arm64), and to prototype the pieces koi
doesn't have yet: key custody, signing, tx history indexing, ENS, openlv. Those
move *into* the core on the core's terms; everything the prototype duplicated gets
deleted in favor of koi code.

| Target | UI | Talks to core via |
|---|---|---|
| Desktop daemon + web | SolidJS (`crates/web`), embedded | HTTP `127.0.0.1:7777` |
| TUI / GUI | ratatui / webview | HTTP (boots embedded daemon if absent) |
| **Mobile (Android/iOS)** | **Flutter (`app/`)** | **FRB bindings, in-process — no daemon, no HTTP** |

```
            desktop/web/tui                      mobile
        ┌──────────────────────┐        ┌───────────────────────┐
        │ SolidJS / ratatui    │        │ Flutter UI (app/)     │
        └─────────┬────────────┘        └──────────┬────────────┘
                  │ HTTP :7777                     │ FRB (in-process FFI)
        ┌─────────▼────────────┐        ┌──────────▼────────────┐
        │ crates/daemon        │        │ crates/ffi            │  ← platform adapter
        │ poem HTTP + web embed│        │ FRB api + mobile hooks│    crates: thin,
        └─────────┬────────────┘        └──────────┬────────────┘    no logic
                  └───────────────┬────────────────┘
                        ┌─────────▼──────────┐
                        │ crates/app (koi)   │  the core — models, state, sqlx,
                        │                    │  networks, quoters, vendors
                        └────────────────────┘  + vault/signing/indexer/ens/lv
```

One core, two platform adapters. The core knows nothing about HTTP or FFI; each
platform crate holds its OS hooks — the daemon crate owns poem, the OpenAPI
service, and the embedded web UI; the ffi crate owns the FRB runtime, the
Android/iOS TLS + path init, and the bridge types.

---

## 1. What is canonical where

**koi-app is the source of truth for** (prototype equivalents get deleted):

| Domain | koi-app | prototype (`crates/koi`) — fate |
|---|---|---|
| Networks + RPC | `models/network/` — endpoint pool per network, health metrics, provider rotation, throttle/retry layers, DB-backed configs | `chain/` + `transport.rs` single-URL provider — **drop** (keep only the 1.2× `with_gas_headroom` logic; the Android TLS lesson is fixed inside koi's transport, §3.1) |
| Accounts | `models/account/` — DB rows, groups, layout, WalletType (eoa/safe/view/railgun) | vault-embedded account list — **drop**; secrets stay in vault, account rows in DB |
| Assets | `models/asset/` — system-wide assets, per-account enablement | `assets_json` blob in vault settings — **drop** |
| Quoting + fiat | `models/quoter/` — manager, uniswap v2/v3 + erc4626 discovery, `Router::from_iter(quoters).with_ecb()` (ECB fiat already works) | `pricing/` (309 lines) — **drop entirely**, redundant |
| Vendors | `vendor/` (blockscout, safe, smoldapp, sourcify, zerion, avara) + DB-backed `VendorManager` | `vendor/` dupes + dead `VENDOR_MANAGER` static — **drop**; adopt the prototype's discovery-only policy (§4) |
| Calldata decode | `models/tx/decode.rs` (664 lines) | `indexer::known_selector` — **merge** into one map (§4) |
| Storage | sqlx/SQLite + migrations | ad-hoc JSON files — **drop** (vault file excepted; tx index moves to SQLite) |
| Naming | already `network` / `network_identity` (BASELINE-compliant) | `chain`/`chain_id` — resolved by adoption, no sweep needed |

**The prototype is the donor for** (koi-app has nothing here — verified):

| Capability | Donor module | Notes |
|---|---|---|
| Encrypted key storage | `vault/` (806 lines) | Argon2id KEK → XChaCha20-Poly1305-wrapped data key, header as AAD, zeroize on lock; passcode unlock + raw-data-key unlock (the biometric/platform-keystore path); retry delay, wipe, change-passcode. This *is* the "next step on koi" — the format is designed and device-proven, take it as-is |
| Biometric/pin plumbing | `keyring/` + Dart `biometric_storage` | OS releases the data key only after user auth |
| Key flows | `api/mod.rs` + vault | import mnemonic / raw private key / **keystore JSON** (`eth-keystore` — not a koi dep yet), watch-only accounts, key creation, mnemonic validation + reveal. koi only has address derivation (`models/account/derive.rs`) |
| Signing + broadcast | `signing/` (158) + `chain/` send/estimate/status | `alloy-signer-local` is already a koi dependency — this is wiring, not new deps |
| Tx history indexer | `indexer/` (625) | local-first, vendors are hash-discovery-only, every displayed fact from user RPC, immutable-once-final, parallel + checkpointed sync, `transaction_detail` (failure notes, balance changes, ERC-20/721 events, gas breakdown, out-of-gas detection). Persist to SQLite instead of `tx_index_{addr}.json` |
| ENS | `ens/` (177) | forward + reverse + name detection; koi-app has zero ENS |
| dapp connectivity | `lv/` (703) | openlv sessions, request approval, event stream (`StreamSink`); web does openlv in JS today — Rust impl can serve both later |
| Auth model | vault unlock semantics | koi's `http/auth/mod.rs` is a placeholder (accepts anything, returns user `"123"`). Desktop unlock/session design should be built on vault unlock; on mobile the trust boundary is the process itself |

The Flutter app (`app/`, ~55 Dart files: riverpod + go_router; onboarding, unlock,
home, send, tx detail, assets, accounts, address book, vendors, settings, QR, lv
overlay) moves over as-is and is the mobile UI, full stop.

---

## 2. Repo layout & Phase A (relocate, unchanged)

```
koi/
├── app/                     # Flutter app (from koi-mobile/app)
├── crates/
│   ├── app/                 # the core (pkg koi-app, lib koi) — http/ extracted out
│   ├── daemon/              # NEW — poem HTTP API + web embed (from crates/app/src/http)
│   ├── ffi/                 # NEW — FRB crate (from koi-mobile/crates/koi, renamed koi-ffi)
│   ├── bin/ client/ gui/ tui/ web/         # desktop family, as today
├── flutter_rust_bridge.yaml # rust_root: crates/ffi, dart_output: app/lib/src/core/bridge
└── justfile                 # + bridge / rust-android / rust-ios / apk / ios recipes
```

Crate roles after the split (§3.4 has the mechanics):

- **`crates/app`** — the shared core. Keeps its package/lib name (`koi-app` / `koi`)
  to avoid churn; a rename (e.g. `crates/koi`) is cosmetic and can happen any time.
  No poem, no rust-embed, no FRB. Everything upstreamed from the prototype lands
  here.
- **`crates/daemon`** — desktop-specific: the poem-openapi routes, auth, the
  OpenAPI service/spec, and the embedded `crates/web` bundle. `crates/bin` invokes
  it for `daemon`/`tui`/`gui` startup; gui/tui/client/web stay separate crates as
  today and remain HTTP consumers.
- **`crates/ffi`** — mobile-specific: FRB api surface + generated glue, bridge
  init (data dir from `path_provider`, TLS/crypto-provider init), FRB mirror
  types, `StreamSink` event plumbing.

Land the prototype byte-for-byte first (it's self-contained — zero imports of
koi-app), so the repo ships the working app before any convergence starts:

1. Copy `crates/koi` → `crates/ffi`; rename package `koi-ffi`, `[lib] name = "koi_ffi"`
   (avoids collision with `crates/bin` pkg `koi` / `crates/app` lib `koi`), keep
   `crate-type = ["cdylib", "staticlib", "rlib"]`. Add to workspace members.
2. Copy `app/` (exclude `build/`, `.dart_tool/`, `.idea/`, committed `jniLibs/*.so`
   — gitignore jniLibs, it's always produced by the build).
3. Copy `flutter_rust_bridge.yaml` (update `rust_root: crates/ffi`), regen with
   `flutter_rust_bridge_codegen generate` (binary pinned **2.9.0**, must match the
   `=2.9.0` pin in both Cargo.toml and pubspec; put `~/.cargo/bin` on the devshell
   PATH — it wasn't in the prototype and every codegen run tripped on it).
4. Update `app/ios/Flutter/koi.xcconfig` lib path to `crates/ffi/.../libkoi_ffi.a`;
   keep `-force_load` (FRB symbols are dlsym-only and get dead-stripped without it).
5. Flake: add a `devShells.mobile` — rust targets (`aarch64-linux-android`,
   `armv7-linux-androideabi`, `x86_64-linux-android`, `aarch64-apple-ios{,-sim}`,
   `x86_64-apple-ios`), `flutter`, `cargo-ndk`, `android-tools`, `temurin-bin-21`,
   composed Android SDK (platforms 35/36). **Pin one NDK** and use
   `$ANDROID_NDK_HOME` in the justfile — the prototype's flake exported NDK 27
   while its justfile used a hand-installed 28.2; fix that here, don't copy it.
6. Port the just recipes (`bridge`, `rust-android` via cargo-ndk → jniLibs,
   `rust-ios`, `rust-ios-sim`, `apk`, `ios`), copy `BASELINE.md`.
7. Verify: `cargo test` in crates/ffi (17 pass, 1 ignored live-ENS), `flutter
   analyze` + `flutter test`, `just apk` on device; `just build` / `just dev`
   untouched on desktop.

---

## 3. The real work: what must be fixed for koi-app to run cross-platform

These are the concrete blockers found by inspection, in priority order. Until all
of them land, koi-app cannot be linked into the mobile app at all.

### 3.1 TLS: make certificate verification work on Android inside koi's transport

koi's endpoint pool / provider construction stays exactly as it is — this is a
dependency-configuration fix, not an architecture change, and the prototype's
custom `transport.rs` is dropped rather than adopted.

The issue (confirmed in `Cargo.lock`): alloy's HTTP transport
(`alloy-transport-http` / `alloy-rpc-client` / `alloy-provider`) pulls
`reqwest 0.13`, whose rustls path uses `rustls-platform-verifier` +
`rustls-platform-verifier-android`. On Android that verifier needs explicit
JVM/JNI initialization, which FRB's tokio worker threads don't get for free — the
panic the prototype hit. koi-app's own `reqwest 0.12` (`rustls-tls` =
webpki-roots) is unaffected.

Resolvable either way inside `EthProvider::update`
(`models/network/endpoint/provider.rs`, ~line 141), where the client is built:

- initialize the platform verifier once with the Android context at bridge init
  (`rustls-platform-verifier` supports this; it's the "correct" path and keeps OS
  trust stores), **or**
- hand alloy's `ClientBuilder` a preconfigured `reqwest::Client` using
  webpki-roots (what the prototype did — two lines of TLS config, identical
  behavior on all platforms).

Either is fine; pick when wiring the bridge. **In both cases**: install exactly
one rustls crypto provider at startup — both `aws-lc-rs` and `ring` are in koi's
tree, and rustls panics when it can't pick one. One
`rustls::crypto::aws_lc_rs::default_provider().install_default()` in `State::new`
(aws-lc-rs cross-compiles fine with the NDK — proven by the prototype).

### 3.2 Paths: configuration assumes a desktop filesystem

- `config.rs` resolves the DB from `KOI_DATABASE_URL`/`DATABASE_URL` env → cwd
  `koi.db` → `dirs::config_dir()`. On Android/iOS: env vars don't exist, cwd is `/`,
  and `dirs` answers are wrong or unusable inside an app sandbox.
- `abi_cache_dir` defaults to relative `"cache/abis"` (resolved against cwd).

**Fix**: add `State::new_with(config: Configuration)` (or a `data_dir` override that
derives `database_url` and `abi_cache_dir` beneath it). Desktop keeps
`Configuration::load()`; mobile constructs the config from the `path_provider`
app-support directory Dart passes across the bridge once at init. This replaces the
prototype's `dir: String` parameter on every single API call — init once, hold the
`AppState` in a static in `crates/ffi` (the pattern its `api/mod.rs` already uses).

### 3.3 sqlx runtime: `runtime-async-std` under an FRB/tokio host

sqlx works on mobile as-is — the `sqlite` feature bundles `libsqlite3-sys`, which
cross-compiles with the NDK, no system sqlite needed. But koi picked
`runtime-async-std`, so the phone would run an async-std reactor *alongside* FRB's
embedded tokio runtime. Switch the sqlx feature to `runtime-tokio` (one line in
`crates/app/Cargo.toml`; the daemon already runs tokio everywhere).

### 3.4 Split the server out of the core: `crates/app/src/http` → `crates/daemon`

`crates/app` today compiles poem + poem-openapi (git fork) and `rust-embed`s
`../web/dist` unconditionally — a mobile build would need the web frontend built
just to compile, and would ship an HTTP server + embedded SolidJS app in the APK.

**Fix: crate split, not feature gates.** Move `src/http/` (routes, `serve()`, the
OpenAPI service, `WebAssets` embed, `Auth`) into a new `crates/daemon`
(pkg `koi-daemon`) that depends on the core. `crates/bin` switches its
`http::serve(state)` calls to `koi_daemon::serve(state)`. The core drops poem,
poem-openapi, and rust-embed from its unconditional deps; `crates/ffi` depends on
the bare core. Platform hooks then live where they belong: daemon-only concerns
(listeners, embedded web, OpenAPI spec, HTTP auth) in `crates/daemon`, mobile-only
concerns (FRB, path/TLS init) in `crates/ffi`, and the core stays OS-agnostic.

One entanglement to unwind (measured): poem-openapi reaches into ~25 non-http
files — `#[derive(Object)]`/`Union` on nearly every model, manual
`impl poem_openapi::types::Type` in `models/alloy.rs`, and
`IntoResponse`/`From<KoiError> for poem::Error` in `error.rs`. The pragmatic cut:

- Models/error keep their OpenAPI derives behind a small **`openapi` feature** in
  the core (optional `poem-openapi` dep, `cfg_attr` on derives, `cfg`-gated impl
  blocks for `alloy.rs` and `error.rs`). `crates/daemon` enables the feature;
  `crates/ffi` doesn't. The orphan rule forces this — the error→poem conversions
  can't move to the daemon crate without newtype boilerplate on every endpoint.
- So: server *code* physically moves out; only inert derive attributes stay
  behind a feature. Desktop builds are unaffected (daemon always enables it), and
  feature unification can't leak into mobile since `cargo ndk` builds
  `-p koi-ffi` as its own graph.

### 3.5 Mobile process lifecycle

Verified: koi-app spawns **no** background tasks (`tokio::spawn`/interval loops —
none in `crates/app/src`); everything is request-driven, caches are in-memory
(`moka`, `BalanceCacheManager`) or on disk. That's exactly right for mobile, where
the OS freezes the process at will. Two rules going forward:

- Anything long-running added to the core (indexer sync!) must be checkpointed and
  resumable, never a persistent daemon loop. The prototype's indexer already works
  this way (saves after every enrichment batch; `last_synced_block` only advances
  when complete) — keep that property when porting.
- The daemon can later add background refresh loops, but they belong in
  `crates/daemon`, never in shared core paths.

### 3.6 Version alignment: upgrade koi-app to alloy 2

**Decision: koi-app moves to alloy 2** as an early step of the merge — for koi's
usage (provider calls, primitives, signer, tx types) the API delta is small, and
`eth-prices` is already alloy-2 compatible (bump 0.0.10 → 0.0.11 alongside). The
payoff: every donor module (vault, signing, indexer, ens, lv) is already written
against alloy 2 and ports **unchanged**, and the reqwest generations converge on
0.13. Touchpoints in koi-app: `models/alloy.rs`, the endpoint/provider layer,
`tx/decode.rs`, `alloy-signer-local`. Do it as its own PR with desktop-only risk,
before any donor module lands. (Edition 2021 → 2024 in the ffi crate: whenever
convenient, no behavior impact.)

### 3.7 Toolchain / build glue

- Rust toolchain needs the six mobile targets (flake, §2.5); edition 2024 needs a
  current stable — pin via rust-overlay.
- Android: `.so` is built by `cargo ndk` outside Gradle (AGP must not try to
  auto-install NDK into the read-only Nix SDK — workaround comments already in the
  prototype's `build.gradle.kts`); artifact lands in
  `app/android/app/src/main/jniLibs/arm64-v8a/libkoi_ffi.so`.
- iOS: static `libkoi_ffi.a` linked via xcconfig with `-force_load`; untested
  (needs a mac) but scaffolding is complete.
- CI: an `aarch64-linux-android` `cargo check -p koi-app` (core, without the
  `openapi` feature) + `-p koi-ffi` catches poem leakage into the core and
  TLS/paths regressions without needing an emulator.

---

## 4. Remaining functional differences (koi-side additions)

What the mobile UI calls today vs what koi-app offers — this is the gap list for
the `crates/ffi` adapter to become thin. Grouped by the prototype's ~65 FRB
functions:

| Mobile API group | koi-app today | Needed |
|---|---|---|
| `generate/validate_mnemonic`, `address_from_{mnemonic,private_key}` | `models/account/derive.rs` has generate + derive | add `validate`; reuse rest |
| `address_from_keystore`, `private_key_from_keystore` | nothing (no `eth-keystore` dep) | add keystore JSON import to core |
| `create_vault`, `unlock_with_{passcode,key}`, `lock`, `change_passcode`, `wipe`, `unlock_retry_delay`, `biometric_unlock_key`, `verify_passcode` | nothing (auth is a stub) | **port vault + keyring wholesale** — this is koi's planned key-storage step; also becomes the base for upgrading the desktop `Auth` placeholder to real unlock sessions |
| `add/remove/rename_account`, `reveal_mnemonic`, watch-only add | accounts CRUD exists (read-only, no secrets) | connect vault secrets ↔ account rows; import/create/watch flows into core |
| `ens_resolve/reverse`, `is_ens_name` | nothing | port `ens/` |
| `get_balance`, `fetch_balances(_with_prices)` | `models/account/balances.rs` + quoters + balance cache — **better** | reuse; expose staleness (`_lastFetched`) for the quote-age UI |
| `estimate_gas`, `send_transaction`, `transaction_status`, `erc20_transfer_data` | decode + simulate exist; **no signing, no broadcast** | port `signing/` + send/estimate (keep 1.2× gas headroom); wire to vault |
| `simulate_transaction` | `models/tx/simulate.rs`, `http/net/simulate.rs` | reuse |
| `get/sync_transaction_history`, `index_transaction`, `transaction_detail` | only Safe-vendor tx fetch (`http/account/tx.rs`) | port `indexer/` onto SQLite (new migration); merge its `known_selector` map with `models/tx/decode.rs` into **one** selector map; enforce vendors-are-discovery-only (never trust vendor timestamps/status/values for display) |
| `get/set_account_assets` | per-account assets in DB — **better** | reuse; drop the JSON-blob settings path |
| `get/set_quoter_config` | QuoterManager in DB — **better**, ECB fiat included | reuse; drop `pricing/` |
| `get/set_vendor_flags` | DB-backed `VendorManager` — **better** | reuse (the prototype's Rust-side flag manager was dead code anyway; its truth lived in vault settings) |
| `zerion/smoldapp_icon_url`, `coingecko_price_url` | vendor modules exist | reuse |
| `lv_*` (connect, approve, reject, events stream, sessions) | nothing in Rust (web does openlv in JS) | port `lv/`; move the remaining Dart-side ERC-20 calldata decode behind the bridge (BASELINE rule) |

Settings split after convergence: vault file keeps secrets + whatever must exist
pre-unlock (appearance, passcode params); SQLite keeps everything else (accounts,
assets, networks, quoters, vendor flags, tx index). Tx-index JSON files need no
migration shim — the index is rebuildable by design.

Dart-side deltas (mechanical): bridge init becomes `init(dataDir)` once instead of
`dir:` on every call; `assets_service` / `quoter_config` / vendor pages re-point at
typed calls backed by koi's DB; `transaction_cache_service` stays a thin memory
cache. Everything else (routing, theming, onboarding, QR, lv overlay) is untouched.

Known prototype debts to fix during the merge, not before (from its HANDOFF.md):
unlock-path biometric fail-open needs verification on a device with enrolled
biometrics; iOS build untested; quote-age not yet surfaced in UI.

---

## 5. Order of work

1. **Phase A** (§2) — repo ships the working mobile app, desktop untouched.
2. koi-app → alloy 2 (+ eth-prices 0.0.11) as its own PR, desktop-only risk (§3.6).
3. Cross-platform fixes (§3.1–3.4): Android TLS verification + single crypto
   provider install, `State::new_with`, sqlx → tokio, and the
   `http/` → `crates/daemon` split (with the `openapi` derive feature in core).
   After this, `crates/ffi` can link the core and start delegating.
4. Re-point the easy wins in `crates/ffi` at koi-app: networks/RPC pool, balances,
   assets, quoters, vendor flags, simulate. Delete `chain/`, `pricing/`,
   `transport.rs`, vendor dupes, the dead flag manager.
5. Port vault + keyring + key flows (keystore/watch/create/import) into koi-app —
   unchanged, they're already alloy 2; wire signing/send/estimate. This is also
   the foundation for replacing the desktop auth placeholder.
6. Port indexer onto SQLite; unify the selector map with `tx/decode.rs`; port `ens/`.
7. Port `lv/`; move lv calldata decoding out of Dart.
8. Daemon-side adoption: expose unlock/signing/history over HTTP with the upgraded
   auth so web/TUI reach parity with mobile.

## 6. Day-to-day reference (post-merge)

```bash
nix develop .#mobile            # rust w/ mobile targets, flutter, SDK/NDK, cargo-ndk, frb codegen
just bridge                     # after any crates/ffi/src/api change (regen Rust + Dart)
just rust-android               # cargo ndk → app/.../jniLibs/…/libkoi_ffi.so
just apk                        # Android debug APK
just ios | just rust-ios-sim    # mac only
cd crates/ffi && cargo test
cd app && flutter test && flutter analyze
just dev | just tui | just gui | just build   # desktop, unchanged
```

Device gotchas proven on the prototype's test device (OnePlus/ColorOS): wifi-adb
shell is sandboxed (`pm clear`/`pm grant` → SecurityException; clean state =
uninstall/reinstall); `flutter run` needs the `.so` built first (Gradle doesn't
drive cargo); crash stacks via `adb logcat | grep 'I flutter'`.
