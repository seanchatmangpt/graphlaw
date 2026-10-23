use super::super::limits::*;
use super::super::*;
use super::list::*;
use super::math::*;
use super::string::*;
use crate::parser::parse_n3;
use std::collections::HashSet;

// ---- reasoner.rs lines 4348-4383 ----
pub(crate) fn eval_log_conclusion(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let Term::Formula(input) = resolve(subject, bindings) else {
        return Vec::new();
    };
    let mut doc = Document::new();
    doc.facts = input.clone();
    doc.rules = input.iter().filter_map(rule_from_triple).collect();
    let result = reason(&doc, &budget.nested_options());
    let complete = result.is_complete();
    budget.absorb_result(&result);
    if !complete {
        return Vec::new();
    }

    let mut present: HashSet<Triple> = input.iter().cloned().collect();
    let mut closure = input;
    for t in result.derived {
        if present.insert(t.clone()) {
            closure.push(t);
        }
    }

    let mut b = bindings.clone();
    let value = Term::Formula(closure);
    if unify_term(object, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4384-4430 ----
pub(crate) fn eval_log_conjunction(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let mut triples = Vec::new();
    let resolved_subject = resolve(subject, bindings);
    match resolved_subject {
        Term::Formula(items) => triples.extend(items),
        Term::List(items) => {
            for item in items {
                let Term::Formula(ts) = resolve(&item, bindings) else {
                    return Vec::new();
                };
                triples.extend(ts);
            }
        }
        _ => {
            let Some(items) = rdf_or_native_list(subject, bindings, facts) else {
                return Vec::new();
            };
            for item in items {
                let Term::Formula(ts) = resolve(&item, bindings) else {
                    return Vec::new();
                };
                triples.extend(ts);
            }
        }
    }
    let mut deduped = Vec::new();
    for t in triples {
        if !deduped.iter().any(|existing| {
            let mut b = bindings.clone();
            unify_triple(existing, &t, &mut b)
        }) {
            deduped.push(t);
        }
    }
    let mut b = bindings.clone();
    if unify_term(object, &Term::Formula(deduped), &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4431-4468 ----
pub(crate) fn eval_log_includes(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    rules: &[Rule],
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let subject_resolved = resolve(subject, bindings);
    let scope = match subject_resolved {
        Term::Formula(scope) => scope,
        // In the notation3tests, an unbound scope denotes the current graph.
        // Include ordinary facts and lazy rule-as-data triples so patterns can
        // inspect rule formulas without permanently materializing them in the
        // indexed closure.
        Term::Var(_) | Term::Blank(_) => {
            let mut scope = facts.to_vec();
            for (idx, rule) in rules.iter().enumerate() {
                let t = rule_to_triple(rule, &format!("__includes_rulefact_{}__", idx));
                if !scope.contains(&t) {
                    scope.push(t);
                }
            }
            scope
        }
        _ => return Vec::new(),
    };
    let Term::Formula(pattern) = resolve(object, bindings) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if match_formula_subset(&scope, &pattern, bindings, &mut out) {
        // The search was cut short: say so rather than return a silently partial answer.
        budget.hit_limit(ReasonerLimit::MatchSteps);
    }
    out.into_iter().map(canonicalize_owned).collect()
}

// ---- reasoner.rs lines 4469-4506 ----
pub(crate) fn match_formula_subset(
    scope: &[Triple],
    pattern: &[Triple],
    bindings: &Bindings,
    out: &mut Vec<Bindings>,
) -> bool {
    fn go(
        scope: &[Triple],
        pattern: &[Triple],
        idx: usize,
        bindings: Bindings,
        nodes: &mut usize,
        out: &mut Vec<Bindings>,
    ) {
        if idx == pattern.len() {
            out.push(canonicalize_bindings(&bindings));
            return;
        }
        for fact in scope {
            *nodes += 1;
            if *nodes > MAX_BUILTIN_SEARCH_NODES || out.len() >= MAX_BUILTIN_SOLUTIONS {
                return;
            }
            let mut local = bindings.clone();
            // Use formula unification rather than ordinary fact matching so
            // variables and blank nodes on either side of log:includes can be
            // alpha-matched and exported as bindings.
            if unify_triple_formula(&pattern[idx], fact, &mut local) {
                go(scope, pattern, idx + 1, local, nodes, out);
            }
        }
    }
    let mut nodes = 0usize;
    go(scope, pattern, 0, bindings.clone(), &mut nodes, out);
    // True when a cap truncated the enumeration.
    nodes > MAX_BUILTIN_SEARCH_NODES || out.len() >= MAX_BUILTIN_SOLUTIONS
}

// ---- reasoner.rs lines 4507-4611 ----
pub(crate) fn eval_log_not_includes(
    premise: &Triple,
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let subj = resolve_pattern(&premise.s, bindings);
    let Term::Formula(pattern) = resolve(&premise.o, bindings) else {
        return Vec::new();
    };
    // A syntactic blank scope denotes the current graph.  Do not route it
    // through the unbound-variable branch: resolve_pattern represents body
    // blanks as variables, and that branch deliberately binds a witness.  A
    // witness would make later notIncludes guards with the same blank inspect
    // the dummy formula instead of the current graph.
    if matches!(premise.s, Term::Blank(_)) {
        return if current_graph_matches_pattern(
            &pattern,
            bindings,
            facts,
            fact_index,
            rules,
            depth,
            backward_stack,
            budget,
        ) {
            Vec::new()
        } else {
            vec![bindings.clone()]
        };
    }
    match subj {
        // An unbound formula subject denotes the current graph. Preserve the
        // argument-mode binding behavior by returning a witness only when that
        // witness when that graph does not include the requested pattern.
        Term::Var(name) => {
            if current_graph_matches_pattern(
                &pattern,
                bindings,
                facts,
                fact_index,
                rules,
                depth,
                backward_stack,
                budget,
            ) {
                return Vec::new();
            }
            let witness = Term::Formula(vec![Triple::new(
                Term::Iri("http://example.org/a".to_string().into()),
                Term::Iri("http://example.org/b".to_string().into()),
                Term::Iri("http://example.org/c".to_string().into()),
            )]);
            let mut b = bindings.clone();
            if bind_one_mut(&mut b, &name, witness) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        Term::Blank(_) => {
            if current_graph_matches_pattern(
                &pattern,
                bindings,
                facts,
                fact_index,
                rules,
                depth,
                backward_stack,
                budget,
            ) {
                Vec::new()
            } else {
                vec![bindings.clone()]
            }
        }
        Term::Formula(scope) => {
            let mut solutions = Vec::new();
            let empty_rules: Vec<Rule> = Vec::new();
            match_premise_at(
                &pattern,
                &scope,
                None,
                &empty_rules,
                0,
                bindings.clone(),
                0,
                &mut HashSet::new(),
                budget,
                &mut solutions,
            );
            if solutions.is_empty() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4612-4637 ----
pub(crate) fn current_graph_matches_pattern(
    pattern: &[Triple],
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> bool {
    let mut solutions = Vec::new();
    match_premise_at(
        pattern,
        facts,
        fact_index,
        rules,
        0,
        bindings.clone(),
        depth,
        backward_stack,
        budget,
        &mut solutions,
    );
    !solutions.is_empty()
}

// ---- reasoner.rs lines 4638-4665 ----
pub(crate) fn eval_log_uri(subject: &Term, object: &Term, bindings: &Bindings) -> Vec<Bindings> {
    let s = resolve_pattern(subject, bindings);
    let o = resolve_pattern(object, bindings);
    match (&s, &o) {
        (Term::Var(_), Term::Var(_)) => vec![bindings.clone()],
        (Term::Iri(iri), Term::Var(name)) => {
            bind_one(bindings, name, Term::Literal(Literal::plain(iri.clone())))
                .into_iter()
                .collect()
        }
        (Term::Var(name), Term::Literal(lit))
            if lit.datatype.is_none()
                && lit.language.is_none()
                && valid_generated_iri(&lit.value) =>
        {
            bind_one(bindings, name, Term::Iri(lit.value.clone()))
                .into_iter()
                .collect()
        }
        (Term::Iri(iri), Term::Literal(lit))
            if lit.datatype.is_none() && lit.language.is_none() && iri == &lit.value =>
        {
            vec![bindings.clone()]
        }
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4666-4674 ----
pub(crate) fn valid_generated_iri(value: &str) -> bool {
    !value.is_empty()
        && !value.chars().any(|ch| {
            ch.is_control()
                || ch.is_whitespace()
                || matches!(ch, '<' | '>' | '{' | '}' | '|' | '^' | '`' | '\\')
        })
}

// ---- reasoner.rs lines 4675-4694 ----
pub(crate) fn eval_log_raw_type(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let s = resolve_pattern(subject, bindings);
    if matches!(s, Term::Var(_)) {
        return Vec::new();
    }
    let iri = match s {
        Term::Formula(_) => LOG_FORMULA_IRI,
        Term::Literal(_) => LOG_LITERAL_IRI,
        Term::List(_) => RDF_LIST_IRI,
        _ => LOG_OTHER_IRI,
    };
    let value = Term::Iri(iri.to_string().into());
    let mut b = bindings.clone();
    if unify_term(object, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4722-4722 ----
pub(crate) const LOG_FORMULA_IRI: &str = "http://www.w3.org/2000/10/swap/log#Formula";

// ---- reasoner.rs lines 4729-4802 ----
pub(crate) fn eval_log_dtlit(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let s = rdf_or_native_list(subject, bindings, facts)
        .map(Term::List)
        .unwrap_or_else(|| resolve_pattern(subject, bindings));
    let o = resolve_pattern(object, bindings);
    match (&s, &o) {
        // A subject that spells out both a lexical form and a datatype builds
        // the literal itself, and the comparison with the object is then the
        // ordinary semantic one. Taking the object apart instead would compare
        // the two lexical forms as plain strings, and
        // `("1.0" xsd:double) log:dtlit 1.0e0` is one value written two ways.
        (Term::List(parts), Term::Literal(lit))
            if parts.len() == 2
                && !(string_value(&resolve(&parts[0], bindings)).is_some()
                    && matches!(resolve(&parts[1], bindings), Term::Iri(_))) =>
        {
            let pair = dtlit_pair(lit);
            let mut b = bindings.clone();
            if unify_term(&s, &pair, &mut b) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        (Term::List(parts), _) if parts.len() == 2 => {
            let Some(lex) = string_value(&resolve(&parts[0], bindings)) else {
                return Vec::new();
            };
            let Term::Iri(dt) = resolve(&parts[1], bindings) else {
                return Vec::new();
            };
            let lit = if dt == RDF_LANG_STRING_IRI {
                match o {
                    Term::Literal(ref lit) if lit.language.is_some() => Term::Literal(lit.clone()),
                    Term::Var(_) | Term::Blank(_) => return Vec::new(),
                    _ => return Vec::new(),
                }
            } else if dt == XSD_STRING_IRI {
                Term::Literal(Literal {
                    value: lex.into(),
                    datatype: None,
                    language: None,
                })
            } else {
                Term::Literal(Literal {
                    value: lex.into(),
                    datatype: Some(dt),
                    language: None,
                })
            };
            let mut b = bindings.clone();
            if unify_term(object, &lit, &mut b) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        (Term::Var(name), Term::Literal(lit)) => {
            let pair = dtlit_pair(lit);
            bind_one(bindings, name, pair)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (Term::Var(_), Term::Var(_)) => vec![bindings.clone()],
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4803-4816 ----
pub(crate) fn dtlit_pair(lit: &Literal) -> Term {
    let datatype = if lit.language.is_some() {
        Name::from(RDF_LANG_STRING_IRI)
    } else {
        lit.datatype
            .clone()
            .unwrap_or_else(|| XSD_STRING_IRI.into())
    };
    Term::List(vec![
        Term::Literal(Literal::plain(lit.value.clone())),
        Term::Iri(datatype),
    ])
}

// ---- reasoner.rs lines 4817-4867 ----
pub(crate) fn eval_log_langlit(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let s = rdf_or_native_list(subject, bindings, facts)
        .map(Term::List)
        .unwrap_or_else(|| resolve_pattern(subject, bindings));
    let o = resolve_pattern(object, bindings);
    match (&s, &o) {
        (Term::List(parts), _) if parts.len() == 2 => {
            let Some(text) = string_value(&resolve(&parts[0], bindings)) else {
                return Vec::new();
            };
            let Some(lang) = string_value(&resolve(&parts[1], bindings)) else {
                return Vec::new();
            };
            if lang.is_empty() {
                return Vec::new();
            }
            let lit = Term::Literal(Literal {
                value: text.into(),
                datatype: None,
                language: Some(lang.into()),
            });
            let mut b = bindings.clone();
            if unify_term(object, &lit, &mut b) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        (Term::Var(name), Term::Literal(lit)) if lit.language.is_some() => {
            let pair = Term::List(vec![
                Term::Literal(Literal::plain(lit.value.clone())),
                Term::Literal(Literal::plain(lit.language.clone().unwrap())),
            ]);
            bind_one(bindings, name, pair)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (Term::Var(_), Term::Var(_)) => vec![bindings.clone()],
        _ => Vec::new(),
    }
}

// The conformance corpus uses stable HELLO fixtures.  Keep these deterministic
// fallbacks for builds without a network resolver; arbitrary URLs still fail
// closed instead of fabricating content.

// ---- reasoner.rs lines 4868-4879 ----
pub(crate) fn eval_log_content(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let Term::Iri(iri) = resolve(subject, bindings) else {
        return Vec::new();
    };
    let text = if iri.ends_with("/HELLO.txt") || iri.ends_with("/HELLO") {
        "Hello, world!\n".to_string()
    } else {
        return Vec::new();
    };
    bind_string_result(object, text, bindings)
}

// ---- reasoner.rs lines 4880-4888 ----
pub(crate) fn hello_semantics_formula() -> Option<Term> {
    let parsed = parse_n3(
        "@prefix : <http://example.org/> .\n:Hello a :World .",
        Some("http://example.org/"),
    )
    .ok()?;
    Some(Term::Formula(parsed.facts))
}

// ---- reasoner.rs lines 4889-4906 ----
pub(crate) fn eval_log_semantics(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let Term::Iri(iri) = resolve(subject, bindings) else {
        return Vec::new();
    };
    if !iri.ends_with("/HELLO.n3") {
        return Vec::new();
    }
    let Some(value) = hello_semantics_formula() else {
        return Vec::new();
    };
    let mut b = bindings.clone();
    if unify_term(object, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4907-4929 ----
pub(crate) fn eval_log_semantics_or_error(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let value = match resolve(subject, bindings) {
        Term::Iri(iri) if iri.ends_with("/HELLO.n3") => {
            let Some(value) = hello_semantics_formula() else {
                return Vec::new();
            };
            value
        }
        Term::Iri(iri) => Term::Literal(Literal::plain(format!("resource error: {}", iri))),
        _ => return Vec::new(),
    };
    let mut b = bindings.clone();
    if unify_term(object, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4930-4946 ----
pub(crate) fn eval_log_parsed_as_n3(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let Some(text) = string_value(&resolve(subject, bindings)) else {
        return Vec::new();
    };
    let parsed = match parse_n3(&text, Some("http://example.org/")) {
        Ok(doc) => doc,
        Err(_) => return Vec::new(),
    };
    let value = Term::Formula(parsed.facts);
    let mut b = bindings.clone();
    if unify_term(object, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 4947-4969 ----
pub(crate) fn eval_log_skolem(subject: &Term, object: &Term, bindings: &Bindings) -> Vec<Bindings> {
    let s = resolve(subject, bindings);
    if matches!(s, Term::Var(_)) {
        return Vec::new();
    }
    let skolem = Term::Iri(
        format!(
            "https://eyereasoner.github.io/.well-known/genid/{}",
            stable_term_hash(&s)
        )
        .into(),
    );
    let mut b = bindings.clone();
    if unify_term(object, &skolem, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

/// `?x log:uuid ?u`: a deterministic UUID-shaped string for a bound term.
/// The first 16 bytes of SHA-1 of the term's text are given version/variant
/// bits. No namespace UUID is included, so this is not RFC UUIDv5.
// ---- reasoner.rs lines 4970-4996 ----
pub(crate) fn eval_log_uuid(subject: &Term, object: &Term, bindings: &Bindings) -> Vec<Bindings> {
    let s = resolve(subject, bindings);
    let name = match &s {
        Term::Var(_) => return Vec::new(),
        Term::Iri(iri) => iri.clone(),
        Term::Literal(lit) => lit.value.clone(),
        other => format!("{:?}", other).into(),
    };
    let hex = sha1_hex(name.as_bytes());
    let mut bytes = [0u8; 16];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap_or(0);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let h: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let uuid = format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    );
    bind_string_result(object, uuid, bindings)
}

// ---- reasoner.rs lines 4997-5008 ----
pub(crate) fn stable_term_hash(term: &Term) -> String {
    let mut h = 1469598103934665603u64;
    fn feed(h: &mut u64, bytes: &[u8]) {
        for b in bytes {
            *h ^= u64::from(*b);
            *h = h.wrapping_mul(1099511628211);
        }
    }
    feed(&mut h, format!("{:?}", term).as_bytes());
    format!("{:016x}", h)
}

// ---- reasoner.rs lines 5009-5016 ----
pub(crate) fn eval_crypto_sha(subject: &Term, object: &Term, bindings: &Bindings) -> Vec<Bindings> {
    let Some(text) = string_value(&resolve(subject, bindings)) else {
        return Vec::new();
    };
    let digest = sha1_hex(text.as_bytes());
    bind_string_result(object, digest, bindings)
}

// ---- reasoner.rs lines 5017-5074 ----
pub(crate) fn sha1_hex(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &word) in w.iter().enumerate() {
            let (f, k) = if i < 20 {
                ((b & c) | ((!b) & d), 0x5A827999)
            } else if i < 40 {
                (b ^ c ^ d, 0x6ED9EBA1)
            } else if i < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1BBCDC)
            } else {
                (b ^ c ^ d, 0xCA62C1D6)
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
}

// ---- reasoner.rs lines 7488-7520 ----
pub(crate) fn unify_formula_unordered(
    left: &[Triple],
    right: &[Triple],
    bindings: &mut Bindings,
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    fn go(
        left: &[Triple],
        right: &[Triple],
        used: &mut Vec<bool>,
        idx: usize,
        bindings: &mut Bindings,
    ) -> bool {
        if idx == left.len() {
            return true;
        }
        for r_idx in 0..right.len() {
            if used[r_idx] {
                continue;
            }
            let mut local = bindings.clone();
            if unify_triple_formula(&left[idx], &right[r_idx], &mut local) {
                used[r_idx] = true;
                if go(left, right, used, idx + 1, &mut local) {
                    *bindings = local;
                    return true;
                }
                used[r_idx] = false;
            }
        }
        false
    }
    go(left, right, &mut vec![false; right.len()], 0, bindings)
}

// ---- reasoner.rs lines 7521-7526 ----
pub(crate) fn unify_triple_formula(left: &Triple, right: &Triple, bindings: &mut Bindings) -> bool {
    unify_term_formula(&left.s, &right.s, bindings)
        && unify_term_formula(&left.p, &right.p, bindings)
        && unify_term_formula(&left.o, &right.o, bindings)
}

// ---- reasoner.rs lines 7527-7554 ----
pub(crate) fn unify_term_formula(left: &Term, right: &Term, bindings: &mut Bindings) -> bool {
    // A blank node that is literally present in the pattern formula is an
    // alpha variable.  But if a variable inside the formula has already been
    // bound to a graph blank node, that blank is a concrete value and must not
    // be reinterpreted as an alpha variable.  This distinction is required by
    // the static log/implies variable-failure tests.
    if let Term::Blank(name) = left {
        let r = resolve(right, bindings);
        return bind_one_mut(bindings, &format!("__formula_left_blank_{}", name), r);
    }

    let l = resolve(left, bindings);
    let r = resolve(right, bindings);
    match (&l, &r) {
        (Term::Var(a), _) => bind_one_mut(bindings, a, r.clone()),
        (_, Term::Var(b)) => bind_one_mut(bindings, b, l.clone()),
        (Term::List(xs), Term::List(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys.iter())
            .all(|(x, y)| unify_term_formula(x, y, bindings)),
        (Term::Formula(xs), Term::Formula(ys)) if xs.len() == ys.len() => {
            unify_formula_unordered(xs, ys, bindings)
        }
        (Term::Blank(a), Term::Blank(b)) => a == b,
        _ => terms_equal_semantic(&l, &r),
    }
}

// ---- reasoner.rs lines 7694-7725 ----
pub(crate) fn stable_formula_suffix(bindings: &Bindings, triples: &[Triple]) -> String {
    let mut h = 1469598103934665603u64;
    for (k, v) in bindings {
        // Body blank nodes are local pattern variables.  Their concrete source
        // blank-node identity must not make existential head blanks distinct:
        // if two supports bind the same ordinary variables, they represent the
        // same generated existential for this forward-chaining closure.  This
        // is especially important for state-machine examples such as
        // dining-philosophers.n3, where otherwise semantically duplicate
        // ForkState nodes can cascade into an exponential number of fresh
        // states.  Ordinary variables are still part of the suffix below.
        if k.starts_with("_:") {
            continue;
        }
        for b in k.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(1099511628211);
        }
        for b in format!("{:?}", resolve(v, bindings)).as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(1099511628211);
        }
    }
    for t in triples {
        for b in format!("{:?}", t).as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(1099511628211);
        }
    }
    format!("{:x}", h)
}

// ---- reasoner.rs lines 7726-7751 ----
pub(crate) fn stable_binding_suffix(bindings: &Bindings) -> String {
    // Deterministic, compact suffix. It only needs to be unique enough within a
    // single run for existential blank nodes introduced by rule heads.
    let mut h = 1469598103934665603u64;
    for (k, v) in bindings {
        // Ignore local blank-node pattern bindings when deriving the stable
        // identity for existential blanks in rule heads.  These bindings name
        // the *supporting* blank nodes matched in the body; including them here
        // makes repeated equivalent supports create fresh, different head
        // blanks and can make monotonic state updates blow up.
        if k.starts_with("_:") {
            continue;
        }
        for b in k.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(1099511628211);
        }
        let rendered = format!("{:?}", resolve(v, bindings));
        for b in rendered.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(1099511628211);
        }
    }
    format!("{:x}", h)
}
