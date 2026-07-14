# Tasks: Atomic artifact materialization

## Phase 1: Define transaction states

- [ ] [serial] M1: Add pure transaction-plan, marker, recovery, and lock-decision types [r[nickel_export.shell.atomic_materialization]]
- [ ] [serial] M2: Define portable crash-consistency claims and stronger pointer-mode claims [r[nickel_export.shell.atomic_materialization]]

## Phase 2: Implement the shell protocol

- [ ] [serial] I1: Add exclusive repository locking for write and check modes [r[nickel_export.shell.atomic_materialization]]
- [ ] [serial] I2: Stage, sync, rename, and directory-sync output and manifest files [r[nickel_export.shell.atomic_materialization]]
- [ ] [serial] I3: Add durable transaction markers, deterministic recovery, and optional generation-pointer mode [r[nickel_export.shell.atomic_materialization]]

## Phase 3: Verify

- [ ] [serial] V1: Add fault-injection, concurrent access, stale-lock, and recovery idempotence tests [r[nickel_export.shell.atomic_materialization]]
- [ ] [serial] V2: Run Rust, Cairn, CLI, and Nix validation and archive the change [r[nickel_export.shell.atomic_materialization]]
