# Tasks: Confine evaluator inputs

## Phase 1: Model captured evaluation

- [x] [serial] M1: Add pure captured-input and confinement-policy types [r[nickel_export.shell.captured_input_evaluation]]
- [x] [serial] M2: Define explicit environment and package-admission rules [r[nickel_export.shell.captured_input_evaluation]]

## Phase 2: Implement the shell boundary

- [x] [serial] I1: Materialize exact declared bytes in a private path-preserving snapshot [r[nickel_export.shell.captured_input_evaluation]]
- [x] [serial] I2: Run Nickel against the snapshot with ambient import authority removed [r[nickel_export.shell.captured_input_evaluation]]
- [x] [serial] I3: Fail closed for unsupported package and filesystem escape behavior [r[nickel_export.shell.captured_input_evaluation]]

## Phase 3: Verify

- [x] [serial] V1: Add positive snapshot and negative mutation, ambient-import, package, escape, and cleanup tests [r[nickel_export.shell.captured_input_evaluation]]
- [x] [serial] V2: Run Rust, Cairn, CLI, and Nix validation and archive the change [r[nickel_export.shell.captured_input_evaluation]]
