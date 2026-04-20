//! ETNA framework-neutral property functions for memchr.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs and
//! returning `PropertyResult`. Framework adapters in `src/bin/etna.rs` and
//! witness tests in `tests/etna_witnesses.rs` all call these functions
//! directly — invariants are never re-implemented inside an adapter.

#![allow(missing_docs)]

use alloc::vec::Vec;

pub enum PropertyResult {
    Pass,
    Fail(alloc::string::String),
    Discard,
}

// ------------------------------------------------------------------
// Naive reference implementations. These mirror
// `src/tests/memchr/naive.rs` but are public so the `etna` binary can
// link against them without depending on internal test modules.

fn naive_memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

fn naive_memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().rposition(|&b| b == needle)
}

fn naive_memchr_iter(needle: u8, haystack: &[u8]) -> Vec<usize> {
    haystack
        .iter()
        .enumerate()
        .filter(move |&(_, &b)| b == needle)
        .map(|t| t.0)
        .collect()
}

// ------------------------------------------------------------------
// Properties.

/// Forward iteration over a single-byte needle must yield exactly the same
/// 0-based positions as `haystack.iter().enumerate().filter(..).map(i)`.
///
/// Historical bug (8313aeb): the `Memchr` iterator started positions at 1
/// instead of 0 and also failed to fuse correctly after the first hit,
/// producing results that disagreed with `Iterator::position`.
pub fn property_memchr_iter_matches_naive(
    needle: u8,
    haystack: Vec<u8>,
) -> PropertyResult {
    let got: Vec<usize> = crate::memchr_iter(needle, &haystack).collect();
    let expected = naive_memchr_iter(needle, &haystack);
    if got == expected {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(alloc::format!(
            "memchr_iter mismatch: got={:?} expected={:?} needle={} haystack={:?}",
            got, expected, needle, haystack,
        ))
    }
}

/// Reverse iteration via `DoubleEndedIterator::next_back` must yield the same
/// positions as the forward iterator, just in reverse order. Equivalently,
/// `memrchr_iter` must equal `naive_memchr_iter` reversed.
///
/// Historical bug (1b37466): `DoubleEndedIterator::next_back` for `Memchr`
/// delegated to a forward (`position`) search instead of a reverse
/// (`rposition`) search, producing positions from the start of the remaining
/// haystack rather than the end. Also catches the 0-based / 1-based off-by-one
/// regression from 8313aeb on the reverse path.
pub fn property_memrchr_iter_matches_naive(
    needle: u8,
    haystack: Vec<u8>,
) -> PropertyResult {
    let got: Vec<usize> = crate::memrchr_iter(needle, &haystack).collect();
    let mut expected = naive_memchr_iter(needle, &haystack);
    expected.reverse();
    if got != expected {
        return PropertyResult::Fail(alloc::format!(
            "memrchr_iter mismatch: got={:?} expected={:?} needle={} haystack={:?}",
            got, expected, needle, haystack,
        ));
    }
    // Additional tie-in: first/last positions via the single-shot functions
    // must agree with the iterator endpoints.
    let last = expected.first().copied();
    let first = expected.last().copied();
    if crate::memchr(needle, &haystack) != first {
        return PropertyResult::Fail(alloc::format!(
            "memchr disagrees with first(memrchr_iter.reverse): needle={} haystack={:?}",
            needle, haystack,
        ));
    }
    if crate::memrchr(needle, &haystack) != last {
        return PropertyResult::Fail(alloc::format!(
            "memrchr disagrees with first(memrchr_iter): needle={} haystack={:?}",
            needle, haystack,
        ));
    }
    let _ = (naive_memchr, naive_memrchr);
    PropertyResult::Pass
}
