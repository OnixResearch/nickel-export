# Export performance and resource limits

## Accepted mechanisms

The shell borrows captured input bytes rather than copy every file. The default aggregate input budget is 128 MiB. Each read uses the smaller of its per-file budget and the remaining aggregate budget. Integrity checks apply the same aggregate policy to supplied files.

Replay consumes one result at a time. It retains the first output and compares subsequent exact bytes against it. The report still includes each selected outcome. A divergence or failed run produces no agreed output or success receipt.

Executable identity uses bounded streaming BLAKE3. Tests compare this identity against the existing whole-buffer function for empty and multichunk files. Artifact-change checks remain in place. Hash checks are not an executable-closure proof.

Artifact verification uses a path index for larger manifests. Manifests with at most 16 exports retain direct scans because index construction cost more in that measured range. Both paths reject conflicting identities for the same supplied path.

## Shared process mechanism

The shell uses Bounded Exec at immutable revision `29dac88ecded94457572db3fdfaaaab95fa91525`:

`https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git`

Version probes and evaluation use null stdin, a clear environment, stream limits, a process deadline, and owned teardown. The default deadline is 30 seconds per process. The default teardown budget is 500 milliseconds. An export can run several processes, so these limits are not a total export deadline.

The shell retains executable identity, version admission, redaction, snapshot policy, and receipt authority. Evaluator failures report bounded counts and status rather than raw stdout or stderr. Bounded Exec does not provide a filesystem sandbox. Unix process-group teardown does not establish equivalent descendant guarantees on every platform.

Nix records the dependency NAR with SHA-256 because that fetch protocol requires it. Export content identities remain BLAKE3.

## Reproduce measurements

Run commands from this repository:

```sh
nix develop -c cargo test --release --workspace measure_ -- --ignored --nocapture
```

The capture measurement compares a source copy with borrowed capture metadata over the same fixed input. The replay control proves that the reference buffer survives without a clone. Its retained-buffer count describes the implementation, not a heap-profiler observation.

The manifest measurement compares the old full scan with production verification. It includes exact-byte agreement and rejected-content controls. Timings include index construction. They exclude manifest construction, Nickel evaluation, and disk I/O.

Checked-in measurement logs record scoped observations. They do not establish an end-to-end export speedup. The maintainers own benchmark updates and regression controls.

## Recorded service-configuration sample

The [CLI measurement](measurements/cli.json) compares baseline `710b6e03a8faf74f6afc6cc8ef36d7667958b70d` against this implementation. Both use Nix-built binaries and the same Nickel executable. Each sample includes `--write`, filesystem synchronization, and an exact output comparison.

The local sample used one warmup and eight measured runs per binary. Mean time changed from 62.8 ms to 59.4 ms. This is a small local gain, not a general latency guarantee. The [mechanism measurements](measurements/mechanisms.log) cover the larger manifest gain separately.

To repeat the CLI comparison, copy `examples/service-config/` into a private benchmark root. Keep an unchanged copy of `generated/service.json` as the expected output. Run each binary with these arguments:

```sh
export --spec examples/service-config/request.json --root "$BENCH_ROOT" \
  --evaluator "$EVALUATOR" --evaluator-identity nixpkgs:nickel \
  --evaluator-version nickel-lang-cli-1.17.0 \
  --manifest examples/service-config/generated/manifest.json --write
```

Each measured command then uses `cmp` against the expected output. Hyperfine records the timings with `--warmup 1 --runs 8 --export-json cli.json`. The benchmark does not alter the source checkout or measure dependency downloads.

## Reuse decisions

`snapshot_only` remains unsafe as a result-cache key. The snapshot contains captured declared files, but the external evaluator can read outside that snapshot. Repeated declared identity does not justify a cache hit.

An embedded `nickel-eval` adapter also needs a published pinned contract, exact format parity, enforced input closure, and consumer-owned admission. The closed evaluator currently rejects host imports and uses JSON output. It is not an equivalent replacement for the CLI import and native-format contract.

The existing read-only `verify --check-artifacts` command remains the inexpensive integrity-only operation. It does not claim fresh evaluation.
