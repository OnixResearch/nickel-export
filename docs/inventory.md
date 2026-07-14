# Consumer inventory

Inventory baseline: 2026-07-13.

| Consumer | Revision | Existing behavior | Shared extraction candidate | Consumer-owned boundary |
|---|---|---|---|---|
| Octet | `49d2262d78462c41c7f732eeeda267c78a813606` | Pure request/spec validation, exact source/dependency/output BLAKE3 identities, secret markers, deterministic manifests, freshness checks, JSON/TOML/YAML/raw command shapes | Canonical request normalization, identities, receipt/manifest ordering, freshness, Octet projection | Candidate-family policy, evidence role, destination admission, release gates |
| Mantle | `732d0f1a59fb7001d38206321e8576b7c0ec2fda` | Embedded evaluator shell, path confinement, exact source refs, evaluator descriptor, receipt/report rendering | Receipt fields, evaluator descriptor, non-claim, Mantle projection | Embedded `crunch_eval` semantics, root authority, output writes, build/release claims |
| Cairn | `7e9ed636203395b3808a65962f6bb6da60f57268` | External `nickel export` for policy and lifecycle data, then Rust semantic checks | External shell adapter and freshness mode | Lifecycle policy parser, gate authority, generated policy destination |
| Trellis | `fe008bda65baf9a335fe837294837427973a4ab4` | Several Rust and Nix external Nickel invocations with byte freshness and negative fixtures | External shell adapter, exact identities, check mode | Verification policy, proof corpus semantics, Verus gates |
| Animus | `f1a8995dca714938042d66336477aa72c518e0a2` | Nix generation of TOML/YAML/JSON field projections followed by byte diffs | Multi-format external shell, selector identity, check mode | Agent/runtime profile contracts, destination ownership, runtime gates |

## Extraction conclusion

The shared layer is request shape, exact artifact identity, a versioned declared-input fingerprint, evaluator descriptors, structured diagnostics, deterministic receipts/manifests, freshness comparison, conservative secret handling, and compatibility projections.

The shared layer explicitly excludes Nickel evaluator semantics, import resolution authority, filesystem roots, output destination authority, consumer contracts and policy meaning, build authority, lifecycle gates, proof claims, and release eligibility.

## Proof-pattern reference

The bounded identity proof work reviewed Trellis `src/serialize_inj.rs`,
`scripts/verus.sh`, and `docs/proof-artifact-manifest.contract.ncl` at
`7f99b1b8f0be0fcec5fad6334a2af6fc8746bf25`. Nickel-export reruns its own
project-local Verus source with a separately recorded correspondence boundary;
no Trellis proof or whole-system claim transfers into this repository.
