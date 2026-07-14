# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Receipts bind the evaluator artifact and execution plan

r[nickel_export.shell.evaluator_execution_identity]
The shell MUST resolve one evaluator artifact for version and export execution, MUST bind its exact BLAKE3 identity and canonical typed execution plan into the receipt, and MUST reject ambiguous or conflicting evaluator options. Stronger closure identities MUST identify their supplying evidence class.

#### Scenario: Evaluator artifact changes

- GIVEN an evaluator was resolved and identified
- WHEN its executable differs before export execution
- THEN execution fails without issuing a receipt.

#### Scenario: Closure evidence is unavailable

- GIVEN only exact executable bytes can be identified
- WHEN the evaluator descriptor is built
- THEN it records artifact identity without claiming dynamic-library or package closure identity.

#### Scenario: Option is duplicated or conflicting

- GIVEN a plan contains duplicate or conflicting evaluator semantics
- WHEN plan validation runs
- THEN validation fails rather than silently sorting or deduplicating the options.

#### Scenario: Receipt plan differs from process arguments

- GIVEN a canonical typed execution plan
- WHEN process arguments and receipt evidence are rendered
- THEN both are derived from that same plan and cannot diverge independently.
