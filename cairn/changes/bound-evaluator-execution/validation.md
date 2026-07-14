# Validation evidence

- The evaluator-binding archive supplied a passing workspace, strict Clippy, Wasm, CLI, and Nix baseline.
- `config/resource-limits.ncl` typechecks and its checked JSON export is embedded in the CLI and freshness-checked by Nix.
- Workspace tests cover exact stream bounds, oversized stream rejection, evaluator timeout/kill/reap behavior, overlong paths, overlong options, and non-zero profile validation.
- Core identity lengths now use checked conversion and return `SizeOverflow`; artifact and diagnostic bounds return staged limit errors.
- Evaluator stdout/stderr collection is bounded, deadline-supervised, terminated, and reaped without `Command::output`.
- Secret-marker scanning no longer allocates a lowercased copy of each artifact.
- Formatting, workspace tests, strict Clippy, and Wasm no-std checks pass.
