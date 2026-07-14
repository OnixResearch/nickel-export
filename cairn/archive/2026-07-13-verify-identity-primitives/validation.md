# Validation evidence

## Completion contract

The change is complete only when project-owned Verus source proves the bounded
pre-hash encoding, path, and admitted-state model obligations; the pinned proof
reruns from Nix; a deliberately false neighboring proof fails; exact proof,
vector, fixture, and Rust source identities are checked; correspondence vectors
pass in ordinary Rust; and the evidence keeps cryptographic, evaluator,
filesystem, verifier-soundness, and automatic Rust/model-equivalence non-claims.
Passing tests alone, importing a Trellis proof, or recording an unchecked proof
log is not completion.

## Portfolio registry and audit

- **Project-owned generic models**: count-prefixed bounded sequences of
  length-delimited fields, component paths, and opaque admission models. This
  route is validated by 30 Verus obligations and is the accepted bounded claim.
- **Schema-specific Rust refinement**: directly verify every receipt/manifest
  encoder branch and parser. This would be stronger than the accepted
  correspondence claim and remains outside this change; no such refinement is
  claimed.
- **Imported Trellis certification**: rejected. Trellis supplied reviewed proof
  patterns only, and its proof results do not transfer to this repository.
- **Tagged-encoding redesign**: rejected as scope drift because it would change
  canonical identities rather than verify the shipped versioned format.

The adversarial audit confirmed that the component path model abstracts Rust
UTF-8 tokenization and that admission identity values are abstract rather than
cryptographic hashes. `proofs/generated/evidence.json` records both limitations,
the verifier/solver trust boundary, and exact BLAKE3 identities for the separate
proof and implementation sources. The attempted advisory VibeThinker review
was unavailable due to an unreachable local endpoint; deterministic Verus,
Rust, Nickel, Nix, and Cairn checks remain authoritative.

## Checks

- Pre-change `nix develop -c cargo test --workspace`: passed with 23 tests.
- Cairn proposal, design, and task gates: passed before implementation.
- `octet-production-verus ... proofs/identity_primitives.rs`: `30 verified, 0 errors`.
- The same verifier rejects `proofs/fixtures/invalid/ambiguous-prefix.rs` with
  `postcondition not satisfied`.
- `nix run path:../trellis#verusfmt -- --check --verus-only proofs/identity_primitives.rs`: passed.
- `nix develop -c cargo test -p nickel-export-core proof_correspondence_vectors_match_rust_primitives`: passed.
- `nix develop -c cargo test --workspace`: passed.
- `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `nix develop -c cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown`: passed.
- `nix build .#checks.x86_64-linux.identity-proofs --no-link -L`: passed, including proof rerun, negative fixture, formatter, Nickel evidence freshness, and artifact identity checks.
- `nix run path:../cairn#cairn -- validate --root .`: passed with two active changes before sync/archive.
- `nix run path:../cairn#cairn -- traceability coverage --root . --policy cairn-policy/generated/cairn-policy.json --profile nickel-export-default --json`: passed with 17 of 17 requirements referenced; receipt `59d27d20bb43b49ef37d197b91183796f00bbb7c9d7c5a34ef007583eef0f74e`.
- `nix flake check -L`: passed all local `x86_64-linux` checks, including the identity proof rail; Nix reported `aarch64-linux` as omitted/incompatible rather than failed.
