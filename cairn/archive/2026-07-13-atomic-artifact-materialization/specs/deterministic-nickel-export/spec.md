# Deterministic Nickel Export Delta

## ADDED Requirements

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
