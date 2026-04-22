# memchr — Injected Bugs

Total mutations: 2

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `memchr_iter_offbyone_8313aeb_1` | `memchr_iter_offbyone` | `src/memchr.rs` | `patch` | `8313aebad08af00dbd7090114f2a70a1842b6f45` |
| 2 | `memrchr_iter_forward_1b37466_1` | `memrchr_iter_forward` | `src/memchr.rs` | `patch` | `1b37466ed6f79fcc40692beccfd16b199830b147` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `memchr_iter_offbyone_8313aeb_1` | `MemchrIterMatchesNaive` | `witness_memchr_iter_matches_naive_case_multi_hit`, `witness_memchr_iter_matches_naive_case_single_hit` |
| `memrchr_iter_forward_1b37466_1` | `MemrchrIterMatchesNaive` | `witness_memrchr_iter_matches_naive_case_multi_hit`, `witness_memrchr_iter_matches_naive_case_two_separated` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `MemchrIterMatchesNaive` | ✓ | ✓ | ✓ | ✓ |
| `MemrchrIterMatchesNaive` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. memchr_iter_offbyone

- **Variant**: `memchr_iter_offbyone_8313aeb_1`
- **Location**: `src/memchr.rs`
- **Property**: `MemchrIterMatchesNaive`
- **Witness(es)**:
  - `witness_memchr_iter_matches_naive_case_multi_hit`
  - `witness_memchr_iter_matches_naive_case_single_hit`
- **Source**: Fix Memchr iterators to use 0-based positions
  > `Memchr::next` post-shifted every match index by `+1`, so `memchr_iter(needle, haystack)` reported positions one ahead of their true offsets. The fix removes the shift so iterator positions match the naive byte scan.
- **Fix commit**: `8313aebad08af00dbd7090114f2a70a1842b6f45` — Fix Memchr iterators to use 0-based positions
- **Invariant violated**: The positions yielded by `memchr_iter(needle, haystack)` match the indices yielded by the naive `haystack.iter().enumerate().filter(|(_, b)| **b == needle).map(|(i, _)| i)` scan, in order.
- **How the mutation triggers**: The buggy forward iterator post-shifts every output via `.map(|i| i + 1)`, so every hit is reported at `index + 1` instead of `index`. A haystack with a match at index 0 gets reported at 1; off-by-one is detected on the very first yielded position.

### 2. memrchr_iter_forward

- **Variant**: `memrchr_iter_forward_1b37466_1`
- **Location**: `src/memchr.rs`
- **Property**: `MemrchrIterMatchesNaive`
- **Witness(es)**:
  - `witness_memrchr_iter_matches_naive_case_multi_hit`
  - `witness_memrchr_iter_matches_naive_case_two_separated`
- **Source**: Modify DoubleEndedIterator to use rposition instead of just position
  > `Memchr::next_back` dispatched the reverse scan through `memchr_raw`, so it returned the leftmost match each time instead of the rightmost. The fix routes `next_back` through `memrchr_raw` so the `DoubleEndedIterator` yields match positions from the end of the haystack.
- **Fix commit**: `1b37466ed6f79fcc40692beccfd16b199830b147` — Modify DoubleEndedIterator to use rposition instead of just position
- **Invariant violated**: `memrchr_iter(needle, haystack).collect::<Vec<_>>()` equals the reverse of `memchr_iter(needle, haystack).collect::<Vec<_>>()` (i.e. the reverse iterator yields match positions from the end of the haystack).
- **How the mutation triggers**: The buggy `next_back` dispatches the reverse scan through `memchr_raw` instead of `memrchr_raw`, so it returns the *first* (leftmost) match every time it's called. The first `.next_back()` of `b"aabaabaab"` searching for `b` yields 2 (leftmost hit) instead of 8 (rightmost hit).
