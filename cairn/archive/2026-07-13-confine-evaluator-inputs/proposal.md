# Proposal: Confine evaluator inputs

## Summary

Evaluate Nickel from an exact captured input snapshot with an explicit subprocess environment so receipts bind the bytes the evaluator actually consumed.

## Motivation

The shell currently reads source and dependency bytes for hashing, then invokes Nickel on the original repository paths. Concurrent mutation can therefore make Nickel evaluate different bytes from those recorded in the receipt. Nickel also searches ambient `NICKEL_IMPORT_PATH` and can consult package lock and cache state that the request does not currently bind.

## Scope

- Capture source, contract, and declared dependency bytes once.
- Materialize those bytes into a private path-preserving evaluation snapshot.
- Invoke Nickel against the snapshot rather than the mutable repository.
- Clear ambient import authority and supply an explicit deterministic environment.
- Reject package imports unless their manifest and package material are explicitly declared and supported.
- Record whether evaluation was snapshot-only or filesystem-sandbox-confined.
- Add mutation, ambient-import, undeclared-package, and cleanup tests.

## Non-Goals

- Proving Nickel evaluator correctness.
- Claiming complete closure from a snapshot without filesystem confinement.
- Adding a cache or content-addressed store.
- Owning Nix or Mantle sandbox semantics.

## Impact

- **Shell**: input capture, snapshot lifecycle, subprocess environment, and cleanup.
- **Schemas**: dependency-observation policy gains an explicitly bounded confinement mode.
- **Testing**: positive snapshot evaluation and negative race, ambient import, package, and escape cases.
