# Validation evidence

- Crash-consistent publication supplied a passing workspace, strict Clippy, CLI, and Nix baseline.
- Bounded generated cases prove request normalization idempotence and dependency-order invariance across empty and multi-dependency families.
- Existing generated identity tests cover every declared-input inclusion and deliberate exclusion; strict wire tests cover unknown and fabricated evidence.
- Checked corpus neighbors contain one admitted request and one unknown-field rejection and run in normal workspace tests.
- The cargo-fuzz `wire` target exercises request decoding/normalization, manifest decoding/integrity admission, and bounded side-effect-free CLI argument parsing.
- The fuzz workspace has a pinned lockfile, formatting check, and dedicated Nix build check.
- Host workspace tests, strict Clippy, fuzz formatting, local fuzz-target compilation, and `nix build .#checks.x86_64-linux.fuzz-target --no-link -L` pass.
- The first Nix fuzz derivation attempt exposed an incorrect build root; setting `cargoRoot = "fuzz"` fixed the deterministic package build.
