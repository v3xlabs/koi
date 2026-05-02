<p align="center">
<a href="https://ethereum.github.io/kohaku/">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/ethereum/kohaku/refs/heads/master/docs/public/kohaku_logo.svg">
<img alt="Kohaku logo" src="https://raw.githubusercontent.com/ethereum/kohaku/refs/heads/master/docs/public/kohaku_logo.svg" width="auto" height="60">
</picture>
</a>
</p>

A local, privacy, and freedom-focused wallet.

> [!IMPORTANT]
> This project is currently work in progress and not ready for production use.

## What is Koi?

Koi runs as a local daemon wallet and serves a web interface on 127.0.0.1:7777.
It allows for wallet management, transaction signing, and dapp connectivity.

## How?

The koi daemon is written in Rust, it serves a strict api over http to the local machine.
It queries data directly via ethereum rpc; or via opt-in 3rd party vendor services.

## Development

```bash
nix develop
just install
just both
```
