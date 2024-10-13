# ATT: All The Things

Prototype knowledge base/note-taking app in Rust with support for semantic data, and augmenting that data with derived data from external sources. Currently a vertical slice with only Rust crates from crates.io as data.

## Running

### Client

Run with: `cargo run --bin att_client_iced`

### Server

- Install the [Diesel CLI](https://github.com/diesel-rs/diesel/tree/master/diesel_cli) if not yet installed.
- Set up a postgresql database connectable from `postgres://att:att@localhost/att`, or change `DATABASE_URL` in `.env` to point to your database.
- Run `diesel setup` to create the database and tables.
- Set the `ATT_CRATES_IO_USER_AGENT` environment variable to a [user agent for accessing the crates.io API, as per their instructions](https://crates.io/data-access#api). You can also create the `user.env` file and set the env var there.
- Run with: `cargo run --bin att_server`
