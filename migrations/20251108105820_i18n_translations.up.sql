CREATE TABLE IF NOT EXISTS i18n_translations (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,
    locale TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(key, locale)
);
