# Tasks: Bind evaluator execution

## Phase 1: Define exact evaluator evidence

- [ ] [serial] M1: Add typed execution-plan, evaluator-artifact, and optional closure-identity models [r[nickel_export.shell.evaluator_execution_identity]]
- [ ] [serial] M2: Define canonical option semantics and duplicate/conflict rejection [r[nickel_export.shell.evaluator_execution_identity]]

## Phase 2: Implement one resolved execution path

- [ ] [serial] I1: Resolve and hash one evaluator artifact for version and export execution [r[nickel_export.shell.evaluator_execution_identity]]
- [ ] [serial] I2: Render argv and receipt semantics from the same typed plan [r[nickel_export.shell.evaluator_execution_identity]]
- [ ] [serial] I3: Add the bounded Nix/Mantle closure evidence adapter surface [r[nickel_export.shell.evaluator_execution_identity]]

## Phase 3: Verify

- [ ] [serial] V1: Add positive identity and negative replacement, ambiguity, and plan-drift tests [r[nickel_export.shell.evaluator_execution_identity]]
- [ ] [serial] V2: Run Rust, Cairn, CLI, and Nix validation and archive the change [r[nickel_export.shell.evaluator_execution_identity]]
