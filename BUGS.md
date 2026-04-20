# memchr — Injected Bugs

Total mutations: 2 (from 3 candidates; 1 terminally inexpressible — see below)

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `memchr_iter_offbyone` | `memchr_iter_offbyone_8313aeb_1` | `patches/memchr_iter_offbyone_8313aeb_1.patch` | `patch` | `8313aebad08af00dbd7090114f2a70a1842b6f45` |
| 2 | `memrchr_iter_forward` | `memrchr_iter_forward_1b37466_1` | `patches/memrchr_iter_forward_1b37466_1.patch` | `patch` | `1b37466ed6f79fcc40692beccfd16b199830b147` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `memchr_iter_offbyone_8313aeb_1` | `property_memchr_iter_matches_naive` | `witness_memchr_iter_matches_naive_case_multi_hit`, `witness_memchr_iter_matches_naive_case_single_hit` |
| `memrchr_iter_forward_1b37466_1` | `property_memrchr_iter_matches_naive` | `witness_memrchr_iter_matches_naive_case_multi_hit`, `witness_memrchr_iter_matches_naive_case_two_separated` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_memchr_iter_matches_naive` | OK | OK | OK | OK |
| `property_memrchr_iter_matches_naive` | OK | OK | OK | OK |

## Bug Details

### 1. memchr_iter_offbyone (8313aeb_1)
- **Variant**: `memchr_iter_offbyone_8313aeb_1`
- **Location**: `src/memchr.rs`, `impl<'h> Iterator for Memchr<'h>::next`
- **Property**: `property_memchr_iter_matches_naive`
- **Witnesses**: `witness_memchr_iter_matches_naive_case_multi_hit`, `witness_memchr_iter_matches_naive_case_single_hit`
- **Fix commit**: `8313aebad08af00dbd7090114f2a70a1842b6f45` — "Fix Memchr iterators to use 0-based positions"
- **Invariant violated**: The positions yielded by `memchr_iter(needle, haystack)` match the indices yielded by the naive `haystack.iter().enumerate().filter(|(_, b)| **b == needle).map(|(i, _)| i)` scan, in order.
- **How the mutation triggers**: The buggy forward iterator post-shifts every output via `.map(|i| i + 1)`, so every hit is reported at `index + 1` instead of `index`. A haystack with a match at index 0 gets reported at 1; off-by-one is detected on the very first yielded position.

### 2. memrchr_iter_forward (1b37466_1)
- **Variant**: `memrchr_iter_forward_1b37466_1`
- **Location**: `src/memchr.rs`, `impl<'h> DoubleEndedIterator for Memchr<'h>::next_back`
- **Property**: `property_memrchr_iter_matches_naive`
- **Witnesses**: `witness_memrchr_iter_matches_naive_case_multi_hit`, `witness_memrchr_iter_matches_naive_case_two_separated`
- **Fix commit**: `1b37466ed6f79fcc40692beccfd16b199830b147` — "Modify DoubleEndedIterator to use rposition instead of just position"
- **Invariant violated**: `memrchr_iter(needle, haystack).collect::<Vec<_>>()` equals the reverse of `memchr_iter(needle, haystack).collect::<Vec<_>>()` (i.e. the reverse iterator yields match positions from the end of the haystack).
- **How the mutation triggers**: The buggy `next_back` dispatches the reverse scan through `memchr_raw` instead of `memrchr_raw`, so it returns the *first* (leftmost) match every time it's called. The first `.next_back()` of `b"aabaabaab"` searching for `b` yields 2 (leftmost hit) instead of 8 (rightmost hit).

## Skipped candidates

### `memchr_fallback_4ee2398` — surface removed
- **Discover hash**: `4ee2398d1ff6edd9cc037ccb8fc7c63ce62b8e8f` — "fallback: fix unaligned read"
- **Why skipped**: The fix targeted the old `fallback::memchr` scanner that operated directly on `usize` word reads. Commit `3120980` ("memchr: rewrite everything") deleted that entire fallback path and replaced it with a new `arch::all::memchr::One::Fallback1` implementation that uses a different primitive. The original unaligned-read mutation is not expressible against the current codebase — there is no matching `usize` word-load site to re-break.
