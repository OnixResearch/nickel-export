# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Only admitted evidence reaches canonical consumers

r[nickel_export.core.admitted_evidence_types]
The core MUST decode untrusted serialized values into non-authoritative wire types, MUST reject unknown fields and invariant violations, and MUST expose opaque admitted receipt and verified manifest types to manifest construction, freshness checks, and compatibility projections.

#### Scenario: Canonical evidence is valid

- GIVEN a supported receipt or manifest with all required invariants
- WHEN pure admission runs
- THEN it produces an opaque admitted value with read-only accessors.

#### Scenario: Unknown nested field is present

- GIVEN a request, evaluator, artifact, receipt, or manifest contains an unknown field
- WHEN strict decoding runs
- THEN decoding fails rather than discarding the field.

#### Scenario: Caller fabricates evidence fields

- GIVEN a wire receipt has a weakened non-claim, malformed identity, unsafe path, or inconsistent declared-input identity
- WHEN admission runs
- THEN no admitted receipt is produced.

#### Scenario: Projection receives unchecked data

- GIVEN an untrusted wire receipt or manifest
- WHEN a compatibility projection is requested
- THEN the public API requires successful admission before projection.
