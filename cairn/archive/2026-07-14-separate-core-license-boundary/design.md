## Context

The no-std core and std shell already form a compiler-enforced authority boundary. The license split follows that architecture and is represented in Nickel-owned release configuration rather than as untracked prose only.

## Decisions

### Use MPL-2.0 for the core

`nickel-export-core` explicitly declares `MPL-2.0`. MPL file-level copyleft preserves modifications to the core without imposing AGPL on a larger linked work.

### Keep the shell AGPL

The workspace default and `nickel-export` package remain `AGPL-3.0-or-later`.

### Make package licenses typed

`config/repository.ncl` remains the source of truth and gains a package-license mapping validated by its Nickel contract. `release/profile.ncl` exports the mapping, and checked JSON projections are refreshed. The existing repository-level `license` remains for compatibility.

### Keep claims bounded

This change does not change evaluator semantics, canonical receipts, or previous license grants and does not relicense dependencies.
