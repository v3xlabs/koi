CREATE TABLE asset_icons (
    asset_identity TEXT PRIMARY KEY NOT NULL REFERENCES assets(asset_identity) ON DELETE CASCADE,
    png_data BLOB NOT NULL
);
