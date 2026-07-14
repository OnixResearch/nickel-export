//! Expected-failure fixture: an unframed first-byte encoding is not injective.

use vstd::prelude::*;

verus! {

pub open spec fn ambiguous_prefix_spec(value: Seq<u8>) -> u8 {
    if value.len() == 0 { 0 } else { value[0] }
}

proof fn false_injectivity_claim(left: Seq<u8>, right: Seq<u8>)
    requires ambiguous_prefix_spec(left) == ambiguous_prefix_spec(right)
    ensures left == right
{}

} // verus!
