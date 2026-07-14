# Tasks: Property and fuzz rails

## Phase 1: Define generators and properties

- [ ] [serial] M1: Add bounded valid and adversarial generators for every canonical schema layer [r[nickel_export.validation.generative_rails]]
- [ ] [serial] M2: Define invariant, no-panic, and resource-bound property assertions [r[nickel_export.validation.generative_rails]]

## Phase 2: Add deterministic property tests

- [ ] [serial] I1: Cover normalization, permutation, identity inclusion/exclusion, admission, and round-trip properties [r[nickel_export.validation.generative_rails]]
- [ ] [serial] I2: Add versioned regression seeds and positive/negative neighboring fixtures [r[nickel_export.validation.generative_rails]]

## Phase 3: Add fuzz targets

- [ ] [serial] I3: Fuzz request/manifest decoding, paths, canonical encoders, admission, and CLI parsing under named budgets [r[nickel_export.validation.generative_rails]]
- [ ] [serial] V1: Run property suites, checked corpora, Rust/Cairn validation, and archive the change [r[nickel_export.validation.generative_rails]]
