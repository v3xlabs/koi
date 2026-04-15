Kohaku Koi is a privacy-first self-custodial wallet.
It runs as a daemon and serves a web interface on 127.0.0.1

User can read balances, manage wallets, craft transactions, etc.
Users should be able to natively do first-class ethereum operations;
- sending to multiple recipients
- batching transactions together
- multisig support (erc4337 / safe)
- wrapping eth
- vaults (erc4626)
- swapping

Transactions should be simulated, and their state changes should be visible to the user.
When approving a transaction the user should be aware of balance changes, gas costs, etc.

All of the above features should be enabled / disabled on a per-wallet per-chain basis.
