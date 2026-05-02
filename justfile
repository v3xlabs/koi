default:
    just --list

install:
    cd ui && pnpm install
    cd crates/app && cargo fmt
    cd crates/bin && cargo fmt

ui:
    cd crates/web && pnpm dev
openapi:
    cd crates/web && pnpm openapi
dev:
    cd crates/bin && cargo run
tui:
    cd crates/bin && cargo run -- tui
lint:
    cd crates/app && cargo fmt && cargo clippy
    cd crates/bin && cargo fmt && cargo clippy
    cd ui && pnpm lint --fix
bacon:
    cd crates/bin && bacon

build:
    cd crates/web && pnpm build
    cd crates/bin && cargo build --release

[parallel]
both: ui dev
