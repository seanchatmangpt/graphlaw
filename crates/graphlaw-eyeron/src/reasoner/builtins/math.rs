use super::super::limits::*;
use super::super::*;
use super::list::*;
use super::log::*;
use super::*;
use num_bigint::BigInt;
use std::collections::BTreeMap;

// ---- reasoner.rs lines 5935-5978 ----
pub(crate) fn eval_math_difference(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    if items.len() != 2 {
        return Vec::new();
    }
    let first = resolve(&items[0], bindings);
    let second = resolve(&items[1], bindings);
    if let (Some(a), Some(b)) = (datetime_seconds(&first), datetime_seconds(&second)) {
        let result = typed_literal(format_duration_seconds(a - b), XSD_DURATION);
        let mut out = bindings.clone();
        return if unify_term(right, &result, &mut out) {
            vec![canonicalize_owned(out)]
        } else {
            Vec::new()
        };
    }
    let Some(a) = numeric_value(&first) else {
        return Vec::new();
    };
    let Some(b) = numeric_value(&second) else {
        return Vec::new();
    };
    let result = match (a.exact.as_ref(), b.exact.as_ref()) {
        (Some(x), Some(y)) => integer_literal(x - y),
        _ => numeric_literal(a.value - b.value, a.integer && b.integer),
    };
    if matches!(resolve(right, bindings), Term::Blank(_)) {
        return vec![bindings.clone()];
    }
    let mut out = bindings.clone();
    if unify_term_loose_numeric(right, &result, &mut out) {
        vec![canonicalize_owned(out)]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 5979-6002 ----
pub(crate) fn is_math_operator(iri: &str) -> bool {
    matches!(
        iri,
        MATH_PRODUCT
            | MATH_QUOTIENT
            | MATH_INTEGER_QUOTIENT
            | MATH_REMAINDER
            | MATH_EXPONENTIATION
            | MATH_NEGATION
            | MATH_ABSOLUTE_VALUE
            | MATH_ROUNDED
            | MATH_SIN
            | MATH_COS
            | MATH_TAN
            | MATH_ASIN
            | MATH_ACOS
            | MATH_ATAN
            | MATH_SINH
            | MATH_COSH
            | MATH_TANH
            | MATH_DEGREES
    )
}

// ---- reasoner.rs lines 6003-6112 ----
pub(crate) fn eval_math_operator(
    pred: &str,
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    match pred {
        MATH_PRODUCT => eval_numeric_list(left, right, bindings, facts, |items| {
            if let Some(exact) = all_exact(&items) {
                // The product's size is the sum of its operands' sizes; refuse
                // before multiplying rather than after stringifying the result.
                if exact.iter().any(|n| n.bits() == 0) {
                    return Some(integer_literal(BigInt::from(0)));
                }
                let bits: u64 = exact.iter().map(|n| n.bits()).sum();
                if bits > MAX_EXACT_PRODUCT_BITS {
                    return None;
                }
                return Some(integer_literal(
                    exact
                        .into_iter()
                        .fold(BigInt::from(1), |product, n| product * n),
                ));
            }
            let all_integer = items.iter().all(|n| n.integer);
            let value = items.iter().fold(1.0, |acc, n| acc * n.value);
            Some(numeric_literal(value, all_integer))
        }),
        MATH_QUOTIENT => eval_numeric_list(left, right, bindings, facts, |items| {
            if items.len() != 2 || items[1].value == 0.0 {
                return None;
            }
            Some(numeric_literal(
                items[0].value / items[1].value,
                items[0].integer && items[1].integer,
            ))
        }),
        MATH_INTEGER_QUOTIENT => eval_numeric_list(left, right, bindings, facts, |items| {
            if items.len() != 2 {
                return None;
            }
            // Both `BigInt` division and `f64::trunc` round toward zero.
            if let Some(exact) = all_exact(&items) {
                if *exact[1] == BigInt::from(0) {
                    return None;
                }
                return Some(integer_literal(exact[0] / exact[1]));
            }
            if items[1].value == 0.0 {
                return None;
            }
            Some(numeric_literal(
                (items[0].value / items[1].value).trunc(),
                true,
            ))
        }),
        MATH_REMAINDER => eval_numeric_list(left, right, bindings, facts, |items| {
            if items.len() != 2 {
                return None;
            }
            // Both `BigInt` and `f64` remainders take the sign of the dividend.
            if let Some(exact) = all_exact(&items) {
                if *exact[1] == BigInt::from(0) {
                    return None;
                }
                return Some(integer_literal(exact[0] % exact[1]));
            }
            if items[1].value == 0.0 {
                return None;
            }
            Some(numeric_literal(items[0].value % items[1].value, true))
        }),
        MATH_EXPONENTIATION => eval_exponentiation(left, right, bindings, facts),
        MATH_NEGATION => eval_unary_numeric(left, right, bindings, |x| -x, |x| -x, true, true),
        MATH_ABSOLUTE_VALUE => eval_unary_numeric(
            left,
            right,
            bindings,
            |x| x.abs(),
            num_traits::Signed::abs,
            true,
            false,
        ),
        // An integer is already rounded.
        MATH_ROUNDED => eval_unary_numeric(
            left,
            right,
            bindings,
            |x| (x + 0.5).floor(),
            BigInt::clone,
            true,
            false,
        ),
        MATH_SIN => eval_unary_numeric_with_inverse(left, right, bindings, f64::sin, f64::asin),
        MATH_COS => eval_unary_numeric_with_inverse(left, right, bindings, f64::cos, f64::acos),
        MATH_TAN => eval_unary_numeric_with_inverse(left, right, bindings, f64::tan, f64::atan),
        MATH_ASIN => eval_unary_numeric_with_inverse(left, right, bindings, f64::asin, f64::sin),
        MATH_ACOS => eval_unary_numeric_with_inverse(left, right, bindings, f64::acos, f64::cos),
        MATH_ATAN => eval_unary_numeric_with_inverse(left, right, bindings, f64::atan, f64::tan),
        MATH_SINH => eval_unary_numeric_with_inverse(left, right, bindings, f64::sinh, f64::asinh),
        MATH_COSH => eval_unary_numeric_with_inverse(left, right, bindings, f64::cosh, f64::acosh),
        MATH_TANH => eval_unary_numeric_with_inverse(left, right, bindings, f64::tanh, f64::atanh),
        MATH_DEGREES => {
            eval_unary_numeric_with_inverse(left, right, bindings, f64::to_degrees, f64::to_radians)
        }
        _ => Vec::new(),
    }
}

// ---- reasoner.rs lines 6113-6125 ----
pub(crate) fn unify_term_loose_numeric(left: &Term, right: &Term, bindings: &mut Bindings) -> bool {
    let l = resolve_pattern(left, bindings);
    let r = resolve_pattern(right, bindings);
    match (&l, &r) {
        (Term::Var(a), _) => bind_one_mut(bindings, a, r.clone()),
        (_, Term::Var(b)) => bind_one_mut(bindings, b, l.clone()),
        _ if numeric_value(&l).is_some() && numeric_value(&r).is_some() => {
            numeric_terms_equal(&l, &r)
        }
        _ => unify_term(&l, &r, bindings),
    }
}

// ---- reasoner.rs lines 6126-6162 ----
pub(crate) fn eval_numeric_list<F>(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
    op: F,
) -> Vec<Bindings>
where
    F: FnOnce(Vec<Numeric>) -> Option<Term>,
{
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    let mut nums = Vec::new();
    for item in items {
        let Some(n) = numeric_value(&resolve(&item, bindings)) else {
            return Vec::new();
        };
        nums.push(n);
    }
    let Some(value) = op(nums) else {
        return Vec::new();
    };
    if matches!(resolve(right, bindings), Term::Blank(_)) {
        return vec![bindings.clone()];
    }
    let mut out = bindings.clone();
    if unify_term_loose_numeric(right, &value, &mut out) {
        vec![canonicalize_owned(out)]
    } else {
        Vec::new()
    }
}

/// `op` is the floating-point operation; `exact_op` is the same operation on
/// an integer, which is where it has to stay exact -- negating 10^16 - 1
/// twice has to give 10^16 - 1 back, and through an `f64` it does not.
// ---- reasoner.rs lines 6163-6236 ----
pub(crate) fn eval_unary_numeric<F, G>(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    op: F,
    exact_op: G,
    integer_if_integral: bool,
    allow_inverse: bool,
) -> Vec<Bindings>
where
    F: Fn(f64) -> f64,
    G: Fn(&BigInt) -> BigInt,
{
    let apply = |n: &Numeric| match n.exact.as_ref() {
        Some(exact) => integer_literal(exact_op(exact)),
        None => numeric_literal(op(n.value), integer_if_integral),
    };
    let l = resolve(left, bindings);
    let r = resolve(right, bindings);
    match (&l, &r) {
        (Term::Var(_), Term::Var(_)) if allow_inverse => vec![bindings.clone()],
        (Term::Var(name), _) if allow_inverse => {
            let Some(n) = numeric_value(&r) else {
                return Vec::new();
            };
            let value = apply(&n);
            bind_one(bindings, name, value)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (Term::Blank(_), _) if allow_inverse => {
            if matches!(r, Term::Blank(_)) || numeric_value(&r).is_some() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            }
        }
        (Term::Var(_), _) => Vec::new(),
        (_, Term::Var(name)) => {
            let Some(n) = numeric_value(&l) else {
                return Vec::new();
            };
            let value = apply(&n);
            bind_one(bindings, name, value)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (_, Term::Blank(_)) => {
            if numeric_value(&l).is_some() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            }
        }
        (_, _) => {
            let Some(n) = numeric_value(&l) else {
                return Vec::new();
            };
            let value = apply(&n);
            let mut out = bindings.clone();
            if unify_term_loose_numeric(right, &value, &mut out) {
                vec![canonicalize_owned(out)]
            } else {
                Vec::new()
            }
        }
    }
}

/// The result keeps the operand's datatype when it can: `1 math:acos 0`, not
/// `1 math:acos "0"^^xsd:decimal`. A result that is not a whole number
/// promotes to decimal on its own, in `numeric_literal`.
// ---- reasoner.rs lines 6237-6317 ----
pub(crate) fn eval_unary_numeric_with_inverse<F, G>(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    forward: F,
    inverse: G,
) -> Vec<Bindings>
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let l = resolve(left, bindings);
    let r = resolve(right, bindings);
    let integral = |term: &Term| numeric_value(term).is_some_and(|n| n.integer);
    let zero = numeric_literal(0.0, integral(&l));
    let inverse_zero = numeric_literal(inverse(0.0), integral(&r));

    match (&l, &r) {
        (Term::Var(_), Term::Var(_)) => vec![bindings.clone()],
        (Term::Var(a), Term::Blank(_)) => bind_one(bindings, a, inverse_zero)
            .into_iter()
            .map(canonicalize_owned)
            .collect(),
        (Term::Blank(_), Term::Var(b)) => bind_one(bindings, b, zero)
            .into_iter()
            .map(canonicalize_owned)
            .collect(),
        (Term::Blank(_), Term::Blank(_)) => vec![bindings.clone()],
        (Term::Var(name), _) => {
            let Some(n) = numeric_value(&r) else {
                return Vec::new();
            };
            let computed = inverse(n.value);
            if !computed.is_finite() && !computed.is_infinite() {
                return Vec::new();
            }
            let value = numeric_literal(computed, n.integer);
            bind_one(bindings, name, value)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (Term::Blank(_), _) => {
            if numeric_value(&r).is_some() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            }
        }
        (_, Term::Var(name)) => {
            let Some(n) = numeric_value(&l) else {
                return Vec::new();
            };
            let value = numeric_literal(forward(n.value), n.integer);
            bind_one(bindings, name, value)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        (_, Term::Blank(_)) => {
            if numeric_value(&l).is_some() {
                vec![bindings.clone()]
            } else {
                Vec::new()
            }
        }
        (_, _) => {
            let Some(n) = numeric_value(&l) else {
                return Vec::new();
            };
            let value = numeric_literal(forward(n.value), n.integer);
            let mut out = bindings.clone();
            if unify_term_loose_numeric(right, &value, &mut out) {
                vec![canonicalize_owned(out)]
            } else {
                Vec::new()
            }
        }
    }
}

// ---- reasoner.rs lines 6318-6372 ----
pub(crate) fn eval_exponentiation(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };
    if items.len() != 2 {
        return Vec::new();
    }
    let base = resolve(&items[0], bindings);
    let exp = resolve(&items[1], bindings);
    let result = resolve(right, bindings);
    match (&base, &exp, &result) {
        (_, Term::Var(name), _) => {
            let Some(b) = numeric_value(&base) else {
                return Vec::new();
            };
            let Some(r) = numeric_value(&result) else {
                return Vec::new();
            };
            if b.value <= 0.0 || r.value <= 0.0 {
                return Vec::new();
            }
            let e = r.value.ln() / b.value.ln();
            let value = numeric_literal(e, true);
            bind_one(bindings, name, value)
                .into_iter()
                .map(canonicalize_owned)
                .collect()
        }
        _ => {
            let Some(b) = numeric_value(&base) else {
                return Vec::new();
            };
            let Some(e) = numeric_value(&exp) else {
                return Vec::new();
            };
            let value = exact_power(&b, &e)
                .unwrap_or_else(|| numeric_literal(b.value.powf(e.value), b.integer && e.integer));
            if matches!(resolve(right, bindings), Term::Blank(_)) {
                return vec![bindings.clone()];
            }
            let mut out = bindings.clone();
            if unify_term_loose_numeric(right, &value, &mut out) {
                vec![canonicalize_owned(out)]
            } else {
                Vec::new()
            }
        }
    }
}

// ---- reasoner.rs lines 6373-6381 ----
pub(crate) fn is_math_comparison(iri: &str) -> bool {
    iri == MATH_GREATER_THAN
        || iri == MATH_LESS_THAN
        || iri == MATH_NOT_GREATER_THAN
        || iri == MATH_NOT_LESS_THAN
        || iri == MATH_EQUAL_TO
        || iri == MATH_NOT_EQUAL_TO
}

// ---- reasoner.rs lines 6382-6418 ----
pub(crate) fn eval_math_sum(
    left: &Term,
    right: &Term,
    bindings: &Bindings,
    facts: &[Triple],
) -> Vec<Bindings> {
    let Some(items) = rdf_or_native_list(left, bindings, facts) else {
        return Vec::new();
    };

    let mut nums = Vec::new();
    for item in items {
        let Some(n) = numeric_value(&resolve(&item, bindings)) else {
            return Vec::new();
        };
        nums.push(n);
    }

    let value = match all_exact(&nums) {
        Some(exact) => integer_literal(exact.into_iter().fold(BigInt::from(0), |sum, n| sum + n)),
        None => numeric_literal(
            nums.iter().map(|n| n.value).sum(),
            nums.iter().all(|n| n.integer),
        ),
    };
    match resolve(right, bindings) {
        Term::Var(name) => bind_one(bindings, &name, value).into_iter().collect(),
        Term::Blank(_) => vec![bindings.clone()],
        other if numeric_terms_equal(&other, &value) => vec![bindings.clone()],
        _ => Vec::new(),
    }
}

/// Most compiled patterns kept per thread. A compiled `Regex` may hold up to
/// the regex crate's default size limit (10 MiB), so the worst case is bounded
/// at `REGEX_CACHE_CAPACITY` times that, not at the number of distinct patterns
/// a hostile document can name.
// ---- reasoner.rs lines 7051-7097 ----
pub(crate) fn eval_math_compare(
    pred: &str,
    left: &Term,
    right: &Term,
    bindings: &Bindings,
) -> Vec<Bindings> {
    let lterm = resolve(left, bindings);
    let rterm = resolve(right, bindings);
    // xsd:dateTime / xsd:date order as instants, and only against each other:
    // a dateTime is never comparable with a number or an untyped string.
    if let (Some(l), Some(r)) = (datetime_seconds(&lterm), datetime_seconds(&rterm)) {
        return compare_ordered(pred, l.partial_cmp(&r), bindings);
    }
    if datetime_seconds(&lterm).is_some() || datetime_seconds(&rterm).is_some() {
        return Vec::new();
    }
    let Some(l) = comparable_number(&lterm) else {
        return Vec::new();
    };
    let Some(r) = comparable_number(&rterm) else {
        return Vec::new();
    };
    // Two integers compare exactly.  Anything else compares through the f64
    // view, where equality has to allow for rounding.
    use std::cmp::Ordering;
    let exact = match (l.exact.as_ref(), r.exact.as_ref()) {
        (Some(x), Some(y)) => Some(x.cmp(y)),
        _ => None,
    };
    let ok = if pred == MATH_GREATER_THAN {
        exact.map_or(l.value > r.value, Ordering::is_gt)
    } else if pred == MATH_LESS_THAN {
        exact.map_or(l.value < r.value, Ordering::is_lt)
    } else if pred == MATH_NOT_GREATER_THAN {
        exact.map_or(l.value <= r.value, Ordering::is_le)
    } else if pred == MATH_NOT_LESS_THAN {
        exact.map_or(l.value >= r.value, Ordering::is_ge)
    } else if pred == MATH_EQUAL_TO {
        exact.map_or((l.value - r.value).abs() <= f64::EPSILON, Ordering::is_eq)
    } else if pred == MATH_NOT_EQUAL_TO {
        exact.map_or((l.value - r.value).abs() > f64::EPSILON, Ordering::is_ne)
    } else {
        false
    };
    if ok {
        vec![bindings.clone()]
    } else {
        Vec::new()
    }
}

/// The six math: comparisons over an already-computed ordering.
// ---- reasoner.rs lines 7098-7128 ----
pub(crate) fn compare_ordered(
    pred: &str,
    ord: Option<std::cmp::Ordering>,
    bindings: &Bindings,
) -> Vec<Bindings> {
    use std::cmp::Ordering::*;
    let Some(ord) = ord else {
        return Vec::new();
    };
    let ok = if pred == MATH_GREATER_THAN {
        ord == Greater
    } else if pred == MATH_LESS_THAN {
        ord == Less
    } else if pred == MATH_NOT_GREATER_THAN {
        ord != Greater
    } else if pred == MATH_NOT_LESS_THAN {
        ord != Less
    } else if pred == MATH_EQUAL_TO {
        ord == Equal
    } else if pred == MATH_NOT_EQUAL_TO {
        ord != Equal
    } else {
        false
    };
    if ok {
        vec![bindings.clone()]
    } else {
        Vec::new()
    }
}

// ---- reasoner.rs lines 7303-7335 ----
pub(crate) fn numeric_literal_value(lit: &Literal) -> Option<Numeric> {
    {
        {
            let dt = lit.datatype.as_deref();
            let is_integer = matches!(dt, Some("http://www.w3.org/2001/XMLSchema#integer"));
            let is_numeric = matches!(
                dt,
                Some("http://www.w3.org/2001/XMLSchema#integer")
                    | Some("http://www.w3.org/2001/XMLSchema#decimal")
                    | Some("http://www.w3.org/2001/XMLSchema#double")
                    | Some("http://www.w3.org/2001/XMLSchema#float")
            );
            if !is_numeric {
                return None;
            }
            let exact = if is_integer && lit.value.len() <= MAX_EXACT_LITERAL_CHARS {
                lit.value.parse::<BigInt>().ok()
            } else {
                None
            };
            parse_numeric_lexical(&lit.value).map(|value| Numeric {
                value,
                integer: is_integer,
                exact,
            })
        }
    }
}

/// The one lexical form a number is indexed under. `0.0` and
/// `"0"^^xsd:decimal` are the same number, and the matcher treats them as
/// equal, so an index that keys on the written form has to fold them
/// together or it will simply fail to find the fact.
// ---- reasoner.rs lines 7336-7345 ----
pub(crate) fn canonical_numeric_lexical(lit: &Literal) -> Option<String> {
    let numeric = numeric_literal_value(lit)?;
    Some(match &numeric.exact {
        Some(exact) => exact.to_string(),
        None => trim_float(numeric.value),
    })
}

/// A term as an index keys it: the term itself, unless it spells a number
/// some other way than `canonical_numeric_lexical` would.
// ---- reasoner.rs lines 7346-7373 ----
pub(crate) fn index_form(term: &Term) -> std::borrow::Cow<'_, Term> {
    use std::borrow::Cow;
    match term {
        Term::Literal(lit) => match canonical_numeric_lexical(lit) {
            Some(value) if value != lit.value => Cow::Owned(Term::Literal(Literal {
                value: value.into(),
                ..lit.clone()
            })),
            _ => Cow::Borrowed(term),
        },
        Term::List(items) => {
            let mut canonical: Option<Vec<Term>> = None;
            for (idx, item) in items.iter().enumerate() {
                if let Cow::Owned(owned) = index_form(item) {
                    canonical.get_or_insert_with(|| items.clone())[idx] = owned;
                }
            }
            match canonical {
                Some(items) => Cow::Owned(Term::List(items)),
                None => Cow::Borrowed(term),
            }
        }
        _ => Cow::Borrowed(term),
    }
}

/// Largest exact power this will materialise, in bits.  Eyeling caps the
/// same operation at two million; Ackermann(4, 2) needs about 65,536.
// ---- reasoner.rs lines 7374-7388 ----
pub(crate) const MAX_EXACT_POWER_BITS: u64 = 2_000_000;

pub(crate) fn integer_literal(value: BigInt) -> Term {
    Term::Literal(Literal {
        value: value.to_string().into(),
        datatype: Some(
            "http://www.w3.org/2001/XMLSchema#integer"
                .to_string()
                .into(),
        ),
        language: None,
    })
}

/// The exact values of `items`, when every one of them has one.
// ---- reasoner.rs lines 7389-7394 ----
pub(crate) fn all_exact(items: &[Numeric]) -> Option<Vec<&BigInt>> {
    items.iter().map(|item| item.exact.as_ref()).collect()
}

/// `base ** exponent` exactly, when both are integers, the exponent is not
/// negative, and the result is small enough to be worth materialising.
// ---- reasoner.rs lines 7395-7433 ----
pub(crate) fn exact_power(base: &Numeric, exponent: &Numeric) -> Option<Term> {
    use num_traits::ToPrimitive;
    let base = base.exact.as_ref()?;
    let exponent = exponent.exact.as_ref()?.to_u32()?;
    // |base|^exponent has at most bits(base) * exponent bits.
    let bits = base.bits().checked_mul(u64::from(exponent))?;
    if bits > MAX_EXACT_POWER_BITS {
        return None;
    }
    Some(integer_literal(base.pow(exponent)))
}

pub(crate) fn numeric_literal(value: f64, prefer_integer: bool) -> Term {
    if prefer_integer && value.fract() == 0.0 {
        return Term::Literal(Literal {
            value: format!("{:.0}", value).into(),
            datatype: Some(
                "http://www.w3.org/2001/XMLSchema#integer"
                    .to_string()
                    .into(),
            ),
            language: None,
        });
    }
    // `xsd:decimal` has no exponential lexical form, so a magnitude that only
    // writes as one is an `xsd:double`.
    let value = trim_float(value);
    let datatype = if value.contains('e') {
        "http://www.w3.org/2001/XMLSchema#double"
    } else {
        "http://www.w3.org/2001/XMLSchema#decimal"
    };
    Term::Literal(Literal {
        value: value.into(),
        datatype: Some(datatype.into()),
        language: None,
    })
}

// ---- reasoner.rs lines 7434-7468 ----
pub(crate) fn numeric_terms_equal(a: &Term, b: &Term) -> bool {
    match (numeric_value(a), numeric_value(b)) {
        (Some(x), Some(y)) => {
            if let (Some(p), Some(q)) = (x.exact.as_ref(), y.exact.as_ref()) {
                p == q
            } else if x.value.is_nan() || y.value.is_nan() {
                x.value.is_nan() && y.value.is_nan()
            } else if x.value.is_infinite() || y.value.is_infinite() {
                x.value == y.value
            } else {
                (x.value - y.value).abs() <= f64::EPSILON
            }
        }
        _ => false,
    }
}

pub(crate) fn terms_equal_semantic(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Literal(x), Term::Literal(y)) => literals_equal_semantic(x, y),
        (Term::List(xs), Term::List(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys.iter())
            .all(|(x, y)| terms_equal_semantic(x, y)),
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

// ---- reasoner.rs lines 7469-7487 ----
pub(crate) fn literals_equal_semantic(a: &Literal, b: &Literal) -> bool {
    let ta = Term::Literal(a.clone());
    let tb = Term::Literal(b.clone());
    if numeric_value(&ta).is_some() && numeric_value(&tb).is_some() {
        return a.datatype == b.datatype && numeric_terms_equal(&ta, &tb);
    }
    if a.language != b.language {
        return false;
    }
    let adt = a.datatype.as_deref();
    let bdt = b.datatype.as_deref();
    let a_string = adt.is_none() || adt == Some(XSD_STRING_IRI);
    let b_string = bdt.is_none() || bdt == Some(XSD_STRING_IRI);
    if a_string && b_string {
        return a.value == b.value;
    }
    a == b
}

// ---- reasoner.rs lines 7758-7812 ----
pub(crate) fn trim_float(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-INF"
        } else {
            "INF"
        }
        .to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }

    // `{:e}` gives the same shortest round-trip digits, already split into a
    // mantissa and a power of ten.
    let exponential = format!("{:e}", value.abs());
    let (mantissa, exponent) = exponential
        .split_once('e')
        .expect("{:e} writes an exponent");
    let exponent: i32 = exponent.parse().expect("{:e} writes an integer exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let count = digits.len() as i32;
    // `value` is `0.<digits> * 10^point`: where the decimal point falls.
    let point = exponent + 1;

    let body = if point >= count && point <= 21 {
        format!("{digits}{}", "0".repeat((point - count) as usize))
    } else if point > 0 && point <= 21 {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    } else if point > -6 && point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else {
        let tail = if count == 1 {
            String::new()
        } else {
            format!(".{}", &digits[1..])
        };
        let sign = if point > 1 { "+" } else { "-" };
        format!("{}{tail}e{sign}{}", &digits[..1], (point - 1).abs())
    };

    if value.is_sign_negative() {
        format!("-{body}")
    } else {
        body
    }
}

// ---- reasoner.rs lines 7813-7821 ----
pub(crate) fn parse_numeric_lexical(value: &str) -> Option<f64> {
    match value {
        "INF" | "+INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        "NaN" => Some(f64::NAN),
        other => other.parse::<f64>().ok(),
    }
}
