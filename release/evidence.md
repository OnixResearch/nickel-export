# Release evidence

Date: 2026-07-14

## Cohorts

- Rust toolchain: `1.88.0`, including `wasm32-unknown-unknown`.
- Nickel evaluator package: `nickel-lang-cli-1.17.0`; CLI invocations verify the evaluator `--version` token before evaluation.
- Root nixpkgs input: `e7a3ca8092b61ff85b6a45bf863ea2b2d6a661b3`.
- Root rust-overlay input: `e013376c32a8fcf07ddb6ec71739552bc118b7bd`.
- Cairn lifecycle input: `a22ea2bff65f16abec4f0f7ba2d7ddc14dc35871`.
- Octet proof-tool provider: `374bd16b26cee2af34211a29bfa531c016811f51`, supplying Verus `0.2026.05.17.e479cce`.
- Project-owned artifact, proof, replay-report, and correspondence identities: BLAKE3 with an explicit `b3:` tag where carried on wire.
- Checked resource-profile identity after adding the four-run replay bound: `b3:5c8d9b0bf5b9ed4991d0d223d6e7cae62ab55ccee9ad93b743a4b6b057809bb6`.
- License: `AGPL-3.0-or-later`; full text is checked in at `LICENSE` and installed by the Nix package.

## Inventory and boundaries

`docs/inventory.md` records Octet, Mantle, Cairn, Trellis, and Animus behavior at immutable revisions. `docs/migration.md` defines pinned dual-run cutover and rollback. The extracted core does not own evaluator semantics, import-resolution authority, filesystem roots, output destination authority, consumer policy, build claims, lifecycle gates, or release eligibility. Bounded Verus models and sequential replay add local detection evidence only; they do not transfer Trellis claims, prove verifier soundness, formally refine all Rust behavior, or establish universal evaluator determinism.

## Verification

- `cargo test --workspace`: passed with 24 core tests and 18 shell tests, pairing positive and negative cases for admission, canonical identities, transactions, proof correspondence, replay agreement/divergence/failure, timeout, oversize, and immutable captured inputs.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown`: passed.
- `nix build .#checks.x86_64-linux.identity-proofs --no-link -L`: passed with `30 verified, 0 errors`; the deliberately false adjacent proof was rejected, Nickel evidence was fresh, and exact proof/vector/Rust BLAKE3 identities matched.
- Four real external Nickel format fixtures and the worked service example passed write and check mode: JSON, TOML, YAML, raw text, and service JSON. Resource-profile drift regenerated their receipts through normal write mode.
- Release-selected JSON and service exports passed three sequential replay runs. Repeated service replay produced identical report and receipt bytes.
- Negative rails reject unsafe traversal, symlink components, secret-like material without opt-in, unbound contracts, incomplete dependency closures, undeclared observed imports, evaluator/contract errors, evaluator version mismatch, mixed evaluator manifests, duplicate outputs, stale manifests, tampered generated output, mismatched declared-input material, invalid replay profiles, alternating output, evaluator failure, timeout, and oversized replay output.
- `cairn validate --root .`: passed with one accepted spec and no active changes after all lifecycle packages were archived.
- `cairn traceability coverage --root . --policy cairn-policy/generated/cairn-policy.json --profile nickel-export-default --json`: passed with 18 of 18 requirements referenced; receipt hash `12a3a9d39db599fa87fc94648d59def8379467467797a3fb446694381e63c2c2`.
- `nix flake check -L`: passed all 11 local release checks covering package tests, format, strict Clippy, fuzz compilation, host/Wasm no-std checks, proof reruns, typed Nickel freshness, Cairn policy/traceability, replay-aware CLI exact-output freshness, malformed input, evaluator drift, and tamper rejection. The local run checked `x86_64-linux`; Nix reported `aarch64-linux` as omitted/incompatible, not failed.

## Publication boundary

This checkout is an independent local Git/Jujutsu repository. Consumer manifests must not depend on it through a workspace-relative path. Octet and other consumers remain on their legacy paths until this repository is published and they can pin an immutable remote revision. Publishing or pushing is intentionally outside this local evidence step.
