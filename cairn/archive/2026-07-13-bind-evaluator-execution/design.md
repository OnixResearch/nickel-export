# Design: Bind evaluator execution

## Context

`verify_evaluator_version` and `run_evaluator` independently invoke the configured program. A `PATH` lookup or mutable executable can therefore resolve differently between calls. The canonical descriptor records a supplied identity and normalized strings rather than the exact artifact and process plan.

## Dependencies

Implement after `confine-evaluator-inputs` so the canonical plan includes snapshot paths and explicit environment semantics.

## Decisions

### 1. Resolve one evaluator artifact

**Choice:** Require or resolve a canonical executable path once, reject symlink drift, hash the executable bytes, and use that resolved path for both version and export execution.

**Rationale:** The receipt then identifies the executable actually invoked rather than a caller assertion.

### 2. Keep artifact and closure identities distinct

**Choice:** Record an exact executable BLAKE3 identity and an optional adapter-verified Nix/Mantle closure identity with its evidence class.

**Rationale:** Executable bytes do not capture dynamic libraries, while closure identity is unavailable to every portable shell.

### 3. Derive arguments and evidence from one typed plan

**Choice:** Define evaluator format, selector, contract, import paths, environment profile, and package mode as typed fields. Render process arguments and receipt fields from that value.

**Rationale:** This eliminates drift between actual argv and the descriptor.

### 4. Reject ambiguous options

**Choice:** Reject duplicate or conflicting typed options rather than sorting and silently deduplicating free-form strings.

**Rationale:** Distinct executions must not collapse to one identity unless the schema explicitly defines them as equivalent.

## Risks / Trade-offs

- Portable executable hashing remains weaker than closure identity.
- Requiring a stable resolved path may break callers that rely on mutable `PATH` lookup.
- Canonical evaluator descriptor changes require a schema migration.

## Validation Plan

- Replace an evaluator between version and export phases and prove execution fails closed.
- Verify executable, closure, typed plan, and rendered argv identities are stable.
- Reject duplicate/conflicting options and descriptor/argv divergence.
