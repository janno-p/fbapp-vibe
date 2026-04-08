## Status

✅ Accepted

## Date

2026-04-08

## Context

The application uses **Axum** (Rust) + **HTMX** + **Askama** templates for server-driven HTML rendering (ADR-0003, ADR-0004). JavaScript has been minimal — only two templates contain inline scripts (~40 lines total) handling tab navigation, player filtering, checkbox enforcement, and HTMX response feedback.

Upcoming tasks introduce client-side interactivity that does not belong in HTMX (which is for server round-trips):

| Task | Feature | JS Scope |
|------|---------|----------|
| ⏱️ **0029** | Per-match countdown timer | `setInterval` updating `[data-kickoff-utc]` elements every second |
| 🔒 **0033** | Prediction revision window | Show/hide form state based on match deadline |
| 🎚️ **0034** | Confidence multiplier | Interactive slider/toggle on group stage predictions |
| 🏆 **0035** | Achievement badges | Animated badge reveal on unlock |

This growth in local UI state logic raises the question: continue with ad-hoc vanilla `<script>` blocks, or adopt a lightweight library?

**Key constraints:**

- 🦀 **No build pipeline for JS**: No bundler, no TypeScript compilation step. All assets are static files served from `assets/`. Any library must be a single-file drop-in.
- 📦 **npm is already in use**: `package.json` manages `htmx.org` and `@tailwindcss/cli`. A `make js` target copies distribution files from `node_modules/` to `assets/js/`. Any new library should follow this pattern.
- 🧠 **Rust-first team**: JavaScript expertise is not assumed. The chosen approach should minimise the JS surface area and learning curve.
- 🔄 **ADR-0003 anticipates this**: "Alpine.js may be introduced alongside HTMX for purely local UI interactions that do not require a server round-trip."

**Alternatives considered:**

| Approach | Size (gz) | npm package | Build needed | HTMX synergy | Verdict |
|----------|-----------|-------------|-------------|--------------|---------|
| 🧠 **Vanilla inline `<script>`** | 0 | — | No | Manual | Good for trivial cases; grows unmaintainable |
| 🏔️ **Alpine.js** | ~15 KB | `alpinejs` | No | Excellent | ✅ Recommended |
| 🎮 **Stimulus** | ~30 KB | `@hotwired/stimulus` | Yes | Good | Build step breaks the no-bundler constraint |
| 🟢 **Petite-Vue** | ~6 KB | `petite-vue` | No | Moderate | Less HTMX ecosystem synergy; smaller community |
| 🔤 **Hyperscript** | ~9 KB | `hyperscript.org` | No | Native | Experimental; niche; steep syntax learning curve |

## Decision

🏔️ Adopt **Alpine.js** for client-side interactivity, complementing HTMX for local UI state that does not require a server round-trip.

Install via npm (consistent with the existing htmx.org and tailwindcss pattern) and copy the distribution file with `make js`.

## Rationale

1. 📦 **Consistent with existing tooling**: `alpinejs` is added to `devDependencies` in `package.json`, version-pinned and reproducible. `make js` copies `node_modules/alpinejs/dist/cdn.min.js` to `assets/js/alpine.js` — exactly the same pattern used for HTMX.

2. 🏔️ **Declarative HTML-first syntax**: Alpine's attribute API (`x-data`, `x-show`, `x-model`, `x-on`, `x-init`) keeps logic close to the markup it affects. This aligns with the server-rendered template philosophy and avoids scattered imperative `<script>` blocks.

3. 🔄 **Natural HTMX complement**: Alpine owns local UI state (show/hide, timers, form toggles); HTMX owns server communication (form posts, partial swaps). They coexist without conflict and are the de-facto standard pairing in the HTMX community.

4. 🧪 **Low learning curve**: Declarative attribute syntax is approachable for developers unfamiliar with JavaScript frameworks. The Alpine docs are concise; onboarding takes hours, not days.

5. 🎯 **Covers all near-term tasks**:
   - **0029 countdown**: `x-data` + `x-init` + `setInterval` — replaces a standalone vanilla timer script
   - **0033 revision window**: `x-show` bound to a deadline-computed property
   - **0034 confidence slider**: `x-model` + `@change` for checkbox enforcement
   - **0035 badge animations**: `x-show` + `x-transition` for CSS-driven reveal

6. 📉 **Replaces inline script blocks**: The existing ~40 lines of vanilla JS in `templates/predictions/index.html` (tab switching, player filter, checkbox max-3) map directly to Alpine directives, reducing template noise and improving maintainability.

## Trade-offs and Risks ⚠️

| Trade-off | Mitigation |
|-----------|-----------|
| 🧠 **New library to learn** | Alpine docs are concise; the team can ramp up in 1–2 code reviews. Patterns are documented in the first template using it. |
| 📄 **Markup gets busier with `x-*` attributes** | Expected trade-off for interactive elements. Net improvement over scattered `<script>` blocks and `document.querySelectorAll` calls. |
| 🔮 **Magic attribute behaviour** | Mitigated by keeping `x-data` objects small and well-commented. Complex logic lives in named functions, not inline expressions. |
| 📦 **15 KB added to every page load** | Acceptable for a friend-group app. Alpine is deferred (`defer` attribute) and does not block rendering. |
| 🚀 **Does not enable complex SPA-style UIs** | Correct by design. If a future feature requires that, it warrants a new ADR. Alpine is the right scope for this application. |

## Consequences

- ✅ Add `"alpinejs": "^3.14"` to `devDependencies` in `package.json`.
- ✅ Extend the `make js` target to copy `node_modules/alpinejs/dist/cdn.min.js` to `assets/js/alpine.js`.
- ✅ Add `<script src="/assets/js/alpine.js" defer></script>` to `templates/layout/base.html` (after HTMX, before `</body>`).
- ✅ New interactive features (task 0029+) use Alpine `x-data` components rather than inline `<script>` blocks.
- ✅ Existing inline scripts in `templates/predictions/index.html` are migrated to Alpine directives as part of the relevant tasks (not a standalone migration pass).
- ℹ️ Vanilla JS utility functions (one-liners, event helpers) may remain inline where Alpine would add more ceremony than it removes.
- ℹ️ `assets/js/alpine.js` is committed to the repository (same as `assets/js/htmx.js`) — it is a build output of `make js`, not a source file.
