# Proposal: Bound evaluator execution

## Summary

Make evaluator time, memory-facing byte streams, artifact sizes, path lengths, option lengths, and canonical integer conversions explicit and fail-closed.

## Motivation

The shell currently uses `Command::output`, which has no deadline and buffers stdout and stderr without project bounds. The core bounds artifact counts but not individual paths, options, diagnostics, or artifact bytes, and some length conversions saturate at `u64::MAX`. These behaviors weaken totality, denial-of-service resistance, and proof claims.

## Scope

- Define a typed `ResourceLimits` contract with named limits and checked defaults.
- Bound source, dependency, output, stderr, path, option, diagnostic, and manifest sizes.
- Add evaluator deadline, termination, and child reaping.
- Replace saturating length conversion with explicit overflow errors.
- Stream or incrementally scan large material where practical.
- Record applied limit profile identity in shell evidence.

## Non-Goals

- Claiming exact operating-system memory or CPU accounting.
- Replacing Nix or Mantle sandbox resource controls.
- Treating a timeout as evidence that Nickel is incorrect.

## Impact

- **Configuration**: human-authored limit profiles use typed Nickel and checked exports.
- **Core**: size-overflow and bound diagnostics become explicit.
- **Shell**: bounded process supervision replaces unbounded `output()` collection.
