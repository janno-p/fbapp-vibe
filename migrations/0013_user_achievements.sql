-- Achievement badges awarded to users per tournament.
-- Implements cavekit-badges.md R2.
CREATE TABLE user_achievements (
    id           BIGSERIAL PRIMARY KEY,
    user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    badge_slug   TEXT NOT NULL,
    awarded_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Prevents duplicate awards: same badge once per user per tournament.
CREATE UNIQUE INDEX user_achievements_unique
    ON user_achievements (user_id, tournament_id, badge_slug);
