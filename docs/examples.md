# Worked examples

## Example: a service that only reads JSON

Imagine a service that reads `service.json` when it starts. People maintain the
configuration in Nickel because they want names, contracts, and early errors,
but the service does not embed a Nickel evaluator.

The files for this example live in [`examples/service-config/`](../examples/service-config/).

Nickel by itself can produce the runtime file:

```console
nickel export --format json examples/service-config/service.ncl
```

That answers **“what JSON does this evaluate to now?”** This repository adds a
checked-in manifest and answers **“does the generated JSON still match these
exact reviewed inputs and this evaluator description?”**

The project commits the Nickel source, contract, export request, generated JSON,
and manifest together. The running service only needs the generated JSON.

### 1. Write the human-owned Nickel source

[`service.ncl`](../examples/service-config/service.ncl) is the source people
review and edit:

```nickel
let LocalDevelopmentPort = 8080 in
{
  service = "payments-api",
  listen = {
    address = "127.0.0.1",
    port = LocalDevelopmentPort,
  },
  logging = {
    format = "json",
    include_request_ids = true,
  },
}
```

[`contract.ncl`](../examples/service-config/contract.ncl) describes the shape
that the service expects. Nickel rejects the export if, for example, `port` is
a string instead of a number.

### 2. Declare the export

[`request.json`](../examples/service-config/request.json) says:

- evaluate `service.ncl`;
- validate it with `contract.ncl`;
- produce JSON at `generated/service.json`;
- treat the contract as an exact dependency;
- reject secret-like source material.

The request does not contain the service's policy. It only describes how to
produce and identify the artifact.

### 3. Generate the artifact and receipt

From the repository root:

```console
nix develop -c cargo run --quiet -p nickel-export -- export \
  --spec examples/service-config/request.json \
  --root . \
  --evaluator nickel \
  --evaluator-identity nixpkgs:nickel \
  --evaluator-version nickel-lang-cli-1.17.0 \
  --manifest examples/service-config/generated/manifest.json \
  --write
```

This writes two files:

1. [`generated/service.json`](../examples/service-config/generated/service.json),
   which the service can read;
2. [`generated/manifest.json`](../examples/service-config/generated/manifest.json),
   which records the exact source, contract, output, and evaluator identities.

The command also prints the individual receipt. The important part is the
relationship it records:

```text
exact service.ncl bytes
+ exact contract.ncl bytes
+ export options
+ recorded Nickel evaluator
= declared_input_identity

then the receipt binds:
declared_input_identity -> exact service.json bytes
```

The declared identity does not include the output or its destination. Two runs
with the same declared identity but different output identities are therefore
a useful warning about hidden inputs or nondeterministic evaluation. Under the
CLI's `declared_only` policy it is not a safe cache key because Nickel did not
report its complete observed import closure.

The manifest does not claim that the service is correct or safe. It only binds
those exact declared inputs and output under the recorded evaluator
description.

### 4. Check freshness in CI

Use the same command with `--check` instead of `--write`:

```console
nix develop -c cargo run --quiet -p nickel-export -- export \
  --spec examples/service-config/request.json \
  --root . \
  --evaluator nickel \
  --evaluator-identity nixpkgs:nickel \
  --evaluator-version nickel-lang-cli-1.17.0 \
  --manifest examples/service-config/generated/manifest.json \
  --check
```

`--check` evaluates the source again but does not write anything. It compares
the newly produced bytes and manifest with the checked-in files.

| Change | Result |
|---|---|
| Nothing changed | The check passes. |
| Someone edits `service.ncl` and its evaluated output changes | The generated JSON is reported as stale. |
| Someone makes a source-only change that produces identical JSON | The manifest is reported as stale because the source identity changed. |
| Someone manually edits `service.json` | The generated JSON is reported as stale. |
| Someone changes only the contract | The manifest is reported as stale, even if the JSON bytes remain identical. |
| The configured Nickel version does not match the executable | Evaluation is rejected before a receipt is issued. |
| The source violates the contract | Nickel fails and no successful receipt is issued. |

## Other places the same pattern helps

- Export a reviewed Nickel release policy to JSON for a Rust release tool.
- Export an agent profile to TOML for a program that does not read Nickel.
- Export selected text from a Nickel document while retaining the identity of
  the source, selector, and evaluator.
- Give multiple repositories one receipt format instead of maintaining several
  slightly different freshness checkers.

## When this tool is unnecessary

Use Nickel directly when you only need a one-off export and do not retain the
output. Use Organist or Nix file generation when they already own the entire
materialization workflow and no consumer needs a portable receipt. This tool is
for generated artifacts that must remain independently reviewable and
freshness-checkable.
