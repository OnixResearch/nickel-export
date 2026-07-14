# Tasks: Verify manifest integrity

## Phase 1: Define bounded verification claims

- [x] [serial] M1: Add typed integrity reports and deterministic diagnostic classes [r[nickel_export.core.manifest_integrity_verification]]
- [x] [serial] M2: Separate structural, supplied-byte, freshness, and semantic claim boundaries [r[nickel_export.core.manifest_integrity_verification]]

## Phase 2: Implement pure verification

- [x] [serial] I1: Recompute canonical declared-input, receipt, and manifest identities [r[nickel_export.core.manifest_integrity_verification]]
- [x] [serial] I2: Validate schema, hash, path, non-claim, uniqueness, ordering, and evaluator invariants [r[nickel_export.core.manifest_integrity_verification]]
- [x] [serial] I3: Add the read-only `verify` CLI shell and optional exact-byte checks [r[nickel_export.core.manifest_integrity_verification]]

## Phase 3: Verify

- [x] [serial] V1: Add positive and negative core and CLI integrity fixtures [r[nickel_export.core.manifest_integrity_verification]]
- [x] [serial] V2: Run Rust, Cairn, compatibility, and Nix validation and archive the change [r[nickel_export.core.manifest_integrity_verification]]
