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
profile bounds artifact counts and bytes, sequential replay runs, evaluator
executable and stderr bytes, path and option lengths, diagnostic bytes,
evaluator deadline, and status polling. The BLAKE3 profile identity is carried in the canonical typed
execution plan. Limit violations, timeouts, and integer conversion overflow
fail before receipt creation.

## Diagnostic: `onix-nickel-export-diagnostic/v1`

Diagnostics contain a stable `class`, `subject`, human-readable `message`, and `note`, `warning`, or `error` severity. Any error diagnostic prevents receipt creation. Contract meaning remains consumer-owned.

## Shell failure: `onix-nickel-export-shell-error/v1`

The CLI emits one JSON object on stderr with a stable `stage` and human-readable `message`. Evaluator spawn/version/execution failures, unsafe paths, source failures, admission failures, stale artifacts, serialization failures, and write failures never include a successful receipt. Replay divergence and run failures additionally carry the deterministic replay report in a `replay` field.

## Replay report: `onix-nickel-export-replay-report/v1`

An explicit `--replay-runs` profile executes one captured snapshot and canonical evaluator plan sequentially within the configured run bound. The report binds the shared plan, evaluator artifact, and resource-profile identities to ordered run statuses, exact output BLAKE3 identities, byte lengths, a fail-closed verdict, and a BLAKE3 identity over schema-owned canonical report bytes. Agreement reports precede the ordinary receipt on stdout. Divergence, failure, timeout, and output overflow produce a shell error carrying the report and no success receipt. Reports contain no clock, process ID, snapshot path, or ambient run identifier. Agreement is empirical evidence for only the selected runs, not proof of future or universal determinism.

## Receipt: `onix-nickel-export-receipt/v3`

A receipt binds:

- `receipt_identity`, BLAKE3 over `onix-nickel-export-receipt-identity/v1` canonical bytes;
- the versioned `declared_input_identity` described above;
- exact source, dependency, and output bytes with `b3:` BLAKE3 identities and byte lengths;
- family, selector, contract, format, import paths, and destination;
- evaluator label, exact artifact BLAKE3 identity, optional adapter-verified closure identity, typed plan identity, version metadata, sorted non-duplicate options, and dependency-observation policy;
- sorted non-fatal diagnostics and an explicit non-claim.

The core rejects source/output path mismatches, incomplete dependency material, undeclared observed imports, incomplete evaluator-observed closures, error diagnostics, unsafe artifact paths, malformed identities, weakened non-claims, and conservative secret markers in authored source/dependencies without opt-in. Versioned request, receipt, manifest, artifact, diagnostic, evaluator, and resource-profile records reject unknown fields. Deserialization produces wire values; pure admission produces opaque `AdmittedReceipt` and `VerifiedManifest` states, and compatibility projections require those admitted states.

## Manifest: `onix-nickel-export-manifest/v3`

A manifest contains receipts sorted by destination and one shared evaluator descriptor. Duplicate destinations and mixed evaluator descriptors are rejected. `manifest_identity` is BLAKE3 over `onix-nickel-export-manifest-identity/v1` canonical bytes before the identity field is attached. Receipt and manifest canonical bytes use checked big-endian lengths and list counts; pretty JSON remains only the human-facing wire representation. Freshness is exact equality against a newly derived manifest.

## Materialization transaction: `onix-nickel-export-materialization-transaction/v1`

Write and check modes coordinate through a repository lock. Write mode stages
and synchronizes output and manifest bytes, then records their temporary and
destination paths plus BLAKE3 identities in a durable transaction marker. A
check rejects any extant marker. A later write deterministically completes a
matching interrupted transaction before starting another. This is
crash-consistent and fail-closed; it does not claim portable atomic replacement
of two unrelated paths. Consumers that support generation directories can use
one atomically replaced pointer file instead.

## Integrity report: `onix-nickel-export-integrity-report/v1`

`nickel-export verify` strictly decodes and admits a stored manifest, recomputes
receipt and manifest canonical identities, and checks schema, path, hash,
non-claim, evaluator-cohort, ordering, and uniqueness invariants without
invoking Nickel. With `--check-artifacts`, it also reads every referenced source,
dependency, and output and verifies exact BLAKE3 identities and byte lengths.
The report explicitly does not claim freshness for absent bytes, evaluator
correctness, or semantic correctness.

## Compatibility projections

`project_octet_manifest` emits `octet-nickel-export-manifest/v1`, including bare hexadecimal BLAKE3 values and replay command fields. `project_mantle_receipt` emits `mantle-nickel-export-receipt-v1`. Projections are intentionally one-way: canonical admission occurs first, then the legacy shape is derived.
