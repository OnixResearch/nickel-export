# Proposal: Replay evaluation determinism

## Summary

Add bounded sequential replay that evaluates one captured input and evaluator plan repeatedly, compares exact outputs, and emits explicit agreement or divergence evidence.

## Motivation

A declared-input identity lets consumers recognize repeated evaluations, but the CLI currently performs only one run. Replaying the same captured plan can detect hidden inputs, evaluator nondeterminism, or plan drift when output identities differ. Repeated agreement remains empirical evidence rather than proof.

## Scope

- Add an explicit replay mode with a typed, named run-count bound.
- Reuse one captured input snapshot, evaluator artifact identity, canonical plan, environment profile, and resource profile.
- Execute runs sequentially and retain bounded per-run status and output identities.
- Fail closed when any run fails, times out, or produces a different output identity.
- Emit deterministic replay evidence without clocks or ambient run identifiers.
- Integrate replay fixtures into release validation for selected export families.

## Non-Goals

- Claiming repeated agreement proves universal determinism.
- Running concurrent evaluations or benchmarking performance.
- Caching successful outputs.
- Masking a failed run because other runs agree.

## Impact

- **CLI**: replay command or explicit replay option.
- **Evidence**: versioned replay report with shared plan identity and per-run outcomes.
- **Release**: selected fixtures gain bounded repeated-evaluation checks.
