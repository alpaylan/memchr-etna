# memchr — ETNA Tasks

Total tasks: 8

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `memchr_iter_offbyone_8313aeb_1` | proptest | `MemchrIterMatchesNaive` | `witness_memchr_iter_matches_naive_case_multi_hit` |
| 002 | `memchr_iter_offbyone_8313aeb_1` | quickcheck | `MemchrIterMatchesNaive` | `witness_memchr_iter_matches_naive_case_multi_hit` |
| 003 | `memchr_iter_offbyone_8313aeb_1` | crabcheck | `MemchrIterMatchesNaive` | `witness_memchr_iter_matches_naive_case_multi_hit` |
| 004 | `memchr_iter_offbyone_8313aeb_1` | hegel | `MemchrIterMatchesNaive` | `witness_memchr_iter_matches_naive_case_multi_hit` |
| 005 | `memrchr_iter_forward_1b37466_1` | proptest | `MemrchrIterMatchesNaive` | `witness_memrchr_iter_matches_naive_case_multi_hit` |
| 006 | `memrchr_iter_forward_1b37466_1` | quickcheck | `MemrchrIterMatchesNaive` | `witness_memrchr_iter_matches_naive_case_multi_hit` |
| 007 | `memrchr_iter_forward_1b37466_1` | crabcheck | `MemrchrIterMatchesNaive` | `witness_memrchr_iter_matches_naive_case_multi_hit` |
| 008 | `memrchr_iter_forward_1b37466_1` | hegel | `MemrchrIterMatchesNaive` | `witness_memrchr_iter_matches_naive_case_multi_hit` |

## Witness Catalog

- `witness_memchr_iter_matches_naive_case_multi_hit` — base passes, variant fails
- `witness_memchr_iter_matches_naive_case_single_hit` — base passes, variant fails
- `witness_memrchr_iter_matches_naive_case_multi_hit` — base passes, variant fails
- `witness_memrchr_iter_matches_naive_case_two_separated` — base passes, variant fails
