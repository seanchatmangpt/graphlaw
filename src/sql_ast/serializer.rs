#![allow(missing_docs)]
#![allow(clippy::result_large_err)]

use super::ast::{
    AggregateFunction, CompiledQuery, JoinClause, OrderTerm, ScopePos, SqlExpr, SqlLiteral,
    TableRef, validate_identifier, validate_qualified_identifier,
};
use super::refusals::{AstError, RefusalCode, refusal_arm};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub from: TableRef,
    pub joins: Vec<JoinClause>,
    pub select_list: Vec<SqlExpr>,
    pub where_clause: Option<SqlExpr>,
    pub group_by: Vec<SqlExpr>,
    /// `sql:having` / `sql:havingClause` predicate (aggregates legal here).
    pub having: Option<SqlExpr>,
    /// `sql:orderBy` terms (aggregates legal here).
    pub order_by: Vec<OrderTerm>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SelectStatement {
    const MAX_DEPTH: usize = 64;

    /// Compiles the AST into parameterized SQL compatible with DuckDB
    /// prepared statements (`$1, $2, ...`).
    ///
    /// Returns the SQL text plus its typed parameter vector in
    /// [`CompiledQuery`]. Identifier validation runs before any
    /// serialization: a table/column/alias name with characters outside the
    /// validated identifier alphabet is a typed
    /// `[REFUSAL-01] MalformedAstRefusal` (see `docs/refusals.md`), never a
    /// panic or an injection surface. Note: this hand-built-AST path does
    /// not anchor a canonical graph; only [`compile_turtle`] produces the
    /// graph-backed form the receipt law keys on.
    pub fn compile(&self) -> Result<CompiledQuery, AstError> {
        validate_identifier(&self.from.name)?;
        if let Some(schema) = &self.from.schema {
            validate_qualified_identifier(schema)?;
        }
        if let Some(alias) = &self.from.alias {
            validate_identifier(alias)?;
        }

        let mut params = Vec::new();
        let mut sql = String::with_capacity(256);

        sql.push_str("SELECT ");
        if self.select_list.is_empty() {
            sql.push('*');
        } else {
            for (idx, item) in self.select_list.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(item, &mut sql, &mut params, 0, ScopePos::SelectList)?;
            }
        }

        sql.push_str(" FROM ");
        if let Some(schema) = &self.from.schema {
            sql.push_str(schema);
            sql.push('.');
        }
        sql.push_str(&self.from.name);
        if let Some(alias) = &self.from.alias {
            sql.push(' ');
            sql.push_str(alias);
        }

        for join in &self.joins {
            validate_identifier(&join.table.name)?;
            if let Some(schema) = &join.table.schema {
                validate_qualified_identifier(schema)?;
            }
            if let Some(alias) = &join.table.alias {
                validate_identifier(alias)?;
            }
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            if let Some(schema) = &join.table.schema {
                sql.push_str(schema);
                sql.push('.');
            }
            sql.push_str(&join.table.name);
            if let Some(alias) = &join.table.alias {
                sql.push(' ');
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            Self::serialize_expr(
                &join.condition,
                &mut sql,
                &mut params,
                0,
                ScopePos::Predicate,
            )?;
        }

        if let Some(where_clause) = &self.where_clause {
            sql.push_str(" WHERE ");
            Self::serialize_expr(where_clause, &mut sql, &mut params, 0, ScopePos::Predicate)?;
        }

        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (idx, item) in self.group_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(item, &mut sql, &mut params, 0, ScopePos::GroupBy)?;
            }
        }

        if let Some(having) = &self.having {
            sql.push_str(" HAVING ");
            Self::serialize_expr(having, &mut sql, &mut params, 0, ScopePos::Having)?;
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            for (idx, term) in self.order_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(&term.expr, &mut sql, &mut params, 0, ScopePos::OrderBy)?;
                if term.descending {
                    sql.push_str(" DESC");
                }
            }
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // C1-a: SelectStatement::compile() alone is not graph-backed; only the
        // `compile_turtle` SelectStatement arm classifies as cacheable.
        Ok(CompiledQuery::raw_compiled(sql, params))
    }

    pub(super) fn serialize_expr(
        expr: &SqlExpr,
        sql: &mut String,
        params: &mut Vec<SqlLiteral>,
        depth: usize,
        pos: ScopePos,
    ) -> Result<(), AstError> {
        if depth > Self::MAX_DEPTH {
            return Err(AstError::RecursionLimitExceeded(Self::MAX_DEPTH));
        }

        match expr {
            SqlExpr::ColumnRef {
                column,
                table_alias,
            } => {
                validate_identifier(column)?;
                if let Some(alias) = table_alias {
                    // Qualified references (`main.regional_sales.region_id`)
                    // validate per dot-separated part.
                    validate_qualified_identifier(alias)?;
                    sql.push_str(alias);
                    sql.push('.');
                }
                sql.push_str(column);
            }
            SqlExpr::Literal(lit) => {
                params.push(lit.clone());
                sql.push('$');
                sql.push_str(&params.len().to_string());
            }
            SqlExpr::Binary { op, left, right } => {
                sql.push('(');
                Self::serialize_expr(left, sql, params, depth + 1, pos)?;
                sql.push(' ');
                sql.push_str(op.as_sql());
                sql.push(' ');
                Self::serialize_expr(right, sql, params, depth + 1, pos)?;
                sql.push(')');
            }
            SqlExpr::FunctionCall { function, argument } => {
                if !pos.allows_aggregate() {
                    // Typed arm: `num` carries the aggregate discriminant that
                    // `aggregate_debug_name` renders on the cold Display path.
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::AGG_IN_POSITION,
                    )
                    .with_num(match function {
                        AggregateFunction::Count => 0,
                        AggregateFunction::Sum => 1,
                        AggregateFunction::Avg => 2,
                        AggregateFunction::Min => 3,
                        AggregateFunction::Max => 4,
                    }));
                }
                sql.push_str(function.as_sql());
                sql.push('(');
                Self::serialize_expr(
                    argument,
                    sql,
                    params,
                    depth + 1,
                    ScopePos::AggregateArgument,
                )?;
                sql.push(')');
            }
            SqlExpr::Alias { expr, alias } => {
                if pos == ScopePos::AggregateArgument {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::ALIASED_AGG_ARG,
                    ));
                }
                validate_identifier(alias)?;
                Self::serialize_expr(expr, sql, params, depth + 1, pos)?;
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
        Ok(())
    }
}
