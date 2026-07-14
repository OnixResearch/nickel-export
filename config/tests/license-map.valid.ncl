let Contracts = import "../contracts.ncl" in
[
  { package = "nickel-export-core", license = "MPL-2.0" },
  { package = "nickel-export", license = "AGPL-3.0-or-later" },
] | Contracts.PackageLicenseMap
