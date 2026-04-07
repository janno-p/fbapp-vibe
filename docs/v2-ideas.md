# v2 Ideas

Brainstormed directions for a second version of the app. These are intentionally ambitious — rethinks of what the game *is*, not incremental feature additions.

---

## Rethink the core mechanic

### Prediction markets instead of forms
Give each user a budget of 100 "chips" per tournament. Allocate them across outcomes rather than picking one. Odds shift dynamically as more users pile on the same side. A contrarian who allocates 30 chips to an upset earns 6× if it lands. Changes the game from trivia to strategy.

### Last man standing mode
Parallel game within a league: each round, pick one team to win. If they lose, you're out. Last person standing wins the season trophy. Completely different risk profile — do you pick safe all tournament or gamble on an upset while others play safe?

### Blind round
One designated match day per tournament where you submit without seeing your current rank or anyone else's picks (locked behind a reveal). Eliminates copycat strategy and makes rank swings more dramatic.

### Prediction revision at a cost
Instead of a free revision window, let users revise any time but changing a pick *costs 1 point*. Real strategic tension between gut instinct vs. reacting to late team news.

---

## Rethink social dynamics

### Trash talk wall per match
Each match has a pre/post comment thread. Pre-match trash talk is preserved and shown *after* the result. Public humiliation as a feature.

### The Oracle — AI league member
An auto-added bot that always makes statistically optimal picks based on betting exchange implied probabilities. Every league competes against the market. Gives a meaningful reference point: "you beat the odds."

### Prediction sharing card
Generate a shareable image of your tournament bracket (like Wordle output) for WhatsApp/social media. Single biggest lever for organic growth — every share is an acquisition funnel.

### Dynasties and season history
Multi-tournament persistence: trophies on user profiles, head-to-head win/loss records across tournaments, a league hall of fame. Transforms a one-off game into a long-running rivalry.

---

## Rethink timing

### Live in-match predictions
During a match, an HTMX-polled pop-up opens with a short window (5 minutes): "Next goal before minute 75? First goalscorer?" Scored instantly. Turns passive spectating into active participation.

### Penalty shootout window
When a knockout match goes to penalties, a timed form appears with a 3-minute window to pick the winner. First-to-correct gets a bonus point. Creates a shared live moment across the league.

---

## Rethink information display

### Rank trajectory chart
A line chart showing each user's rank position after every match day — not just the final standings. Shows who peaked early, who imploded in the knockouts, who is on a run. This is the story of the tournament.

### Calibration report
End-of-tournament personal report card: group stage accuracy, how confident picks performed vs. unconfident ones, which teams you consistently over/under-rated. Pure data storytelling.

### Prediction bracket heatmap
A visual tournament bracket where each cell is colored by consensus accuracy — "the whole league got this wrong." Surfaces the moments the community was collectively fooled.

---

## Rethink scope

### Non-FIFA tournaments
Support Champions League, Copa América, domestic cups — not just one big tournament per cycle. Keeps the app alive year-round. Needs more flexible tournament configuration in admin.

### Public leagues with global leaderboard
Allow leagues to be discoverable. A global rank ("you're in the top 12% of all players this tournament") turns private friend groups into part of something larger.

### White-label / multi-tenant
The app already has invite-token leagues and admin tooling. A simple tenant layer (custom league name, logo, subdomain) makes it deployable for media companies or corporate competitions as a standalone product.

---

## Highest-leverage bets

1. **Prediction sharing card** — acquisition multiplier; every share is a funnel
2. **The Oracle bot** — immediately adds meaning to every existing league with zero user setup
3. **Rank trajectory chart** — turns accumulated data into narrative drama
