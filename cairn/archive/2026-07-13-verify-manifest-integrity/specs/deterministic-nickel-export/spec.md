# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Stored manifests support self-contained integrity verification

r[nickel_export.core.manifest_integrity_verification]
The core MUST provide pure verification that recomputes every derivable canonical identity and validates schema, path, hash, non-claim, evaluator-cohort, ordering, and uniqueness invariants without invoking Nickel. Optional artifact-byte verification MUST require the exact bytes being checked.

#### Scenario: Stored manifest is internally coherent

- GIVEN a supported manifest whose canonical fields and identities are valid
- WHEN integrity verification runs
- THEN it returns a verified manifest and explicitly makes no freshness or semantic-correctness claim.

#### Scenario: Identity or invariant is tampered

- GIVEN a manifest contains a changed identity, malformed BLAKE3 value, duplicate output, mixed evaluator, unsafe path, or weakened non-claim
- WHEN integrity verification runs
- THEN verification fails with deterministic staged diagnostics.

#### Scenario: Artifact bytes are supplied

- GIVEN exact source, dependency, or output bytes are supplied for a manifest artifact
- WHEN optional byte verification runs
- THEN the verifier recomputes and compares that artifact identity and byte length.

#### Scenario: Read-only CLI verification runs

- GIVEN a manifest path and optional artifact paths
- WHEN `nickel-export verify` runs
- THEN it performs no Nickel execution and no filesystem writes.
