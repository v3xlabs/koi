CREATE TABLE account_assets (
    account_identity INTEGER NOT NULL,
    asset_identity TEXT NOT NULL,
    PRIMARY KEY (account_identity, asset_identity),
    FOREIGN KEY (account_identity) REFERENCES accounts(account_identity),
    FOREIGN KEY (asset_identity) REFERENCES assets(asset_identity)
);
