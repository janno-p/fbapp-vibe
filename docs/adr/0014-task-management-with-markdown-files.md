# ADR-0014: Task Management with Markdown Files 📋

## Status

✅ Accepted

## Date

2026-04-05

## Context

The project uses AI agents (Claude) extensively for implementation work. Tasks must be:

- 📋 **Auditable**: the history of what was planned, who did it, and what was actually built must be recoverable from the git log.
- 🤖 **AI-ready**: task files must contain enough context for Claude to begin work without clarifying questions.
- 🔍 **Discoverable**: developers and agents must be able to find open, in-progress, and completed work at a glance.
- 📐 **Spec-first**: tasks are written as specifications before implementation begins, not as post-hoc notes.

Several strategies were evaluated:

| Strategy | Audit quality | AI input simplicity | Overhead |
|----------|--------------|--------------------|---------| 
| **Task files + status directories** | ✅ Excellent | ✅ One file per session | Medium |
| Single board file (`TASKS.md`) | ⚠️ Basic | ✅ Always-on | Very low |
| Spec-first files | ✅ Excellent | ✅ Self-contained | High |
| Epics → tasks hierarchy | ✅ Excellent | ⚠️ Two files to pass | High |

## Decision

We will manage tasks as **individual markdown files** 📋 with YAML frontmatter, organised into status subdirectories under `tasks/`. Each file combines a task specification with enough context for Claude to implement it without clarification.

## Directory Structure

```
tasks/
├── TEMPLATE.md          # Canonical template for new task files
├── open/                # Ready to be picked up
├── in-progress/         # Currently being worked on
└── done/                # Completed or cancelled
```

## File Naming Convention

```
{id}-{short-slug}.md
```

Examples:
- `0001-user-registration.md`
- `0002-login-page.md`
- `0023-password-reset-email.md`

IDs are sequential and never reused. The slug is lowercase, hyphen-separated, and describes the task in 3–5 words.

## Task Lifecycle

```
open  ──►  in-progress  ──►  done
           │
           └──►  open   (if blocked or abandoned)
```

- A file moves between directories as its status changes.
- The git history of a file (via `git log --follow`) provides a full audit trail: when it was created, when work started, and when it was completed.
- Cancelled tasks move to `done/` with `status: cancelled` in frontmatter.

## Rationale

1. 🤖 **One file = one Claude session context**: Passing a single task file to Claude at the start of a session gives it goal, acceptance criteria, relevant files, constraints, and ADR references — everything needed to implement without back-and-forth.

2. 📋 **Spec-first forces clarity**: Writing acceptance criteria and context before implementation surfaces ambiguity early, produces better results from Claude, and creates a permanent record of original intent.

3. 🔍 **Directory-as-status is trivially queryable**: `ls tasks/open/` shows all open work. No tooling required. Works in any shell, CI script, or file browser.

4. 📜 **Git history is the audit log**: `git log --follow tasks/done/0001-user-registration.md` shows the full lifecycle. `git blame` shows who last touched each field. No external tracking system required.

5. 🔗 **ADR references tie decisions to work**: Each task file references the ADRs that govern its implementation, ensuring Claude applies the correct architectural constraints.

## Using Task Files with Claude

At the start of a session, provide the task file as context:

```
Here is the task I would like you to implement: @tasks/open/0001-user-registration.md
```

Claude reads the file and has everything it needs: goal, acceptance criteria, relevant source files, ADR constraints, and implementation notes.

After completion, update the task file's `## Outcome` section and move it to `tasks/done/`.

## Trade-offs and Risks ⚠️

- 📁 **File proliferation**: Long-running projects accumulate many files in `tasks/done/`. This is intentional — the done directory is an archive, not a working set. It can be browsed via git log if it grows large.
- 🔄 **Manual status updates**: Moving files between directories is a manual step. There is no automated enforcement that a file in `in-progress/` is actually being worked on. Discipline and code review are the guardrails.
- 🤖 **Task quality determines Claude output quality**: A vague task file produces vague implementation. The template enforces structure, but the content must be written thoughtfully.

## Consequences

- 📁 All work items are tracked as markdown files under `tasks/`.
- 📐 Tasks are written as specifications before implementation begins — no retroactive task creation.
- 🤖 Claude sessions for implementation work begin by passing the relevant task file as context.
- ✅ Completed tasks are moved to `tasks/done/` with the `## Outcome` section filled in before the implementing PR is merged.
- 🔗 Task files reference relevant ADRs, source file paths, and sibling tasks where applicable.
- 📋 The `tasks/TEMPLATE.md` file is the canonical reference for task file format and must be kept up to date if the format evolves.
