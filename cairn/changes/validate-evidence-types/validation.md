# Validation evidence

- The bounded evaluator archive supplied a passing workspace, strict Clippy, Wasm, CLI, and Nix baseline.
- Canonical Serde wire structs reject unknown fields at request, resource-profile, artifact, diagnostic, evaluator, receipt, and manifest layers.
- `build_receipt` returns opaque `AdmittedReceipt`; `build_manifest` and deserialized admission return opaque `VerifiedManifest`.
- Freshness and Octet/Mantle projection APIs accept admitted states rather than mutable wire DTOs.
- Pure admission validates schema, non-claim, normalized paths, BLAKE3 syntax, bounds, diagnostics, evaluator options, declared-input recomputation, manifest sorting, uniqueness, cohorts, and manifest identity.
- Positive wire round trips and negative unknown-field, fabricated-identity, weakened-non-claim, malformed-hash, and tamper tests pass.
- Formatting, workspace tests, strict Clippy, Wasm no-std, and real CLI end-to-end checks pass.
