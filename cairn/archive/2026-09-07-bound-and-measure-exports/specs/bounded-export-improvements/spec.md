# Bounded export improvements

## ADDED Requirements

### Requirement: Bounded process lifetime

r[nickel_export.improvements.process]
The shell MUST apply byte, deadline, stdin, and owned teardown policy to version probes and evaluation. It MUST reject truncated or timed-out output without a success receipt.

#### Scenario: Hanging version probe
GIVEN an evaluator that hangs or exceeds its stream budget during its version probe
WHEN the shell reaches the declared limit
THEN it stops owned work and produces no success receipt.

### Requirement: Aggregate captured-input limit

r[nickel_export.improvements.aggregate]
The shell MUST enforce a checked aggregate input-byte budget before retaining additional captured content. It MUST reject arithmetic overflow and budget excess.

#### Scenario: Individually valid files exceed total capacity
GIVEN files that each fit the per-file budget but exceed the aggregate budget
WHEN input capture runs
THEN it rejects the request before evaluation.

### Requirement: Exact bounded-memory mechanisms

r[nickel_export.improvements.memory]
The shell MUST avoid full copies of captured input and retain only bounded reference and current replay bytes. Streaming executable hashing MUST preserve the exact BLAKE3 identity and artifact-change checks.

#### Scenario: Replay divergence
GIVEN replay runs with different exact output bytes
WHEN incremental assessment runs
THEN it records divergence and withholds agreed output and the success receipt.

### Requirement: Indexed artifact consistency

r[nickel_export.improvements.index]
Artifact verification MUST reject unknown paths, mismatched bytes, and conflicting expected identities for a supplied path. An index MUST preserve these checks.

#### Scenario: Same path has conflicting identities
GIVEN a verified manifest with distinct expected identities for a shared path
WHEN supplied artifact verification runs
THEN it rejects content that cannot satisfy every expected identity.

### Requirement: Measured optimization without cache escalation

r[nickel_export.improvements.measurement]
The repository MUST record reproducible benchmark observations and exact-output controls. The shell MUST NOT treat `snapshot_only` as a safe result-cache key.

#### Scenario: Declared identity repeats
GIVEN repeated declared input identity without an enforced full read closure
WHEN the export command runs again
THEN it evaluates rather than accepting a result-cache hit.
