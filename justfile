default:
    just --list

install:
    cd crates/web && pnpm install
    cd crates/app && cargo fmt
    cd crates/bin && cargo fmt
    cd crates/gui && cargo fmt

ui:
    cd crates/web && pnpm dev
openapi:
    cd crates/web && pnpm openapi
dev:
    cd crates/bin && cargo run
tui:
    cd crates/bin && cargo run -- tui

gui:
    cd crates/web && pnpm build
    cd crates/bin && cargo run -- gui

lint:
    cd crates/app && cargo fmt && cargo clippy
    cd crates/bin && cargo fmt && cargo clippy
    cd crates/gui && cargo fmt && cargo clippy
    cd ui && pnpm lint --fix
bacon:
    cd crates/bin && bacon

build:
    cd crates/web && pnpm build
    cd crates/bin && cargo build --release

[parallel]
both: ui dev
