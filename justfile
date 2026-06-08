koi-db := justfile_directory() / "crates" / "bin" / "koi.db"
export KOI_DATABASE_URL := "sqlite://" + koi-db

default:
    just --list

install:
    cd crates/web && pnpm install
    cd crates/app && cargo fmt
    cd crates/bin && cargo fmt
    cd crates/gui && cargo fmt
    cd crates/tui && cargo fmt
    cd crates/client && cargo fmt

ui:
    cd crates/web && pnpm dev
openapi:
    cd crates/web && pnpm openapi

[private]
run +args:
    cd crates/bin && cargo run -- {{args}}

[private]
build-web:
    cd crates/web && pnpm build

dev: (run "serve")
tui: (run "tui")
gui: (build-web) (run "gui")
migrate: (run "migrate")
migrate-skip: (run "migrate" "--skip")

lint:
    cd crates/app && cargo fmt && cargo clippy
    cd crates/bin && cargo fmt && cargo clippy
    cd crates/gui && cargo fmt && cargo clippy
    cd crates/tui && cargo fmt && cargo clippy
    cd crates/client && cargo fmt && cargo clippy
    cd ui && pnpm lint --fix
bacon:
    cd crates/bin && bacon

build:
    cd crates/web && pnpm build
    cd crates/bin && cargo build --release

[parallel]
both: ui dev
