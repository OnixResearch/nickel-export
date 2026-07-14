# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Bounded replay detects divergent evaluation outputs

r[nickel_export.shell.determinism_replay]
Replay mode MUST execute one immutable captured input and evaluator plan sequentially under a typed run-count and resource profile, MUST compare exact output bytes and identities, and MUST withhold a normal success receipt when any run fails or diverges. Replay evidence MUST remain deterministic and MUST NOT claim universal determinism from bounded agreement.

#### Scenario: All replay runs agree

- GIVEN one captured plan and evaluator whose selected runs produce identical bytes
- WHEN replay completes within bounds
- THEN the report records one shared plan identity and matching ordered output identities and may proceed to ordinary receipt admission.

#### Scenario: One replay output differs

- GIVEN one selected run produces different output bytes
- WHEN replay compares run outcomes
- THEN replay fails with divergence evidence and issues no normal success receipt.

#### Scenario: One replay run fails

- GIVEN one selected run errors, times out, or exceeds a stream bound
- WHEN replay completes its fail-closed decision
- THEN the failure is recorded and agreement from other runs cannot authorize success.

#### Scenario: Replay report is reproduced

- GIVEN the same captured plan, evaluator artifact, profiles, and ordered outcomes
- WHEN replay evidence is rendered again
- THEN its canonical bytes are identical and contain no clock, process ID, or machine-local path.
