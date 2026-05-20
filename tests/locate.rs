//! Fault-localization integration tests for memchr.

use std::fmt;

use crabcheck::quickcheck::{Arbitrary, Mutate};
use memchr::etna::{
    property_memchr_iter_matches_naive, property_memrchr_iter_matches_naive, PropertyResult,
};
use rand::Rng;

#[derive(Clone)]
struct Needle(u8);

#[derive(Clone)]
struct Haystack(Vec<u8>);

impl fmt::Debug for Needle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Debug for Haystack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

const NEEDLE_POOL: &[u8] = &[b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff];
const HAYSTACK_POOL: &[u8] = &[
    b'a', b'a', b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff, b'_', b'.', b' ',
];
const MAX_HAYSTACK_LEN: usize = 128;

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

impl<R: Rng> Mutate<R> for Needle {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
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
        v[i] = HAYSTACK_POOL[rng.random_range(0..HAYSTACK_POOL.len())];
        Haystack(v)
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn prop_memchr_iter_matches_naive((Needle(n), Haystack(h)): (Needle, Haystack)) -> Option<bool> {
    to_opt(property_memchr_iter_matches_naive(n, h))
}

fn prop_memrchr_iter_matches_naive((Needle(n), Haystack(h)): (Needle, Haystack)) -> Option<bool> {
    to_opt(property_memrchr_iter_matches_naive(n, h))
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank,
            "file": s.region.file,
            "function": s.region.function,
            "start_line": s.region.start_line,
            "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai,
            "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else {
        serde_json::Value::Null
    };
    let top_5: Vec<_> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            serde_json::json!({
                "rank": s.rank,
                "file": s.region.file,
                "function": s.region.function,
                "start_line": s.region.start_line,
                "end_line": s.region.end_line,
                "confidence": format!("{}", s.confidence),
                "confidence_rule": s.confidence_rule,
                "panic_overlap": s.panic_overlap,
            })
        })
        .collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status,
        "passed": r.run.passed,
        "discarded": r.run.discarded,
        "n_panics": r.n_panics,
        "n_suspects": r.suspects.len(),
        "top": top,
        "top_5": top_5,
        "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_memchr_iter_matches_naive() {
    let report = crabcheck::quickcheck_with_locate!(prop_memchr_iter_matches_naive, "memchr");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_memrchr_iter_matches_naive() {
    let report = crabcheck::quickcheck_with_locate!(prop_memrchr_iter_matches_naive, "memchr");
    eprintln!("{report}");
    emit_locate_json(&report);
}
