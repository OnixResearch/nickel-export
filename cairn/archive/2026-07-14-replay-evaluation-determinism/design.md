# Design: Replay evaluation determinism

## Context

Replay is useful only if every run shares the exact captured inputs and evaluator execution identity. Otherwise output divergence cannot be attributed cleanly. The report must also avoid clocks and process IDs so identical outcomes produce identical evidence.

## Dependencies

Implement after `confine-evaluator-inputs`, `bind-evaluator-execution`, and `bound-evaluator-execution` provide a captured plan, exact evaluator evidence, and bounded supervision.

## Decisions

### 1. Replay one immutable plan

**Choice:** Capture and validate inputs once, construct one canonical evaluator plan, then execute that plan sequentially for a configured bounded run count.

**Rationale:** Repository changes and plan rebuilding cannot contaminate comparisons between runs.

### 2. Compare exact output bytes and identities

**Choice:** Compute each output BLAKE3 identity and byte length and require exact byte agreement before ordinary receipt admission.

**Rationale:** Serializer or newline drift is observable evidence, not normalized away.

### 3. Fail on every non-agreement outcome

**Choice:** Any evaluator failure, timeout, oversized stream, or output divergence prevents a normal success receipt and produces a replay failure report.

**Rationale:** Majority voting would hide nondeterminism or operational failure.

### 4. Keep replay evidence deterministic

**Choice:** Record shared input/plan/evaluator/profile identities and ordered run outcomes without wall-clock time, random identifiers, or machine-local paths.

**Rationale:** Replaying the same bounded outcome should reproduce the same evidence bytes.

### 5. Bound the claim

**Choice:** State that agreement demonstrates only the selected runs under the recorded plan and does not prove future or universal determinism.

**Rationale:** Empirical replay is a detection rail, not formal proof.

## Risks / Trade-offs

- Replay multiplies evaluator cost and should be selected by profile.
- Stateful evaluators may perturb external caches unless confinement removes them.
- A nondeterministic evaluator can coincidentally agree across bounded runs.

## Validation Plan

- Deterministic fixture produces identical replay evidence.
- Alternating-output evaluator triggers divergence.
- Failing, hanging, and oversized-output runs each prevent success.
- Repository mutation during replay cannot change the captured plan.
