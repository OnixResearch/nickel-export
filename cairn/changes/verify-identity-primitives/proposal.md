# Proposal: Verify identity primitives

## Summary

Add machine-checked proofs for canonical encoding injectivity before hashing, path-normalization safety and idempotence, and admitted-evidence invariant preservation.

## Motivation

Tests can demonstrate many examples but cannot establish structural properties for every bounded input. The highest-value proof targets are pure local functions whose claims do not require proving Nickel, Serde, filesystems, BLAKE3 collision resistance, or downstream release policy.

## Scope

- Model canonical length-delimited encoding and prove structural injectivity before hashing.
- Model relative path normalization and prove idempotence and traversal exclusion.
- Prove admitted receipt and verified manifest constructors preserve required invariants.
- Connect executable Rust to proof artifacts through reviewed correspondence tests and exact source/proof identities.
- Produce reproducible proof receipts with pinned Verus/Trellis toolchain inputs.
- Preserve explicit non-claims for hashing, evaluator behavior, I/O, and whole-system correctness.

## Non-Goals

- Proving BLAKE3 collision resistance.
- Proving Nickel evaluator correctness or equivalence.
- Claiming that Trellis proofs automatically certify downstream Rust.
- Proving filesystem atomicity or sandbox correctness.

## Impact

- **Proof corpus**: focused Verus/Trellis modules and receipts.
- **Core design**: proof-facing pure primitives may be factored from shell-facing DTOs.
- **Release**: proof reruns and correspondence checks become bounded evidence gates.
