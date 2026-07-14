# Deterministic Nickel Export License Boundary

## Purpose

Represent and validate package-specific licenses for the reusable no-std core and AGPL operator shell.

## Requirements

### Requirement: Core and shell use distinct licenses

r[nickel_export.release.license_boundary.package_split] `nickel-export-core` MUST declare `MPL-2.0`, and `nickel-export` MUST declare `AGPL-3.0-or-later`.

#### Scenario: An embedder selects the core

- GIVEN an application depends only on `nickel-export-core`
- WHEN Cargo metadata is inspected
- THEN the core MUST report MPL-2.0 without inheriting the shell's AGPL expression.

### Requirement: Package license metadata is Nickel-owned

r[nickel_export.release.license_boundary.typed_profile] The typed repository profile MUST map every repository-owned package to exactly one selected license expression and MUST preserve the repository-level AGPL expression for compatibility.

#### Scenario: Every owned package is mapped

- GIVEN the repository profile lists `nickel-export-core` and `nickel-export`
- WHEN its package-license contract is applied
- THEN each listed package MUST have the intended non-empty license expression and unknown package keys MUST be rejected.

### Requirement: Complete license artifacts accompany source

r[nickel_export.release.license_boundary.artifacts] Nickel Export MUST include complete MPL-2.0 and AGPL-3.0-or-later texts and MUST document the package boundary and third-party non-claim.

#### Scenario: A source distribution is reviewed offline

- GIVEN a recipient has no network access
- WHEN package licensing is inspected
- THEN both complete license texts and the package map MUST be available in the source distribution.

### Requirement: Generated exports preserve package licenses

r[nickel_export.release.license_boundary.generated_exports] Checked repository and release-profile JSON projections MUST contain the package-license mapping exported from Nickel and MUST fail freshness checks when stale.

#### Scenario: Generated JSON omits the MPL core mapping

- GIVEN Nickel source maps the core to MPL-2.0 but checked JSON omits or changes that row
- WHEN profile freshness validation runs
- THEN validation MUST fail.

### Requirement: License mapping validation fails closed

r[nickel_export.release.license_boundary.validation] Positive validation MUST accept the intended package map, while negative validation MUST reject missing packages, unknown packages, empty expressions, and reversed core/shell expressions.

#### Scenario: Core and shell expressions are reversed

- GIVEN a malformed profile maps the core to AGPL and the shell to MPL
- WHEN license-boundary validation runs
- THEN validation MUST fail with deterministic package-specific diagnostics.

### Requirement: Focused release checks verify the boundary

r[nickel_export.release.license_boundary.final_validation] The change MUST run package metadata checks, Nickel typechecking/export freshness, complete-license checks, and Cairn validation.

#### Scenario: The final boundary is checked

- GIVEN manifests, typed profiles, generated exports, and license texts agree
- WHEN focused release checks run
- THEN they MUST pass without claiming evaluator equivalence or downstream release eligibility.
