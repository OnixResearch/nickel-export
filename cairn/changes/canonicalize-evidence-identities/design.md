# Design: Canonicalize evidence identities

## Context

JSON remains useful for review and interoperability, but serializer output is a broad and dependency-sensitive identity preimage. A schema-owned encoder narrows the trusted computing base and can be implemented in `no_std` without I/O.

## Dependencies

Implement after `validate-evidence-types` so canonical encoders accept admitted values and cannot encode structurally invalid evidence.

## Decisions

### 1. Separate wire representation from identity representation

**Choice:** Continue rendering pretty JSON while computing identities from explicit schema-owned bytes.

**Rationale:** Human readability no longer controls cryptographic identity stability.

### 2. Use versioned length-delimited encoding

**Choice:** Domain-separate receipt and manifest encodings, prefix variable byte strings with checked lengths, prefix lists with checked counts, and assign stable enum tags.

**Rationale:** The encoding is unambiguous and suitable for local injectivity proofs.

### 3. Expose canonical bytes

**Choice:** Provide pure `encode_*_identity` functions in addition to hash helpers.

**Rationale:** Consumers and independent verifiers can audit exact preimages rather than trusting only a hash API.

### 4. Freeze known-answer vectors

**Choice:** Check in positive vectors for each schema and negative vectors for field, ordering, count, tag, and truncation changes. Verify them with a small independent implementation that does not call core encoding helpers.

**Rationale:** Self-consistency tests alone can preserve the same bug on both sides.

## Risks / Trade-offs

- Canonical identity changes require versioned fixture migration.
- Manual encoding requires careful bounds and tag governance.
- An independent verifier adds maintenance but materially reduces correlated implementation risk.

## Validation Plan

- Verify exact canonical bytes and BLAKE3 values for fixed vectors.
- Prove set-valued permutations normalize before encoding while sequence-valued changes remain visible.
- Confirm Serde upgrades and JSON formatting changes do not alter canonical identities.
