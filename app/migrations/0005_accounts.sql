CREATE TABLE accounts (
    account_identity INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    networks TEXT NOT NULL,
    metadata TEXT NOT NULL
);

CREATE TABLE account_assets (
    account_identity INTEGER NOT NULL,
    asset_id TEXT NOT NULL,
    PRIMARY KEY (account_identity, asset_id)
);
