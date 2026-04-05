# ADR-0003: Use HTMX for Client-Server Integration 🔄

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application needs a strategy for client-server interaction — how the browser communicates with the Axum backend and how the UI stays dynamic without full page reloads.

The main architectural approaches considered:

| Approach | Description | Examples |
|----------|-------------|---------|
| **SPA framework** 🏗️ | JavaScript framework manages client state, communicates via JSON API | React, Vue, Svelte |
| **HTMX** 🔄 | HTML-first, server renders fragments, minimal JavaScript | HTMX |
| **Vanilla JS / Alpine.js** 🧩 | Light JS sprinkled on server-rendered pages | Alpine.js, Stimulus |
| **Full SSR with no JS** 📄 | Traditional form submits, full page reloads | Plain HTML forms |

Key requirements that informed this decision:

- 🎯 **Simplicity**: Avoid a separate frontend build pipeline (npm, bundlers, TypeScript compilation) if it doesn't add proportional value.
- 🦀 **Backend-driven UI**: The Axum backend is the source of truth for application state; the UI should reflect server state without a complex client-side state management layer.
- ⚡ **Dynamic interactions**: The UI needs partial page updates, form submissions without full reloads, and real-time-ish feedback — but does not require complex client-side routing or offline support.
- 🔧 **Maintainability**: A single-language (Rust) team should be able to own the full stack without deep JavaScript expertise.

## Decision

We will use **HTMX** 🔄 for client-server integration.

## Rationale

1. 🏗️ **No separate frontend build pipeline**: HTMX is a single JS file included via a `<script>` tag. There is no npm, no bundler, no TypeScript compilation step, and no `node_modules`. The entire build pipeline remains `cargo`.

2. 🦀 **Keeps logic in Rust**: With HTMX, UI logic lives in Axum handlers that return HTML fragments. Business logic, validation, and state management all stay on the server in Rust, avoiding duplication across a Rust backend and a JS frontend.

3. 🔄 **Partial page updates without JavaScript**: HTMX attributes (`hx-get`, `hx-post`, `hx-swap`, `hx-target`) enable dynamic interactions — lazy loading, inline form submission, live search, optimistic UI patterns — declaratively in HTML, without writing JavaScript.

4. 📉 **Reduced accidental complexity**: SPA frameworks introduce client-side routing, state management (Redux, Zustand, etc.), API contract maintenance, and serialization layers. HTMX eliminates this entire category of complexity for applications that don't require offline capability or native-app-like interactivity.

5. 🧪 **Simpler testing**: Server responses are HTML fragments testable with standard HTTP assertions. There is no need for a browser automation layer to test most interactions.

6. 🌐 **Progressive enhancement**: HTMX degrades gracefully — pages remain functional with JavaScript disabled for core flows, improving accessibility and resilience.

## Trade-offs and Risks ⚠️

- 🧠 **Unfamiliar paradigm**: Developers experienced with React/Vue may find the server-driven HTML model unfamiliar. The HTMX documentation and "Hypermedia Systems" book provide good onboarding material.
- 🚫 **Not suitable for all UI patterns**: Highly interactive, stateful UIs (rich text editors, drag-and-drop canvases, real-time collaborative features) are harder to build with HTMX and may require targeted JavaScript or a dedicated component. Alpine.js can be introduced alongside HTMX for local interactivity without abandoning the overall approach.
- 📡 **Chatty server communication**: Every interaction requires a round-trip to the server. This is acceptable given the application's expected latency profile but would need revisiting if offline support becomes a requirement.
- 📦 **Templating dependency**: HTMX requires the server to render HTML fragments, which means the Axum backend must use a templating engine. This is a coupled decision (addressed separately in ADR-0004).

## Consequences

- 🔄 All dynamic UI interactions are implemented as HTMX attributes on HTML elements triggering Axum endpoints.
- 📄 Axum handlers return either full HTML pages or partial HTML fragments depending on whether the request is a full navigation or an HTMX request (detectable via the `HX-Request` header).
- 🚫 No SPA framework (React, Vue, Svelte, etc.) is introduced unless a specific component has a documented justification in a future ADR.
- 🧩 Alpine.js may be introduced alongside HTMX for purely local UI interactions (toggling visibility, client-side validation feedback) that do not require a server round-trip.
- 📦 A server-side templating engine is required and will be chosen in ADR-0004.
- 🧪 Integration tests assert on rendered HTML content returned by Axum handlers.
