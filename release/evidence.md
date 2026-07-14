# Initial release evidence

Date: 2026-07-13

## Cohorts

- Rust toolchain: `1.88.0`, including `wasm32-unknown-unknown`.
- Nickel evaluator package: `nickel-lang-cli-1.17.0`; CLI invocations verify the evaluator `--version` token before evaluation.
- Root nixpkgs input: `e7a3ca8092b61ff85b6a45bf863ea2b2d6a661b3`.
- Root rust-overlay input: `e013376c32a8fcf07ddb6ec71739552bc118b7bd`.
- Cairn lifecycle input: `a22ea2bff65f16abec4f0f7ba2d7ddc14dc35871`.
- Project-owned artifact identities: BLAKE3 with an explicit `b3:` tag.
- License: `AGPL-3.0-or-later`; full text is checked in at `LICENSE` and installed by the Nix package.

## Inventory and boundaries

`docs/inventory.md` records Octet, Mantle, Cairn, Trellis, and Animus behavior at immutable revisions. `docs/migration.md` defines pinned dual-run cutover and rollback. The extracted core does not own evaluator semantics, import-resolution authority, filesystem roots, output destination authority, consumer policy, build claims, lifecycle gates, or release eligibility.

## Verification

- `cargo test --workspace`: passed with positive and negative core and CLI cases, including declared-input repeatability, deliberate exclusions, semantic-input sensitivity, and malformed-material rejection.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown`: passed.
- `cargo doc --workspace --no-deps`: passed.
- Four real external Nickel format fixtures and the worked service example passed write and check mode: JSON, TOML, YAML, raw text, and service JSON.
- Negative rails reject unsafe traversal, symlink components, secret-like material without opt-in, unbound contracts, incomplete dependency closures, undeclared observed imports, evaluator/contract errors, evaluator version mismatch, mixed evaluator manifests, duplicate outputs, stale manifests, tampered generated output, and mismatched declared-input material.
- `cairn validate --root .`: passed with one accepted spec and no active changes after archival.
- `cairn traceability coverage --root . --profile nickel-export-default --json`: passed, eight of eight requirements referenced; receipt hash `14f1b05c21ed0468cf686853ca7c096faed830ce14673e7a40105419bb505df7`.
- `nix flake check -L`: passed the final release rail covering package tests, format, strict Clippy, host/Wasm no-std checks, typed Nickel freshness, Cairn policy/traceability, CLI exact-output freshness, malformed input, evaluator drift, and tamper rejection. The local run checked `x86_64-linux`; Nix reported `aarch64-linux` as an unevaluated incompatible system, not a failed check.

## Publication boundary

This checkout is an independent local Git/Jujutsu repository. Consumer manifests must not depend on it through a workspace-relative path. Octet and other consumers remain on their legacy paths until this repository is published and they can pin an immutable remote revision. Publishing or pushing is intentionally outside this local evidence step.
