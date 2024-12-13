// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        id -> Uuid,
        status -> Text,
        pk -> Bytea,
        phantom_server -> Bytea,
        encrypted_result -> Bytea,
        client_info -> Bytea,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
