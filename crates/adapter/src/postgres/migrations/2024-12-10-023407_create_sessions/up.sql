-- Your SQL goes here
CREATE TABLE sessions
(
    id               UUID PRIMARY KEY,
    status           TEXT      NOT NULL,
    pk               bytea     NOT NULL,
    phantom_server   bytea     NOT NULL,
    encrypted_result bytea     NOT NULL,
    client_info      bytea     NOT NULL,

    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);