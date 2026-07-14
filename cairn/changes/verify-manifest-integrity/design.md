# Design: Verify manifest integrity

## Context

Integrity, freshness, and semantic correctness are different claims. The core can verify internal hashes and invariants without evaluator authority, while freshness additionally requires current exact artifacts and semantic correctness remains consumer-owned.

## Dependencies

Implement after `validate-evidence-types` and `canonicalize-evidence-identities` so verification consumes strict wire values and schema-owned canonical bytes.

## Decisions

### 1. Return a typed verification report

**Choice:** A pure verifier returns either `VerifiedManifest` or sorted structured diagnostics classified by schema, path, identity, cohort, uniqueness, and canonicalization stage.

**Rationale:** Consumers need deterministic failure evidence rather than a boolean.

### 2. Recompute every derivable identity

**Choice:** Recompute declared-input, receipt, and manifest identities; validate artifact identity syntax and byte lengths; validate required non-claims and evaluator consistency.

**Rationale:** Stored identity fields must not be trusted merely because they deserialize.

### 3. Keep optional byte verification explicit

**Choice:** A second pure operation accepts supplied source, dependency, and output bytes and checks them against artifact identities.

**Rationale:** Structural verification cannot prove facts about absent bytes.

### 4. Keep the CLI read-only

**Choice:** `verify` reads evidence and optional artifacts, emits a report, and never writes or invokes Nickel.

**Rationale:** Integrity verification remains a small auditable imperative shell around the pure core.

## Risks / Trade-offs

- Users may confuse integrity with freshness; result schemas and prose must separate them.
- Canonical verification is only as strong as admitted fields and supplied bytes.
- Supporting compatibility projections requires explicit version adapters, not silent coercion.

## Validation Plan

- Positive canonical manifest and optional-byte verification.
- Negative tampered identity, malformed BLAKE3, duplicate output, mixed evaluator, weakened non-claim, missing byte, and byte mismatch cases.
- CLI tests prove no Nickel process or output write occurs.
