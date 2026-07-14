# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Generative validation covers canonical invariants

r[nickel_export.validation.generative_rails]
The repository MUST provide bounded reproducible property tests and coverage-guided corpus targets for normalization, strict decoding, admission, canonical identity encoding, and shell argument parsing. Generated failures MUST be minimized and promoted to deterministic positive and negative regression fixtures.

#### Scenario: Valid value is normalized repeatedly

- GIVEN an arbitrary valid request or evaluator descriptor within configured bounds
- WHEN normalization runs repeatedly
- THEN the normalized value and canonical identities remain unchanged after the first normalization.

#### Scenario: Identity-bearing field changes

- GIVEN an admitted value and one generated mutation to an included identity field
- WHEN canonical identity construction runs
- THEN the identity changes, while generated mutations to deliberate exclusions leave it unchanged.

#### Scenario: Malformed bytes are decoded or admitted

- GIVEN arbitrary bounded request or manifest bytes
- WHEN decoding and admission run
- THEN they return a valid admitted value or a structured error without panic or out-of-profile allocation.

#### Scenario: Generative failure is found

- GIVEN a property or fuzz target finds a counterexample
- WHEN the failure is accepted for repair
- THEN its minimized input, deterministic seed when applicable, positive neighbor, and negative assertion are checked into the regression corpus.
