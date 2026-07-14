# Validation evidence

- Canonical evidence encoding supplied a passing workspace, strict Clippy, Wasm, CLI, and Nix baseline.
- `verify_manifest_integrity` performs strict pure manifest admission and recomputes nested declared-input, receipt, and manifest identities.
- `verify_supplied_artifacts` checks any supplied exact source, dependency, or output bytes while retaining an explicit partial-set claim boundary.
- `nickel-export verify` is a read-only shell: it reads one manifest and optional referenced artifacts, invokes no evaluator, writes nothing, and emits `onix-nickel-export-integrity-report/v1`.
- Positive structural and complete-artifact checks pass for the service example.
- Negative malformed hash, weakened non-claim, unknown field, mixed/duplicate evidence, mismatched bytes, and unknown artifact paths fail closed.
- Formatting, workspace tests, strict Clippy, and the focused real CLI verify command pass.
