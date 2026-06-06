use super::{RequestLog, RequestTokenStat, Storage, UsageSnapshotRecord};

#[test]
fn observability_prepare_migration_does_not_prune_or_vacuum_legacy_rows() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    let log = RequestLog {
        trace_id: Some("trace-observability-migration".to_string()),
        key_id: Some("key-observability-migration".to_string()),
        account_id: Some("acc-observability-migration".to_string()),
        initial_account_id: Some("acc-observability-migration".to_string()),
        attempted_account_ids_json: Some("[\"acc-observability-migration\"]".to_string()),
        initial_aggregate_api_id: None,
        attempted_aggregate_api_ids_json: None,
        request_path: "/v1/responses".to_string(),
        original_path: Some("/v1/responses".to_string()),
        adapted_path: Some("/v1/responses".to_string()),
        method: "POST".to_string(),
        request_type: Some("http".to_string()),
        gateway_mode: None,
        transparent_mode: None,
        enhanced_mode: None,
        model: Some("gpt-5".to_string()),
        upstream_model: Some("gpt-5".to_string()),
        actual_source_kind: Some("openai_account".to_string()),
        actual_source_id: Some("acc-observability-migration".to_string()),
        reasoning_effort: None,
        service_tier: None,
        effective_service_tier: None,
        response_adapter: None,
        upstream_url: None,
        aggregate_api_supplier_name: None,
        aggregate_api_url: None,
        status_code: Some(200),
        duration_ms: Some(10),
        first_response_ms: Some(5),
        input_tokens: None,
        cached_input_tokens: None,
        output_tokens: None,
        total_tokens: None,
        reasoning_output_tokens: None,
        estimated_cost_usd: None,
        error: None,
        created_at: 1,
    };
    let request_log_id = storage.insert_request_log(&log).expect("insert log");
    storage
        .insert_request_token_stat(&RequestTokenStat {
            request_log_id,
            key_id: Some("key-observability-migration".to_string()),
            account_id: Some("acc-observability-migration".to_string()),
            model: Some("gpt-5".to_string()),
            input_tokens: Some(100),
            cached_input_tokens: Some(10),
            output_tokens: Some(20),
            total_tokens: Some(110),
            reasoning_output_tokens: Some(5),
            estimated_cost_usd: Some(0.5),
            created_at: 1,
        })
        .expect("insert token stat");
    storage
        .insert_usage_snapshot(&UsageSnapshotRecord {
            account_id: "acc-observability-migration".to_string(),
            used_percent: Some(10.0),
            window_minutes: Some(300),
            resets_at: None,
            credits_json: None,
            captured_at: 1,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
        })
        .expect("insert usage snapshot");

    storage
        .prepare_observability_storage_for_existing_databases()
        .expect("prepare observability storage");

    let request_log_count: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request logs");
    assert_eq!(request_log_count, 1);
    assert_eq!(
        storage
            .usage_snapshot_count()
            .expect("count usage snapshots"),
        1
    );
    let token_stats_count: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_token_stats", [], |row| {
            row.get(0)
        })
        .expect("count token stats");
    assert_eq!(token_stats_count, 1);
    let rollup_count: i64 = storage
        .conn
        .query_row(
            "SELECT COUNT(1) FROM request_token_stat_rollups",
            [],
            |row| row.get(0),
        )
        .expect("count rollups");
    assert_eq!(rollup_count, 0);
}
