# Proposal: Validate evidence types

## Summary

Separate untrusted serialized data from opaque admitted receipt and manifest types so invalid evidence cannot reach projections or verification through ordinary public APIs.

## Motivation

Canonical receipt and manifest structs currently expose public fields and derive permissive Serde decoding. Callers can fabricate or mutate schema names, identities, non-claims, evaluator descriptors, or nested artifacts and then pass those values into manifest and compatibility projection functions. Unknown serialized fields are ignored by default.

## Scope

- Introduce wire/data-transfer types for untrusted serialized input.
- Add strict decoding that rejects unknown fields and unsupported versions.
- Validate all structural, identity-shape, schema, non-claim, evaluator, and ordering invariants.
- Construct opaque `AdmittedReceipt` and `VerifiedManifest` values with private fields.
- Restrict manifest construction and Octet/Mantle projections to admitted values.
- Preserve ergonomic read-only accessors and explicit serialization.

## Non-Goals

- Proving that supplied artifact digests match unavailable bytes.
- Rerunning Nickel during structural admission.
- Adding signatures or issuer authentication.

## Impact

- **Core API**: public mutable evidence structs are replaced or deprecated in favor of validated states.
- **Schemas**: strict unknown-field and version rejection becomes normative.
- **Consumers**: adapters must explicitly admit deserialized evidence before projection.
