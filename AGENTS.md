# Agent Notes

`nickel-export-core` is the evaluator-neutral `#![no_std]` functional core. It must not read files, inspect environment state, spawn processes, print, consult clocks, use network I/O, or depend on product repositories.

`nickel-export` is the std shell. Keep file access, external Nickel execution, output writes, and exit handling there.

Use BLAKE3 for project-owned identities. Preserve versioned canonical schemas and explicit Octet/Mantle compatibility projections. Add positive and negative tests together.

Primary checks:

```console
cargo test --workspace
cargo check -p nickel-export-core --no-default-features --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets -- -D warnings
nix flake check -L
```
