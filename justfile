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
