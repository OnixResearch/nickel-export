# Deterministic Nickel Export Delta

## ADDED Requirements

### Requirement: Evaluations consume captured declared inputs

r[nickel_export.shell.captured_input_evaluation]
The shell MUST evaluate an exact captured snapshot of normalized source and declared dependency bytes, MUST remove ambient import authority, and MUST distinguish snapshot-only execution from sandbox-confined closure evidence. It MUST NOT issue a receipt for unsupported package or filesystem escape behavior.

#### Scenario: Repository input changes after capture

- GIVEN exact source and dependency bytes have been captured
- WHEN the corresponding repository files change before Nickel runs
- THEN Nickel evaluates the captured snapshot rather than the changed repository files.

#### Scenario: Ambient import path is present

- GIVEN `NICKEL_IMPORT_PATH` names an undeclared import
- WHEN snapshot evaluation runs
- THEN the ambient import path is unavailable and no receipt can depend on it.

#### Scenario: Package material is undeclared

- GIVEN a Nickel source requires package lock or cache material not present in the captured plan
- WHEN evaluation runs
- THEN evaluation fails without issuing a receipt.

#### Scenario: Snapshot lacks filesystem confinement

- GIVEN evaluation occurs in a private snapshot without a sandbox that denies outside reads
- WHEN a receipt is created
- THEN the receipt MUST NOT claim a complete evaluator-observed or sandbox-confined closure.
