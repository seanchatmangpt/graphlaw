//! R2 court: `SelectStatement::compile` validates the FROM table's schema and
//! alias exactly as it validates join tables (REFUSAL-01 InvalidIdentifier),
//! so a hostile identifier never reaches emitted SQL.

use graphlaw::sql_ast::{
    AstError, QualifiedIdent, RefusalCode, SelectItem, SelectStatement, SqlExpr, SqlIdent,
    TableRef,
};

fn select(schema: Option<&str>, alias: Option<&str>) -> SelectStatement {
    SelectStatement {
        ctes: Vec::new(),
        from: TableRef {
            name: "events".to_string(),
            alias: alias.map(str::to_string),
            schema: schema.map(str::to_string),
        },
        joins: Vec::new(),
        select_list: vec![SelectItem::Expr(SqlExpr::ColumnRef {
            column: "v".to_string(),
            table_alias: None,
        })],
        where_clause: None,
        group_by: Vec::new(),
        having: None,
        order_by: Vec::new(),
        limit: None,
        offset: None,
        qualify: None,
    }
}

fn assert_invalid_identifier(err: AstError) {
    assert_eq!(err.code, RefusalCode::InvalidIdentifier, "{err}");
}

#[test]
fn hostile_from_schema_refuses_invalid_identifier() {
    let err = select(Some("s\"; DROP TABLE t; --"), None)
        .compile()
        .expect_err("hostile schema must refuse");
    assert_invalid_identifier(err);
}

#[test]
fn hostile_from_alias_refuses_invalid_identifier() {
    let err = select(None, Some("a\"; DROP TABLE t; --"))
        .compile()
        .expect_err("hostile alias must refuse");
    assert_invalid_identifier(err);
}

#[test]
fn clean_from_schema_and_alias_compile_and_emit_identifiers() {
    let q = select(Some("main"), Some("e"))
        .compile()
        .expect("clean identifiers compile");
    assert!(q.sql.contains("FROM main.events e"), "{}", q.sql);
}

#[test]
fn sql_ident_and_qualified_ident_admit_valid_and_refuse_hostile() {
    let id = SqlIdent::new("regional_sales_2026").expect("valid SqlIdent");
    assert_eq!(id.as_str(), "regional_sales_2026");

    let qid = QualifiedIdent::new("main.regional_sales").expect("valid QualifiedIdent");
    assert_eq!(qid.as_str(), "main.regional_sales");

    assert_invalid_identifier(SqlIdent::new("").expect_err("empty SqlIdent"));
    assert_invalid_identifier(SqlIdent::new("a.b").expect_err("dotted SqlIdent"));
    assert_invalid_identifier(
        SqlIdent::new("x; DROP TABLE t").expect_err("injection SqlIdent"),
    );

    assert_invalid_identifier(QualifiedIdent::new("").expect_err("empty QualifiedIdent"));
    assert_invalid_identifier(
        QualifiedIdent::new("main..events").expect_err("empty segment QualifiedIdent"),
    );
    assert_invalid_identifier(
        QualifiedIdent::new("main.events; --").expect_err("injection QualifiedIdent"),
    );
}
