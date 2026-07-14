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

### Requirement: Only admitted evidence reaches canonical consumers

r[nickel_export.core.admitted_evidence_types]
The core MUST decode untrusted serialized values into non-authoritative wire types, MUST reject unknown fields and invariant violations, and MUST expose opaque admitted receipt and verified manifest types to manifest construction, freshness checks, and compatibility projections.

#### Scenario: Canonical evidence is valid

- GIVEN a supported receipt or manifest with all required invariants
- WHEN pure admission runs
- THEN it produces an opaque admitted value with read-only accessors.

#### Scenario: Unknown nested field is present

- GIVEN a request, evaluator, artifact, receipt, or manifest contains an unknown field
- WHEN strict decoding runs
- THEN decoding fails rather than discarding the field.

#### Scenario: Caller fabricates evidence fields

- GIVEN a wire receipt has a weakened non-claim, malformed identity, unsafe path, or inconsistent declared-input identity
- WHEN admission runs
- THEN no admitted receipt is produced.

#### Scenario: Projection receives unchecked data

- GIVEN an untrusted wire receipt or manifest
- WHEN a compatibility projection is requested
- THEN the public API requires successful admission before projection.

### Requirement: Evidence identities use schema-owned canonical bytes

r[nickel_export.core.canonical_evidence_encoding]
The core MUST encode admitted receipt and manifest identity preimages with a versioned, length-delimited, schema-owned binary encoding, MUST hash those bytes with BLAKE3, and MUST expose the canonical bytes for independent verification. Human-facing JSON serialization MUST NOT define cryptographic identity.

#### Scenario: JSON renderer changes

- GIVEN one admitted receipt or manifest
- WHEN JSON whitespace, escaping, or serializer implementation changes without changing admitted fields
- THEN its canonical identity remains unchanged.

#### Scenario: Canonical field changes

- GIVEN one canonical evidence value
- WHEN any identity-bearing field, enum tag, list count, or list element changes
- THEN its canonical bytes and BLAKE3 identity change.

#### Scenario: Known-answer vector is checked

- GIVEN a checked versioned vector containing structured fields, canonical bytes, and expected BLAKE3 identity
- WHEN the core and independent verifier evaluate it
- THEN both produce the exact checked bytes and identity.

#### Scenario: Length cannot be represented

- GIVEN a field or list length exceeds the canonical integer representation
- WHEN encoding runs
- THEN encoding fails rather than saturating or truncating the length.

### Requirement: Stored manifests support self-contained integrity verification

r[nickel_export.core.manifest_integrity_verification]
The core MUST provide pure verification that recomputes every derivable canonical identity and validates schema, path, hash, non-claim, evaluator-cohort, ordering, and uniqueness invariants without invoking Nickel. Optional artifact-byte verification MUST require the exact bytes being checked.

#### Scenario: Stored manifest is internally coherent

- GIVEN a supported manifest whose canonical fields and identities are valid
- WHEN integrity verification runs
- THEN it returns a verified manifest and explicitly makes no freshness or semantic-correctness claim.

#### Scenario: Identity or invariant is tampered

- GIVEN a manifest contains a changed identity, malformed BLAKE3 value, duplicate output, mixed evaluator, unsafe path, or weakened non-claim
- WHEN integrity verification runs
- THEN verification fails with deterministic staged diagnostics.

#### Scenario: Artifact bytes are supplied

- GIVEN exact source, dependency, or output bytes are supplied for a manifest artifact
- WHEN optional byte verification runs
- THEN the verifier recomputes and compares that artifact identity and byte length.

#### Scenario: Read-only CLI verification runs

- GIVEN a manifest path and optional artifact paths
- WHEN `nickel-export verify` runs
- THEN it performs no Nickel execution and no filesystem writes.

### Requirement: Artifact publication is crash-consistent and concurrency-safe

r[nickel_export.shell.atomic_materialization]
Write and check modes MUST coordinate through an exclusive repository lock, writes MUST stage and durably publish complete individual files, and a durable transaction marker MUST make any interrupted multi-file publication fail closed. The shell MUST NOT claim portable atomic replacement of unrelated filesystem paths.

#### Scenario: Publication succeeds

- GIVEN rendered output and manifest bytes and an acquired lock
- WHEN staging, synchronization, publication, and marker removal complete
- THEN subsequent check mode observes the complete matching generation.

#### Scenario: Crash occurs between artifact renames

- GIVEN a transaction marker was published
- WHEN the process stops after only one destination was replaced
- THEN the marker remains and check mode rejects the incomplete generation.

#### Scenario: Concurrent check or writer starts

- GIVEN another check or write operation owns the repository lock
- WHEN a second operation starts
- THEN it fails with a structured lock diagnostic and performs no mutation.

#### Scenario: Recovery is requested

- GIVEN an interrupted transaction and staged bytes matching the recorded identities
- WHEN recovery runs repeatedly
- THEN it deterministically completes the recorded generation or leaves destinations untouched with the same failure classification.

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
