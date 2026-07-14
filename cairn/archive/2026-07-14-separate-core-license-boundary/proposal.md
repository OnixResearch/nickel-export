## Why

`nickel-export-core` is an evaluator-neutral `no_std` contract library intended for embedding, while `nickel-export` is the filesystem and external-evaluator shell. The current repository-wide AGPL declaration unnecessarily applies strong copyleft to the reusable core.

## What Changes

- License `nickel-export-core` under MPL-2.0.
- Keep the `nickel-export` shell and repository application under `AGPL-3.0-or-later`.
- Extend typed Nickel repository/release profiles with package-specific license metadata and refresh generated exports.
- Add complete MPL and AGPL texts plus an explicit package map.

## Impact

Core modifications remain file-level copyleft, larger embedding applications may use their own license, and the operator shell retains AGPL network reciprocity.
