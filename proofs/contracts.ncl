let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let PositiveBytes = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let NonNegativeCount = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0
) in
let VerifiedOutcome = std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value ["verified", "rejected-as-expected"]
) in
let ArtifactRole = std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value [
    "proof-source",
    "implementation-source",
    "correspondence-vectors",
    "negative-proof-fixture",
  ]
) in
let EvidenceContractValue = {
  schema_version | NonEmptyString,
  claim_scope | NonEmptyString,
  proof_claims | Array NonEmptyString,
  artifacts | Array {
    role | ArtifactRole,
    path | NonEmptyString,
    blake3 | Blake3Hex,
    bytes | PositiveBytes,
  },
  toolchain | {
    provider | NonEmptyString,
    provider_revision | NonEmptyString,
    verifier_release | NonEmptyString,
    verifier_source_revision | NonEmptyString,
    trellis_reference_revision | NonEmptyString,
    trellis_reference_paths | Array NonEmptyString,
  },
  validation | {
    proof_command | Array NonEmptyString,
    proof_outcome | VerifiedOutcome,
    verified_obligations | NonNegativeCount,
    negative_fixture_command | Array NonEmptyString,
    negative_fixture_outcome | VerifiedOutcome,
    correspondence_command | Array NonEmptyString,
    correspondence_outcome | VerifiedOutcome,
  },
  correspondence | {
    classification | NonEmptyString,
    statement | NonEmptyString,
  },
  axioms_and_assumptions | Array NonEmptyString,
  non_claims | Array NonEmptyString,
} in
{
  EvidenceContract = EvidenceContractValue,
}
