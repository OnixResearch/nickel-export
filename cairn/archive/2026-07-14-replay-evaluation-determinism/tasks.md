# Tasks: Replay evaluation determinism

## Phase 1: Define replay evidence

- [x] [serial] M1: Add typed replay profile, shared plan identity, ordered outcomes, and bounded non-claims [r[nickel_export.shell.determinism_replay]]
- [x] [serial] M2: Define deterministic replay report canonicalization [r[nickel_export.shell.determinism_replay]]

## Phase 2: Implement replay orchestration

- [x] [serial] I1: Execute one captured evaluator plan sequentially under a named run-count bound [r[nickel_export.shell.determinism_replay]]
- [x] [serial] I2: Compare exact output bytes and identities and fail on divergence or any run failure [r[nickel_export.shell.determinism_replay]]
- [x] [serial] I3: Add CLI and release-profile integration without clocks or ambient IDs [r[nickel_export.shell.determinism_replay]]

## Phase 3: Verify

- [x] [serial] V1: Add agreement, alternating-output, failure, timeout, oversize, and mutation tests [r[nickel_export.shell.determinism_replay]]
- [x] [serial] V2: Run Rust, Cairn, CLI, and Nix validation and archive the change [r[nickel_export.shell.determinism_replay]]
