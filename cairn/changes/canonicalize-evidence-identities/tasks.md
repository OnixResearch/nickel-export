# Tasks: Canonicalize evidence identities

## Phase 1: Specify canonical bytes

- [ ] [serial] M1: Define versioned receipt and manifest domains, field order, length encoding, list counts, and enum tags [r[nickel_export.core.canonical_evidence_encoding]]
- [ ] [serial] M2: Define checked conversion and schema-migration rules [r[nickel_export.core.canonical_evidence_encoding]]

## Phase 2: Implement pure encoders

- [ ] [serial] I1: Add no-std canonical receipt and manifest byte encoders over admitted values [r[nickel_export.core.canonical_evidence_encoding]]
- [ ] [serial] I2: Move BLAKE3 identity computation from Serde JSON to canonical bytes [r[nickel_export.core.canonical_evidence_encoding]]
- [ ] [serial] I3: Regenerate schemas, fixtures, and migration projections [r[nickel_export.core.canonical_evidence_encoding]]

## Phase 3: Verify

- [ ] [serial] V1: Add independent positive known-answer and negative mutation/truncation vectors [r[nickel_export.core.canonical_evidence_encoding]]
- [ ] [serial] V2: Run Rust, Wasm, Cairn, compatibility, and Nix validation and archive the change [r[nickel_export.core.canonical_evidence_encoding]]
