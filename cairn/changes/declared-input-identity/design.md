# Design: Declared input identity

## Context

`nickel-export-core` currently hashes exact source, dependency, and output bytes and then hashes the post-evaluation manifest. The core has all declared evaluation material before output exists, but it does not expose one identity for that material. The external CLI records `DeclaredOnly`, so any new identity must remain explicit about incomplete evaluator-observed closure.

## Decisions

### 1. Add a separate pre-output identity

**Choice:** Add `build_declared_input_identity` over `DeclaredInputMaterial` and carry its result as `declared_input_identity` in canonical receipts.

**Rationale:** Consumers can compute and compare the identity without manufacturing output bytes, while receipt construction reuses the same validated pure core.

### 2. Use a versioned length-delimited BLAKE3 encoding

**Choice:** Domain-separate the encoding with `onix-nickel-export-declared-input/v1`. Hash normalized source and dependency artifact identities, import paths, selector, contract metadata, format, evaluator identity and version, sorted evaluator options, and dependency-observation policy. Encode variable-length fields with explicit lengths and lists with explicit counts.

**Rationale:** This is deterministic without Serde, preserves the core's `no_std` build, prevents concatenation ambiguity, and gives future encoding changes a version boundary.

### 3. Exclude materialization and consumer metadata

**Choice:** Exclude `family_id`, destination, output bytes, diagnostics, and `allow_secret_material`.

**Rationale:** Those fields do not define the declared Nickel evaluation. The same evaluation may be materialized for another consumer or destination without changing its declared-input identity.

### 4. Keep the identity diagnostic rather than authoritative

**Choice:** State in code, schemas, receipts, and release policy that a `DeclaredOnly` identity is not proof of complete closure and is not a safe cache key.

**Rationale:** The shell does not observe Nickel's complete import closure, hash the evaluator executable, or provide hermetic execution. Caching or storage remains owned by Nix or Mantle.

### 5. Advance canonical schemas while preserving projections

**Choice:** Advance canonical receipt and manifest schemas and generator identity to v2. Leave Octet and Mantle v1 projections unchanged.

**Rationale:** Adding a required receipt field changes the canonical wire shape and deserves an explicit version boundary. Compatibility projections must not gain claims their consumers did not request.

## Risks / Trade-offs

- Consumers might mistake the fingerprint for a cache key; explicit non-claims and the dependency-observation policy bound that risk.
- Manual canonical encoding is another format to maintain; domain versioning and fixed-vector fixtures make changes visible.
- Receipt and manifest v2 require fixture and consumer migration; legacy projections remain stable for staged adoption.

## Validation Plan

- Positive tests prove repeatability and invariance under family, destination, output, and secret-admission changes.
- Negative/change-sensitive tests cover source, dependency, selector, contract, import path, format, evaluator, and closure-policy changes plus material mismatch rejection.
- Run workspace tests, strict Clippy, formatting, host/Wasm `no_std`, Cairn gates and traceability, CLI write/check fixtures, and `nix flake check -L`.
