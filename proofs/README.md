# Identity primitive proofs

`identity_primitives.rs` contains project-owned Verus models for the narrow,
pure obligations behind Nickel export identity handling:

- Fixed-width big-endian `u64` encoding is injective.
- A bounded count-prefixed sequence of length-delimited fields is injective before hashing.
- Accepted component-model paths are relative, portable, nonempty, and free of parent traversal.
- Path normalization is idempotent.
- Successful receipt and manifest model constructors preserve their declared invariants.

The proof uses Octet's pinned production Verus package. The Nix rail reruns the
proof, requires the deliberately false proof fixture to fail, checks the proof
with pinned `verusfmt`, validates the Nickel-authored evidence contract, and
recomputes every recorded BLAKE3 source identity:

```console
nix build .#checks.x86_64-linux.identity-proofs --no-link -L
```

The Rust core test `proof_correspondence_vectors_match_rust_primitives` checks `correspondence-vectors.json`.

The checked `generated/evidence.json` binds the vectors, Verus source, negative fixture, and Rust implementation.

This evidence is an auditable correspondence argument. It is not a formal refinement proof between the model and Rust.

## Claim boundary

These proofs do not establish BLAKE3 collision impossibility, Nickel evaluator
correctness or determinism, filesystem confinement or atomicity, verifier or
solver soundness, automatic Rust/model equivalence, whole-system correctness,
or release eligibility.

Trellis supplied proof patterns only. Nickel Export does not import or transfer a Trellis proof claim.
