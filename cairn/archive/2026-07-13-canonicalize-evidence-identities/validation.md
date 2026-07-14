# Validation evidence

- Strict evidence admission supplied a passing workspace, strict Clippy, Wasm, CLI, and Nix baseline.
- Canonical receipt v3 carries `receipt_identity`; canonical manifest v3 uses schema-owned identity bytes.
- `encode_receipt_identity` and `encode_manifest_identity` expose no-std length-delimited bytes with checked big-endian lengths, explicit list counts, stable enum names, and versioned domains.
- Pretty and compact JSON no longer define receipt or manifest cryptographic identity.
- Fixed known-answer BLAKE3 vectors pin receipt and manifest identities for the core fixture.
- A separately implemented test encoder reproduces the exact canonical receipt bytes without calling the production encoder.
- Positive identity, independent-byte, JSON round-trip, and negative source/dependency/output/tamper vectors pass.
- Formatting, workspace tests, strict Clippy, Wasm no-std, and real CLI end-to-end checks pass.
