# memchr — ETNA Tasks

Total tasks: 8

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task. The `<PropertyKey>` token in the command column uses the PascalCase key recognised by `src/bin/etna.rs`; passing `All` runs every property for the named framework in a single invocation.

## Property keys

| Property | PropertyKey |
|----------|-------------|
| `property_memchr_iter_matches_naive` | `MemchrIterMatchesNaive` |
| `property_memrchr_iter_matches_naive` | `MemrchrIterMatchesNaive` |

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001 | `memchr_iter_offbyone_8313aeb_1` | proptest | `property_memchr_iter_matches_naive` | `witness_memchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- proptest MemchrIterMatchesNaive` |
| 002 | `memchr_iter_offbyone_8313aeb_1` | quickcheck | `property_memchr_iter_matches_naive` | `witness_memchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- quickcheck MemchrIterMatchesNaive` |
| 003 | `memchr_iter_offbyone_8313aeb_1` | crabcheck | `property_memchr_iter_matches_naive` | `witness_memchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- crabcheck MemchrIterMatchesNaive` |
| 004 | `memchr_iter_offbyone_8313aeb_1` | hegel | `property_memchr_iter_matches_naive` | `witness_memchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- hegel MemchrIterMatchesNaive` |
| 005 | `memrchr_iter_forward_1b37466_1` | proptest | `property_memrchr_iter_matches_naive` | `witness_memrchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- proptest MemrchrIterMatchesNaive` |
| 006 | `memrchr_iter_forward_1b37466_1` | quickcheck | `property_memrchr_iter_matches_naive` | `witness_memrchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- quickcheck MemrchrIterMatchesNaive` |
| 007 | `memrchr_iter_forward_1b37466_1` | crabcheck | `property_memrchr_iter_matches_naive` | `witness_memrchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- crabcheck MemrchrIterMatchesNaive` |
| 008 | `memrchr_iter_forward_1b37466_1` | hegel | `property_memrchr_iter_matches_naive` | `witness_memrchr_iter_matches_naive_case_multi_hit` | `cargo run --release --bin etna -- hegel MemrchrIterMatchesNaive` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails. Witnesses live in `tests/etna_witnesses.rs`.

| Witness | Property | Detects | Input shape |
|---------|----------|---------|-------------|
| `witness_memchr_iter_matches_naive_case_multi_hit` | `property_memchr_iter_matches_naive` | `memchr_iter_offbyone_8313aeb_1` | `b"aaaabaaaabaaaab"` searched for `b'b'` — three spaced hits force the first `.next()` to return a non-zero index, which the `+1` mutation corrupts immediately |
| `witness_memchr_iter_matches_naive_case_single_hit` | `property_memchr_iter_matches_naive` | `memchr_iter_offbyone_8313aeb_1` | `b"zzzzzzza"` searched for `b'a'` — single match at the tail, the buggy iterator reports index 8 instead of 7 |
| `witness_memrchr_iter_matches_naive_case_multi_hit` | `property_memrchr_iter_matches_naive` | `memrchr_iter_forward_1b37466_1` | `b"aabaabaab"` searched for `b'b'` — reverse scan should yield `[8, 5, 2]`; the forward-in-disguise variant yields `[2]` and stops |
| `witness_memrchr_iter_matches_naive_case_two_separated` | `property_memrchr_iter_matches_naive` | `memrchr_iter_forward_1b37466_1` | `b"a____a"` searched for `b'a'` — two separated hits; the reverse-first invariant (`next_back()` returns index 5, not 0) detects the swap |
