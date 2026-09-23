CREATE TABLE rustzen_installation_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    build_id TEXT NOT NULL,
    composition_id TEXT NOT NULL,
    schema_fingerprint TEXT NOT NULL,
    data_contract_id TEXT NOT NULL
);
