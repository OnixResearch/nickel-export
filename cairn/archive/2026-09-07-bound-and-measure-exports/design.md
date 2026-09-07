## Context

The shell snapshots declared files but does not enforce the full evaluator read closure. Current identity checks allocate executable-sized buffers. Replay retains every output. Core artifact verification scans all expected artifacts per supplied file.

## Decisions

### Decision: Reuse bounded process mechanics without transferring authority

**Choice:** Consume the published, immutable Bounded Exec contract for process lifetime and bounded streams. Keep executable admission, identity, redaction, receipt policy, and snapshot meaning in the export shell.

**Rationale:** One reviewed mechanism avoids separate timeout and descendant handling for version and evaluation commands. It does not provide a filesystem sandbox.

### Decision: Retain fewer bytes without weakening exactness

**Choice:** Borrow captured input slices, enforce aggregate byte admission, retain only reference and current replay output, and stream executable hashing. Use a path index that preserves all expected identities for repeated paths.

**Rationale:** These changes remove copies and repeated scans. Exact-byte replay comparison and conflict rejection remain unchanged.

### Decision: Keep unsafe reuse excluded

**Choice:** Preserve the `snapshot_only` non-cacheable boundary. Measure local mechanisms and end-to-end exports. Any future closed-runtime adapter requires a published pinned evaluator contract and separate compatibility evidence.

**Rationale:** Matching declared inputs do not prove that the evaluator read only those inputs. Reduced memory does not prove semantic equivalence.

## Risks / Trade-offs

- Shared process exit classification must preserve existing error stages and fail-closed behavior.
- Aggregate budgets reject requests that fit individual limits but exceed total capacity.
- Proof evidence binds changed source identities and needs regeneration. Correspondence vectors must stay exact.
- Index construction has a fixed cost for small manifests. Measurements determine the useful workload range.
