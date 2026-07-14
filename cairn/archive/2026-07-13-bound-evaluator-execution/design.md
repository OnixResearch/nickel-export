# Design: Bound evaluator execution

## Context

Determinism requires more than stable successful output: failures must also be bounded and classified. An evaluator that hangs or emits unlimited output can prevent any receipt decision. Silent saturation also makes the canonical encoding non-injective on hypothetical wider targets.

## Dependencies

Coordinate with `confine-evaluator-inputs` and `bind-evaluator-execution` so one captured plan carries an explicit resource-limit profile.

## Decisions

### 1. Define limits as typed data

**Choice:** Add a pure `ResourceLimits` type and a typed Nickel profile whose checked runtime export supplies named defaults.

**Rationale:** Limits remain reviewable configuration rather than unexplained numeric literals.

### 2. Validate before allocation-heavy work

**Choice:** Reject overlong paths, options, diagnostics, artifact counts, and known file sizes before cloning, sorting, or serializing them.

**Rationale:** Bounds should protect the work needed to report failures.

### 3. Supervise evaluator streams and lifetime

**Choice:** Spawn with piped output, collect through bounded readers, enforce a deadline, terminate on breach, and always reap the child.

**Rationale:** Timeout and output overflow become deterministic staged failures instead of hangs or unbounded allocation.

### 4. Fail on integer conversion overflow

**Choice:** Replace `unwrap_or(u64::MAX)` with checked conversion and `SizeOverflow` diagnostics.

**Rationale:** Canonical lengths must never saturate or alias.

### 5. Record the applied profile

**Choice:** Bind the limit-profile identity into evaluator execution evidence while keeping limits out of Nickel semantic output identity unless they alter execution semantics.

**Rationale:** Reviewers can distinguish runs governed by different operational bounds.

## Risks / Trade-offs

- Portable timeout and process-tree termination need careful platform handling.
- Conservative defaults can reject legitimate large exports and require explicit profile changes.
- OS accounting remains approximate outside stronger sandboxes.

## Validation Plan

- Positive execution just below each bound.
- Negative oversized source, dependency, path, option, output, stderr, diagnostic, and manifest cases.
- Hanging and child-process fixtures prove termination and reaping.
- Checked conversion tests prove no saturation path remains.
