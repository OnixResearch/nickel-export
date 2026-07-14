# Proposal: Property and fuzz rails

## Summary

Add bounded property-based and coverage-guided validation for normalization, identity construction, strict decoding, admission, canonical encoding, and shell request parsing.

## Motivation

Example tests cover known positive and negative cases but do not systematically explore combinations of Unicode paths, list permutations, boundary lengths, malformed nested evidence, enum variants, and identity-bearing field mutations. Deterministic pure cores are especially suitable for generated tests without mocks.

## Scope

- Add property generators for valid and invalid requests, evaluator descriptors, artifact identities, receipts, and manifests.
- Prove normalization idempotence and set-order invariance through assertions.
- Prove included-field sensitivity and deliberate-field exclusions.
- Exercise strict decode/admit/encode round trips.
- Add fuzz targets for request/manifest decoding, path normalization, canonical encoders, and shell argument parsing.
- Check in minimized regression corpus entries and deterministic seeds where supported.
- Bound generated sizes and execution budgets.

## Non-Goals

- Treating test volume as formal proof.
- Fuzzing the Nickel evaluator itself.
- Adding nondeterministic network-backed fuzz infrastructure.

## Impact

- **Development dependencies**: bounded property and fuzz tooling.
- **CI**: fast deterministic corpus checks plus separately budgeted fuzz jobs.
- **Evidence**: minimized failures become positive/negative regression fixtures.
