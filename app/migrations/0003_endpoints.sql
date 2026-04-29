CREATE TABLE network_endpoints (
    endpoint_identity INTEGER PRIMARY KEY,
    endpoint_label TEXT,
    endpoint_type TEXT NOT NULL,
    endpoint_url TEXT NOT NULL,
    endpoint_disabled BOOLEAN NOT NULL,
    network_identity INTEGER NOT NULL
);
