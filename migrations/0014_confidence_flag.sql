-- Confidence multiplier flag on group stage predictions.
-- Implements cavekit-scoring.md R9 / cavekit-predictions.md confidence feature.
ALTER TABLE group_stage_predictions
    ADD COLUMN is_confident BOOLEAN NOT NULL DEFAULT FALSE;
