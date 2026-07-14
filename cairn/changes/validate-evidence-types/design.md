# Design: Validate evidence types

## Context

Serde-derived structs are convenient wire representations but are not proof that evidence invariants hold. Public fields also allow post-construction mutation. The functional core should make invalid admitted states unrepresentable while leaving parsing and rendering explicit.

## Dependencies

This change should land before `canonicalize-evidence-identities` and `verify-manifest-integrity`, which operate on admitted values.

## Decisions

### 1. Split wire values from admitted values

**Choice:** Decode into `ReceiptWire` and `ManifestWire`, then validate into opaque `AdmittedReceipt` and `VerifiedManifest` types.

**Rationale:** Type state makes the admission boundary visible and prevents accidental projection of unchecked values.

### 2. Deny unknown fields at every versioned layer

**Choice:** Apply strict unknown-field rejection to requests, artifacts, evaluator descriptors, diagnostics, receipts, manifests, and compatibility input surfaces.

**Rationale:** Typos and injected fields must not disappear during decoding.

### 3. Centralize invariant validation

**Choice:** Validate schema versions, required non-claims, normalized paths, BLAKE3 syntax, declared-input recomputation, evaluator consistency, sorted uniqueness, duplicate destinations, and mixed cohorts in one pure admission pipeline.

**Rationale:** Constructors, deserialization, and projections should share one invariant definition.

### 4. Keep mutation explicit

**Choice:** Provide accessors and builders that return newly validated values rather than mutable public fields.

**Rationale:** An admitted value must remain admitted for its entire lifetime.

## Risks / Trade-offs

- This is a public Rust API migration and may require canonical schema v3.
- Tests that mutate evidence directly must switch to wire fixtures or negative builders.
- Strict decoding can reject previously ignored forward fields; schema versioning handles intentional evolution.

## Validation Plan

- Positive round-trip tests preserve admitted evidence.
- Negative tests reject unknown fields at every nesting level, unsupported versions, weakened non-claims, malformed hashes, unsafe paths, duplicates, and fabricated declared identities.
- Compile-time API tests ensure projections cannot accept wire values.
