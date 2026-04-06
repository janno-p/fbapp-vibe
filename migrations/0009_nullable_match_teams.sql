-- Knockout matches are seeded before teams are decided (TBD slots).
-- Team IDs are filled in by the polling job as rounds are confirmed.
ALTER TABLE matches ALTER COLUMN home_team_id DROP NOT NULL;
ALTER TABLE matches ALTER COLUMN away_team_id DROP NOT NULL;
