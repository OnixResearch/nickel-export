# Consumer migration and rollback

<!-- r[impl nickel_export.migration.dual_run] -->

## Publication prerequisite

Publish this repository at an immutable revision before changing consumer dependency manifests. Consumers must pin the repository URL and exact Git revision; workspace-relative paths are development conveniences only and are not a release identity.

## Dual-run cutover

For each consumer family:

1. Inventory the current request, evaluator, dependencies, selector, contract, destination, manifest/receipt shape, and downstream policy checks.
2. Add a thin consumer adapter from the existing local types to `nickel-export-core` canonical types. Do not move evaluator semantics, filesystem authority, destination authority, product policy, or release gates.
3. Run the legacy implementation and canonical core over the same exact source, dependencies, evaluator observation, and output.
4. Compare canonical exact-byte identities and the appropriate Octet or Mantle compatibility projection. Classify any difference before proceeding; do not accept unexplained drift.
5. Keep both paths required for at least one full consumer validation cycle. Record the pinned standalone revision and comparison evidence.
6. Switch the canonical standalone result to the primary path while retaining the legacy projection and a rollback feature or adapter.
7. Remove duplicated local core logic only after downstream checked manifests, receipts, Nix checks, and Cairn gates all pass.

## Canonical v2 identity boundary

Canonical receipt and manifest v2 add `declared_input_identity`. Consumers may
use it to correlate repeated evaluations or detect differing outputs for the
same declared inputs. They must not use a `declared_only` or `snapshot_only`
identity as a trusted cache key. Octet and Mantle v1 projections remain
unchanged and intentionally
do not gain stronger closure or caching claims.

## Consumer-specific ownership

- Octet retains candidate-family admission, evidence roles, checked destinations, and release gates.
- Mantle retains embedded evaluator behavior, sandbox/root authority, writes, and build/release evidence.
- Cairn retains policy parsing, lifecycle semantics, gate authority, and generated policy ownership.
- Trellis retains proof-policy semantics, verification gates, and proof-corpus claims.
- Animus retains profile contracts, generated destinations, runtime policy, and agent-harness gates.

## Rollback

If dual-run output or receipt comparison diverges:

1. keep the legacy path authoritative;
2. preserve both outputs and exact identities as bounded diagnostic evidence;
3. revert only the consumer dependency/adapter change, not the standalone revision history;
4. classify the difference as request normalization, evaluator cohort, dependency closure, selector/contract handling, serialization, or consumer policy;
5. add a positive and negative regression fixture in the owning repository before retrying.

Rollback never authorizes a local path override, mixed evaluator manifest, stale checked-in artifact, or weakened gate.
