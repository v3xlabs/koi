CREATE TABLE quoters (
    quoter_identity TEXT PRIMARY KEY,
    quoter_name TEXT NOT NULL,
    token_a TEXT NOT NULL,
    token_b TEXT NOT NULL,
    config TEXT NOT NULL,
    enabled BOOLEAN,
    watch BOOLEAN
);
