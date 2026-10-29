//! P07 — SHACL boundary-cliff bench (WP P07, measure FIRST, then cap).
//!
//! Three isolated sweeps over the real SHACL admission boundary
//! (`graphlaw::shacl_guard::check_shape_limits` + the purrdf shapes
//! engine), nearest-rank p50/p95/p99 per cell:
//!
//! - **depth** `{1,2,4,8,12,16,17}` — 17 is the typed-refusal control (cap
//!   is 16): it MUST refuse typed, never validate.
//! - **degree** `{1..1024}` — one shape with N `sh:property` children; the
//!   cap is 128, so 256..1024 refuse typed in microseconds.
//! - **targets** `{64 B..16 MiB}` — total shapes-graph byte size (many
//!   independent shallow shapes); the cap is 128 KiB, so 256 KiB..16 MiB
//!   refuse typed at the O(1) byte check before any parse.
//!
//! Each cell times two legs: `guard` = the shipped boundary check
//! (`check_shape_limits`) and `engine` = raw `parse_shapes` +
//! `validate_dataset` (the cost the boundary protects). Post-cap, the
//! over-cap cells witness the typed refusal arriving in microseconds.
//!
//! Two extra sections close the WP P07 evidence:
//!
//! - **canonical surface**: the trusted in-tree shapes the repo itself
//!   feeds through the boundary on every `compile_turtle`
//!   (`ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`,
//!   exact shipped bytes via `include_str!`) — must ADMIT under the caps
//!   and its measured cost is the trusted-side baseline.
//! - **adversarial worst-case-at-caps**: inputs built to sit exactly at
//!   each hard cap (including the all-caps composite and a
//!   every-shape-targets-one-focus-node blob), timed over the combined
//!   admission leg — the at-cap Art. 14 measurement this WP records.
//!
//! Every number in `benches/reports/shacl_boundary_cliff.md` comes from a
//! run of this file.

use std::time::Instant;

use graphlaw::purrdf;
use graphlaw::shacl::engine::{parse_shapes, validate_dataset};

/// Nearest-rank quantile over a sorted slice.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((q * (sorted.len() as f64 - 1.0)) + 0.5).floor() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

struct Leg {
    label: String,
    samples_ns: Vec<f64>,
}

impl Leg {
    fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), samples_ns: Vec::new() }
    }

    fn time<F: FnMut()>(&mut self, iters: usize, mut f: F) {
        for _ in 0..iters {
            let t0 = Instant::now();
            f();
            self.samples_ns.push(t0.elapsed().as_secs_f64() * 1e9);
        }
        self.samples_ns
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn stats(&self) -> (f64, f64, f64) {
        (
            quantile(&self.samples_ns, 0.50),
            quantile(&self.samples_ns, 0.95),
            quantile(&self.samples_ns, 0.99),
        )
    }
}

fn fmt_ns(ns: f64) -> String {
    if ns >= 1e6 {
        format!("{:.1} ms", ns / 1e6)
    } else if ns >= 1e3 {
        format!("{:.1} us", ns / 1e3)
    } else {
        format!("{:.0} ns", ns)
    }
}

fn fmt_bytes(b: usize) -> String {
    const KI: usize = 1024;
    const MI: usize = 1024 * 1024;
    if b >= 16 * MI {
        format!("{:.1} MiB", b as f64 / MI as f64)
    } else if b >= MI {
        format!("{:.2} MiB", b as f64 / MI as f64)
    } else if b >= KI {
        format!("{:.1} KiB", b as f64 / KI as f64)
    } else {
        format!("{b} B")
    }
}

const PREFIX: &str = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n";

/// The shipped boundary surface: all four hard physical caps.
fn guard(shapes_ttl: &str) -> Result<(), String> {
    match graphlaw::shacl_guard::check_shape_limits(shapes_ttl) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Acyclic `sh:node` chain of `n` link edges: s0 -> s1 -> ... -> s(n).
fn node_chain(n: usize) -> String {
    let mut ttl = String::from(PREFIX);
    for i in 0..n {
        ttl.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:node ex:s{} .\n",
            i + 1
        ));
    }
    ttl.push_str(&format!("ex:s{n} a sh:NodeShape .\n"));
    ttl
}

/// One shape with `deg` `sh:property` blank-node children (a star).
fn degree_star(deg: usize) -> String {
    let mut ttl = String::from(PREFIX);
    ttl.push_str("ex:root a sh:NodeShape ; sh:targetNode ex:x");
    for i in 0..deg {
        ttl.push_str(&format!(
            " ; sh:property [ sh:path ex:p{i} ; sh:minCount 1 ]",
            i = i
        ));
    }
    ttl.push_str(" .\n");
    ttl
}

/// Many independent shallow shapes padded until the text is ~`target` bytes
/// (a line is only pushed if the result stays <= target).
/// Degree per shape stays tiny; only total byte size grows.
fn byte_blob(target: usize) -> String {
    let mut ttl = String::from(PREFIX);
    let mut i = 0usize;
    loop {
        let line = format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x{i} ; sh:property [ sh:path ex:p{i} ; sh:minCount 1 ] .\n"
        );
        if ttl.len() + line.len() > target {
            break;
        }
        ttl.push_str(&line);
        i += 1;
    }
    ttl
}

/// Worst-case blob at a fixed byte size: every shape targets the SAME focus
/// node, so the engine must evaluate every shape against the data node (the
/// independent blob lets most shapes find no focus node and skip).
fn byte_blob_worst(single_focus_target: usize) -> String {
    let mut ttl = String::from(PREFIX);
    let mut i = 0usize;
    loop {
        let line = format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:property [ sh:path ex:p{i} ; sh:minCount 1 ] .\n"
        );
        if ttl.len() + line.len() > single_focus_target {
            break;
        }
        ttl.push_str(&line);
        i += 1;
    }
    ttl
}

/// All-caps composite: a chain of `MAX_DEPTH - 1` `sh:node` edges whose
/// deepest linking shape carries `MAX_DEGREE - 1` `sh:property` children
/// plus its one `sh:node` link (degree exactly at cap; children sit at
/// depth `MAX_DEPTH`, the last legal level), padded with independent
/// shapes until the text is ~`target` bytes — depth, degree, and bytes all
/// at cap simultaneously without crossing any of them.
fn composite_worst(target: usize) -> String {
    let depth = graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH;
    let deg = graphlaw::shacl_guard::MAX_SHACL_DEGREE;
    let mut ttl = String::from(PREFIX);
    // Chain of depth-1 link edges: s0 -> s1 -> ... -> s(depth-1).
    for i in 0..depth - 1 {
        ttl.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:node ex:s{} .\n",
            i + 1
        ));
    }
    // The deepest linking shape: one sh:node edge (to the terminal) plus
    // (deg - 1) property children = degree exactly at cap.
    ttl.push_str(&format!(
        "ex:s{} a sh:NodeShape ; sh:targetNode ex:x ; sh:node ex:s{depth}",
        depth - 1
    ));
    for i in 0..deg - 1 {
        ttl.push_str(&format!(" ; sh:property [ sh:path ex:p{i} ; sh:minCount 1 ]"));
    }
    ttl.push_str(" .\n");
    ttl.push_str(&format!("ex:s{depth} a sh:NodeShape .\n"));
    // Independent filler shapes (distinct targets) until ~target bytes.
    let mut i = 0usize;
    loop {
        let line = format!(
            "ex:f{i} a sh:NodeShape ; sh:targetNode ex:x{i} ; sh:property [ sh:path ex:p{i} ; sh:minCount 1 ] .\n"
        );
        if ttl.len() + line.len() > target {
            break;
        }
        ttl.push_str(&line);
        i += 1;
    }
    ttl
}

/// Small data graph so the engine legs validate a real focus node.
const DATA_TTL: &str = "@prefix ex: <http://example.org/> .\nex:x a ex:T ; ex:p0 \"v\" .\n";

/// Exact shipped bytes of the trusted canonical shapes surface
/// (`compile_turtle` feeds this concat through the boundary on every call).
const CANONICAL_SHAPES: &str = concat!(
    include_str!("../ontologies/sql-ast.ttl"),
    "\n",
    include_str!("../ggen/sql-dialect/statement_classes.ttl")
);

struct CellResult {
    verdict: String,
    guard: (f64, f64, f64),
    engine: (f64, f64, f64),
}

/// Time one cell: guard leg + engine leg over the same shapes text.
fn cell(shapes: &str, iters: usize) -> CellResult {
    let data = purrdf::parse_dataset(DATA_TTL.as_bytes(), "text/turtle", None)
        .expect("cliff bench: data parses");

    let verdict = match guard(shapes) {
        Ok(()) => "admit".to_string(),
        Err(msg) => format!("REFUSED typed: {msg}"),
    };

    let mut lg = Leg::new("guard");
    lg.time(iters, || {
        let _ = std::hint::black_box(guard(shapes));
    });
    let mut le = Leg::new("engine");
    le.time(iters, || {
        let shapes = parse_shapes(shapes, None).expect("parse_shapes");
        let report = validate_dataset(&data, &shapes).expect("validate");
        std::hint::black_box(report.conforms);
    });

    CellResult { verdict, guard: lg.stats(), engine: le.stats() }
}

fn row(
    name: &str,
    bytes: usize,
    extra: &str,
    r: &CellResult,
) -> String {
    format!(
        "| {name} | {bytes} | {extra} | {} | {} | {} | {} | {} | {} | {} |",
        r.verdict,
        fmt_ns(r.guard.0), fmt_ns(r.guard.1), fmt_ns(r.guard.2),
        fmt_ns(r.engine.0), fmt_ns(r.engine.1), fmt_ns(r.engine.2),
    )
}

fn sweep_depth(iters: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for d in [1usize, 2, 4, 8, 12, 16, 17] {
        let shapes = node_chain(d);
        let r = cell(&shapes, iters);
        if d == 17 {
            assert!(
                r.verdict.starts_with("REFUSED typed"),
                "cliff bench depth-17 control: must refuse typed, got {}",
                r.verdict
            );
        } else {
            assert_eq!(r.verdict, "admit", "cliff bench depth {d} must admit");
        }
        rows.push(row(&format!("depth {d}"), shapes.len(), "", &r));
    }
    rows
}

fn sweep_degree(iters_small: usize, iters_big: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for deg in [1usize, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        let shapes = degree_star(deg);
        let iters = if deg <= 64 { iters_small } else { iters_big };
        let r = cell(&shapes, iters);
        if deg > graphlaw::shacl_guard::MAX_SHACL_DEGREE {
            assert!(
                r.verdict.starts_with("REFUSED typed"),
                "cliff bench degree {deg}: past the cap must refuse typed, got {}",
                r.verdict
            );
        } else {
            assert_eq!(r.verdict, "admit", "cliff bench degree {deg} must admit");
        }
        rows.push(row(&format!("degree {deg}"), shapes.len(), "", &r));
    }
    rows
}

fn sweep_targets(iters_small: usize, iters_mid: usize, iters_big: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for target in [
        64usize,
        1024,
        4 * 1024,
        16 * 1024,
        64 * 1024,
        128 * 1024 - 1,
        256 * 1024,
        1024 * 1024,
        4 * 1024 * 1024,
        16 * 1024 * 1024,
    ] {
        let shapes = byte_blob(target);
        let iters = if target <= 16 * 1024 {
            iters_small
        } else if target <= 1024 * 1024 {
            iters_mid
        } else {
            iters_big
        };
        let r = cell(&shapes, iters);
        if shapes.len() > graphlaw::shacl_guard::MAX_SHACL_SHAPES_BYTES {
            assert!(
                r.verdict.starts_with("REFUSED typed"),
                "cliff bench bytes {}: past the cap must refuse typed, got {}",
                shapes.len(),
                r.verdict
            );
        } else {
            assert_eq!(r.verdict, "admit", "cliff bench bytes {} must admit", shapes.len());
        }
        let quads = purrdf::parse_dataset(shapes.as_bytes(), "text/turtle", None)
            .map(|ds| ds.quads().count())
            .unwrap_or(usize::MAX);
        rows.push(row(
            &fmt_bytes(shapes.len()),
            quads,
            "quads",
            &r,
        ));
    }
    rows
}

/// Canonical trusted surface + adversarial worst-case-at-caps rows.
fn worst_case_at_caps(it_mid: usize, it_huge: usize) -> Vec<(String, usize, usize, CellResult)> {
    let mut out = Vec::new();
    let cap_b = graphlaw::shacl_guard::MAX_SHACL_SHAPES_BYTES;
    let cap_d = graphlaw::shacl_guard::MAX_SHACL_DEGREE;
    let cap_q = graphlaw::shacl_guard::MAX_SHACL_QUADS;

    // Canonical trusted surface (compile_turtle feeds exactly these bytes).
    let canon = CANONICAL_SHAPES;
    let canon_quads = purrdf::parse_dataset(canon.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(canon, it_mid);
    assert_eq!(
        r.verdict, "admit",
        "canonical surface must admit under the caps (bytes {} quads {canon_quads})",
        canon.len()
    );
    out.push(("canonical (sql-ast+statement_classes)".into(), canon.len(), canon_quads, r));

    // Bytes at cap, independent blob.
    let blob = byte_blob(cap_b - 1);
    let quads = purrdf::parse_dataset(blob.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(&blob, it_huge);
    assert_eq!(r.verdict, "admit", "bytes-at-cap blob must admit");
    out.push(("WORST bytes-at-cap (independent blob)".into(), blob.len(), quads, r));

    // Bytes at cap, every shape on one focus node.
    let worst = byte_blob_worst(cap_b - 1);
    let quads = purrdf::parse_dataset(worst.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(&worst, it_huge);
    assert_eq!(r.verdict, "admit", "bytes-at-cap single-focus blob must admit");
    out.push(("WORST bytes-at-cap (single focus node)".into(), worst.len(), quads, r));

    // Degree at cap (star).
    let star = degree_star(cap_d);
    let quads = purrdf::parse_dataset(star.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(&star, it_mid);
    assert_eq!(r.verdict, "admit", "degree-at-cap star must admit");
    out.push(("WORST degree-at-cap (128-child star)".into(), star.len(), quads, r));

    // Depth at cap (chain).
    let chain = node_chain(graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH);
    let quads = purrdf::parse_dataset(chain.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(&chain, it_mid);
    assert_eq!(r.verdict, "admit", "depth-at-cap chain must admit");
    out.push(("WORST depth-at-cap (16-edge chain)".into(), chain.len(), quads, r));

    // All-caps composite.
    let comp = composite_worst(cap_b - 1);
    let quads = purrdf::parse_dataset(comp.as_bytes(), "text/turtle", None)
        .map(|ds| ds.quads().count())
        .unwrap_or(usize::MAX);
    let r = cell(&comp, it_huge);
    assert_eq!(r.verdict, "admit", "all-caps composite must admit");
    assert!(
        quads < cap_q,
        "composite quads {quads} must stay under the quads cap {cap_q} (quads cap is defense-in-depth under the bytes cap)"
    );
    out.push(("WORST all-caps composite".into(), comp.len(), quads, r));

    out
}

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let (it_depth, it_small, it_big, it_mid, it_huge) = if quick {
        (30, 30, 10, 5, 2)
    } else {
        (200, 200, 50, 40, 5)
    };

    println!("# shacl_boundary_cliff run (POST-CAP: check_shape_limits)");
    println!(
        "# os={} arch={} quick={} caps: bytes={} quads={} degree={} depth={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        quick,
        graphlaw::shacl_guard::MAX_SHACL_SHAPES_BYTES,
        graphlaw::shacl_guard::MAX_SHACL_QUADS,
        graphlaw::shacl_guard::MAX_SHACL_DEGREE,
        graphlaw::shacl_guard::MAX_SHACL_VALIDATION_DEPTH,
    );

    println!("\n== Sweep D: shape-nesting depth (cap 16; 17 = typed-refusal control) ==");
    println!("| cell | bytes | guard verdict | guard p50 | guard p95 | guard p99 | engine p50 | engine p95 | engine p99 |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for r in sweep_depth(it_depth) {
        println!("{r}");
    }

    println!("\n== Sweep G: shape-link degree (cap 128; 256+ = typed refusals) ==");
    println!("| cell | bytes | guard verdict | guard p50 | guard p95 | guard p99 | engine p50 | engine p95 | engine p99 |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for r in sweep_degree(it_small, it_big) {
        println!("{r}");
    }

    println!("\n== Sweep S: total shapes bytes (cap 128 KiB; independent shallow shapes) ==");
    println!("| cell | quads | guard verdict | guard p50 | guard p95 | guard p99 | engine p50 | engine p95 | engine p99 |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for r in sweep_targets(it_small, it_mid, it_huge) {
        println!("{r}");
    }

    println!("\n== Canonical trusted surface + adversarial worst-case-at-caps ==");
    println!("| cell | bytes | quads | guard verdict | guard p50 | guard p95 | guard p99 | engine p50 | engine p95 | engine p99 |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for (name, bytes, quads, r) in worst_case_at_caps(it_mid, it_huge) {
        println!(
            "| {name} | {bytes} | {quads} | {} | {} | {} | {} | {} | {} | {} |",
            r.verdict,
            fmt_ns(r.guard.0), fmt_ns(r.guard.1), fmt_ns(r.guard.2),
            fmt_ns(r.engine.0), fmt_ns(r.engine.1), fmt_ns(r.engine.2),
        );
    }

    println!("\n# p50/p95/p99 are nearest-rank over per-iteration Instant samples;");
    println!("# guard = check_shape_limits (bytes->quads->degree->depth, typed refusals);");
    println!("# engine = raw purrdf parse_shapes+validate_dataset (the cost the guard bounds).");
}
