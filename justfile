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
