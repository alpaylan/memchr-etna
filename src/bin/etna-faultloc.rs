// Crabcheck fault-localization runner for memchr.
// Self-contained. src/bin/etna.rs stays untouched.

use std::fmt;

use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use memchr::etna::{
    property_memchr_iter_matches_naive, property_memrchr_iter_matches_naive, PropertyResult,
};
use rand::Rng;

// ---------- wrapper newtypes (mirroring src/bin/etna.rs:128-156) ----------

#[derive(Clone)]
struct Needle(u8);

#[derive(Clone)]
struct Haystack(Vec<u8>);

impl fmt::Debug for Needle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}
impl fmt::Debug for Haystack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

const NEEDLE_POOL: &[u8] = &[b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff];
const HAYSTACK_POOL: &[u8] = &[
    b'a', b'a', b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff, b'_', b'.', b' ',
];
const MAX_HAYSTACK_LEN: usize = 128;

// ---------- Arbitrary impls ----------

impl<R: Rng> Arbitrary<R> for Needle {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Needle(NEEDLE_POOL[rng.random_range(0..NEEDLE_POOL.len())])
    }
}

impl<R: Rng> Arbitrary<R> for Haystack {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0usize..=MAX_HAYSTACK_LEN);
        Haystack(
            (0..len)
                .map(|_| HAYSTACK_POOL[rng.random_range(0..HAYSTACK_POOL.len())])
                .collect(),
        )
    }
}

// ---------- Mutate impls (single-point perturbation, pool-preserving) ----------

impl<R: Rng> Mutate<R> for Needle {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        // Swap in a different needle from the pool — bit-flipping arbitrary
        // u8 bits would leave the pool and almost always produce a needle
        // that doesn't occur in any haystack we generate.
        Needle(NEEDLE_POOL[rng.random_range(0..NEEDLE_POOL.len())])
    }
}

impl<R: Rng> Mutate<R> for Haystack {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        if self.0.is_empty() {
            return Haystack(vec![HAYSTACK_POOL[rng.random_range(0..HAYSTACK_POOL.len())]]);
        }
        let mut v = self.0.clone();
        let i = rng.random_range(0..v.len());
        // Replace one byte with a random one from the haystack pool.
        v[i] = HAYSTACK_POOL[rng.random_range(0..HAYSTACK_POOL.len())];
        Haystack(v)
    }
}

// ---------- dispatch ----------

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}


fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property> [tests]", args[0]);
        eprintln!("  tool:     crabcheck");
        eprintln!("  property: MemchrIterMatchesNaive | MemrchrIterMatchesNaive");
        return;
    }
    let tool = args[1].as_str();
    let property = args[2].as_str();

    let result = match (tool, property) {
        ("crabcheck", "MemchrIterMatchesNaive") => {
            quickcheck(|(Needle(n), Haystack(h))| {
                to_opt(property_memchr_iter_matches_naive(n, h))
            })
        },
        ("crabcheck", "MemrchrIterMatchesNaive") => {
            quickcheck(|(Needle(n), Haystack(h))| {
                to_opt(property_memrchr_iter_matches_naive(n, h))
            })
        },
        _ => panic!("Unknown tool or property: {tool} {property}"),
    };

    println!("Result: {:?}", result);
}
