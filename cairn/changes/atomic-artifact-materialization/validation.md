# Validation evidence

- Independent integrity verification supplied a passing workspace, strict Clippy, CLI, and Nix baseline.
- Write and check modes acquire one repository lock; live-owner contention fails before mutation and dead Linux process locks are removed within a bounded acquisition loop.
- Output and manifest bytes are staged with exclusive creation, synchronized, and published through same-filesystem rename.
- A strict durable transaction marker records temporary/destination paths and BLAKE3 identities before either rename; check mode rejects any extant marker.
- Pure recovery decisions distinguish publishable staged bytes, already-published destinations, and invalid/missing states.
- Automatic write recovery is idempotent and retains the marker on irrecoverable identity mismatch.
- `publish_generation_pointer` offers single-file atomic publication for consumers using complete generation directories.
- Positive publication/recovery/pointer tests and negative contention/incomplete/mismatch tests pass, as do formatting, workspace tests, strict Clippy, and real CLI end-to-end checks.
