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
    cd ui && pnpm lint
bacon:
    cd app && bacon

[parallel]
both: ui dev
