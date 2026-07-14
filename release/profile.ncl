let ReplayRuns = 3 in
let Repository = import "../config/repository.ncl" in
{
  schema = "onix-nickel-export-release-profile/v1",
  project = Repository.name,
  license = Repository.license,
  package_licenses = Repository.package_licenses,
  license_artifacts = Repository.license_artifacts,
  evaluator = Repository.evaluator,
  required_checks = Repository.release_checks,
  claim_boundary = Repository.non_claims,
  replay = {
    selected_families = ["fixtures.json", "examples.service-config"],
    runs = ReplayRuns,
    non_claim = "Selected replay agreement is bounded empirical evidence, not proof of universal determinism.",
  },
  artifact_families = [
    "rust-packages",
    "nix-package",
    "canonical-schemas",
    "compatibility-projections",
    "fixture-manifests",
    "proof-evidence",
  ],
}
