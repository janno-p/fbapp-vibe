CREATE TABLE group_stage_predictions (
    id                BIGSERIAL PRIMARY KEY,
    user_id           BIGINT NOT NULL REFERENCES users(id),
    match_id          BIGINT NOT NULL REFERENCES matches(id),
    predicted_outcome match_outcome NOT NULL,
    points_awarded    INT,
    UNIQUE (user_id, match_id)
);

CREATE TABLE knockout_predictions (
    id            BIGSERIAL PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id),
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    round         knockout_round NOT NULL,
    team_id       BIGINT NOT NULL REFERENCES teams(id),
    points_awarded INT,
    UNIQUE (user_id, tournament_id, round, team_id)
);

CREATE TABLE top_scorer_predictions (
    id            BIGSERIAL PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id),
    tournament_id BIGINT NOT NULL REFERENCES tournaments(id),
    player_id     BIGINT NOT NULL REFERENCES players(id),
    points_awarded INT,
    UNIQUE (user_id, tournament_id, player_id)
);
