CREATE TABLE accounts (
    account_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    networks TEXT NOT NULL,
    metadata TEXT NOT NULL
);

CREATE TABLE account_assets (
    account_id INTEGER NOT NULL,
    asset_id TEXT NOT NULL,
    PRIMARY KEY (account_id, asset_id)
);
