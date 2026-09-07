## Why

The export shell has a bounded evaluation loop but an unbounded version probe and descendant pipe lifetime. It also clones captured files and retains all replay outputs. Large manifests repeat artifact scans. These costs and gaps affect repeated configuration exports.

## What Changes

- Use bounded process execution for version probes and evaluation, with explicit stdin and owned teardown.
- Add a total captured-input budget and safe diagnostic output.
- Borrow captured bytes, assess replay incrementally, stream executable hashing, and index artifact verification.
- Measure representative workloads and preserve exact receipt and replay semantics.
- Keep `snapshot_only` excluded from cache admission.

## Impact

- **Files**: Shell and core source, typed resource profile, generated profile, tests, benchmark, proof correspondence evidence, and documentation.
- **Testing**: Before/after workspace tests, malicious process controls, aggregate overflow, replay divergence, artifact conflicts, Clippy, Wasm, and Nix checks.
- **Consumer**: The service-configuration example and repeated export callers provide current demand.
- **Owner**: nickel-export maintainers own evidence meaning and publication. Bounded Exec owns the reused process mechanism.
- **Repeatability**: Repository-owned fixtures, benchmark commands, exact-byte tests, and Nix checks reproduce scoped observations.
