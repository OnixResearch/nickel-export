# Design: Confine evaluator inputs

## Context

`execute` reads exact material before `run_evaluator`, but the evaluation plan still points Nickel at repository files. Nickel additionally supports ambient `NICKEL_IMPORT_PATH`, package manifests, and a platform-selected package cache. The existing `DeclaredOnly` policy correctly avoids a closure claim but does not prevent a captured-input/evaluated-input mismatch.

## Dependencies

This is the first shell-hardening change. `bind-evaluator-execution`, `bound-evaluator-execution`, and `replay-evaluation-determinism` build on its captured plan.

## Decisions

### 1. Separate capture from execution

**Choice:** Build a pure captured-input plan containing normalized paths and exact bytes, then let the shell materialize it in a private snapshot.

**Rationale:** Receipt admission and subprocess execution consume one captured value instead of independently rereading mutable repository state.

### 2. Preserve normalized relative paths

**Choice:** Recreate source and dependency paths under the snapshot root and rewrite import and contract arguments to that root.

**Rationale:** Relative Nickel imports keep their reviewed shape while undeclared relative files are absent.

### 3. Remove ambient import authority

**Choice:** Clear the child environment and add only an explicit allowlist. Force deterministic diagnostic options and ensure `NICKEL_IMPORT_PATH` is absent.

**Rationale:** Ambient search paths otherwise act as unrecorded evaluator inputs.

### 4. Fail closed for packages

**Choice:** Reject package evaluation by default. A future supported package mode must declare the lock manifest and immutable package material and use an explicit cache location.

**Rationale:** Platform package caches are not exact declared inputs.

### 5. Keep confinement claims tiered

**Choice:** Distinguish snapshot-only execution from execution inside a filesystem sandbox that denies reads outside the snapshot.

**Rationale:** A private tree prevents ordinary undeclared relative imports but does not by itself prevent absolute-path reads.

## Risks / Trade-offs

- Snapshot creation adds filesystem work and cleanup obligations.
- Some existing configurations may rely on ambient imports and must declare them.
- Portable snapshot-only mode remains weaker than Nix or Mantle sandbox confinement.

## Validation Plan

- Mutate repository inputs after capture and prove evaluation uses the snapshot.
- Set an ambient `NICKEL_IMPORT_PATH` containing a hidden import and prove it is ignored.
- Reject undeclared package and absolute escape attempts without issuing receipts.
- Verify cleanup on evaluator success, failure, and timeout.
