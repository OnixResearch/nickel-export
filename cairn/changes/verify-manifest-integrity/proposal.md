# Proposal: Verify manifest integrity

## Summary

Add pure self-contained receipt and manifest integrity verification plus a read-only CLI command that does not rerun Nickel.

## Motivation

Freshness checking currently derives a new manifest by evaluating Nickel and compares the resulting struct. Consumers also need to determine whether a stored evidence bundle is internally coherent before evaluation, migration, projection, or archival. No single public verifier currently recomputes all canonical identities and structural invariants from stored evidence.

## Scope

- Recompute declared-input, receipt, and manifest identities from admitted fields.
- Validate schema domains, BLAKE3 syntax, non-claims, normalized paths, evaluator cohorts, uniqueness, and ordering.
- Distinguish internal integrity from freshness and evaluator correctness.
- Add `nickel-export verify --manifest ...` as a read-only shell.
- Emit structured verification diagnostics and no success receipt on failure.

## Non-Goals

- Proving that artifact digests match bytes not supplied to the verifier.
- Proving freshness without current input and output bytes.
- Authenticating the issuer or signing evidence.
- Re-evaluating Nickel.

## Impact

- **Core API**: pure integrity verifier over admitted wire evidence and optionally supplied artifact bytes.
- **CLI**: read-only verification command.
- **Consumers**: archives and compatibility adapters gain a common preflight check.
