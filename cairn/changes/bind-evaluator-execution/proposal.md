# Proposal: Bind evaluator execution

## Summary

Bind receipts to the resolved evaluator artifact and one canonical execution plan instead of trusting caller-provided labels and separately reconstructed option strings.

## Motivation

The shell currently runs the evaluator by a caller-provided path twice, accepts a caller-provided identity, and checks only that `--version` output contains an expected token. The receipt options are generated separately from the actual process arguments, and generic adapters can silently sort and deduplicate options whose repetition or order may matter.

## Scope

- Resolve one evaluator executable path before version and export execution.
- Compute and record its BLAKE3 artifact identity.
- Accept a stronger Nix or Mantle closure identity when an adapter can verify it.
- Derive process arguments and receipt semantics from one canonical typed plan.
- Replace untyped option normalization with typed fields and reject ambiguous duplicates.
- Treat human-readable evaluator labels and versions as metadata rather than artifact proof.

## Non-Goals

- Proving dynamic-library or evaluator semantic equivalence from an executable hash alone.
- Reimplementing Nix closure computation in the evaluator-neutral core.
- Caching evaluator outputs.

## Impact

- **Schemas**: evaluator descriptors gain exact artifact and optional closure identities.
- **Shell**: executable resolution, hashing, and plan construction change.
- **Compatibility**: legacy projections retain bounded labels while canonical receipts carry stronger evidence.
