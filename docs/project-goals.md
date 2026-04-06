# Project Goals & Feature Specification

## Purpose

A prediction game for major international football tournaments — UEFA European Championships and FIFA World Cups. A fixed group of friends competes by predicting tournament outcomes before competition begins. Multiple separate leagues are supported; predictions are made once and count across all leagues a user belongs to.

## Users & Access

- **Authentication:** Google OAuth (existing)
- **Leagues:** Created by admins; users join via invite link. A user may belong to multiple leagues simultaneously.
- **Admin role:** Granted per user; admins manage the active tournament and league membership.

## Tournament Management

- One active tournament at a time, registered and activated by an admin.
- Tournament fixture data (teams, groups, matches, players) is pulled from an external API (football-data.org or equivalent free source).
- Real-time match result updates via background polling of the API.
- Prediction editing is open until the tournament kick-off; locked once the competition starts.

## Predictions

Users make the following predictions before the tournament starts:

| Category | What is predicted | Notes |
|---|---|---|
| Group stage | Winner of each match | Home / draw / away |
| Knockout rounds | Which teams advance in each round | R32, R16, QF, SF, Final, Winner |
| Top scorer | 3 player candidates | Points awarded if any is the actual top scorer |

## Scoring

| Prediction | Points |
|---|---|
| Correct group stage match result | 1 pt |
| Correct Round of 32 qualifier | 2 pt per team |
| Correct Round of 16 qualifier | 3 pt per team |
| Correct Quarter-final qualifier | 4 pt per team |
| Correct Semi-final qualifier | 6 pt per team |
| Correct Finalist | 8 pt per team |
| Correct Tournament winner | 10 pt |
| Top scorer in your 3 picks | 5 pt + goals scored by that player |

## In-Tournament Display

Default view: the match nearest in time (most recently finished or next upcoming).

### Leaderboard
- Current points standings per league
- Per-match breakdown showing who gained points from which match

### Future Prospects
- Maximum achievable points for each participant given remaining fixtures
- Scenario modeling: "if teams X and Y win, standings change to…"

### Comparison
- Side-by-side view of two users' predictions and points

## Real-Time Updates

Match results flow in via a background process polling the football API. Results trigger:
- Score updates
- Point recalculation for all affected predictions
- Standings refresh
