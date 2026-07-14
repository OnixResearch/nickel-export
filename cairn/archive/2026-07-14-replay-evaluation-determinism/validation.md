# Validation evidence

- The archived identity-proof change supplied a clean `nix flake check -L` pre-change baseline on `x86_64-linux`.
- Cairn proposal, design, and task gates passed before implementation.
- `--replay-runs` is optional, integer-typed, requires at least two runs, and is bounded by the Nickel-authored `max_replay_runs` resource field.
- One captured snapshot, canonical evaluator plan, plan identity, evaluator artifact identity, and resource-profile identity are reused by every sequential run.
- The pure replay assessment compares exact output bytes, records ordered BLAKE3 identities and lengths, and withholds agreed output for divergence, failure, timeout, and oversize classifications.
- The schema-owned report encoder excludes clocks, process IDs, snapshot roots, and machine-local paths; two three-run service replays produced byte-identical JSONL evidence.
- Positive tests cover stable agreement, canonical report identity, CLI bounds, immutable captured bytes, deterministic rendering, and ordinary receipt admission after agreement.
- Negative tests cover alternating external output, explicit evaluator failure, timeout, oversized stdout, invalid/duplicate run counts, divergence, and shell-error evidence without a success receipt.
- `nix develop -c cargo test -p nickel-export`: passed with 18 shell tests.
- `nix develop -c cargo test --workspace`: passed, including 24 core tests and the shell replay suite.
- `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`: passed after boxing optional replay evidence to preserve a small shell error type.
- `nix develop -c cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown`: passed.
- `nix build .#checks.x86_64-linux.identity-proofs --no-link -L`: passed after refreshing the exact core implementation identity; `30 verified, 0 errors`, and the false neighboring proof remained rejected.
- Fixture manifests for JSON, TOML, YAML, raw text, and the service example were regenerated through normal write mode because the typed resource-profile identity changed.
- Direct service replay with three runs passed twice and `cmp` confirmed identical report and receipt bytes.
- `nix build .#checks.x86_64-linux.cli-e2e --no-link -L`: passed with release-selected JSON and service replay checks, deterministic service evidence comparison, ordinary freshness checks, and existing negative rails.
- `nix run path:../cairn#cairn -- traceability coverage --root . --policy cairn-policy/generated/cairn-policy.json --profile nickel-export-default --json`: passed with 18 of 18 requirements referenced; receipt `12a3a9d39db599fa87fc94648d59def8379467467797a3fb446694381e63c2c2`.
- `nix flake check -L`: passed all 11 local checks on `x86_64-linux`, including replay-aware CLI end-to-end validation and refreshed identity proofs; Nix reported `aarch64-linux` as omitted/incompatible rather than failed.
