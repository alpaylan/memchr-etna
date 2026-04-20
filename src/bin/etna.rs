// ETNA workload runner for memchr.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: MemchrIterMatchesNaive | MemrchrIterMatchesNaive | All
//
// Every invocation prints exactly one JSON line to stdout and exits 0
// (except argv parsing, which exits 2).

use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use memchr::etna::{
    property_memchr_iter_matches_naive, property_memrchr_iter_matches_naive, PropertyResult,
};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError, TestRunner};
use quickcheck_etna::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &["MemchrIterMatchesNaive", "MemrchrIterMatchesNaive"];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(u64::MAX)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ---------- Canonical witness inputs ----------

fn canonical_haystack_multi_hit() -> Vec<u8> {
    b"aaaabaaaabaaaab".to_vec()
}

fn canonical_haystack_two_separated() -> Vec<u8> {
    b"a____a".to_vec()
}

fn check_memchr_iter_matches_naive() -> Result<(), String> {
    to_err(property_memchr_iter_matches_naive(
        b'b',
        canonical_haystack_multi_hit(),
    ))
}

fn check_memrchr_iter_matches_naive() -> Result<(), String> {
    to_err(property_memrchr_iter_matches_naive(
        b'a',
        canonical_haystack_two_separated(),
    ))
}

// ---------- etna (deterministic witness replay) ----------

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "MemchrIterMatchesNaive" => check_memchr_iter_matches_naive(),
        "MemrchrIterMatchesNaive" => check_memrchr_iter_matches_naive(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            );
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ---------- Shared Arbitrary-biased generators (qc + cc) ----------
//
// Two newtype wrappers around the primitive inputs:
//   * `Needle` — a single byte, biased toward 0..16 and a few printable bytes
//     so that random haystacks actually contain hits.
//   * `Haystack` — a Vec<u8> of length 0..=128 drawn from a small pool that
//     overlaps with the needle pool, so at least some positions match.

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
impl fmt::Display for Needle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Display for Haystack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

const NEEDLE_POOL: &[u8] = &[b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff];
const HAYSTACK_POOL: &[u8] = &[
    b'a', b'a', b'a', b'b', b'c', b'z', 0x00, 0x01, 0x7f, 0xff, b'_', b'.', b' ',
];

fn random_needle<R: Rng>(rng: &mut R) -> u8 {
    NEEDLE_POOL[rng.random_range(0..NEEDLE_POOL.len())]
}

fn random_haystack<R: Rng>(rng: &mut R) -> Vec<u8> {
    let len = rng.random_range(0usize..=128);
    (0..len)
        .map(|_| HAYSTACK_POOL[rng.random_range(0..HAYSTACK_POOL.len())])
        .collect()
}

impl QcArbitrary for Needle {
    fn arbitrary(g: &mut Gen) -> Self {
        Needle(NEEDLE_POOL[g.random_range(0..NEEDLE_POOL.len())])
    }
}

impl QcArbitrary for Haystack {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(0usize..=128);
        Haystack(
            (0..len)
                .map(|_| HAYSTACK_POOL[g.random_range(0..HAYSTACK_POOL.len())])
                .collect(),
        )
    }
}

impl<R: Rng> CcArbitrary<R> for Needle {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Needle(random_needle(rng))
    }
}
impl<R: Rng> CcArbitrary<R> for Haystack {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Haystack(random_haystack(rng))
    }
}

// ---------- proptest ----------

fn needle_strategy() -> BoxedStrategy<u8> {
    prop::sample::select(NEEDLE_POOL.to_vec()).boxed()
}

fn haystack_strategy() -> BoxedStrategy<Vec<u8>> {
    prop::collection::vec(prop::sample::select(HAYSTACK_POOL.to_vec()), 0..=128).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = ProptestConfig {
        cases: cases_budget().min(u32::MAX as u64) as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "MemchrIterMatchesNaive" => runner
            .run(&(needle_strategy(), haystack_strategy()), move |(n, h)| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex_h = h.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_memchr_iter_matches_naive(n, h)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({} {:?})", n, cex_h)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "MemrchrIterMatchesNaive" => runner
            .run(&(needle_strategy(), haystack_strategy()), move |(n, h)| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex_h = h.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_memrchr_iter_matches_naive(n, h)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({} {:?})", n, cex_h)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---------- quickcheck (forked crate with `etna` feature) ----------

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_memchr_iter_matches_naive(Needle(n): Needle, Haystack(h): Haystack) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_memchr_iter_matches_naive(n, h) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_memrchr_iter_matches_naive(Needle(n): Needle, Haystack(h): Haystack) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_memrchr_iter_matches_naive(n, h) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let mut qc = QuickCheck::new()
        .tests(budget)
        .max_tests(budget.saturating_mul(2))
        .max_time(Duration::from_secs(86_400));
    let result = match property {
        "MemchrIterMatchesNaive" => {
            qc.quicktest(qc_memchr_iter_matches_naive as fn(Needle, Haystack) -> TestResult)
        }
        "MemrchrIterMatchesNaive" => {
            qc.quicktest(qc_memrchr_iter_matches_naive as fn(Needle, Haystack) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- crabcheck ----------

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_memchr_iter_matches_naive((Needle(n), Haystack(h)): (Needle, Haystack)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_memchr_iter_matches_naive(n, h) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_memrchr_iter_matches_naive((Needle(n), Haystack(h)): (Needle, Haystack)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_memrchr_iter_matches_naive(n, h) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cc_config = crabcheck_qc::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "MemchrIterMatchesNaive" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_memchr_iter_matches_naive)
        }
        "MemrchrIterMatchesNaive" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_memrchr_iter_matches_naive)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- hegel ----------

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_byte_from(tc: &TestCase, pool: &[u8]) -> u8 {
    let idx = tc.draw(
        hgen::integers::<usize>()
            .min_value(0)
            .max_value(pool.len() - 1),
    );
    pool[idx]
}

fn hg_draw_needle(tc: &TestCase) -> u8 {
    hg_draw_byte_from(tc, NEEDLE_POOL)
}

fn hg_draw_haystack(tc: &TestCase) -> Vec<u8> {
    let len = tc.draw(hgen::integers::<usize>().min_value(0).max_value(128));
    (0..len).map(|_| hg_draw_byte_from(tc, HAYSTACK_POOL)).collect()
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "MemchrIterMatchesNaive" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = hg_draw_needle(&tc);
                let h = hg_draw_haystack(&tc);
                let cex_h = h.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_memchr_iter_matches_naive(n, h)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({} {:?})", n, cex_h),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "MemrchrIterMatchesNaive" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = hg_draw_needle(&tc);
                let h = hg_draw_haystack(&tc);
                let cex_h = h.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_memrchr_iter_matches_naive(n, h)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({} {:?})", n, cex_h),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

// ---------- dispatch ----------

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (
            Err(format!("Unknown tool: {tool}")),
            Metrics::default(),
        ),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!("Properties: MemchrIterMatchesNaive | MemrchrIterMatchesNaive | All");
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
