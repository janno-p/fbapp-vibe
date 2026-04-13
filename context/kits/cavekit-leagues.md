---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit: Leagues & Membership

## Scope

League creation by admins, invite-token-based membership, league overview pages with member lists, and membership tracking.

## Requirements

### R1: League Creation
**Description:** Admin users can create leagues for organizing groups of predictors.

**Acceptance Criteria:**
- [ ] Admin visits `/admin/leagues` and sees list of existing leagues
- [ ] Admin submits league creation form with: name (string, unique)
- [ ] POST `/admin/leagues` creates new league record in database
- [ ] Newly created league has: id, name, created_at
- [ ] Admin is redirected to `/admin` after successful creation
- [ ] League creation is idempotent (same name can be submitted twice without error, returns existing league or rejects duplicate based on schema constraint)

**Dependencies:** cavekit-auth (AdminUser extractor)

### R2: Invite Token Generation
**Description:** Each league has a unique invite token for sharing membership invitations.

**Acceptance Criteria:**
- [ ] Each league has a generated invite token (random string, 20+ characters)
- [ ] Token is persistent and does not regenerate
- [ ] Invite token is visible only to league creator/admin users (not public)
- [ ] GET `/leagues/{id}` league overview page shows invite token to eligible users
- [ ] Token format is URL-safe (alphanumeric)

**Dependencies:** R1 (League Creation)

### R3: Token-Based League Joining
**Description:** Users can join a league by visiting a shareable invite link.

**Acceptance Criteria:**
- [ ] Invite link is: GET `/leagues/join/{token}`
- [ ] Valid token automatically adds user to league membership
- [ ] User is a member if a row exists in `league_members(league_id, user_id)`
- [ ] Joining is idempotent: requesting same token twice does not duplicate membership
- [ ] User is redirected to league overview page (`/leagues/{id}`) after joining
- [ ] Invalid or expired tokens return 404 Not Found (or similar error)
- [ ] Any authenticated user can join via token (no admin check needed)

**Dependencies:** R2 (Invite Token), cavekit-auth (AuthSession)

### R4: League Overview Page
**Description:** Members can view league details and see who else is in the league.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}` renders league overview page (members-only)
- [ ] League overview shows: league name, list of members (with user names and avatars)
- [ ] Invite token is displayed prominently (creator/admin only)
- [ ] Page is accessible only to users who are members of the league (401/403 enforced)
- [ ] Page displays "You are not a member of this league" error if user is not a member
- [ ] Member list is sorted consistently (by name or join date)

**Dependencies:** R3 (Token-Based Joining), cavekit-auth (AuthSession)

### R5: Membership Tracking
**Description:** League membership is persisted in the database and used for access control.

**Acceptance Criteria:**
- [ ] `league_members` table exists with: id, league_id, user_id, joined_at (timestamps)
- [ ] (league_id, user_id) has a unique constraint (prevents duplicate memberships)
- [ ] Joining a league inserts a row into `league_members` (see R3)
- [ ] League overview and standings pages query `league_members` to check access
- [ ] Leaderboard and standings pages show points only for users in that league

**Dependencies:** R1 (League Creation), R3 (Token-Based Joining)

### R6: List Leagues on Admin Dashboard
**Description:** Admins can see all leagues on the admin dashboard.

**Acceptance Criteria:**
- [ ] GET `/admin/leagues` lists all leagues (admin-only)
- [ ] List shows: league name, member count, creation date
- [ ] List is paginated or reasonably short (no 10k league limit concern)
- [ ] Admins can click to view or edit each league

**Dependencies:** R1 (League Creation), cavekit-auth (AdminUser)

## Out of Scope

- League deletion
- League member removal or banning
- League privacy settings (public/private/invite-only)
- Recurring or multi-tournament leagues
- Custom league rules or scoring overrides
- League messages or chat
- League administration transfer or multiple admins per league
- Invite token expiration (token is permanent for the league's lifetime)
- Invite limits or rate limiting on joins
- Email-based invitations

## Source Traceability

### Brownfield Status: Complete
All acceptance criteria are satisfied by existing code.

### Source Files
- `src/modules/leagues/mod.rs` — router() with league routes
- `src/modules/leagues/handlers.rs` — create_league, list_leagues, view_league, join_league handlers
- `src/modules/leagues/db.rs` — league CRUD, membership insert, token generation
- `src/modules/leagues/models.rs` — League, LeagueMember types
- `migrations/0006_leagues.sql` — leagues and league_members tables

### Implementation Notes
- Invite token generated using `uuid::Uuid::new_v4()` or similar random generator
- Token is stored directly in the league record; no separate token table
- Membership check is done via `league_members(league_id, user_id)` query before rendering protected pages
- Joining is idempotent: upsert or ignore-on-conflict pattern used in DB insert

## Cross-References
- Depends on: **cavekit-auth.md** (AuthSession, AdminUser extractor, user context)
- Consumed by: **cavekit-standings.md** (league membership for leaderboard isolation)
- Related to: **cavekit-predictions.md** (predictions are per-league in display)
