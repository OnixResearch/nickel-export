let Contracts = import "../contracts.ncl" in
[
  { package = "nickel-export-core", license = "MPL-2.0" },
  { package = "unknown-shell", license = "AGPL-3.0-or-later" },
] | Contracts.PackageLicenseMap
