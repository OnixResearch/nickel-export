# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Evaluator execution and evidence material are bounded

r[nickel_export.shell.bounded_evaluation]
The core and shell MUST enforce a typed named resource-limit profile over artifact counts and sizes, path and option lengths, diagnostics, evaluator output streams, and evaluator lifetime. Length conversion MUST fail on overflow, and every timeout or bound violation MUST produce a staged failure without a successful receipt.

#### Scenario: Evaluation stays within bounds

- GIVEN a valid captured plan and evaluator that complete within the selected profile
- WHEN bounded evaluation runs
- THEN output admission proceeds and records the applied limit-profile identity.

#### Scenario: Evaluator hangs

- GIVEN an evaluator exceeds its configured deadline
- WHEN supervision reaches the deadline
- THEN the shell terminates and reaps the evaluator and emits no receipt.

#### Scenario: Output or diagnostic stream is oversized

- GIVEN evaluator stdout or stderr exceeds its configured byte bound
- WHEN bounded collection detects the excess
- THEN evaluation fails without retaining unbounded data or issuing a receipt.

#### Scenario: Canonical length overflows

- GIVEN a count or byte length cannot be represented by the canonical identity format
- WHEN identity construction runs
- THEN it returns an explicit overflow error rather than saturating or truncating.
