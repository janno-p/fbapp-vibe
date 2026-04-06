CREATE TABLE leagues (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT NOT NULL,
    invite_token TEXT NOT NULL UNIQUE,
    created_by   BIGINT NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE league_members (
    league_id BIGINT NOT NULL REFERENCES leagues(id),
    user_id   BIGINT NOT NULL REFERENCES users(id),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (league_id, user_id)
);
