-- group_stage_predictions already has UNIQUE(user_id, match_id), which covers
-- per-user prediction lookups.  Add a separate index on match_id so the scoring
-- UPDATE (keyed on match_id) and the breakdown queries don't do full table scans.
CREATE INDEX ON group_stage_predictions (match_id);

-- knockout_predictions has UNIQUE(user_id, tournament_id, round, team_id), covering
-- per-user round lookups.  Add an index on (tournament_id, round) for the scoring
-- UPDATE that filters by tournament and round without a leading user_id.
CREATE INDEX ON knockout_predictions (tournament_id, round);

-- top_scorer_predictions has UNIQUE(user_id, tournament_id, player_id), covering
-- per-user tournament lookups.  Add an index on tournament_id for the scoring
-- UPDATE and the leaderboard CTE that filters by tournament only.
CREATE INDEX ON top_scorer_predictions (tournament_id);

-- league_members PK is (league_id, user_id).  Add a reverse index so listing all
-- leagues for a given user (WHERE user_id = ?) is efficient.
CREATE INDEX ON league_members (user_id);
