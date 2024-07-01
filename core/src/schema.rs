// @generated automatically by Diesel CLI.

diesel::table! {
    crate_default_versions (crate_id) {
        crate_id -> Int4,
        version_id -> Int4,
    }
}

diesel::table! {
    crate_downloads (crate_id) {
        crate_id -> Int4,
        downloads -> Int8,
    }
}

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

diesel::joinable!(crate_default_versions -> crate_versions (version_id));
diesel::joinable!(crate_default_versions -> crates (crate_id));
diesel::joinable!(crate_downloads -> crates (crate_id));
diesel::joinable!(crate_versions -> crates (crate_id));
diesel::joinable!(favorite_crates -> crates (crate_id));
diesel::joinable!(favorite_crates -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    crate_default_versions,
    crate_downloads,
    crate_versions,
    crates,
    favorite_crates,
    import_crates_metadata,
    users,
);
