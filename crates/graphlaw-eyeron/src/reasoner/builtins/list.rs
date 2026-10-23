use super::super::limits::*;
use super::super::*;
use super::log::*;
use super::math::*;
use super::*;
use std::collections::{BTreeMap, HashMap, HashSet};

// ---- reasoner.rs lines 5075-5106 ----
pub(crate) fn rdf_or_native_list(
    term: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Option<Vec<Term>> {
    // Most blank nodes seen here are concrete RDF list nodes coming from facts.
    // They must stay concrete so nested RDF lists such as list:append inputs can
    // be followed through rdf:first/rdf:rest.  The one exception is a syntactic
    // blank node from a rule body that has already been bound as a local pattern
    // witness, e.g. after matching `_:x rdf:first 1`.  In that case resolve the
    // blank through the pattern-binding key so the later list built-in sees the
    // native list or graph node it matched.
    let resolved = match term {
        Term::Blank(name) if bindings.contains_key(&blank_binding_name(name)) => {
            resolve_pattern(term, bindings)
        }
        Term::Var(_) => resolve_pattern(term, bindings),
        _ => resolve(term, bindings),
    };
    rdf_or_native_list_resolved(&resolved, facts, &mut HashSet::new())
}

/// Walks an RDF list encoded as `rdf:first`/`rdf:rest` triples (or a native
/// `Term::List`, or `rdf:nil`).  Iterative, not recursive: the earlier
/// recursive version used one stack frame per list cell with no depth bound,
/// so a chain of ordinary Turtle triples -- no nesting, nothing hostile,
/// just a moderately long list -- could overflow the stack (measured:
/// ~3,000 cells on a 1 MiB thread stack, ~27,000 on an 8 MB one), which
/// aborts the process and cannot be caught.
///
/// Also builds one first/rest lookup ([`rdf_list_links`]) instead of
/// `rdf_list_object` scanning the whole fact list per cell, which made a
/// chain of length L cost O(L x len(facts)) (measured: 8,000 cells 1.9 s,
/// 16,000 cells 6.5 s -- quadratic, not linear). The native-list and
/// `rdf:nil` cases below return before touching `facts` at all, so this
/// costs nothing extra for the common case of a parsed `( ... )` list.
// ---- reasoner.rs lines 5107-5145 ----
pub(crate) fn rdf_or_native_list_resolved(
    term: &Term,
    facts: &[Triple],
    seen: &mut HashSet<Term>,
) -> Option<Vec<Term>> {
    match term {
        Term::List(items) => return Some(items.clone()),
        Term::Iri(iri) if iri == RDF_NIL => return Some(Vec::new()),
        Term::Blank(_) | Term::Iri(_) => {}
        _ => return None,
    }
    let (first_of, rest_of) = rdf_list_links(facts);
    let mut out = Vec::new();
    let mut current = term.clone();
    loop {
        match &current {
            Term::Iri(iri) if iri == RDF_NIL => return Some(out),
            // A rest pointing at a native (already-parenthesised) list tail:
            // append it and stop, matching the prior recursive base case.
            Term::List(items) => {
                out.extend(items.iter().cloned());
                return Some(out);
            }
            Term::Blank(_) | Term::Iri(_) => {
                if !seen.insert(current.clone()) {
                    return None;
                }
                out.push(first_of.get(&current)?.clone());
                current = rest_of.get(&current)?.clone();
            }
            _ => return None,
        }
    }
}

/// One O(len(facts)) pass building `rdf:first`/`rdf:rest` lookup maps, so a
/// list of length L costs one scan plus L O(1) lookups instead of L scans.
/// "First matching triple wins" when a subject has more than one `rdf:first`
/// or `rdf:rest` triple, matching the linear-scan `.find()` this replaces.
// ---- reasoner.rs lines 5146-5160 ----
pub(crate) fn rdf_list_links(facts: &[Triple]) -> (HashMap<Term, Term>, HashMap<Term, Term>) {
    let first_pred = Term::Iri(RDF_FIRST.to_string().into());
    let rest_pred = Term::Iri(RDF_REST.to_string().into());
    let mut first_of = HashMap::new();
    let mut rest_of = HashMap::new();
    for t in facts {
        if t.p == first_pred {
            first_of.entry(t.s.clone()).or_insert_with(|| t.o.clone());
        } else if t.p == rest_pred {
            rest_of.entry(t.s.clone()).or_insert_with(|| t.o.clone());
        }
    }
    (first_of, rest_of)
}

// ---- reasoner.rs lines 5161-5181 ----
pub(crate) fn unify_listish(
    term: &Term,
    items: Vec<Term>,
    bindings: &mut Bindings,
    facts: &[Triple],
) -> bool {
    if unify_term(term, &Term::List(items.clone()), bindings) {
        return true;
    }
    let resolved = resolve(term, bindings);
    if let Some(existing) = rdf_or_native_list_resolved(&resolved, facts, &mut HashSet::new()) {
        if existing.len() == items.len()
            && existing
                .iter()
                .zip(items.iter())
                .all(|(a, b)| terms_equal_semantic(a, b))
        {
            return true;
        }
    }
    if items.is_empty() {
        return unify_term(term, &Term::Iri(RDF_NIL.to_string().into()), bindings);
    }
    false
}

// ---- reasoner.rs lines 5182-5210 ----
pub(crate) fn unify_listish_loose_numeric(
    term: &Term,
    items: Vec<Term>,
    bindings: &mut Bindings,
    facts: &[Triple],
) -> bool {
    let list_term = Term::List(items.clone());
    let mut local = bindings.clone();
    if unify_list_terms_loose_numeric(term, &list_term, &mut local, facts) {
        *bindings = local;
        return true;
    }
    let resolved = resolve(term, bindings);
    if let Some(existing) = rdf_or_native_list_resolved(&resolved, facts, &mut HashSet::new()) {
        if existing.len() == items.len()
            && existing
                .iter()
                .zip(items.iter())
                .all(|(a, b)| terms_equal_loose_numeric(a, b))
        {
            return true;
        }
    }
    if items.is_empty() {
        return unify_term(term, &Term::Iri(RDF_NIL.to_string().into()), bindings);
    }
    false
}

// ---- reasoner.rs lines 5211-5236 ----
pub(crate) fn unify_list_terms_loose_numeric(
    left: &Term,
    right: &Term,
    bindings: &mut Bindings,
    facts: &[Triple],
) -> bool {
    let l = resolve_pattern(left, bindings);
    let r = resolve_pattern(right, bindings);
    match (&l, &r) {
        (Term::Var(a), _) => bind_one_mut(bindings, a, r.clone()),
        (_, Term::Var(b)) => bind_one_mut(bindings, b, l.clone()),
        _ => {
            let l_items = rdf_or_native_list_resolved(&l, facts, &mut HashSet::new());
            let r_items = rdf_or_native_list_resolved(&r, facts, &mut HashSet::new());
            if let (Some(xs), Some(ys)) = (l_items, r_items) {
                return xs.len() == ys.len()
                    && xs
                        .iter()
                        .zip(ys.iter())
                        .all(|(x, y)| unify_list_terms_loose_numeric(x, y, bindings, facts));
            }
            terms_equal_loose_numeric(&l, &r)
        }
    }
}

// ---- reasoner.rs lines 5237-5251 ----
pub(crate) fn terms_equal_loose_numeric(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Literal(_), Term::Literal(_))
            if numeric_value(a).is_some() && numeric_value(b).is_some() =>
        {
            numeric_terms_equal(a, b)
        }
        (Term::List(xs), Term::List(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys.iter())
            .all(|(x, y)| terms_equal_loose_numeric(x, y)),
        _ => terms_equal_semantic(a, b),
    }
}

// ---- reasoner.rs lines 5252-5309 ----
pub(crate) fn eval_list_append(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let Some(parts) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };

    // Fast path: preserve the existing deterministic concatenation behavior
    // when every input part is already list-shaped/resolvable.  Fixed-shape
    // native lists may still contain variables; unifying the concatenation
    // with the result binds those element variables as before.
    let mut concatenated = Vec::new();
    let mut all_parts_resolved = true;
    for part in &parts {
        let Some(items) = rdf_or_native_list(part, bindings, facts) else {
            all_parts_resolved = false;
            break;
        };
        concatenated.extend(items);
    }
    if all_parts_resolved {
        let mut b = bindings.clone();
        return if unify_listish_loose_numeric(right, concatenated, &mut b, facts) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }

    // Relational mode: if the result is known, infer unresolved whole-list
    // parts by partitioning that result while respecting the fixed lengths of
    // neighbouring list patterns.  This supports forms such as
    // `((?head) ?tail) list:append (1 2 3)`, yielding ?head=1 and ?tail=(2 3).
    let Some(result_items) = rdf_or_native_list(right, bindings, facts) else {
        return Vec::new();
    };
    let mut out = Vec::<Bindings>::new();
    let mut nodes = 0usize;
    match_list_append_parts(
        &parts,
        &result_items,
        0,
        0,
        bindings,
        facts,
        &mut nodes,
        &mut out,
    );
    if nodes > MAX_BUILTIN_SEARCH_NODES || out.len() >= MAX_BUILTIN_SOLUTIONS {
        budget.hit_limit(ReasonerLimit::MatchSteps);
    }
    out
}

// ---- reasoner.rs lines 5310-5405 ----
pub(crate) fn match_list_append_parts(
    parts: &[Term],
    result_items: &[Term],
    part_index: usize,
    result_index: usize,
    bindings: &Bindings,
    facts: &[Triple],
    nodes: &mut usize,
    out: &mut Vec<Bindings>,
) {
    *nodes += 1;
    if *nodes > MAX_BUILTIN_SEARCH_NODES || out.len() >= MAX_BUILTIN_SOLUTIONS {
        return;
    }
    if part_index == parts.len() {
        if result_index == result_items.len() {
            let solution = canonicalize_bindings(bindings);
            if !out.contains(&solution) {
                out.push(solution);
            }
        }
        return;
    }
    if result_index > result_items.len() {
        return;
    }

    let part = &parts[part_index];
    let resolved = resolve_pattern(part, bindings);

    // Known/fixed-shape parts consume exactly their list length.  A native list
    // containing element variables is fixed-shape even though it is not ground.
    if !matches!(resolved, Term::Var(_)) {
        let Some(items) = rdf_or_native_list(part, bindings, facts) else {
            return;
        };
        let len = items.len();
        if result_index + len > result_items.len() {
            return;
        }
        let mut next = bindings.clone();
        if unify_listish_loose_numeric(
            part,
            result_items[result_index..result_index + len].to_vec(),
            &mut next,
            facts,
        ) {
            match_list_append_parts(
                parts,
                result_items,
                part_index + 1,
                result_index + len,
                &next,
                facts,
                nodes,
                out,
            );
        }
        return;
    }

    // An unresolved top-level variable denotes an unknown list segment.  The
    // known result makes the search finite.  Reserve the minimum length needed
    // by all remaining fixed-shape parts and try each possible segment length.
    let Some(min_remaining) = list_append_minimum_len(&parts[part_index + 1..], bindings, facts)
    else {
        return;
    };
    if result_index + min_remaining > result_items.len() {
        return;
    }
    let max_len = result_items.len() - result_index - min_remaining;

    for len in 0..=max_len {
        let mut next = bindings.clone();
        if !unify_listish_loose_numeric(
            part,
            result_items[result_index..result_index + len].to_vec(),
            &mut next,
            facts,
        ) {
            continue;
        }
        match_list_append_parts(
            parts,
            result_items,
            part_index + 1,
            result_index + len,
            &next,
            facts,
            nodes,
            out,
        );
    }
}

// ---- reasoner.rs lines 5406-5417 ----
pub(crate) fn list_append_minimum_len(
    parts: &[Term],
    bindings: &Bindings,
    facts: &[Triple],
) -> Option<usize> {
    let mut total = 0usize;
    for part in parts {
        let resolved = resolve_pattern(part, bindings);
        if matches!(resolved, Term::Var(_)) {
            continue;
        }
        total = total.checked_add(rdf_or_native_list(part, bindings, facts)?.len())?;
    }
    Some(total)
}

// ---- reasoner.rs lines 5418-5437 ----
pub(crate) fn eval_list_iterate(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (idx, value) in items.into_iter().enumerate() {
        let pair = Term::List(vec![numeric_literal(idx as f64, true), value]);
        let mut b = bindings.clone();
        if unify_term_list_builtin_facts(right, &pair, &mut b, facts) {
            out.push(canonicalize_owned(b));
        }
    }
    out
}

// ---- reasoner.rs lines 5438-5498 ----
pub(crate) fn eval_list_map(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let Some(parts) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    if parts.len() != 2 {
        return Vec::new();
    }
    let Some(inputs) = rdf_or_native_list(&parts[0], bindings, facts) else {
        return Vec::new();
    };
    let Term::Iri(pred) = resolve(&parts[1], bindings) else {
        return Vec::new();
    };

    let y = "__list_map_y".to_string();
    let mut mapped = Vec::new();
    for input in inputs {
        if !input.is_ground() {
            return Vec::new();
        }
        let goal = Triple::new(input, Term::Iri(pred.clone()), Term::Var(y.clone().into()));
        let mut sols = Vec::new();
        match_premise_at(
            &[goal],
            facts,
            fact_index,
            rules,
            0,
            bindings.clone(),
            depth + 1,
            backward_stack,
            budget,
            &mut sols,
        );
        for sol in sols {
            let value = resolve(&Term::Var(y.clone().into()), &sol);
            if !matches!(value, Term::Var(_)) {
                mapped.push(value);
            }
        }
    }

    let result = Term::List(mapped);
    let mut b = bindings.clone();
    if unify_term(right, &result, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 5499-5538 ----
pub(crate) fn eval_list_first_rest(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    if let Some(items) = rdf_or_native_list(left, bindings, facts) {
        if items.is_empty() {
            return Vec::new();
        }
        let pair = Term::List(vec![items[0].clone(), Term::List(items[1..].to_vec())]);
        let mut b = bindings.clone();
        return if unify_term(right, &pair, &mut b) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }

    let right_value = resolve(right, bindings);
    let Term::List(pair) = right_value else {
        return Vec::new();
    };
    if pair.len() != 2 {
        return Vec::new();
    }
    let Some(rest) = rdf_or_native_list(&pair[1], bindings, facts) else {
        return Vec::new();
    };
    let mut items = Vec::with_capacity(rest.len() + 1);
    items.push(pair[0].clone());
    items.extend(rest);
    let mut b = bindings.clone();
    if unify_listish(left, items, &mut b, facts) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 5539-5565 ----
pub(crate) fn eval_list_reverse(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    if let Some(mut items) = rdf_or_native_list(left, bindings, facts) {
        items.reverse();
        let mut b = bindings.clone();
        return if unify_listish(right, items, &mut b, facts) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }
    if let Some(mut items) = rdf_or_native_list(right, bindings, facts) {
        items.reverse();
        let mut b = bindings.clone();
        return if unify_listish(left, items, &mut b, facts) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }
    Vec::new()
}

// ---- reasoner.rs lines 5566-5594 ----
pub(crate) fn eval_list_sort(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let input = rdf_or_native_list(left, bindings, facts)
        .map(|items| (items, true))
        .or_else(|| rdf_or_native_list(right, bindings, facts).map(|items| (items, false)));
    let Some((mut items, left_was_input)) = input else {
        return Vec::new();
    };
    if !items.iter().all(Term::is_ground) {
        return Vec::new();
    }
    items.sort_by(compare_terms_for_list_sort);
    let mut out = bindings.clone();
    let ok = if left_was_input {
        unify_listish(right, items, &mut out, facts)
    } else {
        unify_listish(left, items, &mut out, facts)
    };
    if ok {
        vec![canonicalize_owned(out)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 5595-5623 ----
pub(crate) fn compare_terms_for_list_sort(a: &Term, b: &Term) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Term::Literal(a), Term::Literal(b)) => {
            match (
                numeric_value(&Term::Literal(a.clone())),
                numeric_value(&Term::Literal(b.clone())),
            ) {
                (Some(a), Some(b)) => a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal),
                _ => a.value.cmp(&b.value),
            }
        }
        (Term::List(a), Term::List(b)) => {
            for (a, b) in a.iter().zip(b) {
                let ordering = compare_terms_for_list_sort(a, b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            a.len().cmp(&b.len())
        }
        (Term::Iri(a), Term::Iri(b)) => a.cmp(b),
        (Term::List(_), _) => Ordering::Less,
        (_, Term::List(_)) => Ordering::Greater,
        _ => format!("{:?}", a).cmp(&format!("{:?}", b)),
    }
}

// ---- reasoner.rs lines 5624-5641 ----
pub(crate) fn eval_list_not_member(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    for item in items {
        let mut b = bindings.clone();
        if unify_term(right, &item, &mut b) {
            return Vec::new();
        }
    }
    vec![bindings.clone()]
}

// ---- reasoner.rs lines 5642-5648 ----
pub(crate) fn is_list_builtin(iri: &str) -> bool {
    matches!(
        iri,
        LIST_LAST | LIST_LENGTH | LIST_MEMBER | LIST_IN | LIST_MEMBER_AT | LIST_REMOVE
    )
}

// ---- reasoner.rs lines 5649-5664 ----
pub(crate) fn terms_equal_for_remove(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Literal(x), Term::Literal(y)) => {
            if numeric_value(a).is_some() || numeric_value(b).is_some() {
                return x.datatype == y.datatype && numeric_terms_equal(a, b);
            }
            literals_equal_semantic(x, y)
        }
        (Term::List(xs), Term::List(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys.iter())
            .all(|(x, y)| terms_equal_for_remove(x, y)),
        _ => terms_equal_semantic(a, b),
    }
}

// ---- reasoner.rs lines 5665-5689 ----
pub(crate) fn terms_equal_list_builtin(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Literal(x), Term::Literal(y)) => {
            let xn = numeric_value(a).is_some();
            let yn = numeric_value(b).is_some();
            if xn || yn {
                return x.datatype == y.datatype && numeric_terms_equal(a, b);
            }
            literals_equal_semantic(x, y)
        }
        (Term::List(xs), Term::List(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys.iter())
            .all(|(x, y)| terms_equal_list_builtin(x, y)),
        (Term::List(xs), Term::Iri(iri)) | (Term::Iri(iri), Term::List(xs)) => {
            xs.is_empty() && iri == RDF_NIL
        }
        (Term::Formula(xs), Term::Formula(ys)) if xs.len() == ys.len() => {
            let mut b = BTreeMap::new();
            unify_formula_unordered(xs, ys, &mut b)
        }
        _ => a == b,
    }
}

// ---- reasoner.rs lines 5690-5718 ----
pub(crate) fn unify_term_list_builtin_facts(
    left: &Term,
    right: &Term,
    bindings: &mut Bindings,
    facts: &[Triple],
) -> bool {
    let l = resolve_pattern(left, bindings);
    // The right-hand side is the candidate value supplied by the builtin.
    // Preserve graph blank nodes as concrete values instead of reopening them
    // as local blank-pattern variables.
    let r = resolve(right, bindings);
    match (&l, &r) {
        (Term::Var(a), _) => bind_one_mut(bindings, a, r.clone()),
        (_, Term::Var(b)) => bind_one_mut(bindings, b, l.clone()),
        _ => {
            let l_items = rdf_or_native_list_resolved(&l, facts, &mut HashSet::new());
            let r_items = rdf_or_native_list_resolved(&r, facts, &mut HashSet::new());
            if let (Some(xs), Some(ys)) = (l_items, r_items) {
                return xs.len() == ys.len()
                    && xs
                        .iter()
                        .zip(ys.iter())
                        .all(|(x, y)| unify_term_list_builtin_facts(x, y, bindings, facts));
            }
            terms_equal_list_builtin(&l, &r)
        }
    }
}

// ---- reasoner.rs lines 5719-5831 ----
pub(crate) fn eval_list_builtin(
    pred: &str,
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    match pred {
        LIST_LAST => {
            let Some(items) = rdf_or_native_list(left, bindings, facts) else {
                return Vec::new();
            };
            let Some(last) = items.last().cloned() else {
                return Vec::new();
            };
            let mut b = bindings.clone();
            if unify_term_list_builtin_facts(right, &last, &mut b, facts) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        LIST_LENGTH => {
            let Some(items) = rdf_or_native_list(left, bindings, facts) else {
                return Vec::new();
            };
            let value = numeric_literal(items.len() as f64, true);
            let mut b = bindings.clone();
            if unify_term_list_builtin_facts(right, &value, &mut b, facts) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        LIST_MEMBER => {
            let Some(items) = rdf_or_native_list(left, bindings, facts) else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for item in items {
                let mut b = bindings.clone();
                if unify_term_list_builtin_facts(right, &item, &mut b, facts) {
                    out.push(canonicalize_owned(b));
                }
            }
            out
        }
        LIST_IN => {
            let item = resolve(left, bindings);
            let Some(items) = rdf_or_native_list(right, bindings, facts) else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for candidate in items {
                let mut b = bindings.clone();
                if unify_term_list_builtin_facts(&item, &candidate, &mut b, facts) {
                    out.push(canonicalize_owned(b));
                }
            }
            out
        }
        LIST_MEMBER_AT => {
            let Some(parts) = rdf_or_native_list(left, bindings, facts) else {
                return Vec::new();
            };
            if parts.len() != 2 {
                return Vec::new();
            }
            let Some(items) = rdf_or_native_list(&parts[0], bindings, facts) else {
                return Vec::new();
            };
            let Some(idx) = numeric_value(&resolve(&parts[1], bindings)) else {
                return Vec::new();
            };
            if idx.value < 0.0 || idx.value.fract() != 0.0 {
                return Vec::new();
            }
            let Some(value) = items.get(idx.value as usize).cloned() else {
                return Vec::new();
            };
            let mut b = bindings.clone();
            if unify_term_list_builtin_facts(right, &value, &mut b, facts) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        LIST_REMOVE => {
            let Some(parts) = rdf_or_native_list(left, bindings, facts) else {
                return Vec::new();
            };
            if parts.len() != 2 {
                return Vec::new();
            }
            let Some(items) = rdf_or_native_list(&parts[0], bindings, facts) else {
                return Vec::new();
            };
            let remove = resolve(&parts[1], bindings);
            let kept = items
                .into_iter()
                .filter(|item| !terms_equal_for_remove(item, &remove))
                .collect::<Vec<_>>();
            let mut b = bindings.clone();
            if unify_listish(right, kept, &mut b, facts) {
                vec![canonicalize_owned(b)]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 5832-5884 ----
pub(crate) fn eval_rdf_first(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    if let Some(items) = rdf_or_native_list(left, bindings, facts) {
        let Some(first) = items.first().cloned() else {
            return Vec::new();
        };
        let mut b = bindings.clone();
        return if unify_term_list_builtin_facts(right, &first, &mut b, facts) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }

    // Virtual RDF list matching: a native list term used as a subject behaves as
    // the corresponding rdf:first/rdf:rest chain.  This lets patterns with a
    // blank-list witness match facts whose concrete subject is `( ... )`.
    if matches!(resolve_pattern(left, bindings), Term::Var(_)) {
        let mut candidates = Vec::<Term>::new();
        for t in facts {
            for term in [&t.s, &t.o] {
                if matches!(term, Term::List(items) if !items.is_empty())
                    && !candidates.contains(term)
                {
                    candidates.push(term.clone());
                }
            }
        }
        let mut out = Vec::new();
        for candidate in candidates {
            let Some(items) = rdf_or_native_list_resolved(&candidate, facts, &mut HashSet::new())
            else {
                continue;
            };
            let Some(first) = items.first().cloned() else {
                continue;
            };
            let mut b = bindings.clone();
            if unify_term(left, &candidate, &mut b)
                && unify_term_list_builtin_facts(right, &first, &mut b, facts)
            {
                out.push(canonicalize_owned(b));
            }
        }
        return out;
    }
    Vec::new()
}

// ---- reasoner.rs lines 5885-5934 ----
pub(crate) fn eval_rdf_rest(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    if let Some(items) = rdf_or_native_list(left, bindings, facts) {
        if items.is_empty() {
            return Vec::new();
        }
        let rest = items[1..].to_vec();
        let mut b = bindings.clone();
        return if unify_listish(right, rest, &mut b, facts) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }

    if matches!(resolve_pattern(left, bindings), Term::Var(_)) {
        let mut candidates = Vec::<Term>::new();
        for t in facts {
            for term in [&t.s, &t.o] {
                if matches!(term, Term::List(items) if !items.is_empty())
                    && !candidates.contains(term)
                {
                    candidates.push(term.clone());
                }
            }
        }
        let mut out = Vec::new();
        for candidate in candidates {
            let Some(items) = rdf_or_native_list_resolved(&candidate, facts, &mut HashSet::new())
            else {
                continue;
            };
            if items.is_empty() {
                continue;
            }
            let rest = items[1..].to_vec();
            let mut b = bindings.clone();
            if unify_term(left, &candidate, &mut b) && unify_listish(right, rest, &mut b, facts) {
                out.push(canonicalize_owned(b));
            }
        }
        return out;
    }
    Vec::new()
}
