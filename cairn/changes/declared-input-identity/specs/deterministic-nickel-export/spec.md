# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Declared evaluation inputs have a pre-output identity

r[nickel_export.core.declared_input_identity]
The core MUST derive a versioned BLAKE3 identity from normalized source and dependency identities, import paths, selector, contract metadata, format, evaluator identity and version, sorted evaluator options, and dependency-observation policy, and canonical receipts MUST carry that identity. It MUST exclude consumer family, destination, output bytes, diagnostics, and secret-admission policy.

#### Scenario: Output materialization changes

- GIVEN identical declared evaluation inputs
- WHEN the destination, output bytes, consumer family, or secret-admission policy changes
- THEN the declared input identity MUST remain unchanged.

#### Scenario: Declared evaluation semantics change

- GIVEN an existing declared input identity
- WHEN source bytes, dependency bytes, import paths, selector, contract metadata, format, evaluator descriptor, or dependency-observation policy changes
- THEN the declared input identity MUST change.

#### Scenario: Declared material is malformed

- GIVEN source or dependency material whose path does not match the normalized request
- WHEN declared input identity construction runs
- THEN construction MUST fail without producing an identity.

#### Scenario: Closure is declared only

- GIVEN a declared input identity whose evaluator did not report its observed closure
- WHEN the identity is consumed
- THEN it MUST NOT be represented as proof of complete closure or as a safe cache key.
