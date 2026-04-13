# Dead Ends

## DE-001: ck:task-builder worktree isolation auto-cleanup

**Observed**: Subagent branches created via `isolation: "worktree"` commit changes but Claude Code auto-cleans the worktree and branch before the parent thread can run `git merge`. Tests written by subagents are lost silently.

**Symptom**: Task reported COMPLETE by subagent, impl tracking updated, but `git log` shows no new commits from the wave.

**Occurred**: Sessions 1 and 2 of auth build loop.

**Fix**: Write all tests directly in the main thread. Do not delegate with `isolation: "worktree"` for this build site.
