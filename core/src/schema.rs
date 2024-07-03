// @generated automatically by Diesel CLI.

diesel::table! {
    crate_versions (id) {
        id -> Int4,
        crate_id -> Int4,
        number -> Varchar,
    }
}

diesel::table! {
    crates (id) {
        id -> Int4,
        name -> Varchar,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
        description -> Varchar,
        homepage -> Nullable<Varchar>,
        readme -> Nullable<Varchar>,
        repository -> Nullable<Varchar>,
        downloads -> Int8,
        default_version_id -> Int4,
    }
}

diesel::table! {
    favorite_crates (user_id, crate_id) {
        user_id -> Int4,
        crate_id -> Int4,
    }
}

diesel::table! {
    import_crates_metadata (id) {
        id -> Int4,
        imported_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        password_hash -> Varchar,
    }
}

diesel::joinable!(favorite_crates -> crates (crate_id));
diesel::joinable!(favorite_crates -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    crate_versions,
    crates,
    favorite_crates,
    import_crates_metadata,
    users,
);
