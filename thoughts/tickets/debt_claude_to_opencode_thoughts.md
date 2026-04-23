---
type: debt
priority: high
created: 2026-04-23
status: reviewed
tags: [claude, opencode, thoughts, migration, agentic]
keywords: claude.md, .claude/tasks, .claude/CLAUDE.md, thoughts/tickets, opencode /init, .opencode/agents, .opencode/commands, Cluster444 agentic plugin, migrate tasks on tickets
patterns: instruction-file migration, task-archive migration, repo-wide reference rewrite, agentic workflow spec, markdown config deprecation
---

# DEBT-001: Migrate Claude-specific repo guidance and task archive to thoughts/opencode conventions

## Description
Remove repo-local Claude-specific guidance files and migrate task tracking into `thoughts/` using an agentic workflow model. The end state should rely on opencode-native conventions where they are needed, with obsolete Claude markdown removed once replacements are in place or confirmed unnecessary.

## Context
The repository currently contains nested `CLAUDE.md` files under `src/`, `tests/`, and `.claude/`, plus a populated `.claude/tasks/` archive with both open and completed work items. The intended direction is to stop maintaining Claude-specific repo-local instructions and move task tracking to `thoughts/` as the primary location, following opencode-style conventions for agents and commands.

## Requirements

### Functional Requirements
- Remove repo-local `CLAUDE.md` files after migrating or replacing any required guidance.
- Convert every file under `.claude/tasks/` one-to-one into a `thoughts/tickets` artifact, including completed history.
- Keep migrated task content summary-level, but make each ticket suitable for agentic execution.
- Update markdown/doc references that still point at Claude-specific paths.
- Preserve the intent of the current task archive while moving to new thoughts-native file names.

### Non-Functional Requirements
- Scope must stay repo-local; do not touch user-level Claude/OpenCode config.
- Do not introduce opencode scaffolding unless it is required to complete the migration.
- The migration should be deterministic and easy to review.
- Final state should have no remaining repo-local `CLAUDE.md` files.

## Current State
- Claude guidance lives in `.claude/CLAUDE.md`, `src/CLAUDE.md`, `src/polling/CLAUDE.md`, `src/modules/*/CLAUDE.md`, and `tests/CLAUDE.md`.
- Task tracking lives under `.claude/tasks/` with `open/`, `done/`, and `TEMPLATE.md`.
- No `thoughts/` or `.opencode/` directories currently exist in the repo.

## Desired State
- Claude-specific repo markdown is removed or replaced with opencode-native alternatives.
- All `.claude/tasks/` entries exist as thoughts tickets under `thoughts/tickets`.
- Repo docs reference the new thoughts/opencode workflow instead of Claude-specific files.
- Any needed agentic task handling follows the Cluster444-style migrate-on-ticket approach.

## Research Context

### Keywords to Search
- `CLAUDE.md` - locate all repo-local instruction files to remove or replace.
- `.claude/tasks` - source task archive to migrate one-to-one.
- `thoughts/tickets` - target location for migrated tickets.
- `opencode /init` - determine the expected native project conventions.
- `.opencode/agents` / `.opencode/commands` - target conventions for agentic workflow structure.
- `Cluster444 agentic plugin` - reference behavior for ticket-driven task migration.

### Patterns to Investigate
- instruction-file migration - how repo guidance is encoded and replaced.
- task archive conversion - how to preserve status/history while renaming structure.
- repo-wide reference rewrites - how to update paths in markdown/docs safely.
- agentic workflow specs - how ticket files should express migrate-on-ticket behavior.
- markdown config deprecation - how to remove obsolete instruction files without losing intent.

### Key Decisions Made
- Migrate to opencode-oriented files instead of preserving Claude-specific repo guidance.
- Convert every `.claude/tasks` file, including historical `done/` entries.
- Use `thoughts/` as the primary location for tickets.
- Keep migration repo-local only.
- Treat the opencode scaffolding itself as out of scope for this ticket.

## Success Criteria

### Automated Verification
- [ ] `rg --files -g 'CLAUDE.md'` returns no repo-local hits.
- [ ] `rg --files thoughts/tickets` shows migrated tickets for all `.claude/tasks` entries.
- [ ] `rg -n "\.claude/tasks|CLAUDE.md" docs src tests .claude` returns only intentional historical references, if any.

### Manual Verification
- [ ] Moved tasks retain their original intent in the new thoughts ticket structure.
- [ ] Repo-local Claude guidance is fully removed or replaced.
- [ ] New ticket names and layout follow thoughts conventions.

## Related Information
- `.claude/CLAUDE.md`
- `src/CLAUDE.md`
- `src/polling/CLAUDE.md`
- `src/modules/auth/CLAUDE.md`
- `src/modules/admin/CLAUDE.md`
- `src/modules/predictions/CLAUDE.md`
- `src/modules/standings/CLAUDE.md`
- `tests/CLAUDE.md`
- `.claude/tasks/TEMPLATE.md`
- `.claude/tasks/open/*`
- `.claude/tasks/done/*`

## Notes
`thoughts/` and `.opencode/` do not currently exist, so the migration will likely need to create the new ticket tree and verify the target conventions before deleting legacy files.
