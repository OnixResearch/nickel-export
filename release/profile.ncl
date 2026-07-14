let Repository = import "../config/repository.ncl" in
{
  schema = "onix-nickel-export-release-profile/v1",
  project = Repository.name,
  license = Repository.license,
  evaluator = Repository.evaluator,
  required_checks = Repository.release_checks,
  claim_boundary = Repository.non_claims,
  artifact_families = [
    "rust-packages",
    "nix-package",
    "canonical-schemas",
    "compatibility-projections",
    "fixture-manifests",
    "proof-evidence",
  ],
}
