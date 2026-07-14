# Canonical schemas

## Request: `onix-nickel-export-request/v1`

A request records `family_id`, repository-relative `source`, complete declared `dependencies`, evaluator `import_paths`, optional `selector`, optional consumer-owned `contract` metadata, native `format`, repository-relative `destination`, and explicit `allow_secret_material` policy. Artifact paths are normalized, parent traversal and absolute paths are rejected, and path lists are sorted and deduplicated. The external CLI interprets non-empty contract metadata as a repository-relative contract file and requires that file in the dependency set; embedded adapters may use a stable contract label and supply captured diagnostics directly.

## Declared input: `onix-nickel-export-declared-input/v1`

`declared_input_identity` is BLAKE3 over a versioned, length-delimited canonical
encoding of:

- normalized source path, exact source identity, and byte length;
- sorted dependency paths, exact identities, and byte lengths;
- sorted import paths, selector, contract metadata, and output format;
- normalized evaluator label, exact artifact identity, optional closure identity,
  typed plan identity, version, sorted non-duplicate options, and
  dependency-observation policy.

Consumer `family_id`, destination, output bytes, diagnostics, and secret opt-in
policy are deliberately excluded because they do not change the declared
Nickel evaluation. The identity can therefore be computed before evaluation
and remains the same when the same evaluation is materialized at another
location.

Under `declared_only` or `snapshot_only`, this is explicitly a fingerprint of
declared material, not proof of a complete dependency closure and not a safe
cache key. `snapshot_only` additionally proves that the CLI evaluated a private
copy of the declared bytes with ambient environment variables removed, but it
does not claim sandbox confinement. The core provides
`build_declared_input_identity` for consumers that need the identity before
output bytes exist.

## Resource limits

The CLI embeds the checked export of `config/resource-limits.ncl`. Its typed
profile bounds artifact counts and bytes, evaluator executable and stderr
bytes, path and option lengths, diagnostic bytes, evaluator deadline, and
status polling. The BLAKE3 profile identity is carried in the canonical typed
execution plan. Limit violations, timeouts, and integer conversion overflow
fail before receipt creation.

## Diagnostic: `onix-nickel-export-diagnostic/v1`

Diagnostics contain a stable `class`, `subject`, human-readable `message`, and `note`, `warning`, or `error` severity. Any error diagnostic prevents receipt creation. Contract meaning remains consumer-owned.

## Shell failure: `onix-nickel-export-shell-error/v1`

The CLI emits one JSON object on stderr with a stable `stage` and human-readable `message`. Evaluator spawn/version/execution failures, unsafe paths, source failures, admission failures, stale artifacts, serialization failures, and write failures never include a successful receipt.

## Receipt: `onix-nickel-export-receipt/v2`

A receipt binds:

- the versioned `declared_input_identity` described above;
- exact source, dependency, and output bytes with `b3:` BLAKE3 identities and byte lengths;
- family, selector, contract, format, import paths, and destination;
- evaluator label, exact artifact BLAKE3 identity, optional adapter-verified closure identity, typed plan identity, version metadata, sorted non-duplicate options, and dependency-observation policy;
- sorted non-fatal diagnostics and an explicit non-claim.

The core rejects source/output path mismatches, incomplete dependency material, undeclared observed imports, incomplete evaluator-observed closures, error diagnostics, unsafe artifact paths, malformed identities, weakened non-claims, and conservative secret markers in authored source/dependencies without opt-in. Versioned request, receipt, manifest, artifact, diagnostic, evaluator, and resource-profile records reject unknown fields. Deserialization produces wire values; pure admission produces opaque `AdmittedReceipt` and `VerifiedManifest` states, and compatibility projections require those admitted states.

## Manifest: `onix-nickel-export-manifest/v2`

A manifest contains receipts sorted by destination and one shared evaluator descriptor. Duplicate destinations and mixed evaluator descriptors are rejected. `manifest_identity` is BLAKE3 over the canonical manifest payload before the identity field is attached. Freshness is exact equality against a newly derived manifest.

## Compatibility projections

`project_octet_manifest` emits `octet-nickel-export-manifest/v1`, including bare hexadecimal BLAKE3 values and replay command fields. `project_mantle_receipt` emits `mantle-nickel-export-receipt-v1`. Projections are intentionally one-way: canonical admission occurs first, then the legacy shape is derived.
