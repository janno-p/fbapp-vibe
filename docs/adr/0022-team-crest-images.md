# ADR-0022: Use API-Provided Team Crest Images 🛡️

## Status

✅ Accepted

## Date

2026-04-09

## Context

Displaying a team's visual identity alongside its name went through two prior approaches before the current one:

| Approach | ADR | Outcome |
|----------|-----|---------|
| 🖼️ Custom SVG flag assets per team | _(initial spec)_ | Rejected — high authoring cost, ongoing maintenance burden |
| 🏳 Unicode flag emoji via `src/flags.rs` | [ADR-0019](0019-unicode-emoji-country-flags.md) | Superseded — emoji rendering varies by OS; no support for club crests |
| 🛡️ **API-provided crest images** (current) | **ADR-0022** | Accepted |

The football-data.org API (see [ADR-0018](0018-football-api.md)) returns a `crest` URL for every team it exposes. These are official, high-quality SVG/PNG assets hosted on the API's CDN, covering both national teams and club sides. Task 0036 implemented this approach; this ADR documents the decision retroactively.

## Decision

🛡️ Serve team crest images directly from the URLs provided by the Football Data API, stored in `teams.crest_url`. Fall back to the local placeholder `/assets/default.svg` when the column is `NULL`.

## Rationale

| Concern | Unicode emoji | API crest images |
|---------|---------------|-----------------|
| 🎨 **Visual quality** | OS-dependent, varies widely | Official club/nation artwork |
| 🏟️ **Club tournaments** | ✗ TLAs are for nations only | ✓ Works for any team the API knows |
| 📁 **Asset maintenance** | Zero (OS renders) | Zero (API serves) |
| 🔄 **Always up-to-date** | ✓ | ✓ — API CDN reflects changes |
| 🌐 **Coverage** | Nations only | All teams in the API |
| 📦 **Bundle size** | Zero | Zero (external URLs) |
| ♿ **Accessibility** | Native emoji semantics | `<img alt="team name">` |

The primary motivation for moving away from emoji was **club tournament support**: club sides do not have country TLAs, so `flags.rs` could not produce meaningful output for them. API crests work uniformly across both national and club competitions.

## Implementation

### Database

- `teams.crest_url TEXT` column — populated by `upsert_team` in `src/modules/admin/db.rs` from the API response's `crest` field.
- May be `NULL` for teams that the API does not provide a crest URL for.

### Rust

- `src/crests.rs` — replaces the deleted `src/flags.rs`:
  ```rust
  const DEFAULT_CREST_URL: &str = "/assets/default.svg";

  pub fn find_crest_url(crest_url: Option<&str>) -> String {
      crest_url.unwrap_or(DEFAULT_CREST_URL).to_string()
  }
  ```
- Called in `db.rs` query mapping closures to produce `home_crest_url` / `away_crest_url` / `crest_url` fields on view-model structs.

### Templates

Crests are rendered via a CSS custom property with a circular mask:

```html
<div class="mask-radial-[circle_at_center,...] w-8 h-8 bg-cover bg-center shrink-0"
     style="--crest-url: url('{{ m.home_crest_url }}'); background-image: var(--crest-url)">
</div>
```

The circular mask means the crest asset only needs to be centred — aspect ratio and exact dimensions are handled by CSS.

### Fallback asset

`assets/default.svg` — a 40×40 neutral placeholder shown when `crest_url` is `NULL`. It uses the app's pitch colour as the background with a muted shield outline, matching the visual weight of real crests when cropped to a circle.

## Trade-offs and Risks ⚠️

| Trade-off | Mitigation |
|-----------|-----------|
| 🌐 **External image dependency** | API CDN must be reachable; broken crests show the fallback SVG gracefully |
| 📴 **No offline support** | Acceptable for a web app that requires a live database connection anyway |
| 🎨 **Style varies by team** | Official crests differ in shape, colour, and style — the circular mask normalises them visually |
| 🔗 **URL stability** | football-data.org CDN URLs have been stable; no SLA guarantee, but low observed churn |

## Consequences

- ✅ `src/crests.rs` is the authoritative crest-resolution module; `src/flags.rs` has been deleted.
- ✅ `teams.crest_url` column is populated at seeding time from the Football Data API.
- ✅ `assets/default.svg` provides a graceful fallback for teams without a crest URL.
- ✅ Club tournaments are supported without any additional implementation.
- ⛔ [ADR-0019](0019-unicode-emoji-country-flags.md) (Unicode emoji) is superseded.
- ℹ️ Crest images are loaded by the browser directly from the API CDN — no server-side proxying.
