# Tasks: Verify identity primitives

## Phase 1: Define proof claims

- [ ] [serial] M1: Specify canonical-encoding, path-normalization, and admitted-state lemmas with explicit axioms and non-claims [r[nickel_export.proof.identity_primitives]]
- [ ] [serial] M2: Pin proof toolchain and exact Rust/proof correspondence identities [r[nickel_export.proof.identity_primitives]]

## Phase 2: Implement proofs

- [ ] [serial] I1: Prove canonical encoding injectivity before hashing [r[nickel_export.proof.identity_primitives]]
- [ ] [serial] I2: Prove relative path safety and normalization idempotence [r[nickel_export.proof.identity_primitives]]
- [ ] [serial] I3: Prove admitted receipt and verified manifest invariant preservation [r[nickel_export.proof.identity_primitives]]

## Phase 3: Verify correspondence

- [ ] [serial] V1: Run shared vectors, proof reruns, negative proof fixtures, and correspondence receipt checks [r[nickel_export.proof.identity_primitives]]
- [ ] [serial] V2: Run Octet/Cairn/release validation and archive the bounded proof evidence [r[nickel_export.proof.identity_primitives]]
