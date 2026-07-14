# Identity primitive proofs

`identity_primitives.rs` contains project-owned Verus models for the narrow,
pure obligations behind Nickel export identity handling:

- fixed-width big-endian `u64` encoding is injective;
- a bounded count-prefixed sequence of length-delimited fields is injective
  before hashing;
- accepted component-model paths are relative, portable,
  parent-traversal-free, nonempty, and normalization-idempotent;
- successful receipt and manifest model constructors preserve their declared
  schema, non-claim, uniqueness, evaluator-cohort, diagnostic, and
  recomputed-identity invariants.

The proof uses Octet's pinned production Verus package. The Nix rail reruns the
proof, requires the deliberately false proof fixture to fail, checks the proof
with pinned `verusfmt`, validates the Nickel-authored evidence contract, and
recomputes every recorded BLAKE3 source identity:

```console
nix build .#checks.x86_64-linux.identity-proofs --no-link -L
```

`correspondence-vectors.json` is exercised by the ordinary Rust core test
`proof_correspondence_vectors_match_rust_primitives`. The checked
`generated/evidence.json` binds those vectors, the Verus source, the negative
fixture, and the Rust implementation source. This is an auditable reviewed
correspondence argument, not a formal refinement proof between the model and
Rust.

## Claim boundary

These proofs do not establish BLAKE3 collision impossibility, Nickel evaluator
correctness or determinism, filesystem confinement or atomicity, verifier or
solver soundness, automatic Rust/model equivalence, whole-system correctness,
or release eligibility. Trellis supplied proof patterns only; no Trellis proof
claim is imported or transferred.
