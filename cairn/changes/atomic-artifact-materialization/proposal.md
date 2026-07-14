# Proposal: Atomic artifact materialization

## Summary

Make write and check modes concurrency-safe and crash-consistent with repository locking, same-directory staging, durable renames, and an explicit transaction marker.

## Motivation

Write mode currently writes the generated output and then the manifest directly. A write failure or crash can leave a mixed generation. Check mode reads both files without a lock, so a concurrent writer can change them during verification. Two independent destination paths cannot be updated as one portable filesystem atomic operation, so the system needs an explicit fail-closed transaction protocol.

## Scope

- Acquire a repository-scoped export lock for write and check operations.
- Render all bytes before mutation.
- Stage files with exclusive creation in their destination directories.
- Flush staged bytes and directories before rename.
- Record an in-progress transaction marker before publishing either artifact.
- Reject checks while a transaction marker exists.
- Recover or report interrupted transactions deterministically.
- Offer generation-directory plus atomic-pointer mode where consumers support it.

## Non-Goals

- Claiming portable atomic replacement of unrelated filesystem paths.
- Providing a distributed lock across machines.
- Hiding interrupted writes as successful generations.

## Impact

- **Shell**: write/check orchestration and recovery behavior change.
- **Filesystem**: bounded lock, staging, and transaction metadata are introduced.
- **Testing**: fault injection covers every mutation boundary and concurrent access.
