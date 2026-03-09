CREATE TABLE IF NOT EXISTS refresh_token (
    user_id UUID REFERENCES public.guest_user(uuid) ON DELETE CASCADE,
    token TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL
);
