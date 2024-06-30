CREATE TABLE users (
  id            serial PRIMARY KEY,
  name          varchar NOT NULL UNIQUE,
  password_hash varchar NOT NULL
);


CREATE TABLE crates (
  id          serial PRIMARY KEY,
  name        varchar     NOT NULL,
  updated_at  timestamptz NOT NULL,
  created_at  timestamptz NOT NULL,
  description varchar     NOT NULL,
  homepage    varchar     NULL,
  readme      varchar     NULL,
  repository  varchar     NULL
);

CREATE TABLE crate_downloads (
  crate_id  integer NOT NULL REFERENCES crates (id),
  downloads bigint  NOT NULL,
  PRIMARY KEY (crate_id)
);

CREATE TABLE crate_versions (
  id       serial PRIMARY KEY,
  crate_id integer NOT NULL REFERENCES crates (id),
  version  varchar NOT NULL
);

CREATE TABLE crate_default_versions (
  crate_id   integer NOT NULL REFERENCES crates (id),
  version_id integer NOT NULL REFERENCES crate_versions (id),
  PRIMARY KEY (crate_id)
);

CREATE TABLE import_crates_metadata (
  id          serial PRIMARY KEY,
  imported_at timestamptz NOT NULL
);


CREATE TABLE favorite_crates (
  user_id  integer NOT NULL REFERENCES users (id),
  crate_id integer NOT NULL REFERENCES crates (id),
  PRIMARY KEY (user_id, crate_id)
);
