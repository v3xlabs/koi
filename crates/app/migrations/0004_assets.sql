CREATE TABLE assets (
    asset_identity TEXT PRIMARY KEY,
    asset_name TEXT NOT NULL,
    asset_symbol TEXT NOT NULL,
    asset_decimals INTEGER NOT NULL,
    asset_icon_url TEXT
);
