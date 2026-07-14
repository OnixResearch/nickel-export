# Canonical schemas

## Request: `onix-nickel-export-request/v1`

A request records `family_id`, repository-relative `source`, complete declared `dependencies`, evaluator `import_paths`, optional `selector`, optional repository-relative `contract`, native `format`, repository-relative `destination`, and explicit `allow_secret_material` policy. Paths are normalized, parent traversal and absolute paths are rejected, lists are sorted and deduplicated, and a contract must be included in the dependency set.

## Diagnostic: `onix-nickel-export-diagnostic/v1`

Diagnostics contain a stable `class`, `subject`, human-readable `message`, and `note`, `warning`, or `error` severity. Any error diagnostic prevents receipt creation. Contract meaning remains consumer-owned.

## Shell failure: `onix-nickel-export-shell-error/v1`

The CLI emits one JSON object on stderr with a stable `stage` and human-readable `message`. Evaluator spawn/version/execution failures, unsafe paths, source failures, admission failures, stale artifacts, serialization failures, and write failures never include a successful receipt.

## Receipt: `onix-nickel-export-receipt/v1`

A receipt binds:

- exact source, dependency, and output bytes with `b3:` BLAKE3 identities and byte lengths;
- family, selector, contract, format, import paths, and destination;
- evaluator identity, exact version/package identity, sorted options, and dependency-observation policy;
- sorted non-fatal diagnostics and an explicit non-claim.

The core rejects source/output path mismatches, incomplete dependency material, undeclared observed imports, incomplete evaluator-observed closures, error diagnostics, unsafe paths, and conservative secret markers without opt-in.

## Manifest: `onix-nickel-export-manifest/v1`

A manifest contains receipts sorted by destination and one shared evaluator descriptor. Duplicate destinations and mixed evaluator descriptors are rejected. `manifest_identity` is BLAKE3 over the canonical manifest payload before the identity field is attached. Freshness is exact equality against a newly derived manifest.

## Compatibility projections

`project_octet_manifest` emits `octet-nickel-export-manifest/v1`, including bare hexadecimal BLAKE3 values and replay command fields. `project_mantle_receipt` emits `mantle-nickel-export-receipt-v1`. Projections are intentionally one-way: canonical admission occurs first, then the legacy shape is derived.
