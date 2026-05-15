default:
    just --list

install:
    cd ui && pnpm install
    cd app && cargo fmt

ui:
    cd ui && pnpm dev
openapi:
    cd ui && pnpm openapi
dev:
    cd app && cargo run
lint:
    cd app && cargo fmt && cargo clippy
    cd ui && pnpm lint --fix
bacon:
    cd app && bacon

build:
    cd ui && pnpm build
    cd app && cargo build --release

[parallel]
both: ui dev
