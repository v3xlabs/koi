CREATE TABLE network_endpoints (
    endpoint_identity TEXT PRIMARY KEY,
    endpoint_label TEXT,
    endpoint_type TEXT NOT NULL,
    endpoint_url TEXT NOT NULL,
    endpoint_disabled BOOLEAN NOT NULL,
    network_identity INTEGER NOT NULL,
    FOREIGN KEY (network_identity) REFERENCES networks (network_identity)
);
