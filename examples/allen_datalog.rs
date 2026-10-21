//! Allen interval algebra: composition table as Datalog/N3 facts + path-consistency refinement.
//!
//! 1. TRANSCRIPTION: the published Allen 1983 composition table (Alspaugh's Allen
//!    reference, Table 4a, https://www.ics.uci.edu/~alspaugh/cls/shr/allen.html),
//!    transcribed as row strings in the page's column order `p m o F D s e S d f O M P`
//!    (F=fi, D=di, S=si, O=oi, M=mi, P=pi; `full` = all 13; `concur` = oFDseSdfO).
//! 2. ORACLE: verbatim port of the wasm4pm-cognition breed's brute-force composer
//!    (crates/wasm4pm-cognition/src/breeds/allen_temporal.rs:9-71: `rel()` +
//!    `compute_table()` over concrete intervals, coordinates 1..=6).
//! 3. TABLE CHECK: all 13x13 entries (u16 masks) compared against the oracle; every
//!    mismatch printed; any mismatch fails the run (exit 1 at the very end, after all
//!    other steps have run and reported).
//! 4. EMISSION: the ORACLE table is emitted as N3/Datalog facts to
//!    examples/allen_comp_table.n3 (`:r1 :comp_<r2> ( ... ) .`), parsed back with
//!    eyeron, and the operative table for the case runs is rebuilt FROM the parsed facts.
//! 5. CASES: path-consistency as transitive refinement to fixpoint over the fact-loaded
//!    table: `M[k][j] := M[k][j] INTERSECT (M[k][i] o M[i][j])`, queue-based like the
//!    breed. Vectors emitted as `derived:{i}:{j}` facts with `|`-joined masks.
//!
//! Rule-shape note: forward-chained N3 is monotone (adds facts only); Allen refinement is
//! an intersection (removal), so the transitive refinement rules are applied by this
//! harness's fixpoint loop over the emitted Datalog facts; the rule shape is documented
//! in the emitted file header.

use eyeron::{Document, Term};
use std::collections::VecDeque;
use std::fs;

/// This file's axis order (bit i = NAMES[i]).
const NAMES: [&str; 13] = [
    "p", "pi", "m", "mi", "o", "oi", "d", "di", "s", "si", "f", "fi", "eq",
];

/// Page column order (p m o F D s e S d f O M P) -> index into NAMES.
const PAGE_COLS: [usize; 13] = [0, 2, 4, 11, 7, 8, 12, 9, 6, 10, 5, 3, 1];

fn idx(name: &str) -> usize {
    NAMES.iter().position(|n| *n == name).expect("relation name")
}

fn mask_of(spec: &str) -> u16 {
    let mut m = 0u16;
    for t in spec.split('|') {
        if t.is_empty() { continue; }
        m |= 1 << idx(t);
    }
    m
}

/// Expand a published abbreviation token into our `a|b|c` form.
fn expand(token: &str) -> String {
    match token {
        "full" => return NAMES.join("|"),
        "concur" => return "o|fi|di|s|eq|si|d|f|oi".to_string(),
        "pi" | "mi" | "oi" | "di" | "si" | "fi" | "eq" => return token.to_string(),
        _ => {}
    }
    let mut out: Vec<&str> = Vec::new();
    for c in token.chars() {
        out.push(match c {
            'p' => "p", 'm' => "m", 'o' => "o", 'd' => "d", 's' => "s",
            'f' => "f", 'e' => "eq", 'F' => "fi", 'D' => "di", 'S' => "si",
            'O' => "oi", 'M' => "mi", 'P' => "pi",
            _ => panic!("bad token char {c}"),
        });
    }
    out.join("|")
}

// ---------------------------------------------------------------------------
// Transcribed published table (page col order p m o F D s e S d f O M P)
// ---------------------------------------------------------------------------

const PAGE_ROWS: [[&str; 13]; 13] = [
    ["p", "p", "p", "p", "p", "p", "p", "p", "pmosd", "pmosd", "pmosd", "pmosd", "full"],
    ["p", "p", "p", "p", "p", "m", "m", "m", "osd", "osd", "osd", "Fef", "DSOMP"],
    ["p", "p", "pmo", "pmo", "pmoFD", "o", "o", "oFD", "osd", "osd", "concur", "DSO", "DSOMP"],
    ["p", "m", "o", "F", "D", "o", "F", "D", "osd", "Fef", "DSO", "DSO", "DSOMP"],
    ["pmoFD", "oFD", "oFD", "D", "D", "oFD", "D", "D", "concur", "DSO", "DSO", "DSO", "DSOMP"],
    ["p", "p", "pmo", "pmo", "pmoFD", "s", "s", "seS", "d", "d", "dfO", "M", "P"],
    ["p", "m", "o", "F", "D", "s", "e", "S", "d", "f", "O", "M", "P"],
    ["pmoFD", "oFD", "oFD", "D", "D", "seS", "S", "S", "dfO", "O", "O", "M", "P"],
    ["p", "p", "pmosd", "pmosd", "full", "d", "d", "dfOMP", "d", "d", "dfOMP", "P", "P"],
    ["p", "m", "osd", "Fef", "DSOMP", "d", "f", "OMP", "d", "f", "OMP", "P", "P"],
    ["pmoFD", "oFD", "concur", "DSO", "DSOMP", "dfO", "O", "OMP", "dfO", "O", "OMP", "P", "P"],
    ["pmoFD", "seS", "dfO", "M", "P", "dfO", "M", "P", "dfO", "M", "P", "P", "P"],
    ["full", "dfOMP", "dfOMP", "P", "P", "dfOMP", "P", "P", "dfOMP", "P", "P", "P", "P"],
];

fn transcribed_table() -> [[u16; 13]; 13] {
    let mut t = [[0u16; 13]; 13];
    for (r, row) in PAGE_ROWS.iter().enumerate() {
        for (c, tok) in row.iter().enumerate() {
            t[PAGE_COLS[r]][PAGE_COLS[c]] = mask_of(&expand(tok));
        }
    }
    t
}

// ---------------------------------------------------------------------------
// Oracle: verbatim port of allen_temporal.rs:9-71
// ---------------------------------------------------------------------------

const INVERSE: [usize; 13] = [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 12];

const fn rel(s1: i32, e1: i32, s2: i32, e2: i32) -> usize {
    if e1 < s2 { 0 }
    else if e2 < s1 { 1 }
    else if e1 == s2 { 2 }
    else if e2 == s1 { 3 }
    else if s1 < s2 && e1 > s2 && e1 < e2 { 4 }
    else if s2 < s1 && e2 > s1 && e2 < e1 { 5 }
    else if s1 > s2 && e1 < e2 { 6 }
    else if s1 < s2 && e1 > e2 { 7 }
    else if s1 == s2 && e1 < e2 { 8 }
    else if s1 == s2 && e1 > e2 { 9 }
    else if e1 == e2 && s1 > s2 { 10 }
    else if e1 == e2 && s1 < s2 { 11 }
    else { 12 }
}

const fn compute_table() -> [[u16; 13]; 13] {
    let mut t = [[0u16; 13]; 13];
    let mut x_s = 1;
    while x_s <= 6 {
        let mut x_e = x_s + 1;
        while x_e <= 6 {
            let mut y_s = 1;
            while y_s <= 6 {
                let mut y_e = y_s + 1;
                while y_e <= 6 {
                    let mut z_s = 1;
                    while z_s <= 6 {
                        let mut z_e = z_s + 1;
                        while z_e <= 6 {
                            let r1 = rel(x_s, x_e, y_s, y_e);
                            let r2 = rel(y_s, y_e, z_s, z_e);
                            let r3 = rel(x_s, x_e, z_s, z_e);
                            t[r1][r2] |= 1 << r3;
                            z_e += 1;
                        }
                        z_s += 1;
                    }
                    y_e += 1;
                }
                y_s += 1;
            }
            x_e += 1;
        }
        x_s += 1;
    }
    t
}

static COMPOSE_TABLE: [[u16; 13]; 13] = compute_table();

// ---------------------------------------------------------------------------
// Mask helpers + path-consistency
// ---------------------------------------------------------------------------

const FULL: u16 = 8191;

fn mask_str(mask: u16) -> String {
    let mut parts = Vec::new();
    for (i, n) in NAMES.iter().enumerate() {
        if mask & (1 << i) != 0 {
            parts.push(*n);
        }
    }
    parts.join("|")
}

fn compose_cell(a: u16, b: u16, table: &[[u16; 13]; 13]) -> u16 {
    let mut res = 0u16;
    for (i, row) in table.iter().enumerate() {
        if a & (1 << i) != 0 {
            for (j, cell) in row.iter().enumerate() {
                if b & (1 << j) != 0 {
                    res |= cell;
                }
            }
        }
    }
    res
}

fn inverse_mask(m: u16) -> u16 {
    let mut res = 0u16;
    for (i, inv) in INVERSE.iter().enumerate() {
        if m & (1 << i) != 0 {
            res |= 1 << inv;
        }
    }
    res
}

fn path_consistency(
    n: usize,
    mut matrix: Vec<Vec<u16>>,
    names: &[String],
) -> Result<Vec<Vec<u16>>, (String, String)> {
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();
    q.extend(
        (0..n)
            .flat_map(|i| (0..n).map(move |j| (i, j)))
            .filter(|(i, j)| i != j),
    );
    while let Some((i, j)) = q.pop_front() {
        for k in 0..n {
            if k != i && k != j {
                let t = matrix[k][j] & compose_cell(matrix[k][i], matrix[i][j], &COMPOSE_TABLE);
                if t != matrix[k][j] {
                    if t == 0 {
                        return Err((names[k].clone(), names[j].clone()));
                    }
                    matrix[k][j] = t;
                    matrix[j][k] = inverse_mask(t);
                    q.push_back((k, j));
                    q.push_back((j, k));
                }
            }
        }
    }
    Ok(matrix)
}

// ---------------------------------------------------------------------------
// Cases
// ---------------------------------------------------------------------------

fn seed_matrix(n: usize, edges: &[(usize, usize, &str)]) -> Vec<Vec<u16>> {
    let mut matrix = vec![vec![FULL; n]; n];
    for (i, row) in matrix.iter_mut().enumerate().take(n) {
        row[i] = 1 << 12;
    }
    for (i, j, spec) in edges {
        let m = mask_of(spec);
        matrix[*i][*j] &= m;
        matrix[*j][*i] &= inverse_mask(m);
    }
    matrix
}

fn emit_vectors(tag: &str, names: &[&str], matrix: &[Vec<u16>]) {
    println!("-- vectors {tag}");
    for i in 0..names.len() {
        for j in 0..names.len() {
            if i != j {
                println!("derived:{}:{} = {}", names[i], names[j], mask_str(matrix[i][j]));
            }
        }
    }
}

fn run_case(names: &[&str], edges: &[(usize, usize, &str)], tag: &str) -> Vec<Vec<u16>> {
    let snames: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let matrix = seed_matrix(names.len(), edges);
    let matrix = path_consistency(names.len(), matrix, &snames)
        .unwrap_or_else(|(x, y)| panic!("{tag} unexpectedly inconsistent: {x},{y}"));
    emit_vectors(tag, names, &matrix);
    matrix
}

fn run_case_d() {
    let names = ["A", "B", "C"];
    let snames: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let matrix = seed_matrix(3, &[(0, 1, "m"), (1, 2, "m"), (0, 2, "pi")]);
    let result = path_consistency(3, matrix, &snames);
    match result {
        Ok(_) => panic!("case (d) must refine to empty"),
        Err((x, y)) => {
            println!("case (d) inconsistency: refinement to EMPTY between {x} and {y} OK");
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    // ---- 1. Table check: transcribed published table vs oracle (all 169 entries)
    let transcribed = transcribed_table();
    let mut mismatches = 0usize;
    for r1 in 0..13usize {
        for r2 in 0..13usize {
            if transcribed[r1][r2] != COMPOSE_TABLE[r1][r2] {
                mismatches += 1;
                println!(
                    "mismatch [{} o {}] transcribed={} oracle={}",
                    NAMES[r1], NAMES[r2],
                    mask_str(transcribed[r1][r2]),
                    mask_str(COMPOSE_TABLE[r1][r2])
                );
            }
        }
    }
    println!(
        "table-check: transcribed vs brute-force oracle: {}/169 entries match",
        169 - mismatches
    );

    // ---- 2. Emit the ORACLE table as N3/Datalog comp facts
    let mut n3 = String::new();
    n3.push_str("# Generated by examples/allen_datalog.rs -- DO NOT EDIT BY HAND.\n");
    n3.push_str("# Allen 1983 interval-algebra composition table as Datalog facts.\n");
    n3.push_str("#   comp(r1, r2) = set   encoded as   :r1 :comp_<r2> ( :rA :rB ... ) .\n");
    n3.push_str("# Verified against the wasm4pm-cognition allen_temporal breed's brute-force\n");
    n3.push_str("# composer (allen_temporal.rs:40-71) on all 169 entries (see generator output).\n");
    n3.push('\n');
    n3.push_str("# Path-consistency transitive refinement rules (applied to fixpoint by the\n");
    n3.push_str("# generator's engine loop; forward N3 is monotone, refinement is not):\n");
    n3.push_str("#   refine: M[k][j] := M[k][j] INTERSECT compose(M[k][i], M[i][j])\n");
    n3.push_str("#   compose(A, B) = UNION comp(a, b) over a in A, b in B\n");
    n3.push_str("#   inconsistent :- M[k][j] refines to EMPTY\n");
    n3.push_str("@prefix : <urn:allen:> .\n");
    n3.push('\n');
    for r1 in 0..13usize {
        for r2 in 0..13usize {
            let list = mask_str(COMPOSE_TABLE[r1][r2])
                .split('|')
                .map(|s| format!(":{s}"))
                .collect::<Vec<_>>()
                .join(" ");
            n3.push_str(&format!(":{} :comp_{} ( {} ) .\n", NAMES[r1], NAMES[r2], list));
        }
    }
    fs::write("examples/allen_comp_table.n3", &n3).expect("write facts file");
    println!("emitted examples/allen_comp_table.n3 (169 comp facts)");

    // ---- 3. Reparse with eyeron and rebuild the table FROM the parsed facts
    let text = fs::read_to_string("examples/allen_comp_table.n3").expect("read facts file");
    let doc: Document = eyeron::parse_n3(&text, None).expect("emitted facts must be valid N3");
    let mut loaded: [[u16; 13]; 13] = [[0u16; 13]; 13];
    let mut count = 0usize;
    for t in &doc.facts {
        let Term::Iri(s) = &t.s else { continue };
        let Term::Iri(p) = &t.p else { continue };
        let subj = s.as_str();
        let pred = p.as_str();
        let Some(r1name) = subj.rsplit(':').next() else { continue };
        let Some((_, r2name)) = pred.rsplit_once("_") else { continue };
        let (Some(r1), Some(r2)) = (
            NAMES.iter().position(|n| *n == r1name),
            NAMES.iter().position(|n| *n == r2name),
        ) else { continue };
        let Term::List(items) = &t.o else { continue };
        let mut m = 0u16;
        for term in items {
            let Term::Iri(nm) = term else { panic!("list item not an IRI") };
            let name = nm.as_str().rsplit(':').next().expect("nonempty iri");
            m |= 1 << idx(name);
        }
        loaded[r1][r2] = m;
        count += 1;
    }
    assert_eq!(loaded, COMPOSE_TABLE, "facts file rebuilt table != oracle");
    println!("reparse: loaded {count}/169 comp facts from N3, table == oracle");

    // ---- 4. Cases
    let ca = run_case(&["A", "B", "C"], &[(0, 1, "m"), (1, 2, "m")], "case-a-paper-fixture");
    assert_eq!(mask_str(ca[0][2]), "p", "case (a) derived:A:C must be p (paper fixture)");
    println!("case (a) paper fixture: derived:A:C = p ASSERTED OK");
    let cb = run_case(&["i1", "i2", "i3"], &[(0, 1, "m"), (1, 2, "m")], "case-b-meet-chain");
    assert_eq!(mask_str(cb[0][2]), "p", "case (b) derived:i1:i3 must be p");
    println!("case (b) meet-chain: derived:i1:i3 = p ASSERTED OK");

    let mc = run_case(
        &["A", "B", "C", "D"],
        &[(0, 1, "m"), (1, 2, "o"), (2, 3, "p")],
        "case-c-2step",
    );
    assert!(!mask_str(mc[0][2]).is_empty());
    assert!(!mask_str(mc[0][3]).is_empty());
    println!(
        "case (c) 2-step: derived:A:C = {} ; derived:A:D = {} OK",
        mask_str(mc[0][2]),
        mask_str(mc[0][3])
    );

    run_case_d();

    if mismatches > 0 {
        println!("RESULT: exit 1 ({mismatches} transcription mismatches)");
        std::process::exit(1);
    }
    println!("RESULT: exit 0");
}
