# ADR-0006: Use Tailwind CSS for Styling 🎨

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application uses Askama for server-side HTML templating (ADR-0004) and HTMX for dynamic interactions (ADR-0003). A CSS strategy must be chosen that works well with server-rendered, fragment-based HTML.

The candidates evaluated:

| | **Tailwind CSS** | **Pico CSS** | **Bootstrap** | **Plain CSS** |
|--|----------------|-------------|--------------|--------------|
| Approach | Utility-first classes | Classless / semantic HTML | Component classes | Hand-written |
| Bundle size | Tiny (purged at build) | ~10KB | ~30KB+ | You control it |
| Build step 🔧 | Yes — Tailwind CLI | None | None | None |
| Design system | Composable utilities | Minimal opinionated defaults | Opinionated components | None |
| HTMX community fit | ⭐ Most popular pairing | Popular for small apps | Less common | Common |
| Customisability 🎨 | Very high | Low | Medium | Full |
| Maintenance burden | Low | Very low | Low | High |

Key requirements:

- 🎨 **Flexibility**: The UI should be fully customisable without fighting framework defaults or overriding opinionated component styles.
- 📦 **Minimal shipped CSS**: Only styles actually used in templates should be included in the production bundle.
- 🔄 **Fragment compatibility**: Styles must work correctly on partial HTML fragments returned by HTMX requests, not just full page renders.
- 🛠️ **Simple build integration**: The styling pipeline should fit within the existing `cargo`-based workflow without requiring a full Node.js toolchain.

## Decision

We will use **Tailwind CSS** 🎨 for styling.

## Rationale

1. 🔄 **Natural fit with fragment-based rendering**: Tailwind's utility classes live directly on HTML elements. When HTMX swaps a fragment into the DOM, all styles are self-contained in that fragment — no separate stylesheet coordination is needed.

2. 📦 **Zero unused CSS in production**: Tailwind's build step scans template files and emits only the classes actually used. The resulting CSS bundle is typically a few kilobytes, with no bloat from unused components or utilities.

3. 🎨 **No framework defaults to fight**: Unlike Bootstrap or Pico CSS, Tailwind ships no pre-styled components. Every visual decision is explicit and intentional, making it easy to build a custom design without overriding opinionated defaults.

4. 🛠️ **Standalone CLI — no Node.js required**: The Tailwind CLI is distributed as a single self-contained binary. It can watch Askama template files and regenerate CSS on change without npm, `node_modules`, or a JavaScript runtime in the development or CI environment.

5. ⭐ **Dominant choice in the HTMX community**: Tailwind is the most widely used CSS approach in the HTMX and server-rendered Rust ecosystem, meaning examples, components, and community resources are abundant.

6. 🧩 **Composable design tokens**: Tailwind's configuration file (`tailwind.config.js`) centralises the design system — colours, spacing, typography, breakpoints — making it easy to enforce visual consistency across templates.

## Trade-offs and Risks ⚠️

- 🔧 **Additional build step**: Tailwind requires running its CLI alongside `cargo build`/`cargo watch`. This adds a process to manage during development, mitigated by running both watchers concurrently (e.g. via `cargo-make` or a simple `Makefile`).
- 📄 **Verbose HTML**: Utility-first CSS results in longer `class` attributes in templates. This is a well-known Tailwind trade-off; it is mitigated by Askama's template inheritance and reusable partial templates keeping repetition low.
- 🧠 **Tailwind-specific knowledge**: Developers unfamiliar with utility-first CSS face a short learning curve. Tailwind's documentation and the Tailwind IntelliSense IDE extension significantly reduce this friction.
- 📁 **Config file in project root**: Tailwind requires a `tailwind.config.js` (or `.ts`) file, introducing a single JavaScript config file into an otherwise pure Rust project.

## Consequences

- 🎨 All styling is done via Tailwind CSS utility classes in Askama templates; custom CSS is written only for styles not achievable with Tailwind utilities.
- 📁 A `tailwind.config.js` is maintained at the project root, configured to scan `templates/**/*.html` and `src/**/*.rs` for class names.
- 📦 The Tailwind CLI binary is committed to the repository or pinned in a tooling manifest to ensure reproducible builds.
- 🖥️ The compiled CSS output is written to `assets/css/main.css` and served as a static file by Axum.
- 🔧 Development uses two concurrent watchers: `cargo watch` for Rust/templates and `tailwindcss --watch` for CSS — coordinated via a `Makefile` or `cargo-make` task.
- 🚀 The CI pipeline runs the Tailwind CLI build step before `cargo build` to ensure the CSS asset is present during compilation and testing.
