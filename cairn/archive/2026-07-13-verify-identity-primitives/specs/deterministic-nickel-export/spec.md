# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Identity primitives have bounded machine-checked proofs

r[nickel_export.proof.identity_primitives]
The repository MUST provide reproducible machine-checked proofs that canonical evidence encoding is injective before hashing, accepted path normalization is traversal-safe and idempotent, and admitted evidence constructors preserve their declared invariants. Proof evidence MUST bind exact proof and implementation references and MUST retain explicit cryptographic, evaluator, filesystem, and correspondence non-claims.

#### Scenario: Canonical encodings are equal

- GIVEN two normalized canonical evidence values
- WHEN their pre-hash canonical byte encodings are equal
- THEN the proof establishes equality of all encoded canonical fields.

#### Scenario: Path is accepted and normalized again

- GIVEN any path accepted by the normalization model
- WHEN normalized output is inspected and normalized again
- THEN it is relative, contains no parent traversal, and remains unchanged.

#### Scenario: Admitted evidence is constructed

- GIVEN wire evidence satisfying all constructor preconditions
- WHEN the admitted receipt or verified manifest constructor succeeds
- THEN the proof establishes the schema, identity, non-claim, uniqueness, and evaluator-cohort invariants owned by that type.

#### Scenario: Proof receipt is reviewed

- GIVEN a passing proof artifact and correspondence receipt
- WHEN its claims are evaluated
- THEN it does not claim BLAKE3 collision impossibility, Nickel correctness, filesystem correctness, or automatic equivalence with separately maintained Rust.
