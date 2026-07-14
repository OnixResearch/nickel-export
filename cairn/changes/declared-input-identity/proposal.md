# Proposal: Declared input identity

## Summary

Add a versioned BLAKE3 fingerprint for the exact declared inputs to one Nickel evaluation and carry it in canonical receipt and manifest v2.

## Motivation

The existing receipt identifies source, dependencies, and output separately only after evaluation. Consumers cannot name the declared evaluation before output exists or directly recognize that two receipts used the same declared inputs. A dedicated fingerprint enables cross-run correlation and exposes differing outputs for identical declared inputs without claiming that an external evaluator was hermetic.

## Scope

- Add a pure `no_std` declared-input identity API.
- Canonically bind source and dependency identities, evaluation options, and evaluator descriptor.
- Exclude consumer labels, destination, output bytes, diagnostics, and secret-admission policy.
- Carry the fingerprint in canonical receipt and manifest v2.
- Preserve Octet and Mantle v1 compatibility projections.
- Add positive and negative tests, checked fixtures, documentation, and release validation.

## Non-Goals

- Adding a cache, content-addressed store, or hash-derived output paths.
- Treating a `declared_only` fingerprint as proof of complete dependency closure.
- Hashing or sandboxing the external evaluator executable.
- Changing consumer policy, release, build, or deployment authority.

## Impact

- **Schemas**: canonical receipt and manifest advance to v2; declared-input identity uses its own v1 domain.
- **Files**: pure core, fixtures, accepted specification, schema/example/migration documentation, and release profiles.
- **Testing**: stable identity, semantic-input sensitivity, output independence, malformed material rejection, compatibility projections, no-std/Wasm, CLI freshness, and full Nix validation.
