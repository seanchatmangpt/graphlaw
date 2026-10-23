use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::validate_ast_shacl;
use std::fs;

#[test]
fn test_cold_chain_biologics_verification_flow() {
    let base_dir = format!(
        "/tmp/test_ocel_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let _ = fs::remove_dir_all(&base_dir);
    fs::create_dir_all(&base_dir).unwrap();

    let db = GraphLawDuckDB::open_in_memory().unwrap();

    // 1. Generate events table (partitioned by facility_id)
    let gen_events = format!(
        "COPY (
            SELECT 'FAC-US-01' AS facility_id, 'EVT-101' AS event_id, -18.2 AS temp_celsius, 140 AS duration_seconds
            UNION ALL
            SELECT 'FAC-US-01' AS facility_id, 'EVT-201' AS event_id, -12.4 AS temp_celsius, 450 AS duration_seconds
        ) TO '{}/events' (FORMAT PARQUET, PARTITION_BY (facility_id));",
        base_dir
    );
    db.raw_connection().execute_batch(&gen_events).unwrap();

    let gen_rel = format!(
        "COPY (
            SELECT 'EVT-101' AS event_id, 'BATCH-MAB-9042' AS object_id, 'contains_product' AS qualifier
            UNION ALL
            SELECT 'EVT-201' AS event_id, 'BATCH-MAB-9043' AS object_id, 'contains_product' AS qualifier
        ) TO '{}/event_object_map.parquet' (FORMAT PARQUET);",
        base_dir
    );
    db.raw_connection().execute_batch(&gen_rel).unwrap();

    // 2. Mount virtual zero-copy views
    db.register_parquet_view(
        "ocel_events",
        &format!("{}/events/**/*.parquet", base_dir),
        true,
    )
    .unwrap();
    db.register_parquet_view(
        "ocel_event_object_rel",
        &format!("{}/event_object_map.parquet", base_dir),
        false,
    )
    .unwrap();

    // 3. SHACL validation of AST
    let shapes_ttl = include_str!("../ontologies/sql-ast.ttl");
    let query_ast_ttl = r#"
        @prefix sql:  <http://graphlaw.org/sql#> .
        @prefix fn:   <http://graphlaw.org/functions#> .
        @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
        @prefix ex:   <http://biopharma.org/queries#> .

        ex:ColdChainExcursionQuery a sql:SelectStatement ;
            sql:statementIndex 1 ;
            sql:fromTable [
                sql:tableName "ocel_events" ;
                sql:tableAlias "e"
            ] ;
            sql:joinClause [
                sql:joinType sql:InnerJoin ;
                sql:joinTable [
                    sql:tableName "ocel_event_object_rel" ;
                    sql:tableAlias "r"
                ] ;
                sql:joinCondition [
                    sql:operator sql:Equals ;
                    sql:leftOperand  [ sql:columnName "event_id" ; sql:tableAlias "e" ] ;
                    sql:rightOperand [ sql:columnName "event_id" ; sql:tableAlias "r" ]
                ]
            ] ;
            sql:selectList (
                [ sql:columnName "object_id" ; sql:tableAlias "r" ]
                [
                    sql:aliasName "max_excursion_temp" ;
                    sql:expression [
                        sql:function fn:MAX ;
                        sql:argument [ sql:columnName "temp_celsius" ; sql:tableAlias "e" ]
                    ]
                ]
                [
                    sql:aliasName "total_excursion_duration" ;
                    sql:expression [
                        sql:function fn:SUM ;
                        sql:argument [ sql:columnName "duration_seconds" ; sql:tableAlias "e" ]
                    ]
                ]
            ) ;
            sql:whereClause [
                sql:operator sql:And ;
                sql:leftOperand [
                    sql:operator sql:GreaterThan ;
                    sql:leftOperand  [ sql:columnName "temp_celsius" ; sql:tableAlias "e" ] ;
                    sql:rightOperand [ sql:literalValue "-20.0"^^xsd:double ]
                ] ;
                sql:rightOperand [
                    sql:operator sql:Equals ;
                    sql:leftOperand  [ sql:columnName "qualifier" ; sql:tableAlias "r" ] ;
                    sql:rightOperand [ sql:literalValue "contains_product"^^xsd:string ]
                ]
            ] ;
            sql:groupBy (
                [ sql:columnName "object_id" ; sql:tableAlias "r" ]
            ) .
    "#;

    validate_ast_shacl(query_ast_ttl, shapes_ttl).expect("SHACL validation must pass");

    // 4. Compile and evaluate (SQLRDF-17: admission is compile_turtle —
    // a CompiledQuery with no canon bytes is EvaluatorError::NoCanonicalGraph at eval).
    let compiled =
        graphlaw::sql_ast::compile_turtle(query_ast_ttl).expect("admitted AST graph must compile");
    let query_node = "http://biopharma.org/queries#ColdChainExcursionQuery".to_string();

    let receipt = db
        .eval_ast_query(
            &query_node,
            &compiled,
            "http://biopharma.org/data",
            "http://biopharma.org/vocab",
        )
        .unwrap();

    assert_eq!(receipt.rows_emitted, 2);
    assert_ne!(receipt.query_hash, [0u8; 32]);

    let _ = fs::remove_dir_all(&base_dir);
}
