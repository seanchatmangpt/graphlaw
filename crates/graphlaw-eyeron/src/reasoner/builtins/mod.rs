pub(crate) mod list;
pub(crate) mod log;
pub(crate) mod math;
pub(crate) mod string;

use self::list::*;
use self::log::*;
use self::math::*;
use self::string::*;
use super::limits::*;
use super::*;

use num_bigint::BigInt;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = Date, js_name = now)]
    fn javascript_date_now() -> f64;
}

// ---- reasoner.rs lines 1955-1961 ----
pub(crate) fn is_builtin_premise(triple: &Triple) -> bool {
    let Term::Iri(iri) = &triple.p else {
        return false;
    };
    is_builtin_iri(iri)
}

// ---- reasoner.rs lines 1962-2008 ----
pub(crate) fn is_builtin_iri(iri: &str) -> bool {
    matches!(
        iri,
        LOG_EQUAL_TO
            | LOG_NOT_EQUAL_TO
            | LOG_COLLECT_ALL_IN
            | LOG_FOR_ALL_IN
            | LOG_CONCLUSION
            | LOG_CONJUNCTION
            | LOG_INCLUDES
            | LOG_NOT_INCLUDES
            | LOG_URI
            | LOG_RAW_TYPE
            | LOG_DTLIT
            | LOG_LANGLIT
            | LOG_CONTENT
            | LOG_SEMANTICS
            | LOG_SEMANTICS_OR_ERROR
            | LOG_PARSED_AS_N3
            | LOG_SKOLEM
            | LOG_UUID
            | CRYPTO_SHA
            | DT_DATATYPE
            | DT_LEXICAL_FORM
            | EYELING_DT_DATATYPE
            | EYELING_DT_LEXICAL_FORM
            | RDF_FIRST
            | RDF_REST
            | LIST_FIRST
            | LIST_REST
            | LIST_APPEND
            | LIST_ITERATE
            | LIST_MAP
            | LIST_FIRST_REST
            | LIST_REVERSE
            | LIST_SORT
            | LIST_NOT_MEMBER
            | MATH_SUM
            | MATH_DIFFERENCE
            | SUDOKU_SOLVE
    ) || is_list_builtin(iri)
        || is_math_operator(iri)
        || is_math_comparison(iri)
        || is_string_builtin(iri)
        || is_time_builtin(iri)
}

// ---- reasoner.rs lines 2009-2037 ----
pub(crate) fn is_agenda_safe_builtin_iri(iri: &str) -> bool {
    // These builtins are pure tests or deterministic value constructors over
    // their arguments.  They do not inspect the growing fact closure, so a rule
    // containing them can still be driven by its ordinary fact premises.
    matches!(
        iri,
        LOG_EQUAL_TO
            | LOG_NOT_EQUAL_TO
            | LOG_URI
            | LOG_RAW_TYPE
            | LOG_DTLIT
            | LOG_LANGLIT
            | LOG_CONTENT
            | LOG_SKOLEM
            | LOG_UUID
            | CRYPTO_SHA
            | DT_DATATYPE
            | DT_LEXICAL_FORM
            | EYELING_DT_DATATYPE
            | EYELING_DT_LEXICAL_FORM
            | MATH_SUM
            | MATH_DIFFERENCE
            | SUDOKU_SOLVE
    ) || is_math_operator(iri)
        || is_math_comparison(iri)
        || is_string_builtin(iri)
        || is_time_builtin(iri)
}

// ---- reasoner.rs lines 3276-3296 ----
pub fn builtin_reads_outside_its_triple(predicate: &Term) -> bool {
    let Term::Iri(iri) = predicate else {
        return false;
    };
    matches!(
        iri.as_str(),
        LOG_COLLECT_ALL_IN
            | LOG_FOR_ALL_IN
            | LOG_INCLUDES
            | LOG_NOT_INCLUDES
            | LOG_CONCLUSION
            | LOG_CONJUNCTION
            | LOG_CONTENT
            | LOG_SEMANTICS
            | LOG_SEMANTICS_OR_ERROR
            | LIST_MAP
    ) || iri.as_str() == TIME_LOCAL_TIME
}

/// True iff `goal` holds by evaluating a built-in on it alone — what a
/// proof checker re-performs for a `pe:builtin` step.
// ---- reasoner.rs lines 3297-3320 ----
pub fn verify_builtin_triple(goal: &Triple) -> bool {
    if !is_builtin_premise(goal) {
        return false;
    }
    let mut concrete = BTreeMap::new();
    for term in [&goal.s, &goal.p, &goal.o] {
        bind_concrete_blanks(term, &mut concrete);
    }
    let mut budget = SearchBudget::for_proof(1);
    let mut backward_stack = HashSet::new();
    eval_builtin(
        goal,
        &concrete,
        &[],
        None,
        &[],
        0,
        &mut backward_stack,
        &mut budget,
    )
    .is_some_and(|matches| !matches.is_empty())
}

#[allow(clippy::too_many_arguments)]
// ---- reasoner.rs lines 3894-4069 ----
pub(crate) fn eval_builtin(
    premise: &Triple,
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> Option<Vec<Bindings>> {
    let pred = resolve(&premise.p, bindings);
    match pred {
        Term::Iri(ref iri) if iri == LOG_EQUAL_TO => {
            Some(eval_equal(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LOG_NOT_EQUAL_TO => {
            Some(eval_not_equal(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LOG_COLLECT_ALL_IN => Some(eval_collect_all_in(
            &premise.s,
            &premise.o,
            bindings,
            facts,
            fact_index,
            rules,
            depth,
            backward_stack,
            budget,
        )),
        Term::Iri(ref iri) if iri == LOG_FOR_ALL_IN => Some(eval_for_all_in(
            &premise.s,
            &premise.o,
            bindings,
            facts,
            fact_index,
            rules,
            depth,
            backward_stack,
            budget,
        )),
        Term::Iri(ref iri) if iri == LOG_CONCLUSION => Some(eval_log_conclusion(
            &premise.s, &premise.o, bindings, budget,
        )),
        Term::Iri(ref iri) if iri == LOG_CONJUNCTION => Some(eval_log_conjunction(
            &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if iri == LOG_INCLUDES => Some(eval_log_includes(
            &premise.s, &premise.o, bindings, facts, rules, budget,
        )),
        Term::Iri(ref iri) if iri == LOG_NOT_INCLUDES => Some(eval_log_not_includes(
            premise,
            bindings,
            facts,
            fact_index,
            rules,
            depth,
            backward_stack,
            budget,
        )),
        Term::Iri(ref iri) if iri == LOG_URI => {
            Some(eval_log_uri(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_RAW_TYPE => {
            Some(eval_log_raw_type(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_DTLIT => {
            Some(eval_log_dtlit(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LOG_LANGLIT => {
            Some(eval_log_langlit(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LOG_CONTENT => {
            Some(eval_log_content(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_SEMANTICS => {
            Some(eval_log_semantics(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_SEMANTICS_OR_ERROR => Some(eval_log_semantics_or_error(
            &premise.s, &premise.o, bindings,
        )),
        Term::Iri(ref iri) if iri == LOG_PARSED_AS_N3 => {
            Some(eval_log_parsed_as_n3(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_SKOLEM => {
            Some(eval_log_skolem(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == LOG_UUID => {
            Some(eval_log_uuid(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if matches!(iri.as_str(), DT_DATATYPE | EYELING_DT_DATATYPE) => Some(
            eval_datatype_inspection(&premise.s, &premise.o, bindings, true),
        ),
        Term::Iri(ref iri) if matches!(iri.as_str(), DT_LEXICAL_FORM | EYELING_DT_LEXICAL_FORM) => {
            Some(eval_datatype_inspection(
                &premise.s, &premise.o, bindings, false,
            ))
        }
        Term::Iri(ref iri) if iri == CRYPTO_SHA => {
            Some(eval_crypto_sha(&premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == RDF_FIRST || iri == LIST_FIRST => {
            Some(eval_rdf_first(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == RDF_REST || iri == LIST_REST => {
            Some(eval_rdf_rest(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LIST_APPEND => Some(eval_list_append(
            &premise.s, &premise.o, bindings, facts, budget,
        )),
        Term::Iri(ref iri) if iri == LIST_ITERATE => {
            Some(eval_list_iterate(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LIST_MAP => Some(eval_list_map(
            &premise.s,
            &premise.o,
            bindings,
            facts,
            fact_index,
            rules,
            depth,
            backward_stack,
            budget,
        )),
        Term::Iri(ref iri) if iri == LIST_FIRST_REST => Some(eval_list_first_rest(
            &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if iri == LIST_REVERSE => {
            Some(eval_list_reverse(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LIST_SORT => {
            Some(eval_list_sort(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == LIST_NOT_MEMBER => Some(eval_list_not_member(
            &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if is_list_builtin(iri) => Some(eval_list_builtin(
            iri, &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if iri == MATH_SUM => {
            Some(eval_math_sum(&premise.s, &premise.o, bindings, facts))
        }
        Term::Iri(ref iri) if iri == MATH_DIFFERENCE => Some(eval_math_difference(
            &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if is_math_operator(iri) => Some(eval_math_operator(
            iri, &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if is_math_comparison(iri) => {
            Some(eval_math_compare(iri, &premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if is_string_builtin(iri) => Some(eval_string_builtin(
            iri, &premise.s, &premise.o, bindings, facts,
        )),
        Term::Iri(ref iri) if is_time_builtin(iri) => {
            Some(eval_time_builtin(iri, &premise.s, &premise.o, bindings))
        }
        Term::Iri(ref iri) if iri == SUDOKU_SOLVE => {
            Some(eval_sudoku_solve(&premise.s, &premise.o, bindings))
        }
        _ => None,
    }
}

/// Non-standard extension (see `crate::sudoku`): `?puzzle sudoku:solve
/// ?solution` solves a 9x9 puzzle given as an 81-character string (`.` or
/// `0` for a blank cell), delegating to a native Rust backtracking search
/// rather than N3 rules. A genuinely backward-chained cell-by-cell search
/// (the style four-queens.n3 uses) is correct but impractical here: this
/// engine's per-goal backward-chaining overhead (rule cloning and renaming,
/// no tabling) turns a real 9x9 puzzle's search space into a multi-second-
/// or-worse run even before backtracking, confirmed by isolated timing
/// during sudoku.n3's own design (a single non-backtracking pass over just
/// 16 cells alone took ~4 seconds). Failing to parse (wrong length,
/// non-digit/non-dot character, or conflicting givens) or having no
/// solution both make the premise fail outright, matching how other
/// N3 builtins here treat an invalid or unsatisfiable argument.
// ---- reasoner.rs lines 4070-4079 ----
pub(crate) fn eval_sudoku_solve(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let Some(puzzle) = string_value(&resolve(subject, bindings)) else {
        return Vec::new();
    };
    match crate::sudoku::solve_sudoku_string(&puzzle) {
        Ok(solution) if !solution.is_empty() => bind_string_result(object, solution, bindings),
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4080-4091 ----
pub(crate) fn normalize_for_builtin_equality(term: Term, facts: &[Triple]) -> Term {
    match rdf_or_native_list_resolved(&term, facts, &mut HashSet::new()) {
        Some(items) => Term::List(
            items
                .into_iter()
                .map(|item| normalize_for_builtin_equality(item, facts))
                .collect(),
        ),
        None => term,
    }
}

// ---- reasoner.rs lines 4092-4113 ----
pub(crate) fn eval_equal(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    // Syntactic blank nodes in builtin argument positions are wildcard
    // arguments, but variables already bound to graph blank nodes remain
    // concrete because resolve_pattern only turns literal blank syntax into the
    // internal _: binding variables.
    let l = normalize_for_builtin_equality(resolve_pattern(left, bindings), facts);
    let r = normalize_for_builtin_equality(resolve_pattern(right, bindings), facts);
    match (&l, &r) {
        (Term::Var(a), Term::Var(b)) if a == b => vec![bindings.clone()],
        (Term::Var(a), other) => bind_one(bindings, a, other.clone())
            .into_iter()
            .map(canonicalize_owned)
            .collect(),
        (other, Term::Var(b)) => bind_one(bindings, b, other.clone())
            .into_iter()
            .map(canonicalize_owned)
            .collect(),
        (a, b) if terms_equal_semantic(a, b) => vec![bindings.clone()],
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4114-4128 ----
pub(crate) fn eval_not_equal(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let l = normalize_for_builtin_equality(resolve_pattern(left, bindings), facts);
    let r = normalize_for_builtin_equality(resolve_pattern(right, bindings), facts);
    match (&l, &r) {
        (Term::Var(_), _) | (_, Term::Var(_)) => Vec::new(),
        (a, b) if !terms_equal_semantic(a, b) => vec![bindings.clone()],
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 4129-4242 ----
pub(crate) fn eval_collect_all_in(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let Some(parts) = rdf_or_native_list(subject, bindings, facts) else {
        return Vec::new();
    };
    if parts.len() != 3 {
        return Vec::new();
    }

    let value_template = parts[0].clone();
    let Term::Formula(clause_triples) = parts[1].clone() else {
        return Vec::new();
    };
    let result_template = parts[2].clone();

    // Eyeron treats a blank-node result slot as an existence check only.
    if matches!(result_template, Term::Blank(_)) {
        return vec![bindings.clone()];
    }

    let scoped_facts_storage = match resolve(object, bindings) {
        Term::Formula(scope) => Some(scope),
        _ => None,
    };
    let empty_rules: Vec<Rule> = Vec::new();
    let scope_facts = scoped_facts_storage.as_deref().unwrap_or(facts);
    let scope_index = if scoped_facts_storage.is_some() {
        None
    } else {
        fact_index
    };
    let scope_rules = if scoped_facts_storage.is_some() {
        empty_rules.as_slice()
    } else {
        rules
    };

    let clause_goals = clause_triples
        .iter()
        .map(|triple| resolve_triple(triple, bindings))
        .collect::<Vec<_>>();

    // A blank node that arrives through `bindings` (`?M = _:b1` from the data)
    // is a concrete node; only a blank the scoped clause itself spells out
    // (`_:s`) is an existential pattern. Without this split the substituted
    // data blank matched every node, which is why 2-3 collected scalars needed
    // a "singleton" compatibility second answer.
    // The unresolved subject list shows which blanks the rule itself wrote;
    // `parts` above is already substituted, so it cannot tell them apart. When
    // the list only arrives through a variable there is nothing syntactic to
    // consult and every blank stays a pattern (the previous behaviour).
    let mut initial_bindings = Bindings::new();
    if let Term::List(raw_parts) = subject {
        if let Some(Term::Formula(raw_clause)) = raw_parts.get(1) {
            let mut template_blanks = HashSet::<String>::new();
            for triple in raw_clause {
                collect_blank_labels(&triple.s, &mut template_blanks);
                collect_blank_labels(&triple.p, &mut template_blanks);
                collect_blank_labels(&triple.o, &mut template_blanks);
            }
            let mut goal_blanks = HashSet::<String>::new();
            for triple in &clause_goals {
                collect_blank_labels(&triple.s, &mut goal_blanks);
                collect_blank_labels(&triple.p, &mut goal_blanks);
                collect_blank_labels(&triple.o, &mut goal_blanks);
            }
            for label in goal_blanks.difference(&template_blanks) {
                initial_bindings
                    .insert(blank_binding_name(label), Term::Blank(label.clone().into()));
            }
        }
    }

    let mut solutions = Vec::new();
    match_premise_at(
        &clause_goals,
        scope_facts,
        scope_index,
        scope_rules,
        0,
        initial_bindings,
        depth + 1,
        backward_stack,
        budget,
        &mut solutions,
    );

    let mut collected = Vec::new();
    for sol in solutions {
        let mut combined = bindings.clone();
        for (k, v) in sol {
            combined.insert(k, v);
        }
        collected.push(resolve(&value_template, &combined));
    }

    let collected_list = Term::List(collected.clone());
    let mut out = bindings.clone();
    let mut results = Vec::new();
    if unify_term(&result_template, &collected_list, &mut out) {
        results.push(canonicalize_owned(out));
    }
    results
}

// ---- reasoner.rs lines 4243-4259 ----
pub(crate) fn collect_blank_labels(term: &Term, out: &mut HashSet<String>) {
    match term {
        Term::Blank(label) => {
            out.insert(label.clone().to_string());
        }
        Term::List(items) => items
            .iter()
            .for_each(|item| collect_blank_labels(item, out)),
        Term::Formula(triples) => triples.iter().for_each(|t| {
            collect_blank_labels(&t.s, out);
            collect_blank_labels(&t.p, out);
            collect_blank_labels(&t.o, out);
        }),
        _ => {}
    }
}

// ---- reasoner.rs lines 4260-4347 ----
pub(crate) fn eval_for_all_in(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    fact_index: Option<&FactIndex>,
    rules: &[Rule],
    depth: usize,
    backward_stack: &mut HashSet<String>,
    budget: &mut SearchBudget,
) -> Vec<Bindings> {
    let Some(parts) = rdf_or_native_list(subject, bindings, facts) else {
        return Vec::new();
    };
    if parts.len() != 2 {
        return Vec::new();
    }
    let Term::Formula(generator) = parts[0].clone() else {
        return Vec::new();
    };
    let Term::Formula(condition) = parts[1].clone() else {
        return Vec::new();
    };

    let scoped_facts_storage = match resolve(object, bindings) {
        Term::Formula(scope) => Some(scope),
        _ => None,
    };
    let empty_rules: Vec<Rule> = Vec::new();
    let scope_facts = scoped_facts_storage.as_deref().unwrap_or(facts);
    let scope_index = if scoped_facts_storage.is_some() {
        None
    } else {
        fact_index
    };
    let scope_rules = if scoped_facts_storage.is_some() {
        empty_rules.as_slice()
    } else {
        rules
    };

    let mut generator_matches = Vec::new();
    match_premise_at(
        &generator,
        scope_facts,
        scope_index,
        scope_rules,
        0,
        bindings.clone(),
        depth + 1,
        backward_stack,
        budget,
        &mut generator_matches,
    );

    for gen in &generator_matches {
        let mut condition_matches = Vec::new();
        match_premise_at(
            &condition,
            scope_facts,
            scope_index,
            scope_rules,
            0,
            gen.clone(),
            depth + 1,
            backward_stack,
            budget,
            &mut condition_matches,
        );
        if condition_matches.is_empty() {
            return Vec::new();
        }
    }

    let mut b = bindings.clone();
    match resolve_pattern(object, bindings) {
        Term::Var(name) => {
            if !bind_one_mut(&mut b, &name, Term::Blank("forAllIn".to_string().into())) {
                return Vec::new();
            }
        }
        Term::Blank(_) => {}
        Term::Formula(_) => {}
        _ => {}
    }
    vec![canonicalize_owned(b)]
}

// ---- reasoner.rs lines 4695-4721 ----
pub(crate) fn eval_datatype_inspection(
    subject: &Term,
    object: &Term,
    bindings: &Bindings,
    datatype: bool,
) -> Vec<Bindings> {
    let Term::Literal(lit) = resolve_pattern(subject, bindings) else {
        return Vec::new();
    };
    let value = if datatype {
        let iri = if lit.language.is_some() {
            Name::from(RDF_LANG_STRING_IRI)
        } else {
            lit.datatype.unwrap_or_else(|| XSD_STRING_IRI.into())
        };
        Term::Iri(iri)
    } else {
        Term::Literal(Literal::plain(lit.value))
    };
    let mut next = bindings.clone();
    if unify_term(object, &value, &mut next) {
        vec![canonicalize_bindings(&next)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 6945-6958 ----
pub(crate) fn is_time_builtin(iri: &str) -> bool {
    matches!(
        iri,
        TIME_YEAR
            | TIME_MONTH
            | TIME_DAY
            | TIME_HOUR
            | TIME_MINUTE
            | TIME_SECOND
            | TIME_TIME_ZONE
            | TIME_LOCAL_TIME
    )
}

// ---- reasoner.rs lines 6959-7003 ----
pub(crate) fn eval_time_builtin(
    pred: &str,
    left: &Term,
    right: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    if pred == TIME_LOCAL_TIME {
        let resolved = resolve(right, bindings);
        if !matches!(resolved, Term::Var(_) | Term::Blank(_)) {
            return if datetime_seconds(&resolved).is_some() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            };
        }
        let Some((seconds, millis)) = current_unix_time() else {
            return Vec::new();
        };
        let value = typed_literal(format_datetime_utc(seconds, millis), XSD_DATE_TIME);
        let mut b = bindings.clone();
        return if unify_term(right, &value, &mut b) {
            vec![canonicalize_owned(b)]
        } else {
            Vec::new()
        };
    }
    let Some(dt) = string_value(&resolve(left, bindings)) else {
        return Vec::new();
    };
    let Some(parts) = parse_datetime_parts(&dt) else {
        return Vec::new();
    };
    let value = match pred {
        TIME_YEAR => numeric_literal(parts.year as f64, true),
        TIME_MONTH => numeric_literal(parts.month as f64, true),
        TIME_DAY => numeric_literal(parts.day as f64, true),
        TIME_HOUR => numeric_literal(parts.hour as f64, true),
        TIME_MINUTE => numeric_literal(parts.minute as f64, true),
        TIME_SECOND => numeric_literal(parts.second as f64, true),
        TIME_TIME_ZONE => Term::Literal(Literal::plain(parts.tz)),
        _ => return Vec::new(),
    };
    let mut b = bindings.clone();
    if unify_term(right, &value, &mut b) {
        vec![canonicalize_owned(b)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 7004-7013 ----
pub(crate) struct DateTimeParts {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
    tz: String,
}

// ---- reasoner.rs lines 7014-7042 ----
pub(crate) fn parse_datetime_parts(value: &str) -> Option<DateTimeParts> {
    // Enough ISO-8601 support for the bundled examples: YYYY-MM-DDTHH:MM:SSZ
    // and the same shape with an explicit +/-HH:MM timezone.
    let year = value.get(0..4)?.parse().ok()?;
    let month = value.get(5..7)?.parse().ok()?;
    let day = value.get(8..10)?.parse().ok()?;
    let hour = value.get(11..13)?.parse().ok()?;
    let minute = value.get(14..16)?.parse().ok()?;
    let second = value.get(17..19)?.parse().ok()?;
    let tz = if let Some(z) = value.get(19..) {
        if z.is_empty() {
            "".to_string()
        } else {
            z.to_string()
        }
    } else {
        "".to_string()
    };
    Some(DateTimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
        tz,
    })
}

// ---- reasoner.rs lines 7129-7129 ----
pub(crate) const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";

// ---- reasoner.rs lines 7133-7140 ----
pub(crate) fn typed_literal(value: String, datatype: &str) -> Term {
    Term::Literal(Literal {
        value: value.into(),
        datatype: Some(datatype.into()),
        language: None,
    })
}

// ---- reasoner.rs lines 7141-7150 ----
pub(crate) fn comparable_number(term: &Term) -> Option<Numeric> {
    numeric_value(term).or_else(|| {
        duration_seconds(term).map(|value| Numeric {
            value,
            integer: false,
            exact: None,
        })
    })
}

// ---- reasoner.rs lines 7151-7182 ----
pub(crate) fn duration_seconds(term: &Term) -> Option<f64> {
    let Term::Literal(lit) = term else {
        return None;
    };
    if lit.datatype.as_deref() != Some(XSD_DURATION) {
        return None;
    }
    static DURATION_RE: std::sync::OnceLock<Option<Regex>> = std::sync::OnceLock::new();
    let re = DURATION_RE
        .get_or_init(|| Regex::new(r"^(-)?P(?:(\d+(?:\.\d+)?)Y)?(?:(\d+(?:\.\d+)?)M)?(?:(\d+(?:\.\d+)?)D)?(?:T(?:(\d+(?:\.\d+)?)H)?(?:(\d+(?:\.\d+)?)M)?(?:(\d+(?:\.\d+)?)S)?)?$").ok())
        .as_ref()?;
    let captures = re.captures(&lit.value)?;
    let n = |i| {
        captures
            .get(i)
            .map_or(Some(0.0), |m| m.as_str().parse().ok())
    };
    // XML Schema year/month durations have no fixed length. Eyeling's age
    // comparisons use the conventional Gregorian averages below.
    let seconds = n(2)? * 31_556_952.0
        + n(3)? * 2_629_746.0
        + n(4)? * 86_400.0
        + n(5)? * 3_600.0
        + n(6)? * 60.0
        + n(7)?;
    Some(if captures.get(1).is_some() {
        -seconds
    } else {
        seconds
    })
}

// ---- reasoner.rs lines 7183-7223 ----
pub(crate) fn datetime_seconds(term: &Term) -> Option<f64> {
    let Term::Literal(lit) = term else {
        return None;
    };
    if !matches!(lit.datatype.as_deref(), Some(XSD_DATE | XSD_DATE_TIME)) {
        return None;
    }
    static DATETIME_RE: std::sync::OnceLock<Option<Regex>> = std::sync::OnceLock::new();
    let re = DATETIME_RE
        .get_or_init(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?(Z|[+-]\d{2}:\d{2})?)?(Z|[+-]\d{2}:\d{2})?$").ok())
        .as_ref()?;
    let c = re.captures(&lit.value)?;
    let get = |i| c.get(i).and_then(|m| m.as_str().parse::<i64>().ok());
    let (year, month, day) = (get(1)? as i32, get(2)? as u32, get(3)? as u32);
    let (hour, minute, second) = (
        get(4).unwrap_or(0),
        get(5).unwrap_or(0),
        get(6).unwrap_or(0),
    );
    let fraction = c
        .get(7)
        .and_then(|m| format!("0.{}", m.as_str()).parse::<f64>().ok())
        .unwrap_or(0.0);
    let tz = c
        .get(8)
        .or_else(|| c.get(9))
        .map(|m| m.as_str())
        .unwrap_or("Z");
    let offset = if tz == "Z" {
        0
    } else {
        let sign = if &tz[0..1] == "-" { -1 } else { 1 };
        sign * (tz[1..3].parse::<i64>().ok()? * 3600 + tz[4..6].parse::<i64>().ok()? * 60)
    };
    Some(
        (days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second - offset)
            as f64
            + fraction,
    )
}

// ---- reasoner.rs lines 7224-7232 ----
pub(crate) fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

// ---- reasoner.rs lines 7233-7246 ----
pub(crate) fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    y += i64::from(m <= 2);
    (y as i32, m as u32, d as u32)
}

// ---- reasoner.rs lines 7247-7262 ----
pub(crate) fn format_datetime_utc(seconds: i64, millis: u32) -> String {
    let days = seconds.div_euclid(86_400);
    let sod = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}+00:00",
        sod / 3600,
        sod % 3600 / 60,
        sod % 60
    )
}

/// The current wall-clock time as `(unix seconds, milliseconds)`. `SystemTime::now()`
/// panics unconditionally on `wasm32-unknown-unknown` (there is no clock
/// syscall without a JS bridge), so the wasm32 build instead calls through
/// to JS's `Date.now()`.
// ---- reasoner.rs lines 7263-7268 ----
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(crate) fn current_unix_time() -> Option<(i64, u32)> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some((now.as_secs() as i64, now.subsec_millis()))
}

// ---- reasoner.rs lines 7269-7278 ----
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) fn current_unix_time() -> Option<(i64, u32)> {
    let millis = javascript_date_now();
    if !millis.is_finite() || millis < 0.0 {
        return None;
    }
    let whole = millis.floor() as u64;
    Some(((whole / 1000) as i64, (whole % 1000) as u32))
}

// ---- reasoner.rs lines 7279-7302 ----
pub(crate) fn format_duration_seconds(seconds: f64) -> String {
    let sign = if seconds < 0.0 { "-" } else { "" };
    format!("{sign}PT{}S", trim_float(seconds.abs()))
}

#[derive(Debug, Clone)]
pub(crate) struct Numeric {
    pub(crate) value: f64,
    pub(crate) integer: bool,
    /// The exact value, for an `xsd:integer` whose lexical form really is
    /// one.  An integer literal already carries its value exactly; rounding
    /// it into `value` is a loss past 2^53, and formatting that `f64` back
    /// out is a guess.  Operations closed over the integers use this, and
    /// everything else uses `value`.
    pub(crate) exact: Option<BigInt>,
}

pub(crate) fn numeric_value(term: &Term) -> Option<Numeric> {
    match term {
        Term::Literal(lit) => numeric_literal_value(lit),
        _ => None,
    }
}
