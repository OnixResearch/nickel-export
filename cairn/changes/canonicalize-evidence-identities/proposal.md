# Proposal: Canonicalize evidence identities

## Summary

Compute receipt and manifest identities from an explicit versioned binary encoding rather than Serde JSON output, and publish fixed independent known-answer vectors.

## Motivation

`manifest_identity` currently hashes `serde_json::to_vec` output. The repository pins Serde, but the identity contract still inherits serializer escaping, formatting, and field-emission behavior that is not specified by the evidence schema. The declared-input identity already demonstrates a smaller versioned length-delimited encoding.

## Scope

- Define canonical receipt and manifest identity domains.
- Encode variable fields with explicit lengths and lists with explicit counts.
- Encode enums with stable schema-owned tags.
- Hash canonical bytes with BLAKE3.
- Expose canonical bytes for independent verification.
- Add fixed known-answer vectors checked by the Rust core and an independent verifier implementation.
- Keep pretty JSON as the human-facing wire representation.

## Non-Goals

- Replacing JSON artifact files with a binary format.
- Claiming BLAKE3 collision impossibility.
- Canonicalizing arbitrary JSON values.

## Impact

- **Schemas**: receipt and manifest identity algorithms gain explicit versioned definitions.
- **Fixtures**: canonical identities change under a schema migration.
- **Proof**: encoding injectivity before hashing becomes a tractable local proof obligation.
