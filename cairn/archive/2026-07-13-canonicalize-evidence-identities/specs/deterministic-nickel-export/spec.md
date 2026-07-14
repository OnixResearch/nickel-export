# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Evidence identities use schema-owned canonical bytes

r[nickel_export.core.canonical_evidence_encoding]
The core MUST encode admitted receipt and manifest identity preimages with a versioned, length-delimited, schema-owned binary encoding, MUST hash those bytes with BLAKE3, and MUST expose the canonical bytes for independent verification. Human-facing JSON serialization MUST NOT define cryptographic identity.

#### Scenario: JSON renderer changes

- GIVEN one admitted receipt or manifest
- WHEN JSON whitespace, escaping, or serializer implementation changes without changing admitted fields
- THEN its canonical identity remains unchanged.

#### Scenario: Canonical field changes

- GIVEN one canonical evidence value
- WHEN any identity-bearing field, enum tag, list count, or list element changes
- THEN its canonical bytes and BLAKE3 identity change.

#### Scenario: Known-answer vector is checked

- GIVEN a checked versioned vector containing structured fields, canonical bytes, and expected BLAKE3 identity
- WHEN the core and independent verifier evaluate it
- THEN both produce the exact checked bytes and identity.

#### Scenario: Length cannot be represented

- GIVEN a field or list length exceeds the canonical integer representation
- WHEN encoding runs
- THEN encoding fails rather than saturating or truncating the length.
