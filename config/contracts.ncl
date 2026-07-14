let OwnedPackageCount = 2 in
let OwnedPackages = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value == OwnedPackageCount
  && std.array.elem "nickel-export-core" value
  && std.array.elem "nickel-export" value
) in
let PackageLicenseMapContract = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value == OwnedPackageCount
  && std.array.elem { package = "nickel-export-core", license = "MPL-2.0" } value
  && std.array.elem { package = "nickel-export", license = "AGPL-3.0-or-later" } value
) in
let RepositoryContractValue = {
  name | String,
  repository | String,
  license | String,
  packages | OwnedPackages,
  package_licenses | PackageLicenseMapContract,
  license_artifacts | Array {
    license | String,
    path | String,
  },
  schemas | Array String,
  evaluator | {
    identity | String,
    version | String,
    import_path_policy | String,
  },
  release_checks | Array String,
  non_claims | Array String,
} in
{
  PackageLicenseMap = PackageLicenseMapContract,
  RepositoryContract = RepositoryContractValue,
}
