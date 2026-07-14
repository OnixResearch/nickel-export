let Contracts = import "../contracts.ncl" in
[
  { package = "nickel-export-core", license = "AGPL-3.0-or-later" },
  { package = "nickel-export", license = "MPL-2.0" },
] | Contracts.PackageLicenseMap
