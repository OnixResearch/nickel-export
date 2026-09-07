# nickel-export

`nickel-export` records deterministic Nickel exports. It provides declared-input fingerprints, exact-byte identities, diagnostics, receipts, and freshness manifests.

The boundary is independent of a specific evaluator.

## Purpose

Nickel can export values as JSON, TOML, YAML, or text. `nickel-export` adds a receipt for four facts:

- the exact source and declared dependencies
- the evaluator identity
- the execution-plan identity
- the exact output bytes

Check mode detects stale or manually edited generated artifacts.

Read the [service-configuration example](docs/examples.md) for a complete workflow. It includes expected failures and cases where this tool is unnecessary.

## Architecture

The repository separates pure logic from evaluator and filesystem authority.

### `nickel-export-core`

`nickel-export-core` uses `#![no_std]` plus `alloc`. It performs these functions:

- decodes wire values strictly
- normalizes requests
- checks complete declared dependency sets
- computes BLAKE3 identities
- rejects error diagnostics and secret-like material
- produces opaque `AdmittedReceipt` and `VerifiedManifest` states

Only admitted evidence can enter freshness checks or compatibility projections. The core does not evaluate Nickel or perform I/O.

### `nickel-export`

`nickel-export` is a thin standard-library shell. It performs these functions:

- captures declared files in a private path-preserving snapshot
- removes ambient evaluator environment authority
- runs an explicit external Nickel program
- applies the checked `config/resource-limits.ncl` profile
- applies an optional declared contract
- writes generated artifacts
- implements fail-closed `--check` mode

### `proofs/`

`proofs/` contains bounded Verus models for these properties:

- pre-hash encoding injectivity
- path safety and idempotence
- admitted-state preservation

Exact BLAKE3 source identities and correspondence vectors connect the Rust code to the models. They do not prove formal refinement.

## Claim boundary

An accepted receipt binds one `declared_input_identity` to exact output bytes. Its descriptor includes the evaluator artifact hash and typed execution-plan identity.

The declared identity excludes these values:

- consumer label
- destination
- output
- diagnostics

As a result, repeated evaluations can have equal declared identities and different output bytes.

An executable hash does not prove its dynamic-library closure. An adapter can add a classified Nix or Mantle closure identity after it checks that closure.

A receipt does not prove these properties:

- evaluator equivalence
- deployability
- consumer-policy conformance
- build success
- semantic correctness
- release eligibility

`snapshot_only` has a narrow meaning. The CLI evaluated captured declared files in a private snapshot and removed ambient environment variables.

It did not sandbox every possible filesystem read. It also did not observe the full import closure.

Therefore, a `snapshot_only` identity is not a safe cache key. A consumer that needs observed closure evidence must supply `EvaluatorObservedClosure` evidence.

Receipts always preserve this distinction.

## Usage

A request uses the `onix-nickel-export-request/v1` type. It names these values:

- source and exact dependencies
- import paths
- an optional selector
- optional consumer-owned contract metadata
- native output format
- destination

For the external CLI, nonempty contract metadata names a repository-relative contract file. The request must include that file in `dependencies`.

An embedded consumer can retain a reviewed contract label and supply captured diagnostics directly.

### Check an export

```console
nix develop -c cargo run --quiet -p nickel-export -- export \
  --spec fixtures/requests/json.json \
  --root . \
  --evaluator nickel \
  --evaluator-identity nixpkgs:nickel \
  --evaluator-version nickel-lang-cli-1.17.0 \
  --manifest fixtures/generated/json.manifest.json \
  --check
```

Use `--write` to update the destination and manifest. You must select exactly one of `--write` and `--check`.

Write and check modes take a repository lock. Write mode performs these actions:

1. Stage and synchronize both files.
2. Publish a durable transaction marker.
3. Rename each file atomically.
4. Leave an interrupted transaction in a fail-closed state for deterministic recovery.

An embedded consumer can instead publish one pointer to a complete generation directory.

The embedded resource profile bounds these inputs and observations:

- source and dependency sizes
- evaluator output
- diagnostics
- replay runs
- process time

A timeout, stream overflow, or size-conversion failure produces no receipt.

### Detect replay divergence

Add `--replay-runs 3` to run the same snapshot and evaluator plan three times in sequence.

If all runs agree, the CLI prints a deterministic replay report. Then it prints the ordinary receipt.

If a run diverges or fails, the command exits with a nonzero status. Timeouts and oversized output have the same result.

The shell error contains the replay report. The command does not produce a success receipt.

This report is bounded detection evidence. It does not prove that future runs are deterministic.

### Check stored canonical integrity

Run this command without Nickel and without file writes:

```console
nix develop -c cargo run --quiet -p nickel-export -- verify \
  --manifest examples/service-config/generated/manifest.json \
  --root . \
  --check-artifacts
```

Structural integrity is not freshness or semantic correctness. `--check-artifacts` checks exact bytes only for the manifest paths under the selected root.

## Schemas and compatibility

[docs/schemas.md](docs/schemas.md) documents the canonical schemas. Receipt and manifest identities use versioned, length-delimited bytes that the schemas own.

Pretty JSON is only the reviewable wire form. The [worked examples](docs/examples.md) show the schemas in a complete workflow.

Serialization is an optional core feature. `--no-default-features` keeps the core evaluator-neutral and `no_std`.

Compatibility projections preserve checked legacy fields for these formats:

- Octet `octet-nickel-export-manifest/v1`
- Mantle `mantle-nickel-export-receipt-v1`

These projections are adapters. They are not alternate semantic owners.

Consumers retain evaluation strategy, destination authority, product policy, and release gates.

## Performance and process bounds

Version probes and evaluation share the pinned Bounded Exec mechanism. The shell borrows captured bytes, limits total input, and assesses replay incrementally.

Read [performance and resource limits](docs/performance.md) for benchmark commands, exact limits, and cache restrictions.

## Checks and release

```console
cargo test --workspace
cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
nix build .#checks.x86_64-linux.identity-proofs --no-link -L
nix flake check -L
```

Typed repository and release profiles are in `config/repository.ncl` and `release/profile.ncl`. The checks include generated JSON freshness.

The release boundary also includes these inputs:

- pinned Nix input and Rust toolchain
- Nickel evaluator cohort
- package license map
- positive and negative fixtures
- host and Wasm core checks
- CLI tamper tests

Distribution uses immutable Git revisions and Nix inputs. Both Cargo packages set `publish = false`.

Crates.io is not a release channel. Read [docs/migration.md](docs/migration.md) for dual-run migration and rollback instructions.

## License

`nickel-export-core` uses `MPL-2.0`. The evaluator and file shell uses `AGPL-3.0-or-later`.

Complete license texts and the package map are available in these locations:

- [LICENSE](LICENSE)
- [LICENSES](LICENSES)
- [typed repository contract](config/repository.ncl)

Package licensing is distribution metadata. It is not part of canonical export identity unless a versioned schema adds it.

Earlier grants and third-party terms remain unchanged. The license split does not transfer evaluator authority into the core.

## References

These fixed revisions informed the initial extraction. They remain references and do not transfer consumer policy or evaluator authority.

- [Octet `nickel_export.rs` at `49d2262d78462c41c7f732eeeda267c78a813606`](https://github.com/OnixResearch/octet/blob/49d2262d78462c41c7f732eeeda267c78a813606/crates/octet-standards/src/nickel_export.rs)
- [Mantle `nickel_export.rs` at `732d0f1a59fb7001d38206321e8576b7c0ec2fda`](https://github.com/OnixResearch/mantle/blob/732d0f1a59fb7001d38206321e8576b7c0ec2fda/src/nickel_export.rs)
- [Cairn policy export shell at `7e9ed636203395b3808a65962f6bb6da60f57268`](https://github.com/OnixResearch/cairn/blob/7e9ed636203395b3808a65962f6bb6da60f57268/crates/cairn-cli/src/policy.rs)
- [Trellis policy checker at `fe008bda65baf9a335fe837294837427973a4ab4`](https://github.com/OnixResearch/trellis/blob/fe008bda65baf9a335fe837294837427973a4ab4/scripts/check-verification-policy.rs)
- [Animus generation checks at `f1a8995dca714938042d66336477aa72c518e0a2`](https://github.com/OnixResearch/animus/blob/f1a8995dca714938042d66336477aa72c518e0a2/flake.nix)
- [Trellis serialization proof patterns at `7f99b1b8f0be0fcec5fad6334a2af6fc8746bf25`](https://github.com/OnixResearch/trellis/blob/7f99b1b8f0be0fcec5fad6334a2af6fc8746bf25/src/serialize_inj.rs)
- [Bounded Exec](https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git), consumed at `29dac88ecded94457572db3fdfaaaab95fa91525` for bounded process mechanics. The export shell retains evaluator and receipt authority.
