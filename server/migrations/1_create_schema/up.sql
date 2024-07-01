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
  id          serial,
  name        varchar     NOT NULL,
  updated_at  timestamptz NOT NULL,
  created_at  timestamptz NOT NULL,
  description varchar     NOT NULL,
  homepage    varchar     NULL,
  readme      varchar     NULL,
  repository  varchar     NULL
);
ALTER TABLE ONLY crates
  ADD CONSTRAINT crates_pkey PRIMARY KEY (id);
-- UNIQUE: crate names.
CREATE UNIQUE INDEX crates_name_index ON crates USING btree (name);
CREATE INDEX crates_updated_at_index ON crates USING btree (updated_at);
CREATE INDEX crates_created_at_index ON crates USING btree (created_at);

CREATE TABLE crate_downloads (
  crate_id  integer          NOT NULL,
  downloads bigint DEFAULT 0 NOT NULL
);
-- PK: one crate has one downloads entry.
ALTER TABLE ONLY crate_downloads
  ADD CONSTRAINT crate_downloads_pkey PRIMARY KEY (crate_id);
ALTER TABLE ONLY crate_downloads
  -- ON DELETE CASCADE: delete downloads of crate when create is deleted.
  ADD CONSTRAINT crate_downloads_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES crates (id) ON DELETE CASCADE;

CREATE TABLE crate_versions (
  id       serial,
  crate_id integer NOT NULL,
  number   varchar NOT NULL
);
ALTER TABLE ONLY crate_versions
  ADD CONSTRAINT crate_versions_pkey PRIMARY KEY (id);
ALTER TABLE ONLY crate_versions
  -- ON DELETE CASCADE: delete versions of crate when create is deleted.
  ADD CONSTRAINT crate_versions_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES crates (id) ON DELETE CASCADE;
ALTER TABLE ONLY crate_versions
  -- UNIQUE: combination of crate and version number.
  ADD CONSTRAINT crate_versions_crate_id_number_unique UNIQUE (crate_id, number);

CREATE TABLE crate_default_versions (
  crate_id   integer NOT NULL,
  version_id integer NOT NULL
);
-- PK: one crate has one default version entry.
ALTER TABLE ONLY crate_default_versions
  ADD CONSTRAINT crate_default_versions_pkey PRIMARY KEY (crate_id);
ALTER TABLE ONLY crate_default_versions
  -- DEFERRABLE INITIALLY DEFERRED: in a transaction, allow versions of a crate to be deleted before the crate itself.
  ADD CONSTRAINT crate_default_versions_version_id_fkey FOREIGN KEY (version_id) REFERENCES crate_versions (id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE ONLY crate_default_versions
  -- ON DELETE CASCADE: delete versions of crate when create is deleted.
  ADD CONSTRAINT crate_default_versions_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES crates (id) ON DELETE CASCADE;
-- UNIQUE: each version can only be associated with one crate.
CREATE UNIQUE INDEX crate_default_versions_version_id_index ON crate_default_versions USING btree (version_id);


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


-- ON DELETE CASCADE

-- CREATE TABLE import_crates as
--   TABLE crates
--   WITH NO DATA;
-- ALTER TABLE import_crates
--   ADD CONSTRAINT import_crates_pkey PRIMARY KEY (id);
--
-- CREATE TABLE import_crate_downloads as
--   TABLE crate_downloads
--   WITH NO DATA;
-- ALTER TABLE import_crate_downloads
--   ADD CONSTRAINT import_crate_downloads_pkey PRIMARY KEY (crate_id);
-- ALTER TABLE import_crate_downloads
--   ADD CONSTRAINT import_crate_downloads_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES import_crates (id);
--
-- CREATE TABLE import_crate_versions as
--   TABLE crate_versions
--   WITH NO DATA;
-- ALTER TABLE import_crate_versions
--   ADD CONSTRAINT import_crate_versions_pkey PRIMARY KEY (id);
-- ALTER TABLE import_crate_versions
--   ADD CONSTRAINT import_crate_versions_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES import_crates (id);
--
-- CREATE TABLE import_crate_default_versions as
--   TABLE crate_default_versions
--   WITH NO DATA;
-- ALTER TABLE import_crate_default_versions
--   ADD CONSTRAINT import_crate_default_versions_pkey PRIMARY KEY (crate_id);
-- ALTER TABLE import_crate_default_versions
--   ADD CONSTRAINT import_crate_default_versions_crate_id_fkey FOREIGN KEY (crate_id) REFERENCES import_crates (id);
-- ALTER TABLE import_crate_default_versions
--   ADD CONSTRAINT import_crate_default_versions_version_id_fkey FOREIGN KEY (version_id) REFERENCES import_crate_versions (id);
