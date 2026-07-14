//! Bounded Verus models for nickel-export identity primitives.
//!
//! This file proves properties of project-owned mathematical models. It does
//! not prove BLAKE3 collision resistance, Nickel evaluator behavior,
//! filesystem behavior, verifier soundness, or automatic equivalence with the
//! separately maintained Rust implementation.
use vstd::prelude::*;

verus! {

pub const LENGTH_PREFIX_BYTES: usize = 8;

pub open spec fn length_prefix_bytes_spec() -> int {
    LENGTH_PREFIX_BYTES as int
}

pub const BYTE_MASK: u64 = 0xff;

pub const SHIFT_BYTE_1: u64 = 8;

pub const SHIFT_BYTE_2: u64 = 16;

pub const SHIFT_BYTE_3: u64 = 24;

pub const SHIFT_BYTE_4: u64 = 32;

pub const SHIFT_BYTE_5: u64 = 40;

pub const SHIFT_BYTE_6: u64 = 48;

pub const SHIFT_BYTE_7: u64 = 56;

// r[depends nickel_export.proof.identity_primitives]
/// Model the core's fixed-width big-endian count and length encoding.
pub open spec fn encode_u64_be_spec(value: u64) -> Seq<u8> {
    seq![
        ((value >> SHIFT_BYTE_7) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_6) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_5) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_4) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_3) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_2) & BYTE_MASK) as u8,
        ((value >> SHIFT_BYTE_1) & BYTE_MASK) as u8,
        (value & BYTE_MASK) as u8,
    ]
}

/// Model decoding one fixed-width big-endian count or length.
pub open spec fn decode_u64_be_spec(bytes: Seq<u8>) -> u64
    recommends
        bytes.len() == length_prefix_bytes_spec(),
{
    ((bytes[0] as u64) << SHIFT_BYTE_7) | ((bytes[1] as u64) << SHIFT_BYTE_6) | ((bytes[2] as u64)
        << SHIFT_BYTE_5) | ((bytes[3] as u64) << SHIFT_BYTE_4) | ((bytes[4] as u64) << SHIFT_BYTE_3)
        | ((bytes[5] as u64) << SHIFT_BYTE_2) | ((bytes[6] as u64) << SHIFT_BYTE_1) | (
    bytes[7] as u64)
}

// r[verify nickel_export.proof.identity_primitives]
pub proof fn encode_u64_be_has_fixed_width(value: u64)
    ensures
        encode_u64_be_spec(value).len() == length_prefix_bytes_spec(),
{
}

// r[verify nickel_export.proof.identity_primitives]
pub proof fn encode_u64_be_round_trip(value: u64)
    ensures
        decode_u64_be_spec(encode_u64_be_spec(value)) == value,
{
    let bytes = encode_u64_be_spec(value);
    let byte_0: u8 = bytes[0int];
    let byte_1: u8 = bytes[1int];
    let byte_2: u8 = bytes[2int];
    let byte_3: u8 = bytes[3int];
    let byte_4: u8 = bytes[4int];
    let byte_5: u8 = bytes[5int];
    let byte_6: u8 = bytes[6int];
    let byte_7: u8 = bytes[7int];
    assert(((byte_0 as u64) << SHIFT_BYTE_7) | ((byte_1 as u64) << SHIFT_BYTE_6) | ((byte_2 as u64)
        << SHIFT_BYTE_5) | ((byte_3 as u64) << SHIFT_BYTE_4) | ((byte_4 as u64) << SHIFT_BYTE_3) | (
    (byte_5 as u64) << SHIFT_BYTE_2) | ((byte_6 as u64) << SHIFT_BYTE_1) | (byte_7 as u64) == value)
        by (bit_vector)
        requires
            byte_0 == ((value >> SHIFT_BYTE_7) & BYTE_MASK) as u8,
            byte_1 == ((value >> SHIFT_BYTE_6) & BYTE_MASK) as u8,
            byte_2 == ((value >> SHIFT_BYTE_5) & BYTE_MASK) as u8,
            byte_3 == ((value >> SHIFT_BYTE_4) & BYTE_MASK) as u8,
            byte_4 == ((value >> SHIFT_BYTE_3) & BYTE_MASK) as u8,
            byte_5 == ((value >> SHIFT_BYTE_2) & BYTE_MASK) as u8,
            byte_6 == ((value >> SHIFT_BYTE_1) & BYTE_MASK) as u8,
            byte_7 == (value & BYTE_MASK) as u8,
    ;
}

// r[verify nickel_export.proof.identity_primitives]
/// Equal fixed-width count bytes imply equal count values.
pub proof fn encode_u64_be_injective(left: u64, right: u64)
    requires
        encode_u64_be_spec(left) == encode_u64_be_spec(right),
    ensures
        left == right,
{
    encode_u64_be_round_trip(left);
    encode_u64_be_round_trip(right);
}

// r[depends nickel_export.proof.identity_primitives]
/// Model one canonical variable-width field as length followed by exact bytes.
pub open spec fn encode_length_delimited_spec(field: Seq<u8>) -> Seq<u8>
    recommends
        field.len() <= u64::MAX as int,
{
    encode_u64_be_spec(field.len() as u64) + field
}

// r[verify nickel_export.proof.identity_primitives]
/// Equal canonical variable-width field bytes imply equal field values.
pub proof fn encode_length_delimited_injective(left: Seq<u8>, right: Seq<u8>)
    requires
        left.len() <= u64::MAX as int,
        right.len() <= u64::MAX as int,
        encode_length_delimited_spec(left) == encode_length_delimited_spec(right),
    ensures
        left == right,
{
    let left_encoded = encode_length_delimited_spec(left);
    let right_encoded = encode_length_delimited_spec(right);
    let left_length = encode_u64_be_spec(left.len() as u64);
    let right_length = encode_u64_be_spec(right.len() as u64);
    encode_u64_be_has_fixed_width(left.len() as u64);
    encode_u64_be_has_fixed_width(right.len() as u64);
    assert(left_length =~= left_encoded.subrange(0, length_prefix_bytes_spec()));
    assert(right_length =~= right_encoded.subrange(0, length_prefix_bytes_spec()));
    assert(left_length =~= right_length);
    encode_u64_be_injective(left.len() as u64, right.len() as u64);
    assert(left =~= left_encoded.subrange(length_prefix_bytes_spec(), left_encoded.len() as int));
    assert(right =~= right_encoded.subrange(
        length_prefix_bytes_spec(),
        right_encoded.len() as int,
    ));
    assert(left =~= right);
}

// r[depends nickel_export.proof.identity_primitives]
/// Every field in a canonical field sequence must fit the u64 prefix.
pub open spec fn canonical_fields_bounded(fields: Seq<Seq<u8>>) -> bool {
    forall|index: int| 0 <= index < fields.len() ==> fields[index].len() <= u64::MAX as int
}

// r[depends nickel_export.proof.identity_primitives]
/// Model a normalized canonical field sequence.
pub open spec fn encode_canonical_fields_spec(fields: Seq<Seq<u8>>) -> Seq<u8>
    decreases fields.len(),
{
    if fields.len() == 0 {
        seq![]
    } else {
        encode_length_delimited_spec(fields[0]) + encode_canonical_fields_spec(
            fields.subrange(1, fields.len() as int),
        )
    }
}

proof fn canonical_fields_tail_bounded(fields: Seq<Seq<u8>>)
    requires
        fields.len() > 0,
        canonical_fields_bounded(fields),
    ensures
        canonical_fields_bounded(fields.subrange(1, fields.len() as int)),
{
    assert forall|index: int|
        0 <= index < fields.subrange(1, fields.len() as int).len() implies fields.subrange(
        1,
        fields.len() as int,
    )[index].len() <= u64::MAX as int by {
        assert(fields.subrange(1, fields.len() as int)[index] == fields[index + 1]);
    }
}

// r[verify nickel_export.proof.identity_primitives]
/// Equal pre-hash canonical field bytes imply equality of every field.
pub proof fn encode_canonical_fields_injective(left: Seq<Seq<u8>>, right: Seq<Seq<u8>>)
    requires
        canonical_fields_bounded(left),
        canonical_fields_bounded(right),
        left.len() == right.len(),
        encode_canonical_fields_spec(left) == encode_canonical_fields_spec(right),
    ensures
        left == right,
    decreases left.len(),
{
    if left.len() == 0 {
        assert(right.len() == 0);
    } else {
        let left_tail = left.subrange(1, left.len() as int);
        let right_tail = right.subrange(1, right.len() as int);
        let left_head_encoded = encode_length_delimited_spec(left[0]);
        let right_head_encoded = encode_length_delimited_spec(right[0]);
        let left_encoded = encode_canonical_fields_spec(left);
        let right_encoded = encode_canonical_fields_spec(right);
        let left_head_end = length_prefix_bytes_spec() + left[0].len();
        let right_head_end = length_prefix_bytes_spec() + right[0].len();

        assert(left[0].len() <= u64::MAX as int);
        assert(right[0].len() <= u64::MAX as int);
        assert(left_head_encoded =~= left_encoded.subrange(0, left_head_end));
        assert(right_head_encoded =~= right_encoded.subrange(0, right_head_end));

        let left_length = encode_u64_be_spec(left[0].len() as u64);
        let right_length = encode_u64_be_spec(right[0].len() as u64);
        encode_u64_be_has_fixed_width(left[0].len() as u64);
        encode_u64_be_has_fixed_width(right[0].len() as u64);
        assert(left_length =~= left_encoded.subrange(0, length_prefix_bytes_spec()));
        assert(right_length =~= right_encoded.subrange(0, length_prefix_bytes_spec()));
        assert(left_length =~= right_length);
        encode_u64_be_injective(left[0].len() as u64, right[0].len() as u64);
        assert(left_head_end == right_head_end);
        assert(left_head_encoded =~= right_head_encoded);
        encode_length_delimited_injective(left[0], right[0]);

        assert(encode_canonical_fields_spec(left_tail) =~= left_encoded.subrange(
            left_head_end,
            left_encoded.len() as int,
        ));
        assert(encode_canonical_fields_spec(right_tail) =~= right_encoded.subrange(
            right_head_end,
            right_encoded.len() as int,
        ));
        assert(encode_canonical_fields_spec(left_tail) =~= encode_canonical_fields_spec(
            right_tail,
        ));
        canonical_fields_tail_bounded(left);
        canonical_fields_tail_bounded(right);
        assert(left_tail.len() == right_tail.len());
        encode_canonical_fields_injective(left_tail, right_tail);
        assert forall|index: int| 0 <= index < left.len() implies left[index] == right[index] by {
            if index == 0 {
                assert(left[0] == right[0]);
            } else {
                assert(left_tail[index - 1] == left[index]);
                assert(right_tail[index - 1] == right[index]);
            }
        }
        assert(left =~= right);
    }
}

// r[depends nickel_export.proof.identity_primitives]
/// A modeled canonical evidence value has a bounded field count and bounded fields.
pub open spec fn canonical_evidence_bounded(fields: Seq<Seq<u8>>) -> bool {
    fields.len() <= u64::MAX as int && canonical_fields_bounded(fields)
}

// r[depends nickel_export.proof.identity_primitives]
/// Model the count-prefixed sequence of canonical length-delimited fields.
pub open spec fn encode_canonical_evidence_spec(fields: Seq<Seq<u8>>) -> Seq<u8> {
    encode_u64_be_spec(fields.len() as u64) + encode_canonical_fields_spec(fields)
}

// r[verify nickel_export.proof.identity_primitives]
/// Equal modeled pre-hash evidence bytes imply equality of every canonical field.
pub proof fn encode_canonical_evidence_injective(left: Seq<Seq<u8>>, right: Seq<Seq<u8>>)
    requires
        canonical_evidence_bounded(left),
        canonical_evidence_bounded(right),
        encode_canonical_evidence_spec(left) == encode_canonical_evidence_spec(right),
    ensures
        left == right,
{
    let left_encoded = encode_canonical_evidence_spec(left);
    let right_encoded = encode_canonical_evidence_spec(right);
    let left_count = encode_u64_be_spec(left.len() as u64);
    let right_count = encode_u64_be_spec(right.len() as u64);
    encode_u64_be_has_fixed_width(left.len() as u64);
    encode_u64_be_has_fixed_width(right.len() as u64);
    assert(left_count =~= left_encoded.subrange(0, length_prefix_bytes_spec()));
    assert(right_count =~= right_encoded.subrange(0, length_prefix_bytes_spec()));
    assert(left_count =~= right_count);
    encode_u64_be_injective(left.len() as u64, right.len() as u64);
    assert(encode_canonical_fields_spec(left) =~= left_encoded.subrange(
        length_prefix_bytes_spec(),
        left_encoded.len() as int,
    ));
    assert(encode_canonical_fields_spec(right) =~= right_encoded.subrange(
        length_prefix_bytes_spec(),
        right_encoded.len() as int,
    ));
    assert(encode_canonical_fields_spec(left) =~= encode_canonical_fields_spec(right));
    encode_canonical_fields_injective(left, right);
}

/// Abstract one portable path component after UTF-8 tokenization.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathComponent {
    Empty,
    Current,
    Parent,
    Safe(u64),
}

/// Proof-facing path input. `portable` excludes backslashes and platform-only forms.
pub struct PathInput {
    pub absolute: bool,
    pub portable: bool,
    pub components: Seq<PathComponent>,
}

pub open spec fn contains_parent_spec(components: Seq<PathComponent>) -> bool {
    exists|index: int| 0 <= index < components.len() && components[index] == PathComponent::Parent
}

pub open spec fn normalized_components_spec(components: Seq<PathComponent>) -> Seq<u64>
    decreases components.len(),
{
    if components.len() == 0 {
        seq![]
    } else {
        let tail = normalized_components_spec(components.subrange(1, components.len() as int));
        match components[0] {
            PathComponent::Safe(value) => seq![value] + tail,
            PathComponent::Empty | PathComponent::Current | PathComponent::Parent => tail,
        }
    }
}

// r[depends nickel_export.proof.identity_primitives]
pub open spec fn path_admitted_spec(path: PathInput) -> bool {
    !path.absolute && path.portable && !contains_parent_spec(path.components)
        && normalized_components_spec(path.components).len() > 0
}

// r[depends nickel_export.proof.identity_primitives]
pub open spec fn normalize_path_spec(path: PathInput) -> Option<Seq<u64>> {
    if path_admitted_spec(path) {
        Some(normalized_components_spec(path.components))
    } else {
        None
    }
}

pub open spec fn normalize_normalized_path_spec(path: Seq<u64>) -> Option<Seq<u64>> {
    if path.len() > 0 {
        Some(path)
    } else {
        None
    }
}

// r[verify nickel_export.proof.identity_primitives]
/// An admitted path is relative, portable, nonempty, and parent-traversal-free.
pub proof fn admitted_path_is_safe(path: PathInput)
    requires
        normalize_path_spec(path).is_some(),
    ensures
        !path.absolute,
        path.portable,
        !contains_parent_spec(path.components),
        normalize_path_spec(path).unwrap().len() > 0,
{
}

// r[verify nickel_export.proof.identity_primitives]
/// Normalizing an accepted normalized path again is idempotent.
pub proof fn admitted_path_normalization_is_idempotent(path: PathInput)
    requires
        normalize_path_spec(path).is_some(),
    ensures
        normalize_normalized_path_spec(normalize_path_spec(path).unwrap()) == normalize_path_spec(
            path,
        ),
{
}

/// Proof-facing receipt fields. Identity values are abstract, not cryptographic hashes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ReceiptWireModel {
    pub schema_matches: bool,
    pub non_claim_matches: bool,
    pub paths_safe: bool,
    pub dependencies_sorted_unique: bool,
    pub evaluator_valid: bool,
    pub diagnostics_have_error: bool,
    pub declared_identity: u64,
    pub recomputed_declared_identity: u64,
    pub receipt_identity: u64,
    pub recomputed_receipt_identity: u64,
    pub evaluator_cohort: u64,
    pub output_key: u64,
}

pub open spec fn receipt_invariants_spec(wire: ReceiptWireModel) -> bool {
    wire.schema_matches && wire.non_claim_matches && wire.paths_safe
        && wire.dependencies_sorted_unique && wire.evaluator_valid && !wire.diagnostics_have_error
        && wire.declared_identity == wire.recomputed_declared_identity && wire.receipt_identity
        == wire.recomputed_receipt_identity
}

/// Opaque proof-facing admitted receipt state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AdmittedReceiptModel {
    wire: ReceiptWireModel,
}

pub closed spec fn admitted_receipt_invariants_spec(admitted: AdmittedReceiptModel) -> bool {
    receipt_invariants_spec(admitted.wire)
}

pub closed spec fn admit_receipt_model_spec(wire: ReceiptWireModel) -> Option<
    AdmittedReceiptModel,
> {
    if receipt_invariants_spec(wire) {
        Some(AdmittedReceiptModel { wire })
    } else {
        None
    }
}

// r[impl nickel_export.proof.identity_primitives]
pub fn admit_receipt_model(wire: ReceiptWireModel) -> (result: Option<AdmittedReceiptModel>)
    ensures
        result == admit_receipt_model_spec(wire),
        result.is_some() ==> admitted_receipt_invariants_spec(result.unwrap()),
        result.is_none() ==> !receipt_invariants_spec(wire),
{
    if wire.schema_matches && wire.non_claim_matches && wire.paths_safe
        && wire.dependencies_sorted_unique && wire.evaluator_valid && !wire.diagnostics_have_error
        && wire.declared_identity == wire.recomputed_declared_identity && wire.receipt_identity
        == wire.recomputed_receipt_identity {
        Some(AdmittedReceiptModel { wire })
    } else {
        None
    }
}

// r[verify nickel_export.proof.identity_primitives]
pub proof fn admitted_receipt_constructor_preserves_invariants(wire: ReceiptWireModel)
    requires
        admit_receipt_model_spec(wire).is_some(),
    ensures
        admitted_receipt_invariants_spec(admit_receipt_model_spec(wire).unwrap()),
{
    reveal(admit_receipt_model_spec);
    reveal(admitted_receipt_invariants_spec);
}

/// Proof-facing manifest fields. Identity values are abstract, not hashes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ManifestWireModel {
    pub schema_matches: bool,
    pub generator_matches: bool,
    pub exports_nonempty: bool,
    pub outputs_sorted_unique: bool,
    pub nested_receipts_admitted: bool,
    pub evaluator_cohort_matches: bool,
    pub manifest_identity: u64,
    pub recomputed_manifest_identity: u64,
}

pub open spec fn manifest_invariants_spec(wire: ManifestWireModel) -> bool {
    wire.schema_matches && wire.generator_matches && wire.exports_nonempty
        && wire.outputs_sorted_unique && wire.nested_receipts_admitted
        && wire.evaluator_cohort_matches && wire.manifest_identity
        == wire.recomputed_manifest_identity
}

/// Opaque proof-facing verified manifest state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VerifiedManifestModel {
    wire: ManifestWireModel,
}

pub closed spec fn verified_manifest_invariants_spec(verified: VerifiedManifestModel) -> bool {
    manifest_invariants_spec(verified.wire)
}

pub closed spec fn verify_manifest_model_spec(wire: ManifestWireModel) -> Option<
    VerifiedManifestModel,
> {
    if manifest_invariants_spec(wire) {
        Some(VerifiedManifestModel { wire })
    } else {
        None
    }
}

// r[impl nickel_export.proof.identity_primitives]
pub fn verify_manifest_model(wire: ManifestWireModel) -> (result: Option<VerifiedManifestModel>)
    ensures
        result == verify_manifest_model_spec(wire),
        result.is_some() ==> verified_manifest_invariants_spec(result.unwrap()),
        result.is_none() ==> !manifest_invariants_spec(wire),
{
    if wire.schema_matches && wire.generator_matches && wire.exports_nonempty
        && wire.outputs_sorted_unique && wire.nested_receipts_admitted
        && wire.evaluator_cohort_matches && wire.manifest_identity
        == wire.recomputed_manifest_identity {
        Some(VerifiedManifestModel { wire })
    } else {
        None
    }
}

// r[verify nickel_export.proof.identity_primitives]
pub proof fn verified_manifest_constructor_preserves_invariants(wire: ManifestWireModel)
    requires
        verify_manifest_model_spec(wire).is_some(),
    ensures
        verified_manifest_invariants_spec(verify_manifest_model_spec(wire).unwrap()),
{
    reveal(verify_manifest_model_spec);
    reveal(verified_manifest_invariants_spec);
}

} // verus!
