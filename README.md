# nickel-export

`nickel-export` is the independent, evaluator-neutral boundary for deterministic Nickel export requests, declared-input fingerprints, exact-byte identities, diagnostics, receipts, and freshness manifests.

## Start here

Nickel already converts Nickel values to JSON, TOML, YAML, and text. This tool
adds a receipt that answers: **which exact source, dependencies, evaluator, and
output bytes belonged to this export?** Its check mode then detects a stale or
manually edited generated artifact.

See the [worked service-configuration example](docs/examples.md) for the source,
contract, generated JSON, receipt manifest, CI command, expected failures, and
cases where this tool is unnecessary.

The repository separates a pure core from evaluator and filesystem authority:

- `nickel-export-core` is `#![no_std]` + `alloc`. It strictly decodes wire values, normalizes requests, validates complete declared dependency sets, computes BLAKE3 identities, rejects error diagnostics and secret-like material, and produces opaque `AdmittedReceipt` and `VerifiedManifest` states. Only admitted evidence reaches freshness checks or legacy Octet and Mantle projections. It never evaluates Nickel or performs I/O.
- `nickel-export` is a thin std shell. It captures declared files into a private path-preserving snapshot, removes ambient evaluator environment authority, invokes an explicit external Nickel program under the checked `config/resource-limits.ncl` profile, applies an optional declared contract, writes generated artifacts, and implements fail-closed `--check` mode.
- `proofs/` contains bounded Verus models for pre-hash encoding injectivity, path safety/idempotence, and admitted-state preservation. Exact BLAKE3 source identities and checked correspondence vectors make the separate Rust/model mapping auditable without claiming formal refinement.

## Claim boundary

An accepted receipt binds one `declared_input_identity` to exact output bytes
under a descriptor containing the resolved evaluator artifact hash and typed
execution-plan identity. The declared identity excludes the
consumer label, destination, output, and diagnostics, so equal declared
identities can expose differing outputs across repeated evaluations.

An executable hash does not prove its dynamic-library closure; adapters may add
an explicitly classified Nix or Mantle closure identity when they can verify it.
The receipt does **not** prove evaluator equivalence, deployability,
consumer-policy conformance, build success, semantic correctness, or release
eligibility.
`snapshot_only` means the CLI evaluated the captured declared files in a private
snapshot with ambient environment variables removed, but did not sandbox every
possible filesystem read or observe the full import closure. The identity is
therefore not a safe cache key. Consumers requiring an evaluator-observed
closure must use an adapter that supplies `EvaluatorObservedClosure` evidence
to the core. Receipts never conceal this distinction.

## Usage

A request is typed by `onix-nickel-export-request/v1` and names a source, all exact dependencies, import paths, optional selector, optional consumer-owned contract metadata, native output format, and destination. The external CLI interprets non-empty contract metadata as a repository-relative contract file and requires it in `dependencies`; embedded consumers may retain a reviewed contract label while supplying captured diagnostics directly.

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

Use `--write` to update the destination and manifest. Exactly one of `--write` and `--check` is required. Write and check modes take a repository lock; writes stage and sync both files, publish a durable transaction marker, atomically rename each file, and leave interrupted transactions fail-closed for deterministic recovery. Embedded consumers may instead atomically publish one pointer to a complete generation directory. Source, dependency, evaluator, output, diagnostic, replay-run, and process-time bounds come from the Nickel-authored `config/resource-limits.ncl` profile embedded in the CLI; timeout, stream overflow, and size conversion failures issue no receipt.

Add `--replay-runs 3` to execute the same captured snapshot and typed evaluator plan three times sequentially. Agreement prints a deterministic replay report followed by the ordinary receipt. Divergence, evaluator failure, timeout, or oversized output exits nonzero with the replay report nested in the shell error and no success receipt. This is bounded detection evidence, not proof that future runs are deterministic.

Verify stored canonical integrity without running Nickel or writing files:

```console
nix develop -c cargo run --quiet -p nickel-export -- verify \
  --manifest examples/service-config/generated/manifest.json \
  --root . \
  --check-artifacts
```

Structural integrity is not freshness or semantic correctness. `--check-artifacts`
only verifies exact bytes for the manifest paths supplied from the selected
repository root.

## Schemas and compatibility

Canonical schemas are documented in [docs/schemas.md](docs/schemas.md). Receipt
and manifest identities use versioned schema-owned length-delimited bytes;
pretty JSON is only the reviewable wire form. The
[worked examples](docs/examples.md) show how those schemas fit into a concrete
workflow. Serialization is feature-gated in the core; `--no-default-features`
keeps the core evaluator-neutral and `no_std`.

Compatibility projections preserve the checked legacy fields used by:

- Octet `octet-nickel-export-manifest/v1`;
- Mantle `mantle-nickel-export-receipt-v1`.

These are adapters, not alternate semantic owners. Consumer evaluation strategy, destination authority, product policy, and release gates remain outside this repository.

## Validation and release

```console
cargo test --workspace
cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets -- -D warnings
cargo check --manifest-path fuzz/Cargo.toml
nix build .#checks.x86_64-linux.identity-proofs --no-link -L
nix flake check -L
```

The typed repository and release profiles live in `config/repository.ncl` and `release/profile.ncl`. Checked JSON exports are freshness-tested. The pinned Nix input, Rust toolchain, Nickel evaluator cohort, package license map, positive/negative fixtures, host/Wasm core checks, and CLI tamper tests make the release boundary reproducible.

Distribution is through immutable Git revisions and Nix inputs. Both Cargo packages set `publish = false`; crates.io is not a release channel for this project.

See [docs/migration.md](docs/migration.md) for the dual-run consumer cutover and rollback procedure.

## License

`nickel-export-core` is `MPL-2.0`; the `nickel-export` evaluator/file shell is `AGPL-3.0-or-later`. Complete texts and the package map are in [LICENSE](LICENSE), [LICENSES](LICENSES), and the typed [repository contract](config/repository.ncl).

Package licensing is distribution metadata and is not included in canonical export identity unless a versioned schema explicitly adds it. Earlier grants and third-party terms remain intact; the split does not transfer evaluator authority into the core.

## References

The initial extraction compared these codebases at fixed revisions. They remain references only; they do not transfer consumer-owned policy or evaluator authority into this repository.

- [Octet `nickel_export.rs` at `49d2262d78462c41c7f732eeeda267c78a813606`](https://github.com/OnixResearch/octet/blob/49d2262d78462c41c7f732eeeda267c78a813606/crates/octet-standards/src/nickel_export.rs)
- [Mantle `nickel_export.rs` at `732d0f1a59fb7001d38206321e8576b7c0ec2fda`](https://github.com/OnixResearch/mantle/blob/732d0f1a59fb7001d38206321e8576b7c0ec2fda/src/nickel_export.rs)
- [Cairn policy export shell at `7e9ed636203395b3808a65962f6bb6da60f57268`](https://github.com/OnixResearch/cairn/blob/7e9ed636203395b3808a65962f6bb6da60f57268/crates/cairn-cli/src/policy.rs)
- [Trellis policy checker at `fe008bda65baf9a335fe837294837427973a4ab4`](https://github.com/OnixResearch/trellis/blob/fe008bda65baf9a335fe837294837427973a4ab4/scripts/check-verification-policy.rs)
- [Animus generation checks at `f1a8995dca714938042d66336477aa72c518e0a2`](https://github.com/OnixResearch/animus/blob/f1a8995dca714938042d66336477aa72c518e0a2/flake.nix)
- [Trellis serialization-injectivity proof patterns at `7f99b1b8f0be0fcec5fad6334a2af6fc8746bf25`](https://github.com/OnixResearch/trellis/blob/7f99b1b8f0be0fcec5fad6334a2af6fc8746bf25/src/serialize_inj.rs)
