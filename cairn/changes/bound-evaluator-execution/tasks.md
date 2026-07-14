# Tasks: Bound evaluator execution

## Phase 1: Define resource contracts

- [ ] [serial] M1: Add typed `ResourceLimits`, staged bound diagnostics, and Nickel-authored profiles [r[nickel_export.shell.bounded_evaluation]]
- [ ] [serial] M2: Classify semantic versus operational limit identity fields [r[nickel_export.shell.bounded_evaluation]]

## Phase 2: Enforce bounds

- [ ] [serial] I1: Validate core artifact, path, option, diagnostic, and canonical-length bounds [r[nickel_export.shell.bounded_evaluation]]
- [ ] [serial] I2: Add bounded evaluator stdout/stderr collection, deadline, termination, and reaping [r[nickel_export.shell.bounded_evaluation]]
- [ ] [serial] I3: Replace saturating conversions and avoid allocation-heavy secret scanning [r[nickel_export.shell.bounded_evaluation]]

## Phase 3: Verify

- [ ] [serial] V1: Add positive boundary and negative overflow, oversize, hang, and process-tree tests [r[nickel_export.shell.bounded_evaluation]]
- [ ] [serial] V2: Run Rust, Wasm, Cairn, CLI, and Nix validation and archive the change [r[nickel_export.shell.bounded_evaluation]]
