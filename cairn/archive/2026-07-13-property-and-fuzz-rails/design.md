# Design: Property and fuzz rails

## Context

The core is pure, deterministic, `no_std`-capable, and operates over in-memory values, making it possible to test logic without filesystem or evaluator mocks. Generators must remain bounded so failures are reproducible and CI costs explicit.

## Dependencies

Land after or alongside `validate-evidence-types` and `canonicalize-evidence-identities`; keep generators versioned with each schema.

## Decisions

### 1. Test invariants rather than random examples

**Choice:** Define properties for normalization idempotence, permutation behavior, identity sensitivity/exclusions, strict admission, canonical round trips, and no-panic failure handling.

**Rationale:** These properties directly mirror specification requirements.

### 2. Keep generation bounded and reproducible

**Choice:** Use named generation-size, case-count, shrink, and time budgets. Persist seeds and minimized inputs for every failure.

**Rationale:** CI and local reruns must reproduce the same defect.

### 3. Separate valid and adversarial generators

**Choice:** Generate admitted structures through valid builders and malformed structures through wire DTO mutation.

**Rationale:** Positive coverage should not be dominated by early parse rejection, while negative coverage targets one invariant at a time.

### 4. Fuzz parsers and canonical boundaries

**Choice:** Add coverage-guided targets for JSON request/manifest decoding, relative path normalization, canonical identity encoders, admission, and CLI argument parsing.

**Rationale:** These are the highest-risk attacker-controlled and ambiguity-sensitive boundaries.

### 5. Promote every defect to a regression fixture

**Choice:** Minimized fuzz/property failures are checked in with one positive neighbor and one negative assertion.

**Rationale:** Long fuzz runs are advisory; deterministic fixtures become the release evidence.

## Risks / Trade-offs

- Poor generators can create duplicate-family noise rather than useful coverage.
- Fuzzing dependencies may not support every no-std target, so fuzz harnesses remain dev-only.
- Case counts are diagnostic evidence, not proof.

## Validation Plan

- Run deterministic property suites in normal workspace tests.
- Run checked corpus targets under fixed CI budgets.
- Verify malformed inputs never panic, allocate beyond profile, or produce admitted evidence.
