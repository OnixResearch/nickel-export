# Design: Verify identity primitives

## Context

The project already separates a `no_std` functional core from I/O. Formal work should target local mathematical obligations and avoid translating the unbounded shell or overclaiming from reference proofs.

## Dependencies

Implement after `validate-evidence-types` and `canonicalize-evidence-identities` stabilize admitted invariants and canonical bytes. Reuse Trellis patterns as references while rerunning all project-owned proofs.

## Decisions

### 1. Prove the pre-hash encoder, not cryptographic collision resistance

**Choice:** Prove that two encoded structured inputs with equal canonical bytes have equal canonical fields, subject to schema normalization.

**Rationale:** This establishes absence of encoding ambiguity while leaving BLAKE3 security as an explicit assumption.

### 2. Prove path normalization locally

**Choice:** Prove accepted paths are relative, contain no parent traversal, and normalize idempotently.

**Rationale:** Path safety is central to both core identity and shell confinement and has a small pure state space.

### 3. Prove admitted-state preservation

**Choice:** Model constructors for admitted receipts and verified manifests and prove schema, uniqueness, cohort, non-claim, and recomputed-identity invariants.

**Rationale:** Opaque type-state APIs become useful proof boundaries.

### 4. Keep executable correspondence explicit

**Choice:** Record exact Rust and proof source identities, run shared known-answer vectors, and emit a correspondence receipt. Do not label imported Trellis artifacts as project certification.

**Rationale:** A proof of a model is not automatically a proof of separately maintained Rust.

## Risks / Trade-offs

- Proof maintenance can pressure runtime APIs; keep proof cores small and stable.
- Correspondence remains a bounded engineering argument unless Rust itself is verified.
- Toolchain churn can invalidate receipts without changing mathematical content.

## Validation Plan

- Run all proofs from pinned tooling with no admitted assumptions beyond documented cryptographic and platform axioms.
- Mutate proof fixtures to demonstrate negative rails reject false statements or broken correspondence.
- Verify proof receipt identities through Octet/Valence conventions without escalating claims.
