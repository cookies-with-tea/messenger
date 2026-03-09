CREATE TYPE user_role as ENUM ('user', 'admin');

CREATE TYPE user_status as ENUM ('active', 'inactive', 'in_moderation');

CREATE TABLE IF NOT EXISTS guest_user
(
    uuid        UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    first_name  TEXT             NOT NULL DEFAULT '',
    second_name TEXT             NOT NULL DEFAULT '',
    last_name   TEXT             NOT NULL DEFAULT '',
    phone       TEXT NOT NULL DEFAULT '',
    email       TEXT NOT NULL DEFAULT '',
    birth_date  DATE,
    password_hash    TEXT             NOT NULL DEFAULT '',
    avatar TEXT NOT NULL DEFAULT '',
    street TEXT NOT NULL DEFAULT '',
    gender TEXT NOT NULL DEFAULT '',
    city TEXT NOT NULL DEFAULT '',
    role        user_role  NOT NULL DEFAULT 'user',
    status      user_status       NOT NULL DEFAULT 'active',
    created_at  TIMESTAMP        NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP        NOT NULL DEFAULT NOW()
);
