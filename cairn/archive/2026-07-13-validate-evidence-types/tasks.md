# Tasks: Validate evidence types

## Phase 1: Define type-state boundaries

- [x] [serial] M1: Add wire DTOs and opaque admitted receipt and verified manifest types [r[nickel_export.core.admitted_evidence_types]]
- [x] [serial] M2: Define one pure invariant-admission pipeline and read-only accessors [r[nickel_export.core.admitted_evidence_types]]

## Phase 2: Enforce strict serialization

- [x] [serial] I1: Reject unknown fields and unsupported versions at every canonical nesting layer [r[nickel_export.core.admitted_evidence_types]]
- [x] [serial] I2: Restrict manifest construction and compatibility projections to admitted values [r[nickel_export.core.admitted_evidence_types]]
- [x] [serial] I3: Migrate shell and compatibility adapters to explicit admission [r[nickel_export.core.admitted_evidence_types]]

## Phase 3: Verify

- [x] [serial] V1: Add positive round-trip and negative unknown-field, mutation, identity, schema, and non-claim tests [r[nickel_export.core.admitted_evidence_types]]
- [x] [serial] V2: Run Rust, Cairn, compatibility, and Nix validation and archive the change [r[nickel_export.core.admitted_evidence_types]]
