# Design: Atomic artifact materialization

## Context

Individual rename operations can be atomic within one filesystem, but the output and manifest may live at separate paths. The correctness goal is therefore crash consistency and fail-closed observability, not an impossible universal two-path atomicity claim.

## Dependencies

This change is largely independent, but should consume validated manifest bytes from `validate-evidence-types` and canonical rendering from `canonicalize-evidence-identities` when available.

## Decisions

### 1. Serialize writers and checkers

**Choice:** Use an exclusively created repository lock containing bounded diagnostic metadata. Check and write both require the lock.

**Rationale:** A checker observes one stable generation instead of racing a publisher.

### 2. Stage before mutation

**Choice:** Render output and manifest completely, create same-directory temporary files exclusively, write and `sync_all` them, then sync parent directories.

**Rationale:** Rename publication never exposes partially written files.

### 3. Use a fail-closed transaction marker

**Choice:** Durably publish a transaction marker before the first rename and remove it only after both renames and directory syncs succeed. Check mode rejects any extant marker.

**Rationale:** A crash between independent renames remains visible and cannot be mistaken for a complete generation.

### 4. Support deterministic recovery

**Choice:** Recovery inspects marker state and staged artifact identities, then either completes the exact recorded transaction or leaves existing files untouched with a structured error.

**Rationale:** Recovery must not guess which bytes belong together.

### 5. Offer stronger generation-pointer mode

**Choice:** Consumers able to read through a pointer may stage a complete generation directory and atomically switch one pointer.

**Rationale:** This provides true pairwise publication without imposing it on existing plain-file consumers.

## Risks / Trade-offs

- Lock portability and stale-lock handling need explicit ownership rules without clocks as authority.
- Directory durability semantics vary by platform.
- Transaction metadata adds repository files during interrupted operations.

## Validation Plan

- Inject failure before and after every write, sync, rename, and marker transition.
- Prove checks fail while a transaction is incomplete.
- Run concurrent writer/checker tests and stale-lock negative cases.
- Verify successful recovery is idempotent.
