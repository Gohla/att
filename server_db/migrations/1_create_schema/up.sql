--
-- Users
--

CREATE TABLE users (
  id            serial,
  name          varchar NOT NULL,
  password_hash varchar NOT NULL
);
ALTER TABLE ONLY users
  ADD CONSTRAINT users_pkey PRIMARY KEY (id);
-- UNIQUE: user names.
CREATE UNIQUE INDEX users_name_unique ON users USING btree (name);


--
-- Crates
--

CREATE TABLE crates (
  id                 serial,
  name               varchar          NOT NULL,
  updated_at         timestamptz      NOT NULL,
  created_at         timestamptz      NOT NULL,
  description        varchar          NOT NULL,
  homepage           varchar          NULL,
  readme             varchar          NULL,
  repository         varchar          NULL,

  downloads          bigint DEFAULT 0 NOT NULL,

  default_version_id integer          NOT NULL
);
ALTER TABLE ONLY crates
  ADD CONSTRAINT crates_pkey PRIMARY KEY (id);
-- UNIQUE: crate names.
CREATE UNIQUE INDEX crates_name_index ON crates USING btree (name);
CREATE INDEX crates_updated_at_index ON crates USING btree (updated_at);
CREATE INDEX crates_created_at_index ON crates USING btree (created_at);

CREATE TABLE crate_versions (
  id       serial,
  crate_id integer NOT NULL,
  number   varchar NOT NULL
);
ALTER TABLE ONLY crate_versions
  ADD CONSTRAINT crate_versions_pkey PRIMARY KEY (id);

-- Link crates and crate_versions
ALTER TABLE ONLY crates
  -- DEFERRABLE INITIALLY DEFERRED: in a transaction, allow versions of a crate to be deleted before the crate itself.
  ADD CONSTRAINT crates_default_version_id_fkey FOREIGN KEY (default_version_id) REFERENCES crate_versions (id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE ONLY crate_versions
  -- ON DELETE CASCADE: delete versions of crate when create is deleted.
  ADD CONSTRAINT crate_versions_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES crates (id) ON DELETE CASCADE;
ALTER TABLE ONLY crate_versions
  -- UNIQUE: combination of crate and version number.
  ADD CONSTRAINT crate_versions_crate_id_number_unique UNIQUE (crate_id, number);


--
-- User-Crate data
--

CREATE TABLE favorite_crates (
  user_id  integer NOT NULL,
  crate_id integer NOT NULL
);
ALTER TABLE ONLY favorite_crates
  ADD CONSTRAINT favorite_crates_pkey PRIMARY KEY (user_id, crate_id);
ALTER TABLE ONLY favorite_crates
  -- ON DELETE CASCADE: delete favorite crate when user is deleted.
  ADD CONSTRAINT favorite_crates_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;
ALTER TABLE ONLY favorite_crates
  -- DEFERRABLE INITIALLY DEFERRED: in a transaction, allow crates to be re-imported from a database dump.
  ADD CONSTRAINT favorite_crates_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES crates (id) DEFERRABLE INITIALLY DEFERRED;


--
-- Importing Crates
--

CREATE TABLE import_crates_metadata (
  id          serial,
  imported_at timestamptz NOT NULL
);
ALTER TABLE ONLY import_crates_metadata
  ADD CONSTRAINT import_crates_metadata_pkey PRIMARY KEY (id);
