CREATE TABLE account_groups (
    group_identity INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE accounts ADD COLUMN group_id INTEGER REFERENCES account_groups(group_identity);
ALTER TABLE accounts ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

UPDATE accounts SET display_order = account_identity WHERE display_order = 0;
