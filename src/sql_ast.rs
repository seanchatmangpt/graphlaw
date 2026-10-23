//! Declarative SQL AST representation and deterministic compiler for GraphLaw.
//!
//! Provides in-memory AST terms matching the `ontologies/sql-ast.ttl` ontology,
//! identifier validation eliminating structural injection, parameterized SQL serialization,
//! and recursive bounded RDF collection unrolling.

#![allow(missing_docs)]
// Clippy `result_large_err` residual (coordinator decision, scoped to this
// file only): `AstError` intentionally embeds the ~208-byte `RefusalContext`
// by design — refusals are pure data (docs/refusals.md typed-refusal law).
// Boxing the context is an API change deferred to the switchover phase, not
// a lane-level edit.
#![allow(clippy::result_large_err)]

use std::collections::{HashMap, HashSet};
use std::fmt;

mod ast;
mod refusals;
mod serializer;

pub use ast::*;
pub use refusals::refusal_arm;
pub use refusals::*;
pub use serializer::*;

use ast::{aggregate_argument_pos, aggregate_from_name, canonical_graph_bytes};

// =============================================================================
// Graph Unrolling Helper (purrdf / RDF list extraction)
// =============================================================================

pub trait RdfAstGraphReader {
    type Node: Clone + fmt::Display;

    fn get_single_object(&self, subject: &Self::Node, predicate: &str) -> Option<Self::Node>;
    fn get_datatype_literal(
        &self,
        subject: &Self::Node,
        predicate: &str,
    ) -> Option<(String, Option<String>)>;
    fn get_iri(&self, node: &Self::Node) -> Option<String>;

    /// Number of objects on `(node, predicate)`, across IRIs and literals.
    fn object_count(&self, node: &Self::Node, predicate: &str) -> usize;

    /// Counts only IRI-valued objects of `predicate` (dual-form
    /// discriminator refusals must not count string-literal objects).
    fn iri_object_count(&self, node: &Self::Node, predicate: &str) -> usize;

    /// Predicate IRIs asserted on `node` — backs the unknown-statement-
    /// predicate gate (D1/RUST-7: a rename must not be a silent-ignore
    /// escape hatch past the closed sql: predicate enum). Default: empty,
    /// so the gate degrades to a no-op for readers that cannot enumerate
    /// predicates (never a false refusal on a lawful document).
    fn predicate_iris(&self, _node: &Self::Node) -> Vec<String> {
        Vec::new()
    }

    /// SQLRDF-17: the admitted Turtle AST document this reader is backed
    /// by, when the reader is document-backed. Builtin eval admits only
    /// graph-backed queries compiled via `compile_turtle`, so a reader that
    /// cannot supply the document (default: `None`) is a typed REFUSAL-04
    /// at the builtin, never a hand-built-AST admission.
    fn query_document_turtle(&self) -> Option<String> {
        None
    }

    /// Cardinality-checked object read (audit E3/fuzz-4): more than one
    /// object on a single-cardinality clause predicate is REFUSAL-01; a
    /// duplicate must not win by document order.
    fn single_object_checked(
        &self,
        node: &Self::Node,
        predicate: &str,
    ) -> Result<Option<Self::Node>, AstError> {
        if self.object_count(node, predicate) > 1 {
            return Err(AstError::new(
                RefusalCode::MalformedAst,
                refusal_arm::DUP_CLAUSE_PREDICATE,
            )
            .with_ident(predicate)
            .with_aux_ident(node));
        }
        Ok(self.get_single_object(node, predicate))
    }

    /// Deterministically unwraps an rdf:List (rdf:first / rdf:rest chain) up to max_items.
    fn unroll_rdf_list(
        &self,
        head: &Self::Node,
        max_items: usize,
    ) -> Result<Vec<Self::Node>, AstError> {
        let rdf_first = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
        let rdf_rest = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
        let rdf_nil = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

        let mut items = Vec::new();
        let mut curr = head.clone();
        let mut visited = HashSet::new();

        while let Some(iri) = self.get_iri(&curr) {
            if iri == rdf_nil {
                break;
            }

            if items.len() >= max_items {
                return Err(AstError::RecursionLimitExceeded(max_items));
            }

            if !visited.insert(iri.clone()) {
                return Err(AstError::new(RefusalCode::MalformedList, 0));
            }

            let first = self.get_single_object(&curr, rdf_first).ok_or_else(|| {
                AstError::MissingProperty(rdf_first.to_string(), curr.to_string())
            })?;
            items.push(first);

            curr = self
                .get_single_object(&curr, rdf_rest)
                .ok_or_else(|| AstError::MissingProperty(rdf_rest.to_string(), curr.to_string()))?;
        }

        Ok(items)
    }

    /// Recursively extracts a `SqlExpr` from an AST node in the graph.
    /// Legacy entry point: expression is assumed to sit in a permissive
    /// position (see [`Self::extract_expr_scoped`] for the scope-integrity
    /// aware path used by the clause compiler).
    fn extract_expr(&self, node: &Self::Node, depth: usize) -> Result<SqlExpr, AstError> {
        self.extract_expr_scoped(node, depth, ScopePos::SelectList)
    }

    /// Scope-integrity aware extraction (rec 1.3/3.3/5). `pos` is the
    /// syntactic position of `node`; aggregates are only legal in SELECT
    /// list, HAVING and ORDER BY; an explicit `sql:ProcessScope` column may
    /// not surface in a projection list; an explicit `sql:ViewScope` column
    /// may not filter inside a predicate. A property-stuffing node carrying
    /// more than one expression discriminator is REFUSAL-01 (never silently
    /// extract one term and let the aggregate vanish).
    fn extract_expr_scoped(
        &self,
        node: &Self::Node,
        depth: usize,
        pos: ScopePos,
    ) -> Result<SqlExpr, AstError> {
        if depth > 32 {
            return Err(AstError::RecursionLimitExceeded(32));
        }

        let sql_ns = "http://graphlaw.org/sql#";

        // sql:scope is single-cardinality and only meaningful on column
        // references; a scope annotation on any other expression node is
        // REFUSAL-01, never silently ignored (fuzz 1/4).
        let scope_objects = self.object_count(node, "http://graphlaw.org/sql#scope");
        if scope_objects > 1 {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_CARDINALITY)
                    .with_ident(node)
                    .with_num(scope_objects),
            );
        }
        let explicit_scope = self
            .get_single_object(node, "http://graphlaw.org/sql#scope")
            .and_then(|n| self.get_iri(&n))
            .or_else(|| {
                self.get_datatype_literal(node, "http://graphlaw.org/sql#scope")
                    .map(|(l, _)| l)
            });
        if explicit_scope.is_some() {
            // Scope is meaningful on column references and on output
            // expressions (the section 7 graph annotates the aliased
            // aggregate). It is meaningless on literals and binary/operator
            // nodes: carrying it there is REFUSAL-01, never silence.
            let is_column = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#columnName")
                .is_some();
            let is_output_expr = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#aliasName")
                .is_some()
                || self
                    .get_datatype_literal(node, "http://graphlaw.org/sql#alias")
                    .is_some()
                || self
                    .get_datatype_literal(node, "http://graphlaw.org/sql#functionName")
                    .is_some()
                || self
                    .get_single_object(node, "http://graphlaw.org/sql#function")
                    .is_some();
            if !is_column && !is_output_expr {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::SCOPE_WRONG_NODE,
                )
                .with_ident(node));
            }
        }

        // Ambiguity gate: at most one expression discriminator per node.
        let has_column = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#columnName")
            .is_some();
        let has_literal = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#literalValue")
            .or_else(|| self.get_datatype_literal(node, "http://graphlaw.org/sql#value"))
            .is_some();
        let has_alias_wrapper = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#aliasName")
            .is_some();
        let has_func_iri = self
            .get_single_object(node, "http://graphlaw.org/sql#function")
            .is_some();
        let has_func_name = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#functionName")
            .is_some();
        let has_operator = self
            .get_single_object(node, "http://graphlaw.org/sql#operator")
            .is_some()
            || self
                .get_datatype_literal(node, "http://graphlaw.org/sql#operator")
                .is_some();
        let discriminators = [
            has_column,
            has_literal,
            has_alias_wrapper,
            has_func_iri,
            has_func_name,
            has_operator,
        ]
        .iter()
        .filter(|b| **b)
        .count();
        if discriminators > 1 {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DISCRIMINATORS)
                    .with_ident(node)
                    .with_num(discriminators),
            );
        }

        // D3: `sql:expression` is the property of the alias wrapper
        // (sql:aliasName + sql:expression). Without sql:aliasName it is a
        // transparent expression wrapper (e.g. an sql:orderBy item annotating
        // an aggregate with sql:direction plus sql:expression). Carrying it
        // alongside any other discriminator is a laundering marker
        // (REFUSAL-01), never first-key-wins.
        let has_expr_wrapper = self
            .get_single_object(node, "http://graphlaw.org/sql#expression")
            .is_some();
        if has_expr_wrapper && !has_alias_wrapper && discriminators > 0 {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::EXPRESSION_WRAPPER)
                    .with_ident(node),
            );
        }

        // Check if node is a ColumnRef
        if has_column {
            let (col_name, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#columnName")
                .expect("checked above");
            let alias = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#tableAlias")
                .map(|(a, _)| a);
            // sql:qualifier (section 7) renders as a dotted prefix.
            let qualifier = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#qualifier")
                .map(|(q, _)| q)
                .or(alias);

            // Scope integrity on explicit scope annotations (section 1.3/5).
            if let Some(scope_iri) = explicit_scope {
                let scope = Scope::from_iri(&scope_iri)?;
                if scope == Scope::Process && pos.is_view_surface() {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::PROCESS_SCOPE_IN_SELECT,
                    )
                    .with_ident(&col_name));
                }
                if scope == Scope::View
                    && matches!(
                        pos,
                        ScopePos::Predicate | ScopePos::GroupBy | ScopePos::Having
                    )
                {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::VIEW_SCOPE_IN_PREDICATE,
                    )
                    .with_ident(&col_name));
                }
                // TC-MUT-01 arm B: an explicit scope annotation on an
                // aggregate ARGUMENT beneath a predicate-rooted aggregate is
                // a laundering marker — the argument position is process-only
                // by construction, so the marker can only re-scope the
                // aggregate's output past the gate. The aggregate's own
                // legality (e.g. HAVING) must not inherit. Under an
                // output-position aggregate the section 7 example stays
                // lawful (AggregateArgument, no refusal).
                if pos == ScopePos::PredicateAggregateArgument {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::SCOPE_IN_PRED_AGG_ARG,
                    )
                    .with_ident(&col_name)
                    .with_aux_ident(&scope_iri));
                }
            }

            let expr = SqlExpr::ColumnRef {
                column: col_name,
                table_alias: qualifier,
            };
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // Check if node is a Literal
        if has_literal {
            let (val_str, dtype) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#literalValue")
                .or_else(|| self.get_datatype_literal(node, "http://graphlaw.org/sql#value"))
                .expect("checked above");
            let lit = match dtype.as_deref() {
                // Typed refusals (LITERAL_NOT_INT / LITERAL_NOT_FLOAT): a
                // typed literal that fails to parse refuses; it is never
                // silently stringified or coerced to 0/0.0.
                Some("http://www.w3.org/2001/XMLSchema#integer") => {
                    SqlLiteral::Integer(val_str.parse().map_err(|_| {
                        AstError::new(RefusalCode::LiteralNotInt, 0)
                            .with_ident(&val_str)
                            .with_aux_ident(node)
                    })?)
                }
                Some("http://www.w3.org/2001/XMLSchema#double") => {
                    SqlLiteral::Float(val_str.parse().map_err(|_| {
                        AstError::new(RefusalCode::LiteralNotFloat, 0)
                            .with_ident(&val_str)
                            .with_aux_ident(node)
                    })?)
                }
                Some("http://www.w3.org/2001/XMLSchema#boolean") => {
                    SqlLiteral::Boolean(val_str == "true" || val_str == "1")
                }
                _ => SqlLiteral::String(val_str),
            };
            return Self::finish_alias(self, node, SqlExpr::Literal(lit), sql_ns, pos);
        }

        // Check if node is an Alias wrapper
        if has_alias_wrapper {
            if matches!(
                pos,
                ScopePos::AggregateArgument | ScopePos::PredicateAggregateArgument
            ) {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::ALIAS_IN_AGG_ARG,
                )
                .with_ident(node));
            }
            let (alias_name, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#aliasName")
                .expect("checked above");
            let target_expr_node = self
                .get_single_object(node, "http://graphlaw.org/sql#expression")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#expression".to_string(),
                        node.to_string(),
                    )
                })?;
            let inner_expr = self.extract_expr_scoped(&target_expr_node, depth + 1, pos)?;
            return Ok(SqlExpr::Alias {
                expr: Box::new(inner_expr),
                alias: alias_name,
            });
        }

        // Check if node is a FunctionCall (axiom IRI form: sql:function + sql:argument)
        if has_func_iri {
            let func_node = self
                .get_single_object(node, "http://graphlaw.org/sql#function")
                .expect("checked above");
            let func_iri = self.get_iri(&func_node).ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#function".to_string(),
                    node.to_string(),
                )
            })?;
            let function = AggregateFunction::from_iri(&func_iri)?;
            let arg_node = self
                .get_single_object(node, "http://graphlaw.org/sql#argument")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#argument".to_string(),
                        node.to_string(),
                    )
                })?;
            let argument =
                self.extract_expr_scoped(&arg_node, depth + 1, aggregate_argument_pos(pos))?;
            let expr = Self::check_aggregate_position(
                SqlExpr::FunctionCall {
                    function,
                    argument: Box::new(argument),
                },
                pos,
            )?;
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // FunctionCall (section 7 literal form: sql:functionName + sql:arguments list)
        if has_func_name {
            let (fname, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#functionName")
                .expect("checked above");
            let function = aggregate_from_name(&fname)?;
            let args_head = self
                .get_single_object(node, "http://graphlaw.org/sql#arguments")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#arguments".to_string(),
                        node.to_string(),
                    )
                })?;
            let args = self.unroll_rdf_list(&args_head, 8)?;
            let arg_node = args.first().ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#arguments".to_string(),
                    node.to_string(),
                )
            })?;
            let argument =
                self.extract_expr_scoped(arg_node, depth + 1, aggregate_argument_pos(pos))?;
            let expr = Self::check_aggregate_position(
                SqlExpr::FunctionCall {
                    function,
                    argument: Box::new(argument),
                },
                pos,
            )?;
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // Check if node is a BinaryExpression
        if has_operator {
            let op = self.read_operator(node)?.ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#operator".to_string(),
                    node.to_string(),
                )
            })?;

            let left_legacy =
                self.single_object_checked(node, "http://graphlaw.org/sql#leftOperand")?;
            let left_norm = self.single_object_checked(node, "http://graphlaw.org/sql#left")?;
            if left_legacy.is_some() && left_norm.is_some() {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::DUAL_LEFT_OPERAND,
                )
                .with_ident(node));
            }
            let right_legacy =
                self.single_object_checked(node, "http://graphlaw.org/sql#rightOperand")?;
            let right_norm = self.single_object_checked(node, "http://graphlaw.org/sql#right")?;
            if right_legacy.is_some() && right_norm.is_some() {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::DUAL_RIGHT_OPERAND,
                )
                .with_ident(node));
            }
            let left_node = self
                .get_single_object(node, "http://graphlaw.org/sql#leftOperand")
                .or_else(|| self.get_single_object(node, "http://graphlaw.org/sql#left"))
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#leftOperand".to_string(),
                        node.to_string(),
                    )
                })?;
            let right_node = self
                .get_single_object(node, "http://graphlaw.org/sql#rightOperand")
                .or_else(|| self.get_single_object(node, "http://graphlaw.org/sql#right"))
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#rightOperand".to_string(),
                        node.to_string(),
                    )
                })?;

            let left = self.extract_expr_scoped(&left_node, depth + 1, pos)?;
            let right = self.extract_expr_scoped(&right_node, depth + 1, pos)?;

            return Ok(SqlExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        // D3: a bare sql:expression wrapper (no sql:aliasName, no other
        // discriminator — laundering guarded above) is transparent: extract
        // the inner expression in the SAME syntactic position, so an
        // sql:orderBy item carrying sql:expression -> aggregate extracts the
        // aggregate under ScopePos::OrderBy (which allows aggregates, rec 5)
        // instead of dying as an untyped "Unexpected terminal node".
        if let Some(inner) = self.get_single_object(node, "http://graphlaw.org/sql#expression") {
            return self.extract_expr_scoped(&inner, depth + 1, pos);
        }

        Err(AstError::MalformedNode(node.to_string()))
    }

    /// REFUSAL-03 gate: an aggregate is legal only in SELECT list, HAVING and
    /// ORDER BY; nested aggregates (COUNT(SUM(x))) refuse everywhere.
    fn check_aggregate_position(expr: SqlExpr, pos: ScopePos) -> Result<SqlExpr, AstError> {
        if !pos.allows_aggregate() {
            return Err(AstError::new(
                RefusalCode::ScopeIntegrity,
                refusal_arm::AGG_IN_PREDICATE,
            ));
        }
        Ok(expr)
    }

    /// Wraps an extracted expression in an output alias when the node carries
    /// `sql:alias` (section 7 attribute form; not an expression discriminator).
    fn finish_alias(
        this: &Self,
        node: &Self::Node,
        expr: SqlExpr,
        sql_ns: &str,
        pos: ScopePos,
    ) -> Result<SqlExpr, AstError> {
        if let Some((alias, _)) = this.get_datatype_literal(node, &format!("{sql_ns}alias")) {
            if matches!(
                pos,
                ScopePos::AggregateArgument | ScopePos::PredicateAggregateArgument
            ) {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::SQL_ALIAS_IN_AGG_ARG,
                )
                .with_ident(node));
            }

            return Ok(SqlExpr::Alias {
                expr: Box::new(expr),
                alias,
            });
        }
        Ok(expr)
    }

    /// Reads `sql:operator` as either a section 4 string token (xsd:string
    /// literal) or a legacy axiom IRI. Disallowed tokens are REFUSAL-01.
    fn read_operator(&self, node: &Self::Node) -> Result<Option<Operator>, AstError> {
        // D5: dual-carry (string token AND IRI operator) is REFUSAL-01 — the
        // string used to win silently and the IRI term laundered past review.
        let has_string_operator = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#operator")
            .is_some();
        // MINIMAL UNBLOCK (compile-freeze SLA, lane R1, disclosed): count
        // only IRI objects — `object_count` includes string-literal
        // operators, which made every literal-operator expression a false
        // dual-carry refusal. Repaired (repair lane): order-independent via
        // a dedicated `iri_object_count` trait method — `get_single_object`
        // only inspects the first term, so the string-first ordering slipped
        // past the dual-carry gate.
        let iri_operator_objects = self.iri_object_count(node, "http://graphlaw.org/sql#operator");
        if has_string_operator && iri_operator_objects > 0 {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_OPERATOR_FORMS)
                    .with_ident(node),
            );
        }
        if let Some((token, _)) =
            self.get_datatype_literal(node, "http://graphlaw.org/sql#operator")
        {
            return Operator::from_token(&token).map(Some);
        }
        if let Some(op_node) = self.get_single_object(node, "http://graphlaw.org/sql#operator") {
            let iri = self.get_iri(&op_node).ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#operator".to_string(),
                    node.to_string(),
                )
            })?;
            return Operator::from_iri(&iri).map(Some);
        }
        Ok(None)
    }

    /// Reads a data source: either the legacy `sql:fromTable` alias (single
    /// TableRef node) or the normative `sql:from` rdf:List of table
    /// identifiers with optional `sql:schemaName` (rec 3.3).
    fn extract_table_ref(&self, node: &Self::Node) -> Result<TableRef, AstError> {
        // D1 (fuzz law, rec 1.3/3.3): sql:scope is defined on column and
        // output-expression nodes only. A scope marker on a table-reference
        // node (sql:fromTable / sql:joinTable target) is a term out of
        // place — REFUSAL-01, never silently ignored.
        let scope_marker = self.object_count(node, "http://graphlaw.org/sql#scope") > 0
            || self
                .get_datatype_literal(node, "http://graphlaw.org/sql#scope")
                .is_some();
        if scope_marker {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_ON_TABLE_REF)
                    .with_ident(node),
            );
        }
        let (name, _) = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#tableName")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#tableName".to_string(),
                    node.to_string(),
                )
            })?;
        let alias = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#tableAlias")
            .map(|(a, _)| a);
        let schema = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#schemaName")
            .map(|(s, _)| s);
        Ok(TableRef {
            name,
            alias,
            schema,
        })
    }

    /// Reads the FROM clause: `sql:from` rdf:List first, `sql:fromTable`
    /// alias second. Neither present is REFUSAL-01.
    fn extract_from(&self, query_node: &Self::Node) -> Result<TableRef, AstError> {
        // Mixed-source documents are REFUSAL-01: the validated sql:fromTable
        // must not be a decoy for an unreviewed sql:from list (audit E2).
        let from_list = self.single_object_checked(query_node, "http://graphlaw.org/sql#from")?;
        let from_table =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#fromTable")?;
        if from_list.is_some() && from_table.is_some() {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_FROM)
                    .with_ident(query_node),
            );
        }
        if let Some(from_head) = from_list {
            let items = self.unroll_rdf_list(&from_head, 64)?;
            let first = items.first().ok_or_else(|| {
                AstError::new(RefusalCode::MalformedAst, refusal_arm::EMPTY_FROM)
                    .with_ident(query_node)
            })?;
            return self.extract_table_ref(first);
        }
        if let Some(table_ref_node) = from_table {
            return self.extract_table_ref(&table_ref_node);
        }
        Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::NO_FROM).with_ident(query_node))
    }

    /// Extracts a full `SelectStatement` AST from a target query node.
    /// Normative property names (rec 3.3: sql:select/from/where/having/
    /// orderBy/groupBy) are read first; the SQLRDF-01 aliases (selectList/
    /// fromTable/whereClause/havingClause) stay legal.
    fn extract_select_statement(
        &self,
        query_node: &Self::Node,
    ) -> Result<SelectStatement, AstError> {
        // D1/RUST-7 unknown-predicate gate: every sql:-namespaced predicate on
        // the statement node must be in the closed enum the reader knows how
        // to review. A renamed clause (sql:whereX) must refuse typed
        // REFUSAL-01, never be dropped on the floor and admit the statement
        // without the filter the author wrote.
        const SQL_NS: &str = "http://graphlaw.org/sql#";
        const KNOWN_SQL_PREDICATES: &[&str] = &[
            "statementIndex",
            "selectList",
            "select",
            "from",
            "fromTable",
            "joinClause",
            "whereClause",
            "where",
            "having",
            "havingClause",
            "groupBy",
            "orderBy",
            "limit",
            "offset",
        ];
        for pred in self.predicate_iris(query_node) {
            if let Some(local) = pred.strip_prefix(SQL_NS)
                && !KNOWN_SQL_PREDICATES.contains(&local)
            {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::UNKNOWN_STATEMENT_PREDICATE,
                )
                .with_ident(query_node)
                .with_aux_ident(&pred));
            }
        }
        let from = self.extract_from(query_node)?;

        // Joins (optional)
        let mut joins = Vec::new();
        if let Some(join_node) =
            self.get_single_object(query_node, "http://graphlaw.org/sql#joinClause")
        {
            let join_type = if let Some(type_node) =
                self.get_single_object(&join_node, "http://graphlaw.org/sql#joinType")
            {
                let type_iri = self.get_iri(&type_node).unwrap_or_default();
                // Unknown join-type IRI: typed REFUSAL-01 refusal (arm
                // JOIN_TYPE_IRI) — never a silent downgrade to Inner. A
                // MISSING sql:joinType predicate stays Inner (default), and
                // both behaviors are pinned by courts.
                JoinType::from_iri(&type_iri).map_err(|_| {
                    AstError::new(RefusalCode::MalformedAst, refusal_arm::JOIN_TYPE_IRI)
                        .with_ident(&type_iri)
                })?
            } else {
                JoinType::Inner
            };

            let join_table_node = self
                .get_single_object(&join_node, "http://graphlaw.org/sql#joinTable")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#joinTable".to_string(),
                        join_node.to_string(),
                    )
                })?;
            let join_table = self.extract_table_ref(&join_table_node)?;

            let join_cond_node = self
                .get_single_object(&join_node, "http://graphlaw.org/sql#joinCondition")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#joinCondition".to_string(),
                        join_node.to_string(),
                    )
                })?;

            let join_condition =
                self.extract_expr_scoped(&join_cond_node, 0, ScopePos::Predicate)?;

            joins.push(JoinClause {
                join_type,
                table: join_table,
                condition: join_condition,
            });
        }

        // Unroll projection list (rdf:List): sql:selectList or sql:select.
        // Carrying both is REFUSAL-01: the reviewer must see the list the
        // engine executes (audit E1).
        let select_legacy =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#selectList")?;
        let select_norm =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#select")?;
        if select_legacy.is_some() && select_norm.is_some() {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_SELECT)
                    .with_ident(query_node),
            );
        }
        let select_head = select_legacy.or(select_norm);
        let select_head = select_head.ok_or_else(|| {
            AstError::new(RefusalCode::MalformedAst, refusal_arm::NO_PROJECTION)
                .with_ident(query_node)
        })?;
        let select_nodes = self.unroll_rdf_list(&select_head, 64)?;
        let mut select_list = Vec::with_capacity(select_nodes.len());
        for node in select_nodes {
            select_list.push(self.extract_expr_scoped(&node, 0, ScopePos::SelectList)?);
        }
        // D1: an empty projection list (rdf:nil head) is malformed, never
        // SELECT *: the ontology says projection is never zero-column
        // (both sql:selectList and sql:select paths funnel through here).
        if select_list.is_empty() {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::EMPTY_PROJECTION)
                    .with_ident(query_node),
            );
        }

        // Where clause (optional): sql:whereClause or sql:where; both is REFUSAL-01.
        let where_legacy =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#whereClause")?;
        let where_norm = self.single_object_checked(query_node, "http://graphlaw.org/sql#where")?;
        if where_legacy.is_some() && where_norm.is_some() {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_WHERE)
                    .with_ident(query_node),
            );
        }
        let where_clause = if let Some(where_node) = where_legacy.or(where_norm) {
            Some(self.extract_expr_scoped(&where_node, 0, ScopePos::Predicate)?)
        } else {
            None
        };

        // Group by (optional rdf:List)
        let group_head =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#groupBy")?;
        let group_by = if let Some(group_head) = group_head {
            let group_nodes = self.unroll_rdf_list(&group_head, 64)?;
            let mut list = Vec::with_capacity(group_nodes.len());
            for node in group_nodes {
                list.push(self.extract_expr_scoped(&node, 0, ScopePos::GroupBy)?);
            }
            list
        } else {
            Vec::new()
        };

        // Having (optional): sql:having or sql:havingClause; both is REFUSAL-01.
        let having_norm =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#having")?;
        let having_alias =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#havingClause")?;
        if having_norm.is_some() && having_alias.is_some() {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_HAVING)
                    .with_ident(query_node),
            );
        }
        let having = if let Some(having_node) = having_norm.or(having_alias) {
            Some(self.extract_expr_scoped(&having_node, 0, ScopePos::Having)?)
        } else {
            None
        };

        // Order by (optional rdf:List): sql:orderBy; items carry optional
        // sql:direction ("ASC" | "DESC", ASC default).
        let order_head =
            self.single_object_checked(query_node, "http://graphlaw.org/sql#orderBy")?;
        let order_by = if let Some(order_head) = order_head {
            let order_nodes = self.unroll_rdf_list(&order_head, 64)?;
            let mut list = Vec::with_capacity(order_nodes.len());
            for node in order_nodes {
                let expr = self.extract_expr_scoped(&node, 0, ScopePos::OrderBy)?;
                // A malformed direction is REFUSAL-01, never a silent fold to
                // ASC (same law as limit/offset below): only ASC/DESC admitted.
                // D4-adjacent: a present-but-IRI sql:direction is REFUSAL-01,
                // never a silent fold to ASC (distinguish present-but-IRI
                // from absent via object_count).
                let descending = if self.object_count(&node, "http://graphlaw.org/sql#direction")
                    > 0
                {
                    match self.get_datatype_literal(&node, "http://graphlaw.org/sql#direction") {
                        Some((d, _)) => {
                            if d.eq_ignore_ascii_case("DESC") {
                                true
                            } else if d.eq_ignore_ascii_case("ASC") {
                                false
                            } else {
                                return Err(AstError::new(
                                    RefusalCode::MalformedAst,
                                    refusal_arm::DIRECTION_LEX,
                                )
                                .with_ident(&d)
                                .with_aux_ident(query_node));
                            }
                        }
                        None => {
                            return Err(AstError::new(
                                RefusalCode::MalformedAst,
                                refusal_arm::DIRECTION_IRI,
                            )
                            .with_ident(query_node));
                        }
                    }
                } else {
                    false
                };
                list.push(OrderTerm { expr, descending });
            }
            list
        } else {
            Vec::new()
        };

        // Limit & offset: a malformed integer is REFUSAL-01, never a silent
        // clause drop (audit E4: unbounded scan from a SHACL-clean doc).
        let parse_clause_int = |lex: String, prop: &str| -> Result<usize, AstError> {
            let arm = if prop == "limit" {
                refusal_arm::LIMIT_MALFORMED
            } else {
                refusal_arm::OFFSET_MALFORMED
            };
            lex.parse::<usize>().map_err(|_| {
                AstError::new(RefusalCode::MalformedAst, arm)
                    .with_ident(&lex)
                    .with_aux_ident(query_node)
            })
        };
        // E4 (lane E1-court defect fix, disclosed): sql:limit/sql:offset are
        // literal-valued, and `get_single_object` only surfaces IRI objects —
        // a literal clause previously read as ABSENT, silently dropping LIMIT
        // and never refusing malformed integers. Drive both clauses off
        // `get_datatype_literal` + `object_count` instead.
        let limit_clause = |prop: &str| -> Result<Option<usize>, AstError> {
            let count = self.object_count(query_node, &format!("http://graphlaw.org/sql#{prop}"));
            if count > 1 {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::DUP_CLAUSE_PREDICATE,
                )
                .with_ident(prop)
                .with_aux_ident(query_node));
            }
            match self.get_datatype_literal(query_node, &format!("http://graphlaw.org/sql#{prop}"))
            {
                Some((lex, _)) => parse_clause_int(lex, prop).map(Some),
                None if count > 0 => {
                    let arm = if prop == "limit" {
                        refusal_arm::LIMIT_NOT_INT
                    } else {
                        refusal_arm::OFFSET_NOT_INT
                    };
                    Err(AstError::new(RefusalCode::MalformedAst, arm).with_ident(query_node))
                }
                None => Ok(None),
            }
        };
        let limit = limit_clause("limit")?;
        let offset = limit_clause("offset")?;

        Ok(SelectStatement {
            from,
            joins,
            select_list,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        })
    }

    /// Extracts a section 7 `MergeIntoStatement`: target table identifier,
    /// grouped USING sub-select, merge ON condition, and WHEN MATCHED update
    /// assignments. Any other shape falls back to the legacy scalar template.
    fn extract_merge_statement(&self, node: &Self::Node) -> Result<MergeStatement, AstError> {
        let target_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#targetTable")?
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#targetTable".to_string(),
                    node.to_string(),
                )
            })?;
        let target = self.extract_table_ref(&target_node)?;

        let using_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#usingSource")?
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#usingSource".to_string(),
                    node.to_string(),
                )
            })?;
        let source = self.extract_select_statement(&using_node)?;

        let on_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#mergeOn")?
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#mergeOn".to_string(),
                    node.to_string(),
                )
            })?;
        let on = self.extract_expr_scoped(&on_node, 0, ScopePos::Predicate)?;

        let when_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#whenMatched")?
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#whenMatched".to_string(),
                    node.to_string(),
                )
            })?;
        let assigns_head = self
            .get_single_object(&when_node, "http://graphlaw.org/sql#setAssignments")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#setAssignments".to_string(),
                    when_node.to_string(),
                )
            })?;
        let mut assignments = Vec::new();
        for assign in self.unroll_rdf_list(&assigns_head, 64)? {
            let (column, _) = self
                .get_datatype_literal(&assign, "http://graphlaw.org/sql#targetColumn")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#targetColumn".to_string(),
                        assign.to_string(),
                    )
                })?;
            let value_node = self
                .get_single_object(&assign, "http://graphlaw.org/sql#valueExpression")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#valueExpression".to_string(),
                        assign.to_string(),
                    )
                })?;
            let value = self.extract_expr_scoped(&value_node, 0, ScopePos::Predicate)?;
            assignments.push((column, value));
        }

        Ok(MergeStatement {
            target,
            source,
            on,
            assignments,
        })
    }
}

/// Validates an RDF AST dataset in Turtle syntax against the canonical SHACL rules in `ontologies/sql-ast.ttl`.
pub fn validate_ast_shacl(turtle_dataset: &str, shapes_ttl: &str) -> Result<(), AstError> {
    if let Err(over) = crate::shacl_guard::check_shape_depth(shapes_ttl) {
        return Err(
            AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_DEPTH)
                .with_num(over.observed)
                .with_aux_num(crate::shacl_guard::MAX_SHACL_VALIDATION_DEPTH),
        );
    }

    let ds = purrdf::parse_dataset(turtle_dataset.as_bytes(), "text/turtle", None)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_DATASET))?;

    let shapes = crate::shacl::engine::parse_shapes(shapes_ttl, None)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_SHAPES))?;

    let report = crate::shacl::engine::validate_dataset(&ds, &shapes)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_ENGINE))?;

    if !report.conforms {
        // Typed SHACL_VIOLATIONS carrier: first violation renders on the cold
        // path, `num` counts the remainder ("(+N more violations)").
        let first = report.results.first().ok_or_else(|| {
            AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_VIOLATIONS)
        })?;
        return Err(
            AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_VIOLATIONS)
                .with_ident(&first.focus_node)
                .with_aux_ident(first.message.as_deref().unwrap_or("violation"))
                .with_num(report.results.len()),
        );
    }

    Ok(())
}

/// SHACL-check `turtle` against the SQL AST shapes and compile its statement
/// into SQL. `SelectStatement` uses the expression compiler. Other admitted
/// statement classes compile from `sql:targetTable` and assignment literals.
/// `ConnectStatement` and `DisconnectStatement` refuse here, before any engine call.
/// Charter-A04 seam: process-wide `h_ast → Arc<CompiledQuery>` memoization.
/// A repeated identical admitted graph (same `h_ast = BLAKE3(Canon(G_AST))`)
/// must never re-run `serialize_expr` (or SHACL validation / RDFC-1.0
/// canonicalization for the re-render): the memo returns the already-admitted
/// compiled form. Arc-fed so the stored compiled query is never re-built —
/// callers clone out of the Arc. Capped (flush-on-overflow at the same 256
/// capacity class as the prepared-statement cache) and keyed by the digest,
/// so a flipped literal in the graph forks to a distinct entry by
/// construction.
static AST_COMPILE_MEMO: std::sync::OnceLock<
    std::sync::Mutex<HashMap<[u8; 32], std::sync::Arc<CompiledQuery>>>,
> = std::sync::OnceLock::new();

const AST_COMPILE_MEMO_CAPACITY: usize = 256;

fn ast_compile_memo() -> &'static std::sync::Mutex<HashMap<[u8; 32], std::sync::Arc<CompiledQuery>>>
{
    AST_COMPILE_MEMO.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Sole admission path for SQL-in-RDF: validates a Turtle description of a
/// SQL statement against the SHACL shapes in `ontologies/sql-ast.ttl` plus
/// the dialect pack (`ggen/sql-dialect/statement_classes.ttl`), parses the
/// statement graph, and compiles it to parameterized DuckDB SQL
/// (`$1, $2, ...` placeholders) pinned to the graph that produced it.
///
/// Returns the [`CompiledQuery`] carrying:
///
/// - `sql` + `params`: the parameterized statement text and its typed
///   parameter vector.
/// - `ast_graph_turtle` / `canon_graph_bytes`: the admitted graph and its
///   RDFC-1.0 canonical form (`Canon(G_AST)`), computed once here so the
///   eval/receipt path never re-parses.
/// - the admission seal: [`CompiledQuery::verify_seal`] must hold before
///   execution; `crate::duckdb_eval::GraphLawDuckDB::eval_ast_query`
///   enforces it.
///
/// # Refusals
/// Typed and stable-coded (`docs/refusals.md`), never a panic or generic DB
/// error:
///
/// - `[REFUSAL-01] MalformedAstRefusal` — shape violation: missing or
///   duplicated `sql:statementIndex`, non-integer or negative index,
///   invalid identifier characters.
/// - `[REFUSAL-02] UnimplementedStatementRefusal` — the statement class is
///   valid SHACL but outside the supported set (e.g. an unsupported
///   statement kind).
/// - `[REFUSAL-03] ScopeIntegrityRefusal` — a clause scope references the
///   wrong node or wrong cardinality.
///
/// # Examples
/// ```
/// use graphlaw::sql_ast::compile_turtle;
///
/// let doc = concat!(
///     "@prefix sql: <http://graphlaw.org/sql#> .\n",
///     "@prefix ex: <http://example.org/queries#> .\n",
///     "ex:S a sql:SelectStatement ;\n",
///     "    sql:statementIndex 1 ;\n",
///     "    sql:fromTable [ sql:tableName \"t\" ] ;\n",
///     "    sql:selectList ( [ sql:columnName \"c\" ] ) .\n",
/// );
/// let compiled = compile_turtle(doc)?;
/// assert!(compiled.sql.starts_with("SELECT"));
/// assert!(compiled.verify_seal());
/// # Ok::<(), graphlaw::sql_ast::AstError>(())
/// ```
pub fn compile_turtle(turtle: &str) -> Result<CompiledQuery, AstError> {
    let shapes = format!(
        "{}\n{}",
        include_str!("../ontologies/sql-ast.ttl"),
        include_str!("../ggen/sql-dialect/statement_classes.ttl")
    );
    validate_ast_shacl(turtle, &shapes)?;
    let reader = DatasetReader::from_turtle(turtle)?;
    let (node, kind) = reader.statement_kind()?;
    // Section 4 (StatementBaseShape): every statement MUST carry exactly one
    // integer sql:statementIndex. Missing, duplicated, wrongly-typed, or
    // negative is REFUSAL-01 and refuses before any engine call (fuzz D1).
    let require_index = |reader: &DatasetReader, n: &String| -> Result<(), AstError> {
        match reader.object_count(n, "http://graphlaw.org/sql#statementIndex") {
            0 => {
                return Err(
                    AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_MISSING)
                        .with_ident(n),
                );
            }
            c if c > 1 => {
                return Err(
                    AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_DUP)
                        .with_ident(n)
                        .with_num(c),
                );
            }
            _ => {}
        }
        let (index_lex, datatype) = reader
            .get_datatype_literal(n, "http://graphlaw.org/sql#statementIndex")
            .ok_or_else(|| {
                AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_NOT_INT).with_ident(n)
            })?;
        if datatype.as_deref() != Some("http://www.w3.org/2001/XMLSchema#integer") {
            return Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_BAD_DTYPE)
                    .with_ident(n),
            );
        }
        match index_lex.parse::<i64>() {
            Ok(v) if v >= 0 => Ok(()),
            _ => Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_NEGATIVE)
                    .with_ident(&index_lex)
                    .with_aux_ident(n),
            ),
        }
    };
    require_index(&reader, &node)?;
    // Fuzz D2: nested statement nodes are statements too — the §7 usingSource
    // sub-select carries its own index per §7's example graph.
    if let Some(using) = reader.get_single_object(&node, "http://graphlaw.org/sql#usingSource") {
        require_index(&reader, &using)?;
    }
    // R1: canonicalize ONCE at compile time (not per receipt). Failure is a
    // typed REFUSAL-01 AstError, never a crash.
    let canon_bytes = canonical_graph_bytes(turtle)?;
    // Charter-A04 memoization: identical h_ast → reuse the compiled form,
    // skipping serialize_expr entirely. The memo is consulted only on the
    // SelectStatement (cacheable-class) arm; merge/command/refusal arms are
    // never memoized. `ast_graph_turtle` is restamped from THIS call's
    // source (canonically identical graphs may differ textually).
    let h_ast = *blake3::hash(&canon_bytes).as_bytes();
    if kind.as_str() == "SelectStatement"
        && let Some(arc) = ast_compile_memo()
            .lock()
            .ok()
            .and_then(|m| m.get(&h_ast).cloned())
    {
        let mut cq = (*arc).clone();
        cq.ast_graph_turtle = Some(turtle.to_string());
        return Ok(cq);
    }
    let compiled = match kind.as_str() {
        "SelectStatement" => reader.extract_select_statement(&node)?.compile(),
        // REFUSAL-02 (UnimplementedStatementRefusal): registered carrier is
        // the typed AstError::UnsupportedTerm carrying the statement-class
        // IRI, with the "compile refusal" marker the execution court pins.
        "ConnectStatement" | "DisconnectStatement" => Err(AstError::new(
            RefusalCode::UnimplementedStatement,
            refusal_arm::UNIMPLEMENTED_STATEMENT,
        )
        .with_ident(&kind)),
        "MergeIntoStatement"
            if reader
                .get_single_object(&node, "http://graphlaw.org/sql#usingSource")
                .is_some() =>
        {
            reader.extract_merge_statement(&node)?.compile_merge()
        }
        other => reader.compile_command(&node, other),
    };
    let compiled = compiled.map(|mut cq| {
        // P28-SIL §3.1 + TC-HOT-01 two-node formula: pin the receipt anchor
        // `h_ast = BLAKE3(Canon(G_AST))` exactly ONCE at compile time —
        // BEFORE moving the bytes into `canon_graph_bytes`. Eval then
        // computes `H_query = BLAKE3(h_ast || P_params)` with one fresh
        // hasher per receipt.
        cq.receipt_state = Some(h_ast);
        cq.ast_graph_turtle = Some(turtle.to_string());
        cq.canon_graph_bytes = Some(canon_bytes);
        // C1-a classification (designs/C1-cache.md): only the SelectStatement
        // arm of `compile_turtle` is ever cacheable. Merge and command arms
        // (and every hand-built constructor) compile with `cacheable: false`;
        // no cache store exists yet — this bit only gates future admission.
        cq.cacheable = kind.as_str() == "SelectStatement";
        // RUST-5 sole-admission seal: recompute over the final stamped state
        // so eval can verify the query came through this compile path.
        cq.reseal();
        cq
    })?;
    // Charter-A04 memoization store: only cacheable-class (SelectStatement)
    // compiles enter the memo. Flush-on-overflow at the pinned capacity so
    // the memo never grows unbounded.
    if kind.as_str() == "SelectStatement"
        && let Ok(mut memo) = ast_compile_memo().lock()
    {
        if memo.len() >= AST_COMPILE_MEMO_CAPACITY && !memo.contains_key(&h_ast) {
            memo.clear();
        }
        memo.insert(h_ast, std::sync::Arc::new(compiled.clone()));
    }
    Ok(compiled)
}

struct DatasetReader {
    outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>>,
    /// The admitted Turtle document this reader was parsed from
    /// (SQLRDF-17: lets document-backed callers compile via `compile_turtle`
    /// so every admission carries canon bytes).
    turtle: String,
}

#[derive(Clone)]
enum RdfTerm {
    Iri(String),
    Literal {
        lexical: String,
        datatype: Option<String>,
    },
}

impl DatasetReader {
    fn from_turtle(turtle: &str) -> Result<Self, AstError> {
        let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
            .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_DATASET))?;
        let mut outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>> = HashMap::new();
        for quad in dataset.quads() {
            let subj = match dataset.term_value(quad.s) {
                purrdf::TermValue::Iri(i) => i,
                purrdf::TermValue::Blank { label, scope } => format!("_:{label}_{}", scope.0),
                _ => continue,
            };
            let pred = match dataset.term_value(quad.p) {
                purrdf::TermValue::Iri(i) => i,
                _ => continue,
            };
            let term = match dataset.term_value(quad.o) {
                purrdf::TermValue::Iri(i) => RdfTerm::Iri(i),
                purrdf::TermValue::Blank { label, scope } => {
                    RdfTerm::Iri(format!("_:{label}_{}", scope.0))
                }
                purrdf::TermValue::Literal {
                    lexical_form,
                    datatype,
                    ..
                } => RdfTerm::Literal {
                    lexical: lexical_form,
                    datatype: Some(datatype),
                },
                _ => continue,
            };
            outgoing
                .entry(subj)
                .or_default()
                .entry(pred)
                .or_default()
                .push(term);
        }
        Ok(Self {
            outgoing,
            turtle: turtle.to_string(),
        })
    }

    /// Finds the top-level statement node. Nested statement-class nodes
    /// (e.g. the section 7 `sql:usingSource` sub-select) are objects of other
    /// subjects, so a candidate that is nobody's object wins; ties break on
    /// the lexicographically smallest subject for determinism.
    fn statement_kind(&self) -> Result<(String, String), AstError> {
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let prefix = "http://graphlaw.org/sql#";
        let is_object: HashSet<String> = self
            .outgoing
            .values()
            .flat_map(|preds| preds.values().flatten())
            .filter_map(|t| match t {
                RdfTerm::Iri(i) if i.starts_with("_:") => Some(i.clone()),
                _ => None,
            })
            .collect();
        let mut candidates: Vec<(String, String)> = Vec::new();
        for (subj, preds) in &self.outgoing {
            let Some(objs) = preds.get(ty) else { continue };
            for term in objs {
                let RdfTerm::Iri(iri) = term else { continue };
                if let Some(kind) = iri.strip_prefix(prefix)
                    && kind.ends_with("Statement")
                    && !is_object.contains(subj)
                {
                    candidates.push((subj.clone(), kind.to_string()));
                }
            }
        }
        candidates.sort();
        candidates
            .into_iter()
            .next()
            .ok_or_else(|| AstError::MissingProperty(ty.into(), "sql:*Statement".into()))
    }

    fn ident(&self, node: &str, pred: &str) -> Result<String, AstError> {
        let node = node.to_string();
        let (lex, _) = self
            .get_datatype_literal(&node, pred)
            .ok_or_else(|| AstError::MissingProperty(pred.into(), node))?;
        validate_identifier(&lex).map(str::to_string)
    }

    fn lit(&self, node: &str, pred: &str) -> Result<String, AstError> {
        let node = node.to_string();
        let (lex, dtype) = self
            .get_datatype_literal(&node, pred)
            .ok_or_else(|| AstError::MissingProperty(pred.into(), node))?;
        let numeric = dtype
            .as_deref()
            .is_some_and(|d| d.ends_with("integer") || d.ends_with("double"))
            || lex.chars().all(|c| c.is_ascii_digit());
        if numeric {
            Ok(lex)
        } else {
            Ok(format!("'{}'", lex.replace('\'', "''")))
        }
    }

    /// Compiles the expression-level form of a command class from its
    /// structured graph (nested operand nodes, rdf:Lists, nested SELECT
    /// expressions). Returns `Ok(None)` when the statement does not use the
    /// expression-level vocabulary, so the scalar template vocabulary remains
    /// a lawful fallback (rec 3.2: both forms are the same admitted class).
    fn compile_expression_form(&self, node: &str, kind: &str) -> Result<Option<String>, AstError> {
        let node = node.to_string();
        let p = |name: &str| format!("http://graphlaw.org/sql#{name}");
        match kind {
            "CreateStatement" => {
                let Some(def) = self.get_single_object(&node, &p("tableDefinition")) else {
                    return Ok(None);
                };
                let table = self.ident(&def, &p("tableName"))?;
                let cols_head = self
                    .get_single_object(&def, &p("columns"))
                    .ok_or_else(|| AstError::MissingProperty(p("columns"), def.clone()))?;
                let mut cols = Vec::new();
                for col in self.unroll_rdf_list(&cols_head, 64)? {
                    let name = self.ident(&col, &p("columnName"))?;
                    let ty = self.column_type(&col)?;
                    cols.push(format!("{name} {ty}"));
                }
                Ok(Some(format!("CREATE TABLE {table} ({})", cols.join(", "))))
            }
            "DropStatement" => {
                let Some(target) = self.get_single_object(&node, &p("dropTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                let if_exists = self
                    .get_datatype_literal(&target, &p("ifExists"))
                    .map(|(v, _)| v == "true" || v == "1")
                    .unwrap_or(false);
                Ok(Some(if if_exists {
                    format!("DROP TABLE IF EXISTS {table}")
                } else {
                    format!("DROP TABLE {table}")
                }))
            }
            "AlterStatement" => {
                let Some(action) = self.get_single_object(&node, &p("alterAction")) else {
                    return Ok(None);
                };
                let table = self.ident(&action, &p("tableName"))?;
                let col = self.ident(&action, &p("columnName"))?;
                let ty = self.column_type(&action)?;
                Ok(Some(format!("ALTER TABLE {table} ADD COLUMN {col} {ty}")))
            }
            "TruncateStatement" => {
                let Some(target) = self.get_single_object(&node, &p("truncateTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                Ok(Some(format!("TRUNCATE {table}")))
            }
            "AnalyzeStatement" => {
                let Some(target) = self.get_single_object(&node, &p("analyzeTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                Ok(Some(format!("ANALYZE {table}")))
            }
            "CommentStatement" => {
                let Some(comment) = self.get_single_object(&node, &p("comment")) else {
                    return Ok(None);
                };
                let object = self
                    .get_single_object(&comment, &p("commentObject"))
                    .ok_or_else(|| {
                        AstError::MissingProperty(p("commentObject"), comment.clone())
                    })?;
                let object = match object.as_str() {
                    "http://graphlaw.org/sql#Table" => "TABLE",
                    other => {
                        return Err(AstError::UnsupportedTerm(other.to_string()));
                    }
                };
                let name = self.ident(&comment, &p("commentTargetName"))?;
                let text = self
                    .get_datatype_literal(&comment, &p("commentText"))
                    .ok_or_else(|| AstError::MissingProperty(p("commentText"), comment.clone()))?;
                Ok(Some(format!(
                    "COMMENT ON {object} {name} IS '{}'",
                    text.0.replace('\'', "''")
                )))
            }
            "CopyStatement" => {
                let Some(op) = self.get_single_object(&node, &p("copyOperation")) else {
                    return Ok(None);
                };
                let table = self.ident(&op, &p("tableName"))?;
                let (path, _) = self
                    .get_datatype_literal(&op, &p("filePath"))
                    .ok_or_else(|| AstError::MissingProperty(p("filePath"), op.clone()))?;
                let fmt = self
                    .get_datatype_literal(&op, &p("fileFormat"))
                    .map(|(f, _)| f)
                    .unwrap_or_else(|| "CSV".to_string());
                if !matches!(fmt.as_str(), "CSV" | "PARQUET" | "JSON") {
                    return Err(AstError::UnsupportedTerm(fmt));
                }
                Ok(Some(format!(
                    "COPY {table} TO '{}' (FORMAT {fmt})",
                    path.replace('\'', "''")
                )))
            }
            "ExportStatement" => {
                let Some((path, _)) = self.get_datatype_literal(&node, &p("exportPath")) else {
                    return Ok(None);
                };
                Ok(Some(format!(
                    "EXPORT DATABASE '{}'",
                    path.replace('\'', "''")
                )))
            }
            "ImportStatement" => {
                let Some((path, _)) = self.get_datatype_literal(&node, &p("importPath")) else {
                    return Ok(None);
                };
                Ok(Some(format!(
                    "IMPORT DATABASE '{}'",
                    path.replace('\'', "''")
                )))
            }
            "PrepareStatement" => {
                let Some(target) = self.get_single_object(&node, &p("prepareTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                let query_node = self
                    .get_single_object(&target, &p("prepareQuery"))
                    .ok_or_else(|| AstError::MissingProperty(p("prepareQuery"), target.clone()))?;
                let query = self.inline_select(&query_node)?;
                Ok(Some(format!("PREPARE {name} AS {query}")))
            }
            "ExecuteStatement" => {
                let Some(target) = self.get_single_object(&node, &p("executeTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                Ok(Some(format!("EXECUTE {name}")))
            }
            "DeallocateStatement" => {
                let Some(target) = self.get_single_object(&node, &p("deallocateTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                Ok(Some(format!("DEALLOCATE {name}")))
            }
            "ExplainStatement" => {
                let Some(query_node) = self.get_single_object(&node, &p("explainQuery")) else {
                    return Ok(None);
                };
                let query = self.inline_select(&query_node)?;
                Ok(Some(format!("EXPLAIN {query}")))
            }
            "CallStatement" => {
                let Some(target) = self.get_single_object(&node, &p("callTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("routineName"))?;
                let args = match self.get_single_object(&target, &p("arguments")) {
                    Some(head) => {
                        let items = self.unroll_rdf_list(&head, 64)?;
                        let mut rendered = Vec::with_capacity(items.len());
                        for item in items {
                            rendered.push(self.inline_expr(&item)?);
                        }
                        rendered.join(", ")
                    }
                    None => String::new(),
                };
                Ok(Some(format!("CALL {name}({args})")))
            }
            "PragmaStatement" => {
                let Some(target) = self.get_single_object(&node, &p("pragmaTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("pragmaName"))?;
                Ok(Some(format!("PRAGMA {name}")))
            }
            "SetStatement" => {
                let Some(setting) = self.get_single_object(&node, &p("configSetting")) else {
                    return Ok(None);
                };
                let key = self.ident(&setting, &p("configKey"))?;
                let value_node = self
                    .get_single_object(&setting, &p("configValue"))
                    .ok_or_else(|| AstError::MissingProperty(p("configValue"), setting.clone()))?;
                let value = self.inline_expr(&value_node)?;
                Ok(Some(format!("SET {key} = {value}")))
            }
            "ResetStatement" => {
                let Some(target) = self.get_single_object(&node, &p("resetTarget")) else {
                    return Ok(None);
                };
                let key = self.ident(&target, &p("configKey"))?;
                Ok(Some(format!("RESET {key}")))
            }
            "TransactionStatement" => {
                // BEGIN/COMMIT/ROLLBACK compile to engine text; transaction
                // state is a connection-runtime property, so COMMIT/ROLLBACK
                // without an active BEGIN surfaces the engine's
                // TransactionContext error through the typed
                // EvaluatorError::DuckDb path at execution (documented
                // behavior, same class as a missing-table engine error).
                let action = self
                    .get_single_object(&node, &p("transactionAction"))
                    .unwrap_or_else(|| "http://graphlaw.org/sql#BeginTransaction".to_string());
                let sql = match action.as_str() {
                    "http://graphlaw.org/sql#BeginTransaction" => "BEGIN",
                    "http://graphlaw.org/sql#CommitTransaction" => "COMMIT",
                    "http://graphlaw.org/sql#RollbackTransaction" => "ROLLBACK",
                    other => return Err(AstError::UnsupportedTerm(other.to_string())),
                };
                Ok(Some(sql.to_string()))
            }
            "LoadStatement" => {
                let Some(target) = self.get_single_object(&node, &p("loadTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("extensionName"))?;
                Ok(Some(format!("LOAD {name}")))
            }
            "InstallStatement" => {
                let Some(target) = self.get_single_object(&node, &p("installTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("extensionName"))?;
                let from = match self.get_datatype_literal(&target, &p("repositoryPath")) {
                    Some((path, _)) => format!(" FROM '{}'", path.replace('\'', "''")),
                    None => String::new(),
                };
                Ok(Some(format!("INSTALL {name}{from}")))
            }
            "AttachStatement" => {
                let Some(target) = self.get_single_object(&node, &p("attachTarget")) else {
                    return Ok(None);
                };
                let (path, _) = self
                    .get_datatype_literal(&target, &p("databasePath"))
                    .ok_or_else(|| AstError::MissingProperty(p("databasePath"), target.clone()))?;
                let alias = self.ident(&target, &p("attachAlias"))?;
                Ok(Some(format!(
                    "ATTACH '{}' AS {alias}",
                    path.replace('\'', "''")
                )))
            }
            "DetachStatement" => {
                let Some(target) = self.get_single_object(&node, &p("detachTarget")) else {
                    return Ok(None);
                };
                let alias = self.ident(&target, &p("attachAlias"))?;
                Ok(Some(format!("DETACH {alias}")))
            }
            "UseStatement" => {
                let Some(target) = self.get_single_object(&node, &p("useTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("databaseName"))?;
                Ok(Some(format!("USE {name}")))
            }
            "ExtensionRepositoryStatement" => {
                let Some(target) = self.get_single_object(&node, &p("repositoryTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("repositoryName"))?;
                let (prefix, _) = self
                    .get_datatype_literal(&target, &p("repositoryPrefix"))
                    .ok_or_else(|| {
                        AstError::MissingProperty(p("repositoryPrefix"), target.clone())
                    })?;
                let (key, _) = self
                    .get_datatype_literal(&target, &p("publicKey"))
                    .ok_or_else(|| AstError::MissingProperty(p("publicKey"), target.clone()))?;
                Ok(Some(format!(
                    "SET allow_extension_repositories='allowed'; CREATE OR REPLACE EXTENSION REPOSITORY {name} WITH PREFIX '{}' USING PUBLIC KEY '{}'",
                    prefix.replace('\'', "''"),
                    key.replace('\'', "''")
                )))
            }
            _ => Ok(None),
        }
    }

    /// Validates a column-type literal against the closed allowlist.
    fn column_type(&self, node: &str) -> Result<String, AstError> {
        let ty = "http://graphlaw.org/sql#columnType";
        let (lex, _) = self
            .get_datatype_literal(&node.to_string(), ty)
            .ok_or_else(|| AstError::MissingProperty(ty.to_string(), node.to_string()))?;
        let upper = lex.to_ascii_uppercase();
        match upper.as_str() {
            "INTEGER" | "BIGINT" | "VARCHAR" | "DOUBLE" | "BOOLEAN" | "DATE" => Ok(upper),
            other => Err(AstError::UnsupportedTerm(format!("column type {other}"))),
        }
    }

    /// Compiles a nested SELECT expression graph into inline SQL (literals
    /// rendered in place, not as bind parameters) for embedding in EXPLAIN /
    /// PREPARE.
    fn inline_select(&self, query_node: &str) -> Result<String, AstError> {
        let stmt = self.extract_select_statement(&query_node.to_string())?;
        validate_identifier(&stmt.from.name)?;
        let mut sql = String::from("SELECT ");
        if stmt.select_list.is_empty() {
            sql.push('*');
        } else {
            for (idx, node) in stmt.select_list.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&self.inline_expr_node(node)?);
            }
        }
        sql.push_str(" FROM ");
        sql.push_str(&stmt.from.name);
        if let Some(alias) = &stmt.from.alias {
            validate_identifier(alias)?;
            sql.push(' ');
            sql.push_str(alias);
        }
        for join in &stmt.joins {
            validate_identifier(&join.table.name)?;
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            sql.push_str(&join.table.name);
            if let Some(alias) = &join.table.alias {
                validate_identifier(alias)?;
                sql.push(' ');
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            sql.push_str(&self.inline_expr_node(&join.condition)?);
        }
        if let Some(w) = &stmt.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(&self.inline_expr_node(w)?);
        }
        if !stmt.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (idx, g) in stmt.group_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&self.inline_expr_node(g)?);
            }
        }
        if let Some(limit) = stmt.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = stmt.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        Ok(sql)
    }

    /// Renders an already-extracted `SqlExpr` inline (literals in place).
    fn inline_expr_node(&self, expr: &SqlExpr) -> Result<String, AstError> {
        let mut sql = String::new();
        self.write_inline_expr(expr, &mut sql, 0)?;
        Ok(sql)
    }

    /// Extracts and renders an operand expression node from the graph.
    fn inline_expr(&self, node: &str) -> Result<String, AstError> {
        let expr = self.extract_expr(&node.to_string(), 0)?;
        self.inline_expr_node(&expr)
    }

    fn write_inline_expr(
        &self,
        expr: &SqlExpr,
        sql: &mut String,
        depth: usize,
    ) -> Result<(), AstError> {
        if depth > 32 {
            return Err(AstError::RecursionLimitExceeded(32));
        }
        match expr {
            SqlExpr::ColumnRef {
                column,
                table_alias,
            } => {
                validate_identifier(column)?;
                if let Some(alias) = table_alias {
                    validate_identifier(alias)?;
                    sql.push_str(alias);
                    sql.push('.');
                }
                sql.push_str(column);
            }
            SqlExpr::Literal(lit) => match lit {
                SqlLiteral::Integer(i) => sql.push_str(&i.to_string()),
                SqlLiteral::Float(f) => sql.push_str(&f.to_string()),
                SqlLiteral::Boolean(b) => sql.push_str(if *b { "TRUE" } else { "FALSE" }),
                SqlLiteral::Null => sql.push_str("NULL"),
                SqlLiteral::String(s) => {
                    sql.push('\'');
                    sql.push_str(&s.replace('\'', "''"));
                    sql.push('\'');
                }
            },
            SqlExpr::Binary { op, left, right } => {
                sql.push('(');
                self.write_inline_expr(left, sql, depth + 1)?;
                sql.push(' ');
                sql.push_str(op.as_sql());
                sql.push(' ');
                self.write_inline_expr(right, sql, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::FunctionCall { function, argument } => {
                sql.push_str(function.as_sql());
                sql.push('(');
                self.write_inline_expr(argument, sql, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::Alias { expr, alias } => {
                validate_identifier(alias)?;
                self.write_inline_expr(expr, sql, depth + 1)?;
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
        Ok(())
    }

    fn compile_command(&self, node: &str, kind: &str) -> Result<CompiledQuery, AstError> {
        if let Some(sql) = self.compile_expression_form(node, kind)? {
            return Ok(CompiledQuery::raw_compiled(sql, Vec::new()));
        }
        let table = "http://graphlaw.org/sql#targetTable";
        let col = "http://graphlaw.org/sql#assignColumn";
        let val = "http://graphlaw.org/sql#assignValue";
        let wcol = "http://graphlaw.org/sql#whereColumn";
        let wval = "http://graphlaw.org/sql#whereValue";
        let sql = match kind {
            "InsertStatement" => format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.ident(node, table)?,
                self.ident(node, col)?,
                self.lit(node, val)?
            ),
            "UpdateStatement" => {
                let mut sql = format!(
                    "UPDATE {} SET {} = {}",
                    self.ident(node, table)?,
                    self.ident(node, col)?,
                    self.lit(node, val)?
                );
                if self.get_datatype_literal(&node.to_string(), wcol).is_some() {
                    sql.push_str(&format!(
                        " WHERE {} = {}",
                        self.ident(node, wcol)?,
                        self.lit(node, wval)?
                    ));
                }
                sql
            }
            "DeleteStatement" => {
                let t = self.ident(node, table)?;
                if self.get_datatype_literal(&node.to_string(), wcol).is_some() {
                    format!(
                        "DELETE FROM {t} WHERE {} = {}",
                        self.ident(node, wcol)?,
                        self.lit(node, wval)?
                    )
                } else {
                    format!("DELETE FROM {t}")
                }
            }
            "CreateStatement" => format!(
                "CREATE TABLE {} ({} INTEGER)",
                self.ident(node, table)?,
                self.ident(node, col)?
            ),
            "DropStatement" => format!("DROP TABLE IF EXISTS {}", self.ident(node, table)?),
            "AlterStatement" => format!(
                "ALTER TABLE {} ADD COLUMN {} INTEGER",
                self.ident(node, table)?,
                self.ident(node, col)?
            ),
            "TruncateStatement" => format!("TRUNCATE {}", self.ident(node, table)?),
            "AnalyzeStatement" => format!("ANALYZE {}", self.ident(node, table)?),
            "ExplainStatement" => format!(
                "EXPLAIN SELECT {} FROM {}",
                self.ident(node, col)?,
                self.ident(node, table)?
            ),
            "CopyStatement" => {
                let t = self.ident(node, table)?;
                format!("COPY {t} TO '/tmp/graphlaw-rdf-{t}.csv' (FORMAT CSV)")
            }
            "ExportStatement" => {
                let n = self.ident(node, table)?;
                format!("EXPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'")
            }
            "ImportStatement" => {
                let n = self.ident(node, table)?;
                format!(
                    "ATTACH '/tmp/graphlaw-rdf-{n}.duckdb' AS {n}; USE {n}; CREATE OR REPLACE TABLE only_src(id INTEGER); EXPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'; USE memory; DETACH {n}; IMPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'"
                )
            }
            "MergeIntoStatement" => {
                let t = self.ident(node, table)?;
                let c = self.ident(node, col)?;
                let v = self.lit(node, val)?;
                format!(
                    "MERGE INTO {t} USING (SELECT {v} AS {c}) s ON {t}.{c} = s.{c} WHEN MATCHED THEN UPDATE SET {c} = s.{c}"
                )
            }
            "VacuumStatement" => "VACUUM".into(),
            "CheckpointStatement" => "CHECKPOINT".into(),
            "TransactionStatement" => "BEGIN".into(),
            "ExternalResourceStatement" => "SHOW EXTERNAL RESOURCES".into(),
            "UpdateExtensionsStatement" => "UPDATE EXTENSIONS".into(),
            "SetStatement" => format!(
                "SET {} = {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "ResetStatement" => format!("RESET {}", self.ident(node, table)?),
            "PragmaStatement" => format!("PRAGMA {}", self.ident(node, table)?),
            "CallStatement" => format!("CALL {}()", self.ident(node, table)?),
            "PrepareStatement" => format!(
                "PREPARE {} AS SELECT {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "ExecuteStatement" => format!(
                "PREPARE {} AS SELECT {}; EXECUTE {}",
                self.ident(node, table)?,
                self.lit(node, val)?,
                self.ident(node, table)?
            ),
            "DeallocateStatement" => format!(
                "PREPARE {} AS SELECT 1; DEALLOCATE {}",
                self.ident(node, table)?,
                self.ident(node, table)?
            ),
            "CommentStatement" => format!(
                "COMMENT ON TABLE {} IS {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "LoadStatement" => format!("LOAD {}", self.ident(node, table)?),
            "InstallStatement" => format!(
                "INSTALL {} FROM {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "AttachStatement" => format!("ATTACH ':memory:' AS {}", self.ident(node, table)?),
            "UseStatement" => {
                let n = self.ident(node, table)?;
                format!("ATTACH ':memory:' AS {n}; USE {n}")
            }
            "DetachStatement" => {
                let n = self.ident(node, table)?;
                format!("ATTACH ':memory:' AS {n}; DETACH {n}")
            }
            "ExtensionRepositoryStatement" => format!(
                "SET allow_extension_repositories='allowed'; CREATE OR REPLACE EXTENSION REPOSITORY {} WITH PREFIX {} USING PUBLIC KEY {}",
                self.ident(node, table)?,
                self.lit(node, col)?,
                self.lit(node, val)?
            ),
            other => {
                return Err(AstError::UnsupportedTerm(format!(
                    "compile refusal: {other} has no SQL template"
                )));
            }
        };
        Ok(CompiledQuery::raw_compiled(sql, Vec::new()))
    }
}

impl RdfAstGraphReader for DatasetReader {
    type Node = String;

    fn query_document_turtle(&self) -> Option<String> {
        Some(self.turtle.clone())
    }

    fn object_count(&self, node: &String, predicate: &str) -> usize {
        self.outgoing
            .get(node)
            .and_then(|preds| preds.get(predicate))
            .map_or(0, Vec::len)
    }

    fn predicate_iris(&self, node: &String) -> Vec<String> {
        self.outgoing
            .get(node)
            .map(|preds| preds.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn iri_object_count(&self, node: &String, predicate: &str) -> usize {
        self.outgoing
            .get(node)
            .and_then(|preds| preds.get(predicate))
            .map_or(0, |terms| {
                terms
                    .iter()
                    .filter(|t| matches!(t, RdfTerm::Iri(_)))
                    .count()
            })
    }

    fn get_single_object(&self, subject: &Self::Node, predicate: &str) -> Option<Self::Node> {
        self.outgoing
            .get(subject)?
            .get(predicate)?
            .first()
            .and_then(|term| match term {
                RdfTerm::Iri(i) => Some(i.clone()),
                _ => None,
            })
    }

    fn get_datatype_literal(
        &self,
        subject: &Self::Node,
        predicate: &str,
    ) -> Option<(String, Option<String>)> {
        self.outgoing
            .get(subject)?
            .get(predicate)?
            .first()
            .and_then(|term| match term {
                RdfTerm::Literal { lexical, datatype } => Some((lexical.clone(), datatype.clone())),
                _ => None,
            })
    }

    fn get_iri(&self, node: &Self::Node) -> Option<String> {
        Some(node.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifier_compilation() {
        let stmt = SelectStatement {
            from: TableRef {
                name: "telemetry_events".to_string(),
                alias: Some("t".to_string()),
                schema: None,
            },
            joins: vec![],
            select_list: vec![
                SqlExpr::ColumnRef {
                    column: "batch_id".to_string(),
                    table_alias: None,
                },
                SqlExpr::Alias {
                    expr: Box::new(SqlExpr::FunctionCall {
                        function: AggregateFunction::Sum,
                        argument: Box::new(SqlExpr::ColumnRef {
                            column: "cost".to_string(),
                            table_alias: None,
                        }),
                    }),
                    alias: "total_cost".to_string(),
                },
            ],
            where_clause: Some(SqlExpr::Binary {
                op: Operator::GreaterThan,
                left: Box::new(SqlExpr::ColumnRef {
                    column: "latency".to_string(),
                    table_alias: None,
                }),
                right: Box::new(SqlExpr::Literal(SqlLiteral::Integer(500))),
            }),
            group_by: vec![SqlExpr::ColumnRef {
                column: "batch_id".to_string(),
                table_alias: None,
            }],
            having: None,
            order_by: Vec::new(),
            limit: Some(100),
            offset: None,
        };

        let compiled = stmt.compile().unwrap();
        assert_eq!(
            compiled.sql,
            "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry_events t WHERE (latency > $1) GROUP BY batch_id LIMIT 100"
        );
        assert_eq!(compiled.params, vec![SqlLiteral::Integer(500)]);
    }

    #[test]
    fn test_injection_attempt_rejected() {
        let stmt = SelectStatement {
            from: TableRef {
                name: "events; DROP TABLE users; --".to_string(),
                alias: None,
                schema: None,
            },
            joins: vec![],
            select_list: vec![],
            where_clause: None,
            group_by: vec![],
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let result = stmt.compile();
        assert!(matches!(result, Err(e) if matches!(e.code, RefusalCode::InvalidIdentifier)));
    }
}

#[cfg(test)]
mod r1_canon_once_tests {
    use super::*;
    fn count() -> usize {
        CANONICALIZATION_COUNT.get()
    }

    /// A minimal valid section-5 SELECT graph (real compile, real purrdf
    /// canonicalization — Chicago, no mocks). Single test owns the global
    /// CANONICALIZATION_COUNT to avoid cross-test parallel races.
    fn select_graph() -> String {
        "@prefix : <http://example.org/> .\n@prefix sql: <http://graphlaw.org/sql#> .\n:g a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"t\" ] ;\n    sql:selectList ( [ sql:columnName \"v\" ] ) .".to_string()
    }

    /// R1 non-vacuity: compile_turtle canonicalizes exactly ONCE per compile,
    /// carries the bytes, and the carried bytes are bit-identical to an
    /// independent RDFC-1.0 canonicalization.
    #[test]
    fn canon_happens_once_per_compile_and_bytes_are_bit_identical() {
        let before = count();
        let compiled = compile_turtle(&select_graph()).expect("graph must compile");
        let after_compile = count();
        assert_eq!(
            after_compile,
            before + 1,
            "canonicalization must run exactly once per compile_turtle"
        );
        assert!(
            compiled.canon_graph_bytes.is_some(),
            "compile_turtle must carry Canon(G_AST) bytes"
        );

        // Bit-identity: carried canon bytes equal an independent RDFC-1.0
        // canonical form computed outside the compile path.
        let dataset =
            purrdf::parse_dataset(select_graph().as_bytes(), "text/turtle", None).expect("parse");
        let independent = purrdf::try_canonicalize(&dataset)
            .expect("canonicalize")
            .nquads;
        assert_eq!(
            compiled.canon_graph_bytes.as_deref(),
            Some(independent.as_bytes()),
            "carried Canon(G_AST) bytes must be bit-identical to an independent canonicalization"
        );

        // A second compile canonicalizes again (once per compile, no global
        // memoization) — the counter is a true counter.
        compile_turtle(&select_graph()).expect("compile 2");
        compile_turtle(&select_graph()).expect("compile 3");
        assert_eq!(count(), after_compile + 2);
    }
}

// ===========================================================================
// Lane L4 (v26.10.22) courts: typed literal refusals (LITERAL_NOT_INT /
// LITERAL_NOT_FLOAT), join-type refusal/Default pinning, and the RUST-5
// CompiledQuery admission seal. Chicago: real compile_turtle compiles over
// real Turtle graphs, no mocks, no sleeps. Anti-vacuity is INLINED: every
// court carries its own mutation twin (the mutated form asserts the flipped
// outcome), so deleting the refusal makes the twin fail.
// ===========================================================================
#[cfg(test)]
mod literal_join_seal_courts {
    use super::*;

    const SELECT_HEAD: &str = concat!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n",
        "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n",
        "@prefix ex: <urn:lit:q#> .\n",
        "ex:Q a sql:SelectStatement ;\n",
        "    sql:statementIndex 1 ;\n",
        "    sql:fromTable [ sql:tableName \"t\" ] ;\n",
        "    sql:selectList ( [ sql:columnName \"v\" ] ) ;\n",
        "    sql:whereClause [ sql:operator sql:GreaterThan ;\n",
        "        sql:leftOperand [ sql:columnName \"n\" ] ;\n",
    );

    fn lit_graph(lexical: &str, dtype: &str) -> String {
        format!(
            "{}        sql:rightOperand [ sql:literalValue \"{}\"^^{} ] ] .",
            SELECT_HEAD, lexical, dtype
        )
    }

    /// LITERAL_NOT_INT (RefusalCode = 36): an xsd:integer-typed
    /// sql:literalValue that does not parse refuses typed — it is never
    /// silently coerced to 0 (the legacy `unwrap_or(0)` behavior).
    #[test]
    fn literal_not_int_refuses_typed() {
        let err = compile_turtle(&lit_graph("not-a-number", "xsd:integer"))
            .expect_err("non-integer lexical must refuse");
        assert_eq!(err.code, RefusalCode::LiteralNotInt);
        let msg = err.to_string();
        assert!(
            msg.contains(REFUSAL_01_MALFORMED_AST) && msg.contains("not-a-number"),
            "message carries the REFUSAL-01 marker and the offending lexical: {msg}"
        );

        // ANTI-VACUITY MUTATION TWIN: the same graph with a VALID integer
        // lexical compiles — the court flips exactly on the refusal path.
        let ok = compile_turtle(&lit_graph("42", "xsd:integer")).expect("valid integer compiles");
        assert_eq!(ok.params, vec![SqlLiteral::Integer(42)]);
    }

    /// LITERAL_NOT_FLOAT (RefusalCode = 37): an xsd:double-typed
    /// sql:literalValue that does not parse refuses typed (never 0.0).
    #[test]
    fn literal_not_float_refuses_typed() {
        let err = compile_turtle(&lit_graph("1.2.3 garbage", "xsd:double"))
            .expect_err("non-double lexical must refuse");
        assert_eq!(err.code, RefusalCode::LiteralNotFloat);
        let msg = err.to_string();
        assert!(
            msg.contains(REFUSAL_01_MALFORMED_AST) && msg.contains("1.2.3 garbage"),
            "message carries the REFUSAL-01 marker and the offending lexical: {msg}"
        );

        // ANTI-VACUITY MUTATION TWIN: valid double compiles to a Float param.
        let ok = compile_turtle(&lit_graph("2.5", "xsd:double")).expect("valid double compiles");
        assert_eq!(ok.params, vec![SqlLiteral::Float(2.5)]);
    }

    fn join_graph(join_type: Option<&str>) -> String {
        let type_clause = match join_type {
            Some(iri) => format!("    sql:joinType <{}> ;\n", iri),
            None => String::new(),
        };
        format!(
            concat!(
                "@prefix sql: <http://graphlaw.org/sql#> .\n",
                "@prefix ex: <urn:join:q#> .\n",
                "ex:Q a sql:SelectStatement ;\n",
                "    sql:statementIndex 1 ;\n",
                "    sql:fromTable [ sql:tableName \"a\" ] ;\n",
                "    sql:selectList ( [ sql:columnName \"v\" ] ) ;\n",
                "    sql:joinClause [\n",
                "{type_clause}",
                "        sql:joinTable [ sql:tableName \"b\" ] ;\n",
                "        sql:joinCondition [ sql:operator sql:Equals ;\n",
                "            sql:leftOperand [ sql:columnName \"id\" ] ;\n",
                "            sql:rightOperand [ sql:columnName \"a_id\" ] ] ] ."
            ),
            type_clause = type_clause
        )
    }

    /// Unknown join-type IRI refuses typed (REFUSAL-01, arm JOIN_TYPE_IRI).
    #[test]
    fn unknown_join_type_iri_refuses_typed() {
        let err = compile_turtle(&join_graph(Some("http://graphlaw.org/sql#FullJoin")))
            .expect_err("unknown join-type IRI must refuse");
        assert_eq!(err.code, RefusalCode::MalformedAst);
        let msg = err.to_string();
        assert!(
            msg.contains("sql:joinType IRI") && msg.contains("FullJoin"),
            "message names the offending IRI: {msg}"
        );

        // ANTI-VACUITY MUTATION TWIN: the known join-type IRIs compile and pin
        // their exact SQL rendering — the court flips on the refusal path.
        let inner = compile_turtle(&join_graph(Some("http://graphlaw.org/sql#InnerJoin")))
            .expect("InnerJoin compiles");
        assert!(
            inner.sql.contains(" JOIN "),
            "InnerJoin renders JOIN: {}",
            inner.sql
        );
        let left = compile_turtle(&join_graph(Some("http://graphlaw.org/sql#LeftJoin")))
            .expect("LeftJoin compiles");
        assert!(
            left.sql.contains(" LEFT JOIN "),
            "LeftJoin renders LEFT JOIN: {}",
            left.sql
        );
    }

    /// MISSING sql:joinType stays Inner (pinning court for the default).
    #[test]
    fn missing_join_type_defaults_to_inner() {
        let compiled = compile_turtle(&join_graph(None)).expect("missing joinType compiles");
        assert!(
            compiled.sql.contains(" JOIN ") && !compiled.sql.contains(" LEFT JOIN "),
            "missing joinType must default to Inner, got: {}",
            compiled.sql
        );
    }

    /// RUST-5 admission seal: compile_turtle products carry a valid seal;
    /// mutating any sealed field (sql, cacheable, canon bytes, receipt
    /// anchor) invalidates `verify_seal`. Params are unsealed (lawful
    /// re-binding).
    #[test]
    fn admission_seal_valid_on_compile_and_flips_on_mutation() {
        let compiled = compile_turtle(&lit_graph("42", "xsd:integer")).expect("compile");
        assert!(
            compiled.verify_seal(),
            "fresh compile_turtle product verifies"
        );
        assert_eq!(compiled.admission_seal, compiled.seal_bytes());

        // Mutation twin 1: sql text mutation flips the seal.
        let mut m = compiled.clone();
        m.sql = m.sql.replace("SELECT", "SELECT ALL");
        assert!(!m.verify_seal(), "sql mutation must flip the seal");

        // Mutation twin 2: canon-graph mutation flips the seal.
        let mut m2 = compiled.clone();
        if let Some(c) = m2.canon_graph_bytes.as_mut() {
            c[0] ^= 0xFF;
        }
        assert!(!m2.verify_seal(), "canon bytes mutation must flip the seal");

        // Mutation twin 3: receipt_state mutation flips the seal.
        let mut m3 = compiled.clone();
        m3.receipt_state = Some([7u8; 32]);
        assert!(
            !m3.verify_seal(),
            "receipt_state mutation must flip the seal"
        );

        // Mutation twin 4: reseal() restores validity after a mutation
        // (the seal is a function of state, and reseal is the only writer).
        let mut m4 = m.clone();
        m4.reseal();
        assert!(
            m4.verify_seal(),
            "reseal over the mutated state re-validates"
        );

        // Params are unsealed: mutating params keeps the seal valid.
        let mut p = compiled.clone();
        p.params.push(SqlLiteral::Integer(1));
        assert!(p.verify_seal(), "param re-binding is lawful and unsealed");
    }
}
