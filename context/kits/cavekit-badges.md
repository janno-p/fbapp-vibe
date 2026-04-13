---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit: Achievement Badges

## Scope

An achievement system that awards badges to users based on their tournament performance. Badges are determined by background job after scoring completes, stored in the database, and displayed on member stats and optionally on the leaderboard.

## Requirements

### R1: Badge Types and Definitions
**Description:** System defines at least 5 distinct achievement badge types with associated logic.

**Acceptance Criteria:**
- [ ] At least 5 badge types are defined
- [ ] Each badge has: slug (unique identifier), name, description, icon/emoji
- [ ] Badges are defined as constants or enums in code (not user-editable)
- [ ] Initial badge set includes (at minimum):
  - [ ] `perfect_group_round`: All group stage predictions correct in a single match day
  - [ ] `underdog_caller`: Correctly predicted 3+ away wins (matches where away team won)
  - [ ] `top_scorer`: Finished #1 on leaderboard at end of active tournament
  - [ ] `consistent_predictor`: Group stage accuracy > 70% (correct out of total)
  - [ ] `oracle`: Correctly predicted the tournament winner
- [ ] Each badge definition includes clear logic for awarding (see R3)

**Dependencies:** None (can be defined independently)

### R2: Badge Storage and Retrieval
**Description:** Awarded badges are persisted and can be queried efficiently.

**Acceptance Criteria:**
- [ ] Table `user_achievements` exists with columns: id, user_id, tournament_id, badge_slug, awarded_at (timestamp)
- [ ] Unique constraint on (user_id, tournament_id, badge_slug) prevents duplicate awards
- [ ] Same badge can be awarded in different tournaments (multiple rows with same slug, different tournament_id)
- [ ] Same user can earn multiple badges in one tournament (multiple rows with same user_id/tournament_id, different slug)
- [ ] Query to fetch all badges for a user in a tournament: `SELECT * FROM user_achievements WHERE user_id = ? AND tournament_id = ?`
- [ ] Query to fetch all users with a specific badge: `SELECT DISTINCT user_id FROM user_achievements WHERE badge_slug = ? AND tournament_id = ?`

**Dependencies:** None (database schema)

### R3: Badge Award Job
**Description:** After scoring completes, a background job evaluates all users against badge criteria and awards earned badges.

**Acceptance Criteria:**
- [ ] Job runs after polling/scoring loop completes (in same task or triggered by scoring)
- [ ] Job queries all users in all leagues of the active tournament
- [ ] For each badge type, job evaluates criteria for each user:
  - [ ] `perfect_group_round`: finds match days where user has >0 predictions all correct; awards if found
  - [ ] `underdog_caller`: counts away wins in user's correct group predictions; awards if >= 3
  - [ ] `top_scorer`: queries leaderboard; if user is rank #1 at tournament end, awards badge
  - [ ] `consistent_predictor`: calculates group stage accuracy = correct / total; awards if > 0.70
  - [ ] `oracle`: if user predicted correct tournament winner and it was actually the winner, awards badge
- [ ] Job inserts a row into `user_achievements` for each earned badge
- [ ] Job does not re-award already earned badges (unique constraint prevents duplicates)
- [ ] Job logs awarded badges at info level: "Badge awarded: user_id={}, badge={}, tournament_id={}"
- [ ] Job continues if any single evaluation fails (error handling for each badge type)
- [ ] Job is idempotent: running twice does not duplicate awards

**Dependencies:** R1, R2, cavekit-scoring (prediction and match data)

### R4: Badge Display on Member Stats
**Description:** Member stats page shows earned badges prominently.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/members/{user_id}` renders member stats page with badges section
- [ ] Badges section displays all badges earned in the active tournament
- [ ] Each badge shows: icon/emoji, badge name, short description
- [ ] Badges are displayed in chronological order (awarded_at ASC)
- [ ] If no badges earned, section shows "No badges earned yet"
- [ ] Badges section is visible to all league members (not private/hidden)
- [ ] Completed achievement count shown: "3 / 5 badges earned"

**Dependencies:** R2, cavekit-standings (member stats page)

### R5: Badge Display on Leaderboard (Optional)
**Description:** Main leaderboard optionally shows top badge per user.

**Acceptance Criteria:**
- [ ] Main leaderboard may add optional column: "Top Badge" or "Badge"
- [ ] Column shows the most notable badge earned (if any) using icon/emoji
- [ ] If multiple badges, shows most recent or most rare (implementation choice)
- [ ] Hovering badge shows badge name and description
- [ ] If no badge earned, cell is empty or shows "—"

**Dependencies:** R4, cavekit-standings (main leaderboard)

### R6: Badge Metadata
**Description:** Badge definitions include human-readable metadata for display.

**Acceptance Criteria:**
- [ ] Each badge slug is a string identifier (e.g., "perfect_group_round")
- [ ] Each badge has a display name (e.g., "Perfect Round")
- [ ] Each badge has a short description (e.g., "Predicted all matches in a group stage day correctly")
- [ ] Each badge has an emoji or icon representation
- [ ] Badge metadata is not stored in database (defined in code as constants)

**Dependencies:** R1 (Badge Types)

## Out of Scope

- Loot boxes or randomized badge rewards
- Trading or transferring badges between users
- User-defined custom badges
- Badge rarity levels or tiering
- Badge expiration or seasonal resets
- Badge-specific actions or privileges (badges are cosmetic only)
- Notifications when badges are awarded
- Badge leaderboard or "Rarest Badge" global statistics
- Unlocking additional tournaments via badges
- Social sharing of badges

## Implementation Notes

### Badge Definition Pattern (Rust pseudocode)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeSlug {
    PerfectGroupRound,
    UnderdogCaller,
    TopScorer,
    ConsistentPredictor,
    Oracle,
}

impl BadgeSlug {
    pub fn name(&self) -> &'static str { /* ... */ }
    pub fn description(&self) -> &'static str { /* ... */ }
    pub fn emoji(&self) -> char { /* ... */ }
    pub fn all() -> Vec<Self> { /* ... */ }
}
```

### Criteria Details

**Perfect Group Round:** Match day is a set of all matches played on the same date in group stage. User must have made predictions for all matches in that day and all predictions must be correct. If tournament has no match days (flat schedule), use 1-match threshold or disable this badge.

**Underdog Caller:** Count only away wins (MatchOutcome::Away) where user predicted correctly. Must have >= 3 such correct away predictions.

**Top Scorer:** Query final leaderboard after tournament ends. If user is rank #1, award badge. Must wait until all matches finished.

**Consistent Predictor:** Calculate accuracy as `correct_group_predictions / total_group_predictions`. Must be > 0.70 (70%). Only group stage predictions count.

**Oracle:** Query tournament winner (team that won final match or tournament property). If user has a knockout_prediction with round=Winner and team_id=actual_winner, award badge.

## Source Traceability

### Greenfield Status: New Domain (Task 0035)
This cavekit describes a new achievement system not yet implemented.

### Related Task
- **Task 0035** — Achievement badges — full implementation in one task

### Source Files (To Be Created)
- `src/achievements.rs` — badge definitions, criteria functions, award job
- `migrations/` — sequential migration for user_achievements table and unique constraint
- Template update: `templates/standings/member_stats.html` — badges section
- Optional template update: `templates/standings/leaderboard.html` — top badge column

### Implementation Checklist
- [ ] Create `src/achievements.rs` with BadgeSlug enum and metadata functions
- [ ] Write pure predicate functions for each badge (no DB calls, take prediction data as input)
- [ ] Create migration for `user_achievements` table with unique constraint
- [ ] Integrate award job into polling loop (after scoring completes)
- [ ] Update member stats template to query and display badges
- [ ] Write unit tests for each badge criteria (pure functions)
- [ ] Write integration test for award job (using test data)

## Cross-References
- Depends on: **cavekit-scoring.md** (prediction and match result data)
- Depends on: **cavekit-standings.md** (leaderboard display, member stats template)
- Depends on: cavekit-auth (user context)
- Related to: **cavekit-observability.md** (may log badge awards)
