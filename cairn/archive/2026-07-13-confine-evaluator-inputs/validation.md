# Validation evidence

- Baseline `nix develop -c cargo test --workspace`: passed with 17 core tests and the existing shell suite.
- `nix develop -c cargo fmt --check`: passed.
- `nix develop -c cargo test --workspace`: passed after snapshot implementation.
- `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `nix develop -c cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown`: passed.
- `nix build .#checks.x86_64-linux.cli-e2e --no-link -L`: passed real JSON, TOML, YAML, raw, service-example, evaluator-drift, unsafe-path, and tamper rails with snapshot evaluation.
- Focused tests cover captured-byte stability after repository mutation, snapshot cleanup, private package cache creation, and an empty evaluator environment.
