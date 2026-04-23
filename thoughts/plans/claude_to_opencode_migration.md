# Claude to OpenCode Migration Implementation Plan

## Overview

Migrate repo-local Claude guidance and task tracking to thoughts/opencode conventions. The end state removes `CLAUDE.md` files from the repo, moves every `.claude/tasks/*` entry into `thoughts/tickets`, and updates repo docs so the new workflow is the source of truth.

## Current State Analysis

The repo currently has multiple Claude-specific instruction files and a task archive rooted in `.claude/`.

Key evidence:
- `.claude/CLAUDE.md:119-139` defines the canonical task lifecycle (`open` → `in-progress` → `done`) and spec-first task creation.
- `src/CLAUDE.md:1-39` contains repo guidance that overlaps with architecture docs and task references.
- `README.md:191` still documents `.claude/tasks/` as the AI-assisted development surface.
- `AGENTS.md:10,15` still instructs contributors to read `CLAUDE.md` and use `.claude/tasks`.
- `docs/architecture-structure-issues.md:20,78` references `src/CLAUDE.md` and explicitly notes that `thoughts/` did not exist at the time of writing.

The repo has no existing `thoughts/` or `.opencode/` directory, so the migration needs to create the target tree and establish the new convention in-place.

## Desired End State

After this migration:
- No repo-local `CLAUDE.md` files remain.
- Every `.claude/tasks/*` file exists as a corresponding thoughts ticket under `thoughts/tickets`.
- Repo docs point at `thoughts/` and opencode-oriented workflow notes instead of Claude-specific paths.
- The migration is traceable from old task/archive names to new thoughts ticket names.

Verification:
- `rg --files -g 'CLAUDE.md'` returns no repo-local paths.
- `rg --files thoughts/tickets` lists a migrated ticket for every former `.claude/tasks/*` file.
- `rg -n "\.claude/tasks|CLAUDE.md" README.md AGENTS.md docs src tests .claude` shows only intentional historical references, if any.

### Key Discoveries
- `src/CLAUDE.md:1-39` is a repo guidance file, not executable code.
- `.claude/tasks/TEMPLATE.md:1-67` is the canonical task template and needs a thoughts equivalent.
- `docs/architecture-structure-issues.md:19-33` already explains the current module organization, so `src/CLAUDE.md` is redundant as architecture guidance.
- `README.md:185-191` and `AGENTS.md:10,15` are the main outward-facing references that must change for the workflow shift to be believable.

## What We're NOT Doing

- Not touching user-level Claude/OpenCode config outside the repo.
- Not creating full `.opencode/` scaffolding unless the migration proves it is required.
- Not redesigning the underlying task model beyond moving it into `thoughts/`.
- Not rewriting code behavior, only docs, task files, and workflow references.

## Implementation Approach

Use a staged migration so references are updated before legacy files disappear. First, inventory and map every Claude/task file to its thoughts replacement. Second, migrate the task archive into `thoughts/tickets` with new names while preserving content and history. Third, update repo docs and any markdown references to the new workflow. Finally, delete the obsolete Claude files and verify the repo has no remaining local `CLAUDE.md` files.

## Phase 1: Inventory and Mapping

### Overview

Build a complete source-to-target map for the migration so nothing is missed.

### Changes Required:

#### 1. Claude instruction inventory
**Files**: `.claude/CLAUDE.md`, `src/CLAUDE.md`, `src/polling/CLAUDE.md`, `src/modules/*/CLAUDE.md`, `tests/CLAUDE.md`
**Changes**: enumerate each file, determine whether it is pure guidance or overlaps with another canonical doc, and map its replacement target in `thoughts/` or another repo doc.

#### 2. Task archive inventory
**Files**: `.claude/tasks/TEMPLATE.md`, `.claude/tasks/open/*`, `.claude/tasks/done/*`
**Changes**: list every task file to migrate, preserving source status and IDs in a mapping table for traceability.

#### 3. Reference inventory
**Files**: `README.md`, `AGENTS.md`, `docs/architecture-structure-issues.md`, any other markdown under `docs/`, `src/`, `tests/`
**Changes**: identify every textual reference to `CLAUDE.md`, `.claude/tasks`, or related instructions that must be rewritten.

### Success Criteria:

#### Automated Verification:
- [x] `rg --files -g 'CLAUDE.md'` enumerates all repo-local Claude guidance files.
- [x] `rg --files .claude/tasks` enumerates the full task archive.
- [x] `rg -n "CLAUDE.md|\.claude/tasks|thoughts/|opencode" README.md AGENTS.md docs src tests .claude` shows all affected references.

#### Manual Verification:
- [x] The source-to-target mapping covers every Claude file and task file in the repo.
- [x] Each file has a clear replacement or deletion decision.

---

## Phase 2: Migrate Tasks into Thoughts

### Overview

Convert the entire `.claude/tasks` archive into `thoughts/tickets` using a new thoughts-native naming scheme.

### Changes Required:

#### 1. Ticket template translation
**Files**: `.claude/tasks/TEMPLATE.md` → new `thoughts` template/ticket pattern
**Changes**: translate the current task schema into the thoughts ticket format, preserving the parts that matter for execution while dropping Claude-specific framing.

#### 2. Open task migration
**Files**: `.claude/tasks/open/*`
**Changes**: create corresponding `thoughts/tickets/*` entries for every open task, with new names and equivalent task content.

#### 3. Done task migration
**Files**: `.claude/tasks/done/*`
**Changes**: create corresponding `thoughts/tickets/*` entries for every completed task so the historical archive is preserved in the new location.

#### 4. Agentic workflow notes
**Files**: migrated thoughts tickets
**Changes**: embed the migrate-on-ticket / agentic workflow expectations in the ticket content so downstream agents know to operate from thoughts instead of Claude task files.

### Success Criteria:

#### Automated Verification:
- [x] `rg --files thoughts/tickets` contains one migrated file for every `.claude/tasks/*` source file.
- [x] Migrated tickets use the new thoughts naming scheme.
- [x] Ticket content retains the original task intent and completion state.

#### Manual Verification:
- [x] Open tasks still read as actionable work items.
- [x] Done tasks still preserve their outcome/history context.
- [x] The new task archive is usable without referencing `.claude/tasks`.

---

## Phase 3: Update Repo Docs and References

### Overview

Rewrite repo docs so the new thoughts workflow is discoverable and the Claude workflow is no longer advertised.

### Changes Required:

#### 1. README workflow section
**File**: `README.md`
**Changes**: replace the `.claude/tasks/` mention with the thoughts-based task workflow and update any adjacent developer guidance.

#### 2. AGENTS guidance
**File**: `AGENTS.md`
**Changes**: remove instructions that require reading `CLAUDE.md` and replace task-workflow guidance with thoughts/opencode-oriented references.

#### 3. Historical research note cleanup
**File**: `docs/architecture-structure-issues.md`
**Changes**: update the references to `src/CLAUDE.md` and the note about `thoughts/` not existing so the document reflects the migration without losing its research context.

#### 4. In-repo path rewrites
**Files**: any markdown under `docs/`, `src/`, `tests/`, `.claude/`
**Changes**: rewrite obvious path references to the new thoughts workflow where they are part of active guidance rather than historical record.

### Success Criteria:

#### Automated Verification:
- [x] `rg -n "\.claude/tasks|CLAUDE.md" README.md AGENTS.md docs src tests .claude` returns only intentional historical references.
- [x] No doc still instructs the reader to use repo-local Claude task files as the active workflow.

#### Manual Verification:
- [x] A new contributor can discover the thoughts workflow from the repo docs.
- [x] The architecture research note still makes sense after its reference cleanup.

---

## Phase 4: Remove Legacy Claude Files and Verify

### Overview

Delete the obsolete Claude instruction files once the new thoughts references are in place, then prove the repo is clean.

### Changes Required:

#### 1. Remove Claude instruction files
**Files**: `.claude/CLAUDE.md`, `src/CLAUDE.md`, `src/polling/CLAUDE.md`, `src/modules/*/CLAUDE.md`, `tests/CLAUDE.md`
**Changes**: delete the legacy repo-local guidance files after their useful content has been migrated or made redundant.

#### 2. Final migration verification
**Files**: repo-wide
**Changes**: run repository searches to confirm there are no remaining repo-local `CLAUDE.md` files and that task references now point to thoughts.

### Success Criteria:

#### Automated Verification:
- [x] `rg --files -g 'CLAUDE.md'` returns no repo-local paths.
- [x] `rg -n "\.claude/tasks|CLAUDE.md" README.md AGENTS.md docs src tests .claude` shows only intentional historical references, if any.

#### Manual Verification:
- [x] No active workflow depends on `.claude/CLAUDE.md` or `src/CLAUDE.md`.
- [x] The new thoughts archive is the primary task surface in the repo.

## Testing Strategy

### Unit Tests:
- No code changes are expected, so there are no Rust unit tests for the migration itself.
- Validate text transformations and file moves through repository searches and file existence checks.

### Integration Tests:
- Not applicable unless the migration uncovers a hidden code dependency on the deleted guidance files.

### Manual Testing Steps:
1. Open the new `thoughts/tickets` entries and confirm they correspond one-to-one with the old `.claude/tasks` archive.
2. Read `README.md` and `AGENTS.md` to confirm the repo now points at thoughts-based workflow conventions.
3. Run `rg --files -g 'CLAUDE.md'` to verify no repo-local Claude files remain.
4. Run `rg -n "\.claude/tasks|CLAUDE.md" README.md AGENTS.md docs src tests .claude` to confirm only intentional historical references remain.

## Performance Considerations

None. This is a documentation and workflow migration.

## Migration Notes

- Preserve the historical content of completed tasks when moving them into `thoughts/tickets`.
- Use new thoughts-native filenames rather than keeping the old `.claude/tasks` naming scheme.
- Remove legacy files only after the replacement references are in place.

## References

- Original ticket: `thoughts/tickets/debt_claude_to_opencode_thoughts.md`
- Existing Claude guidance: `.claude/CLAUDE.md:1-139`
- Repo guidance overlap: `src/CLAUDE.md:1-39`
- Task workflow references: `AGENTS.md:10,15`, `README.md:191`
- Historical research note: `docs/architecture-structure-issues.md:20,78`
