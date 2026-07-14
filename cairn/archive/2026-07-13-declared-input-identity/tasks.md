# Tasks: Declared input identity

## Phase 1: Define the identity boundary

- [x] [serial] S1: Specify included inputs, deliberate exclusions, closure non-claims, and schema versioning [r[nickel_export.core.declared_input_identity]]

## Phase 2: Implement the pure core and wire format

- [x] [serial] I1: Add the pre-output BLAKE3 identity API and canonical length-delimited encoding [r[nickel_export.core.declared_input_identity]]
- [x] [serial] I2: Carry the identity in canonical receipt and manifest v2 while preserving compatibility projections [r[nickel_export.core.declared_input_identity]]
- [x] [serial] I3: Add positive invariance tests and negative change-sensitivity and malformed-material tests [r[nickel_export.core.declared_input_identity]]

## Phase 3: Update artifacts and explanations

- [x] [serial] D1: Regenerate fixtures and release profiles and document the identity, migration, examples, and cache non-claim [r[nickel_export.core.declared_input_identity]]

## Phase 4: Verify and archive

- [x] [serial] V1: Run focused Rust, no-std/Wasm, Cairn lifecycle, CLI freshness, and full Nix validation [r[nickel_export.core.declared_input_identity]]
- [x] [serial] V2: Sync the accepted specification and archive the completed change with updated release evidence [r[nickel_export.core.declared_input_identity]]
