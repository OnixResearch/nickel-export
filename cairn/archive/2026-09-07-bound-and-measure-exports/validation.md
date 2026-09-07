# Validation

## Scope and tests

Implementation commit: `6008bcc`, with later test and Nix integration corrections. The branch preserves the existing detached-checkout documentation commits through `0de12cd` by merge.

Baseline `710b6e03a8faf74f6afc6cc8ef36d7667958b70d` passed 42 tests in its Nix package build. Final workspace tests passed 52 tests. Two measurement tests remain ignored in the ordinary suite and passed through an explicit release-mode invocation.

Formatting, all-target Clippy with `-D warnings`, no-default-feature Wasm compilation, and fuzz-target compilation passed. `nix flake check -L` passed all 11 project checks on x86_64-linux. The Verus rail verified 30 obligations and rejected the negative fixture. CLI checks covered native formats, replay, wrong versions, unsafe paths, and modified generated output.

The first full Nix attempt found dangling new requirement links and six pre-existing missing license links. Accepted-spec sync resolved the new links. A real license-boundary check now supplies the older evidence links. The fuzz lockfile also needed Cargo regeneration after the shared dependency change. No flake lockfile was edited manually.

## Mechanism checks

Positive and negative tests cover the version probe, output limits, stdin EOF, descendant teardown, raw diagnostic suppression, aggregate input overflow, exact streaming hashes, borrowed buffers, replay divergence, and conflicting manifest paths.

The final traceability report covers 29 of 29 accepted requirements without missing or dangling IDs. Canonical receipt known-answer tests remain unchanged. The resource profile changed deliberately, so the CLI regenerated fixture manifests with new plan identities and unchanged output bytes.

## Measurements

`docs/measurements/mechanisms.log` records the capture, replay, and manifest experiments. The large manifest sample reduced verification time from 180.8 ms to 15.5 ms across 32 runs. This sample contains 1,024 exports and excludes evaluation and I/O.

`docs/measurements/cli.json` records eight measured service-configuration exports per binary after one warmup. Each invocation includes an exact output comparison. Mean elapsed time changed from 62.8 ms to 59.4 ms. This is a scoped local observation, not a general speed guarantee.

`snapshot_only` still does not qualify for result caching. The closed JSON evaluator is not an equivalent replacement for the CLI import and native-format boundary. No unsafe cache or implicit evaluator replacement was added.
