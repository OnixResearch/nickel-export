# Deterministic Nickel Export Specification

## Purpose

Provide evaluator-neutral, exact-byte Nickel export identities, receipts, compatibility projections, and freshness checks without taking evaluator, filesystem, consumer-policy, or release authority.

## Requirements

### Requirement: The shared core is evaluator-neutral

r[nickel_export.core.evaluator_neutral] `nickel-export-core` MUST compile with `#![no_std]` plus `alloc` and MUST NOT read files, inspect environment state, spawn processes, print, consult clocks, perform network I/O, or evaluate Nickel.

#### Scenario: Core is built for Wasm
- GIVEN default and serialization features are disabled
- WHEN the core is checked for `wasm32-unknown-unknown`
- THEN it MUST compile without std or evaluator dependencies.

### Requirement: Exact identities bind complete declared material

r[nickel_export.core.identity] The core MUST bind exact source, complete declared dependency, selector, consumer-owned contract metadata, evaluator descriptor, output format, destination, and output bytes into versioned receipts using BLAKE3 for project-owned identities.

#### Scenario: Any exact artifact changes
- GIVEN an admitted receipt input
- WHEN one source, dependency, or output byte changes
- THEN the corresponding identity and canonical receipt MUST change.

#### Scenario: Dependency material is incomplete
- GIVEN a request names a dependency with no matching exact material
- WHEN receipt admission runs
- THEN admission MUST fail closed.

### Requirement: Evaluation and contract failures produce no receipt

r[nickel_export.core.fail_closed] The core MUST reject evaluator or contract error diagnostics, undeclared observed dependencies, incomplete evaluator-observed closures, unsafe paths, mixed evaluator manifests, duplicate destinations, and conservative secret-like material without explicit opt-in.

#### Scenario: Contract returns an error diagnostic
- GIVEN exact output bytes and a contract error
- WHEN receipt admission runs
- THEN no successful receipt MUST be produced.

#### Scenario: Manifest mixes evaluator cohorts
- GIVEN individually valid receipts from different evaluator descriptors
- WHEN manifest construction runs
- THEN manifest construction MUST fail.

### Requirement: Filesystem and evaluator authority stay in a thin shell

r[nickel_export.shell.authority] The std CLI MUST keep file reads, explicit external Nickel execution, output writes, process diagnostics, and check-mode orchestration outside the pure core.

#### Scenario: Check mode sees tampered output
- GIVEN a valid checked-in output and manifest
- WHEN output bytes are changed
- THEN `nickel-export ... --check` MUST fail without rewriting either artifact.

#### Scenario: Request path escapes the root
- GIVEN an absolute or parent-traversing request path
- WHEN the CLI validates the request
- THEN evaluation and output writes MUST NOT occur.

### Requirement: Compatibility does not create competing semantic owners

r[nickel_export.compat.projections] The core MUST provide versioned one-way Octet and Mantle compatibility projections while consumer evaluator semantics, destination authority, policy, lifecycle gates, and release authority remain consumer-owned.

#### Scenario: Legacy projection is requested
- GIVEN a canonical admitted receipt or manifest
- WHEN an Octet or Mantle projection is derived
- THEN exact identities and non-claims MUST be preserved without re-evaluating Nickel.

### Requirement: Consumer cutover is pinned and reversible

r[nickel_export.migration.dual_run] Consumers MUST pin an immutable standalone revision, dual-run legacy and canonical paths over identical material, classify all differences, retain rollback, and prohibit workspace-relative release dependencies.

#### Scenario: Dual-run results diverge
- GIVEN legacy and canonical output or identity evidence differs
- WHEN migration validation runs
- THEN the legacy path MUST remain authoritative until an owning regression fixture explains the difference.

### Requirement: Release evidence is reproducible and bounded

r[nickel_export.release.profile] The repository MUST publish typed configuration, checked exports, pinned Nix and Rust inputs, exact evaluator cohort, positive and negative fixtures, host and Wasm core checks, CLI tamper tests, license metadata, and explicit non-claims.

#### Scenario: Typed export is stale
- GIVEN repository or release Nickel source differs from its checked JSON export
- WHEN Nix release validation runs
- THEN validation MUST fail closed.

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
