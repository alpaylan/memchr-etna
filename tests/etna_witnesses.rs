//! Deterministic witness tests for ETNA memchr variants.
//!
//! Each `witness_<property>_case_<tag>` is a fixed concrete input chosen so
//! that:
//!   * on base HEAD the property holds (test passes),
//!   * on the `etna/<variant>` branch (or with the mutation patch applied),
//!     the property fails (test fails).
//!
//! The tests delegate to `property_<name>` — no invariant logic lives here.

use memchr::etna::{
    property_memchr_iter_matches_naive, property_memrchr_iter_matches_naive,
    PropertyResult,
};

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Discard => {}
        PropertyResult::Fail(m) => panic!("property failed: {m}"),
    }
}

// --- memchr_iter_offbyone_8313aeb_1 -------------------------------

/// Forward iteration over a single-byte needle in a haystack with multiple
/// hits. A correct iterator yields 0-based positions; the off-by-one mutation
/// shifts every position by +1 and hits this case immediately on the first
/// output.
#[test]
fn witness_memchr_iter_matches_naive_case_multi_hit() {
    let haystack = b"aaaabaaaabaaaab".to_vec();
    assert_pass(property_memchr_iter_matches_naive(b'b', haystack));
}

/// Single match at a non-zero position. Forces the first `next()` to return a
/// non-zero index — which the off-by-one mutation corrupts into `index + 1`.
#[test]
fn witness_memchr_iter_matches_naive_case_single_hit() {
    let haystack = b"zzzzzzza".to_vec();
    assert_pass(property_memchr_iter_matches_naive(b'a', haystack));
}

// --- memrchr_iter_forward_1b37466_1 -------------------------------

/// Reverse iteration across a haystack with multiple hits. A correct
/// `next_back` yields positions from the end; the "forward search" mutation
/// yields them from the start, producing a mismatched sequence.
#[test]
fn witness_memrchr_iter_matches_naive_case_multi_hit() {
    let haystack = b"aabaabaab".to_vec();
    assert_pass(property_memrchr_iter_matches_naive(b'b', haystack));
}

/// Simpler trigger: a single hit near the end of the haystack. The forward
/// mutation returns the same position as the forward scan, which happens to
/// be correct — so we also assert an extra reverse-first invariant via a
/// witness with two separated hits.
#[test]
fn witness_memrchr_iter_matches_naive_case_two_separated() {
    let haystack = b"a____a".to_vec();
    assert_pass(property_memrchr_iter_matches_naive(b'a', haystack));
}
