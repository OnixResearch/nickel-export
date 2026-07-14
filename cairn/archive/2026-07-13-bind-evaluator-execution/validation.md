# Validation evidence

- The preceding snapshot archive established a passing workspace, Clippy, Wasm, CLI, and Nix baseline.
- Workspace tests pass with exact evaluator artifact-change rejection and duplicate-option rejection.
- Strict Clippy passes for all workspace targets.
- Real CLI fixtures pass while recording the resolved evaluator BLAKE3 identity and typed plan identity.
- The shell resolves one canonical executable, verifies its version, checks its bytes before and after export, and derives process arguments and receipt options from one `CanonicalEvaluationPlan`.
- Canonical descriptors retain an optional separately classified closure identity; legacy Octet and Mantle projections remain unchanged.
