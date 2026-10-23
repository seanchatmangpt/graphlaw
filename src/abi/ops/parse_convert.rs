use serde_json::{Value, json};

use super::super::{Res, bad, dialect_by_name, opt_str, spec_dialect, str_field};
use crate::dialect::{Dialect, Refusal, check};
use crate::law::LawState;

pub(in crate::abi) fn op_parse(v: &Value) -> Res<Value> {
    let text = str_field(v, "text")?;
    let dialect = spec_dialect(v)?;
    // SQL-in-RDF is Turtle plus a closed-world shape check. The shape check
    // has to run before the dataset parse, or a malformed statement would
    // still receive a canonical id.
    if dialect == Dialect::SqlRdf {
        check(text.as_bytes(), dialect, opt_str(v, "base"))?;
    }
    if dialect.media_type().is_some() {
        let s = LawState::parse(text.as_bytes(), dialect, opt_str(v, "base"))?;
        return Ok(
            json!({"dialect": format!("{dialect:?}"), "quads": s.quad_count(), "id": s.id()}),
        );
    }
    check(text.as_bytes(), dialect, opt_str(v, "base"))?;
    Ok(json!({"dialect": format!("{dialect:?}"), "valid": true}))
}

pub(in crate::abi) fn op_convert(v: &Value) -> Res<Value> {
    let s = LawState::parse(
        str_field(v, "text")?.as_bytes(),
        spec_dialect(v)?,
        opt_str(v, "base"),
    )?;
    let to = dialect_by_name(str_field(v, "to")?)?;
    let media = to
        .media_type()
        .ok_or_else(|| bad("`to` must be an RDF dialect"))?;
    let bytes = purrdf::serialize_dataset(s.dataset(), media, purrdf::SerializeGraph::Dataset)
        .map_err(|e| Refusal::engine(to, e))?;
    Ok(json!({"text": String::from_utf8(bytes).map_err(|e| Refusal::engine(to, e))?, "id": s.id()}))
}
