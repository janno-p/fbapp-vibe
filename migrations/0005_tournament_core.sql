CREATE TABLE tournaments (
    id                    BIGSERIAL PRIMARY KEY,
    external_id           TEXT NOT NULL UNIQUE,
    name                  TEXT NOT NULL,
    season                TEXT NOT NULL,
    is_active             BOOLEAN NOT NULL DEFAULT FALSE,
    predictions_locked_at TIMESTAMPTZ,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX one_active_tournament ON tournaments (is_active) WHERE is_active = TRUE;

CREATE TABLE teams (
    id            BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    external_id   TEXT NOT NULL,
    name          TEXT NOT NULL,
    short_name    TEXT NOT NULL,
    crest_url     TEXT,
    UNIQUE (tournament_id, external_id)
);

CREATE TABLE groups (
    id            BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    name          TEXT NOT NULL,
    UNIQUE (tournament_id, name)
);

CREATE TABLE group_memberships (
    group_id BIGINT NOT NULL REFERENCES groups(id),
    team_id  BIGINT NOT NULL REFERENCES teams(id),
    PRIMARY KEY (group_id, team_id)
);

CREATE TABLE players (
    id            BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    external_id   TEXT NOT NULL,
    name          TEXT NOT NULL,
    team_id       BIGINT NOT NULL REFERENCES teams(id),
    goals_scored  INT NOT NULL DEFAULT 0,
    UNIQUE (tournament_id, external_id)
);

CREATE TYPE match_outcome AS ENUM ('home', 'draw', 'away');
CREATE TYPE knockout_round AS ENUM ('r16', 'qf', 'sf', 'final', 'winner');

CREATE TABLE matches (
    id            BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    external_id   TEXT NOT NULL,
    group_id      BIGINT REFERENCES groups(id),
    round         knockout_round,
    home_team_id  BIGINT NOT NULL REFERENCES teams(id),
    away_team_id  BIGINT NOT NULL REFERENCES teams(id),
    scheduled_at  TIMESTAMPTZ NOT NULL,
    home_score    INT,
    away_score    INT,
    outcome       match_outcome,
    UNIQUE (tournament_id, external_id),
    CHECK (
        (group_id IS NOT NULL AND round IS NULL) OR
        (group_id IS NULL AND round IS NOT NULL)
    )
);
