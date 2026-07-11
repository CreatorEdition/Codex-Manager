use super::{RequestLog, RequestTokenStat, Storage};
use crate::storage::{ApiKey, ApiKeyOwner, AppUser};
use std::sync::{Mutex, OnceLock};

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

fn test_api_key(id: &str, created_at: i64) -> ApiKey {
    ApiKey {
        id: id.to_string(),
        name: Some(id.to_string()),
        model_slug: Some("gpt-5".to_string()),
        reasoning_effort: None,
        service_tier: None,
        rotation_strategy: "account_rotation".to_string(),
        aggregate_api_id: None,
        account_plan_filter: None,
        aggregate_api_url: None,
        client_type: "codex".to_string(),
        protocol_type: "openai_compat".to_string(),
        auth_scheme: "authorization_bearer".to_string(),
        upstream_base_url: None,
        static_headers_json: None,
        key_hash: format!("hash-{id}"),
        status: "active".to_string(),
        created_at,
        last_used_at: None,
    }
}
/// 函数 `collect_query_plan_details`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - storage: 参数 storage
/// - sql: 参数 sql
///
/// # 返回
/// 返回函数执行结果
fn collect_query_plan_details(storage: &Storage, sql: &str) -> Vec<String> {
    let mut stmt = storage.conn.prepare(sql).expect("prepare explain");
    let mut rows = stmt.query([]).expect("query explain");
    let mut details = Vec::new();
    while let Some(row) = rows.next().expect("next explain row") {
        let detail: String = row.get(3).expect("detail");
        details.push(detail.to_ascii_lowercase());
    }
    details
}

/// 函数 `method_exact_query_matches_composite_index`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn method_exact_query_matches_composite_index() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let details = collect_query_plan_details(
        &storage,
        "EXPLAIN QUERY PLAN
         SELECT key_id, account_id, request_path, method, model, reasoning_effort, upstream_url, status_code, error, created_at
         FROM request_logs
         WHERE method = 'POST'
         ORDER BY created_at DESC, id DESC
         LIMIT 100",
    );
    assert!(details
        .iter()
        .any(|detail| detail.contains("idx_request_logs_method_created_at")));
}

/// 函数 `key_exact_query_matches_composite_index`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn key_exact_query_matches_composite_index() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let details = collect_query_plan_details(
        &storage,
        "EXPLAIN QUERY PLAN
         SELECT key_id, account_id, request_path, method, model, reasoning_effort, upstream_url, status_code, error, created_at
         FROM request_logs
         WHERE key_id = 'gk_1'
         ORDER BY created_at DESC, id DESC
         LIMIT 100",
    );
    assert!(details
        .iter()
        .any(|detail| detail.contains("idx_request_logs_key_id_created_at")));
}

#[test]
fn key_in_query_matches_composite_index() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let details = collect_query_plan_details(
        &storage,
        "EXPLAIN QUERY PLAN
         SELECT key_id, account_id, request_path, method, model, reasoning_effort, upstream_url, status_code, error, created_at
         FROM request_logs
         WHERE key_id IN ('gk_1', 'gk_2')
         ORDER BY created_at DESC, id DESC
         LIMIT 100",
    );
    assert!(details
        .iter()
        .any(|detail| detail.contains("idx_request_logs_key_id_created_at")));
}

/// 函数 `insert_request_log_with_token_stat_is_visible_via_join`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn insert_request_log_with_token_stat_is_visible_via_join() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    let created_at = 123456_i64;
    let log = RequestLog {
        trace_id: Some("trc-1".to_string()),
        key_id: Some("gk_1".to_string()),
        account_id: Some("acc_1".to_string()),
        initial_account_id: Some("acc_1".to_string()),
        attempted_account_ids_json: Some(r#"["acc_1"]"#.to_string()),
        request_path: "/v1/responses".to_string(),
        original_path: Some("/v1/chat/completions".to_string()),
        adapted_path: Some("/v1/responses".to_string()),
        method: "POST".to_string(),
        request_type: Some("http".to_string()),
        route_strategy: Some("balanced".to_string()),
        route_source: Some("conversation_bound".to_string()),
        client_model: Some("gpt-5-client".to_string()),
        model: Some("gpt-5".to_string()),
        model_source: Some("gateway_override".to_string()),
        upstream_model: Some("gpt-provider-5".to_string()),
        actual_source_kind: Some("openai_account".to_string()),
        actual_source_id: Some("acc_1".to_string()),
        client_reasoning_effort: Some("low".to_string()),
        reasoning_effort: Some("medium".to_string()),
        reasoning_source: Some("api_key_profile".to_string()),
        service_tier: Some("fast".to_string()),
        effective_service_tier: Some("priority".to_string()),
        service_tier_source: Some("gateway_override".to_string()),
        response_adapter: Some("OpenAIChatCompletionsJson".to_string()),
        upstream_url: Some("https://example.test".to_string()),
        aggregate_api_supplier_name: None,
        aggregate_api_url: None,
        status_code: Some(200),
        duration_ms: Some(1234),
        first_response_ms: Some(456),
        input_tokens: None,
        cached_input_tokens: None,
        output_tokens: None,
        total_tokens: None,
        reasoning_output_tokens: None,
        estimated_cost_usd: None,
        error: None,
        created_at,
        ..Default::default()
    };

    let stat = RequestTokenStat {
        request_log_id: 0,
        key_id: log.key_id.clone(),
        account_id: log.account_id.clone(),
        model: log.model.clone(),
        input_tokens: Some(10),
        cached_input_tokens: Some(1),
        output_tokens: Some(2),
        total_tokens: Some(12),
        reasoning_output_tokens: Some(3),
        estimated_cost_usd: Some(0.123),
        created_at,
    };

    let (_request_log_id, token_err) = storage
        .insert_request_log_with_token_stat(&log, &stat)
        .expect("insert request log with token stat");
    assert!(token_err.is_none(), "token stat should insert");

    let logs = storage
        .list_request_logs(None, 10)
        .expect("list request logs");
    assert_eq!(logs.len(), 1);
    let row = &logs[0];
    assert_eq!(row.trace_id.as_deref(), Some("trc-1"));
    assert_eq!(row.initial_account_id.as_deref(), Some("acc_1"));
    assert_eq!(
        row.attempted_account_ids_json.as_deref(),
        Some(r#"["acc_1"]"#)
    );
    assert_eq!(row.request_path, log.request_path);
    assert_eq!(row.original_path.as_deref(), Some("/v1/chat/completions"));
    assert_eq!(row.adapted_path.as_deref(), Some("/v1/responses"));
    assert_eq!(row.request_type.as_deref(), Some("http"));
    assert_eq!(row.route_strategy.as_deref(), Some("balanced"));
    assert_eq!(row.route_source.as_deref(), Some("conversation_bound"));
    assert_eq!(row.client_model.as_deref(), Some("gpt-5-client"));
    assert_eq!(row.model.as_deref(), Some("gpt-5"));
    assert_eq!(row.model_source.as_deref(), Some("gateway_override"));
    assert_eq!(row.upstream_model.as_deref(), Some("gpt-provider-5"));
    assert_eq!(row.actual_source_kind.as_deref(), Some("openai_account"));
    assert_eq!(row.actual_source_id.as_deref(), Some("acc_1"));
    assert_eq!(row.client_reasoning_effort.as_deref(), Some("low"));
    assert_eq!(row.reasoning_effort.as_deref(), Some("medium"));
    assert_eq!(row.reasoning_source.as_deref(), Some("api_key_profile"));
    assert_eq!(row.service_tier.as_deref(), Some("fast"));
    assert_eq!(row.effective_service_tier.as_deref(), Some("priority"));
    assert_eq!(row.service_tier_source.as_deref(), Some("gateway_override"));
    assert_eq!(row.first_response_ms, Some(456));
    assert_eq!(
        row.response_adapter.as_deref(),
        Some("OpenAIChatCompletionsJson")
    );
    assert_eq!(row.input_tokens, Some(10));
    assert_eq!(row.cached_input_tokens, Some(1));
    assert_eq!(row.output_tokens, Some(2));
    assert_eq!(row.total_tokens, Some(12));
    assert_eq!(row.reasoning_output_tokens, Some(3));
    assert_eq!(row.estimated_cost_usd, Some(0.123));
}

#[test]
fn insert_request_log_with_empty_token_stat_skips_token_stats_row() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let created_at = 84_i64;
    let log = RequestLog {
        trace_id: Some("trc-empty-stat".to_string()),
        key_id: Some("gk_empty".to_string()),
        account_id: Some("acc_empty".to_string()),
        request_path: "/v1/responses".to_string(),
        method: "POST".to_string(),
        model: Some("gpt-5".to_string()),
        status_code: Some(502),
        error: Some("upstream error".to_string()),
        created_at,
        ..Default::default()
    };
    let stat = RequestTokenStat {
        request_log_id: 0,
        key_id: log.key_id.clone(),
        account_id: log.account_id.clone(),
        model: log.model.clone(),
        input_tokens: Some(0),
        cached_input_tokens: Some(0),
        output_tokens: Some(0),
        total_tokens: Some(0),
        reasoning_output_tokens: Some(0),
        estimated_cost_usd: Some(0.0),
        created_at,
    };

    let (_request_log_id, token_err) = storage
        .insert_request_log_with_token_stat(&log, &stat)
        .expect("insert request log with empty token stat");
    assert!(token_err.is_none(), "empty token stat should be skipped");

    let logs = storage
        .list_request_logs(None, 10)
        .expect("list request logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].trace_id.as_deref(), Some("trc-empty-stat"));
    assert_eq!(logs[0].total_tokens, None);
    let summary = storage
        .summarize_request_token_stats_by_key_ids(&["gk_empty".to_string()])
        .expect("summarize empty key");
    assert!(summary.is_empty());
}

/// 函数 `token_stat_failure_still_commits_request_log`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn token_stat_failure_still_commits_request_log() {
    let storage = Storage::open_in_memory().expect("open");
    // Only create request_logs table, so request_token_stats insert fails.
    storage
        .ensure_request_logs_table()
        .expect("ensure logs table");

    let created_at = 42_i64;
    let log = RequestLog {
        trace_id: Some("trc-2".to_string()),
        key_id: Some("gk_1".to_string()),
        account_id: Some("acc_1".to_string()),
        initial_account_id: Some("acc_1".to_string()),
        attempted_account_ids_json: Some(r#"["acc_1"]"#.to_string()),
        request_path: "/v1/responses".to_string(),
        original_path: Some("/v1/responses".to_string()),
        adapted_path: Some("/v1/responses".to_string()),
        method: "POST".to_string(),
        model: Some("gpt-5".to_string()),
        reasoning_effort: None,
        response_adapter: Some("Passthrough".to_string()),
        upstream_url: None,
        aggregate_api_supplier_name: None,
        aggregate_api_url: None,
        status_code: Some(200),
        duration_ms: None,
        first_response_ms: None,
        input_tokens: None,
        cached_input_tokens: None,
        output_tokens: None,
        total_tokens: None,
        reasoning_output_tokens: None,
        estimated_cost_usd: None,
        error: None,
        created_at,
        ..Default::default()
    };

    let stat = RequestTokenStat {
        request_log_id: 0,
        key_id: log.key_id.clone(),
        account_id: log.account_id.clone(),
        model: log.model.clone(),
        input_tokens: Some(1),
        cached_input_tokens: None,
        output_tokens: None,
        total_tokens: None,
        reasoning_output_tokens: None,
        estimated_cost_usd: None,
        created_at,
    };

    let (_request_log_id, token_err) = storage
        .insert_request_log_with_token_stat(&log, &stat)
        .expect("insert request log with token stat");
    assert!(token_err.is_some(), "token stat insert should fail");

    let count: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request_logs");
    assert_eq!(count, 1);
}

/// 函数 `request_logs_support_backend_pagination_and_status_filters`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn request_logs_support_backend_pagination_and_status_filters() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    for index in 0..5_i64 {
        let created_at = 1_000 + index;
        let status_code = match index {
            0 | 1 => Some(200),
            2 => Some(404),
            _ => Some(502),
        };
        let error = if status_code.unwrap_or_default() >= 500 {
            Some("upstream interrupted".to_string())
        } else {
            None
        };
        let request_log_id = storage
            .insert_request_log(&RequestLog {
                trace_id: Some(format!("trc-{index}")),
                key_id: Some("gk-log".to_string()),
                account_id: Some("acc-log".to_string()),
                initial_account_id: Some("acc-log".to_string()),
                attempted_account_ids_json: Some(r#"["acc-log"]"#.to_string()),
                request_path: format!("/v1/responses/{index}"),
                original_path: Some("/v1/responses".to_string()),
                adapted_path: Some("/v1/responses".to_string()),
                method: "POST".to_string(),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some("high".to_string()),
                response_adapter: Some("Passthrough".to_string()),
                upstream_url: Some("https://chatgpt.com/backend-api/codex/responses".to_string()),
                aggregate_api_supplier_name: None,
                aggregate_api_url: None,
                status_code,
                duration_ms: Some(200 + index),
                first_response_ms: None,
                input_tokens: None,
                cached_input_tokens: None,
                output_tokens: None,
                total_tokens: None,
                reasoning_output_tokens: None,
                estimated_cost_usd: None,
                error,
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some("gk-log".to_string()),
                account_id: Some("acc-log".to_string()),
                model: Some("gpt-5".to_string()),
                input_tokens: Some(10 + index),
                cached_input_tokens: Some(1),
                output_tokens: Some(2),
                total_tokens: Some(20 + index),
                reasoning_output_tokens: Some(0),
                estimated_cost_usd: Some(0.01),
                created_at,
            })
            .expect("insert token stat");
    }

    let page = storage
        .list_request_logs_paginated(None, Some("5xx"), None, None, 0, 1)
        .expect("list paginated logs");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].trace_id.as_deref(), Some("trc-4"));

    let total_5xx = storage
        .count_request_logs(None, Some("5xx"), None, None)
        .expect("count 5xx logs");
    assert_eq!(total_5xx, 2);
}

/// 函数 `request_logs_filtered_summary_aggregates_counts_and_tokens`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// 无
///
/// # 返回
/// 无
#[test]
fn request_logs_filtered_summary_aggregates_counts_and_tokens() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    for (index, status_code, total_tokens, error) in [
        (0_i64, Some(200_i64), Some(30_i64), None),
        (1_i64, Some(200_i64), Some(50_i64), None),
        (2_i64, Some(502_i64), Some(70_i64), Some("upstream error")),
    ] {
        let created_at = 2_000 + index;
        let request_log_id = storage
            .insert_request_log(&RequestLog {
                trace_id: Some(format!("trc-sum-{index}")),
                key_id: Some("gk-sum".to_string()),
                account_id: Some("acc-sum".to_string()),
                initial_account_id: Some("acc-sum".to_string()),
                attempted_account_ids_json: Some(r#"["acc-sum"]"#.to_string()),
                request_path: "/v1/responses".to_string(),
                original_path: Some("/v1/responses".to_string()),
                adapted_path: Some("/v1/responses".to_string()),
                method: "POST".to_string(),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some("medium".to_string()),
                response_adapter: Some("Passthrough".to_string()),
                upstream_url: Some("https://chatgpt.com/backend-api/codex/responses".to_string()),
                aggregate_api_supplier_name: None,
                aggregate_api_url: None,
                status_code,
                duration_ms: Some(900),
                first_response_ms: None,
                input_tokens: None,
                cached_input_tokens: None,
                output_tokens: None,
                total_tokens: None,
                reasoning_output_tokens: None,
                estimated_cost_usd: None,
                error: error.map(|value| value.to_string()),
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some("gk-sum".to_string()),
                account_id: Some("acc-sum".to_string()),
                model: Some("gpt-5".to_string()),
                input_tokens: None,
                cached_input_tokens: None,
                output_tokens: None,
                total_tokens,
                reasoning_output_tokens: Some(0),
                estimated_cost_usd: Some(0.01),
                created_at,
            })
            .expect("insert token stat");
    }

    let summary = storage
        .summarize_request_logs_filtered(None, Some("all"), None, None)
        .expect("summarize filtered logs");
    assert_eq!(summary.count, 3);
    assert_eq!(summary.success_count, 2);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.total_tokens, 150);
    assert_eq!(summary.estimated_cost_usd, 0.03);
}

#[test]
fn request_logs_support_time_range_filters() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    for (index, created_at) in [1_000_i64, 1_900_i64, 3_100_i64].into_iter().enumerate() {
        let request_log_id = storage
            .insert_request_log(&RequestLog {
                trace_id: Some(format!("trc-time-{index}")),
                key_id: Some("gk-time".to_string()),
                account_id: Some("acc-time".to_string()),
                request_path: "/v1/responses".to_string(),
                method: "POST".to_string(),
                status_code: Some(200),
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some("gk-time".to_string()),
                account_id: Some("acc-time".to_string()),
                model: Some("gpt-5".to_string()),
                total_tokens: Some(10),
                estimated_cost_usd: Some(0.01),
                created_at,
                ..Default::default()
            })
            .expect("insert token stat");
    }

    let page = storage
        .list_request_logs_paginated(None, None, Some(1_500), Some(3_000), 0, 10)
        .expect("list paginated logs");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].trace_id.as_deref(), Some("trc-time-1"));

    let total = storage
        .count_request_logs(None, None, Some(1_500), Some(3_000))
        .expect("count logs");
    assert_eq!(total, 1);

    let summary = storage
        .summarize_request_logs_filtered(None, None, Some(900), Some(2_000))
        .expect("summarize time range");
    assert_eq!(summary.count, 2);
    assert_eq!(summary.total_tokens, 20);
}

#[test]
fn request_token_stats_key_id_summaries_merge_rollups_and_filter_keys() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    for (request_log_id, key_id, model, total_tokens, cost) in [
        (1_i64, "gk-a", "gpt-5", 30_i64, 0.30_f64),
        (2_i64, "gk-b", "gpt-5", 50_i64, 0.50_f64),
        (3_i64, "gk-other", "gpt-5", 900_i64, 9.00_f64),
    ] {
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some(key_id.to_string()),
                account_id: Some(format!("acc-{key_id}")),
                model: Some(model.to_string()),
                total_tokens: Some(total_tokens),
                estimated_cost_usd: Some(cost),
                created_at: 1_000 + request_log_id,
                ..Default::default()
            })
            .expect("insert old token stat");
    }
    storage
        .rollup_request_token_stats_before(2_000)
        .expect("roll up old token stats");
    storage
        .insert_request_token_stat(&RequestTokenStat {
            request_log_id: 10,
            key_id: Some("gk-a".to_string()),
            account_id: Some("acc-gk-a".to_string()),
            model: Some("gpt-5-mini".to_string()),
            total_tokens: Some(20),
            estimated_cost_usd: Some(0.20),
            created_at: 3_000,
            ..Default::default()
        })
        .expect("insert hot token stat");

    let key_summary = storage
        .summarize_request_token_stats_by_key_ids(&["gk-a".to_string(), "gk-b".to_string()])
        .expect("summarize by key ids");
    let key_totals = key_summary
        .iter()
        .map(|item| (item.key_id.as_str(), item.total_tokens))
        .collect::<Vec<_>>();
    assert_eq!(key_totals, vec![("gk-a", 50), ("gk-b", 50)]);
    assert!(!key_summary.iter().any(|item| item.key_id == "gk-other"));

    let model_summary = storage
        .summarize_request_token_stats_by_key_ids_and_model(&["gk-a".to_string()], None, None)
        .expect("summarize by key ids and model");
    let model_totals = model_summary
        .iter()
        .map(|item| (item.key_id.as_str(), item.model.as_str(), item.total_tokens))
        .collect::<Vec<_>>();
    assert_eq!(
        model_totals,
        vec![("gk-a", "gpt-5", 30), ("gk-a", "gpt-5-mini", 20)]
    );
    assert!(!model_summary.iter().any(|item| item.key_id == "gk-other"));
}

#[test]
fn rollup_request_token_stats_before_limited_rolls_only_one_batch() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    for request_log_id in 1_i64..=3 {
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some("gk-batch".to_string()),
                account_id: Some("acc-batch".to_string()),
                model: Some("gpt-5".to_string()),
                total_tokens: Some(10 * request_log_id),
                estimated_cost_usd: Some(0.01 * request_log_id as f64),
                created_at: 1_000 + request_log_id,
                ..Default::default()
            })
            .expect("insert token stat");
    }

    let first_batch = storage
        .rollup_request_token_stats_before_limited(2_000, 2)
        .expect("roll up first batch");
    assert_eq!(first_batch, 2);
    let remaining: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_token_stats", [], |row| {
            row.get(0)
        })
        .expect("count remaining token stats");
    assert_eq!(remaining, 1);
    let rolled_tokens: i64 = storage
        .conn
        .query_row(
            "SELECT total_tokens FROM request_token_stat_rollups
             WHERE key_id = 'gk-batch' AND account_id = 'acc-batch' AND model = 'gpt-5'",
            [],
            |row| row.get(0),
        )
        .expect("load rollup");
    assert_eq!(rolled_tokens, 30);

    let second_batch = storage
        .rollup_request_token_stats_before_limited(2_000, 2)
        .expect("roll up second batch");
    assert_eq!(second_batch, 1);
    let rolled_tokens: i64 = storage
        .conn
        .query_row(
            "SELECT total_tokens FROM request_token_stat_rollups
             WHERE key_id = 'gk-batch' AND account_id = 'acc-batch' AND model = 'gpt-5'",
            [],
            |row| row.get(0),
        )
        .expect("load rollup");
    assert_eq!(rolled_tokens, 60);
}

#[test]
fn observability_prune_defers_request_log_delete_until_token_stats_batch_finishes() {
    let _guard = env_guard();
    std::env::set_var("CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT", "1");
    std::env::set_var("CODEXMANAGER_REQUEST_TOKEN_STATS_RETENTION_DAYS", "1");
    std::env::set_var("CODEXMANAGER_REQUEST_LOG_RETENTION_DAYS", "1");

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    for request_log_id in 1_i64..=2 {
        storage
            .insert_request_log(&RequestLog {
                trace_id: Some(format!("trc-batch-{request_log_id}")),
                key_id: Some("gk-batch".to_string()),
                account_id: Some("acc-batch".to_string()),
                request_path: "/v1/responses".to_string(),
                method: "POST".to_string(),
                status_code: Some(200),
                created_at: 1_000 + request_log_id,
                ..Default::default()
            })
            .expect("insert request log");
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id,
                key_id: Some("gk-batch".to_string()),
                account_id: Some("acc-batch".to_string()),
                model: Some("gpt-5".to_string()),
                total_tokens: Some(10),
                estimated_cost_usd: Some(0.01),
                created_at: 1_000 + request_log_id,
                ..Default::default()
            })
            .expect("insert token stat");
    }

    storage
        .prune_observability_history(200_000)
        .expect("first maintenance");
    let request_logs_after_first: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request logs");
    assert_eq!(request_logs_after_first, 2);

    storage
        .prune_observability_history(200_000)
        .expect("second maintenance");
    let request_logs_after_second: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request logs");
    assert_eq!(request_logs_after_second, 2);

    storage
        .prune_observability_history(200_000)
        .expect("third maintenance");
    let request_logs_after_third: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request logs");
    assert_eq!(request_logs_after_third, 1);

    std::env::remove_var("CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT");
    std::env::remove_var("CODEXMANAGER_REQUEST_TOKEN_STATS_RETENTION_DAYS");
    std::env::remove_var("CODEXMANAGER_REQUEST_LOG_RETENTION_DAYS");
}

/// 函数 `summarize_request_log_error_codes_groups_and_dedups`
///
/// 中文注释：验证 E 项错误码聚合查询——按 error_code 分组计数、取最近代表样例、
/// NULL error_code 归入 unknown、空 error 不计入、时间窗口过滤生效、按 count 降序。
#[test]
fn summarize_request_log_error_codes_groups_and_dedups() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");

    // 中文注释：辅助闭包——按 error_code/error/created_at 插入一条最小请求日志。
    let insert = |code: Option<&str>, error: Option<&str>, created_at: i64| {
        storage
            .insert_request_log(&RequestLog {
                request_path: "/v1/responses".to_string(),
                method: "POST".to_string(),
                error: error.map(str::to_string),
                error_code: code.map(str::to_string),
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
    };

    // refresh_token_reused 出现 3 次（代表样例应取最近一条，created_at=300）。
    insert(Some("refresh_token_reused"), Some("reused #1"), 100);
    insert(Some("refresh_token_reused"), Some("reused #2"), 200);
    insert(Some("refresh_token_reused"), Some("reused #3 latest"), 300);
    // network_error 出现 2 次。
    insert(Some("network_error"), Some("conn reset"), 150);
    insert(Some("network_error"), Some("conn reset again"), 250);
    // NULL error_code 但有 error 文本：应归入 unknown。
    insert(None, Some("unclassified boom"), 180);
    // 成功行（error 为空）：不应计入聚合。
    insert(None, None, 190);
    insert(Some("should_be_ignored_empty_error"), Some("   "), 195);

    // 不限窗口：应得到 3 类（reused / network_error / unknown）。
    let all = storage
        .summarize_request_log_error_codes(None, None, 50)
        .expect("aggregate all");
    assert_eq!(all.len(), 3, "应聚合为 3 类错误：{all:#?}");
    // 按 count 降序：reused(3) 在首位。
    assert_eq!(all[0].error_code, "refresh_token_reused");
    assert_eq!(all[0].count, 3);
    assert_eq!(all[0].last_seen, 300);
    assert_eq!(all[0].sample_message.as_deref(), Some("reused #3 latest"));
    // unknown 类（NULL code 的那一条）。
    let unknown = all
        .iter()
        .find(|item| item.error_code == "unknown")
        .expect("unknown 类应存在");
    assert_eq!(unknown.count, 1);
    assert_eq!(unknown.sample_message.as_deref(), Some("unclassified boom"));

    // 时间窗口过滤：仅 created_at>=250 的行——reused #3(300) + network_error(250)。
    let windowed = storage
        .summarize_request_log_error_codes(Some(250), None, 50)
        .expect("aggregate windowed");
    assert_eq!(windowed.len(), 2, "窗口内应仅 2 类：{windowed:#?}");
    let reused = windowed
        .iter()
        .find(|item| item.error_code == "refresh_token_reused")
        .expect("reused 应在窗口内");
    assert_eq!(reused.count, 1, "窗口内 reused 仅 1 条（created_at=300）");

    // limit 截断：只取 count 最高的 1 类。
    let limited = storage
        .summarize_request_log_error_codes(None, None, 1)
        .expect("aggregate limited");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].error_code, "refresh_token_reused");
}

#[test]
fn daily_rollup_mixed_dashboard_queries_survive_token_stat_prune() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let day_start = 1_700_000_000;
    let next_day = day_start + 86_400;
    let day_after = next_day + 86_400;
    let key_id = "gk-daily-rollup".to_string();
    let account_id = "acc-daily-rollup".to_string();
    let user_id = "user-daily-rollup".to_string();

    storage
        .insert_api_key(&test_api_key(&key_id, day_start))
        .expect("insert api key");
    storage
        .insert_app_user(&AppUser {
            id: user_id.clone(),
            username: user_id.clone(),
            display_name: None,
            password_hash: "test-hash".to_string(),
            role: "member".to_string(),
            status: "active".to_string(),
            created_at: day_start,
            updated_at: day_start,
            last_login_at: None,
        })
        .expect("insert user");
    storage
        .upsert_api_key_owner(&ApiKeyOwner {
            key_id: key_id.clone(),
            owner_kind: "user".to_string(),
            owner_user_id: Some(user_id.clone()),
            project_id: None,
            updated_at: day_start,
        })
        .expect("insert owner");

    for (request_id, trace_id, created_at, total_tokens) in [
        (1_i64, "trace-rollup-history", day_start + 60, 30_i64),
        (2_i64, "trace-rollup-live", next_day + 60, 20_i64),
    ] {
        let log_id = storage
            .insert_request_log(&RequestLog {
                trace_id: Some(trace_id.to_string()),
                key_id: Some(key_id.clone()),
                account_id: Some(account_id.clone()),
                request_path: "/v1/responses".to_string(),
                method: "POST".to_string(),
                model: Some("gpt-5".to_string()),
                actual_source_kind: Some("openai_account".to_string()),
                actual_source_id: Some(account_id.clone()),
                status_code: Some(200),
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
        assert_eq!(log_id, request_id);
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id: log_id,
                key_id: Some(key_id.clone()),
                account_id: Some(account_id.clone()),
                model: Some("gpt-5".to_string()),
                input_tokens: Some(total_tokens / 2),
                output_tokens: Some(total_tokens / 2),
                total_tokens: Some(total_tokens),
                estimated_cost_usd: Some(total_tokens as f64 / 100.0),
                created_at,
                ..Default::default()
            })
            .expect("insert token stat");
    }

    assert_eq!(
        storage
            .rollup_request_token_stats_daily_before_limited(next_day, next_day, 86_400, 100)
            .expect("daily rollup"),
        1
    );
    assert_eq!(
        storage
            .rollup_request_token_stats_daily_before_limited(next_day, next_day, 86_400, 100)
            .expect("daily rollup idempotent"),
        0
    );
    let daily_rows = storage
        .query_request_token_stat_daily_rollups(day_start)
        .expect("query daily rollup");
    assert_eq!(daily_rows.len(), 1);
    assert_eq!(daily_rows[0].total_tokens, 30);
    assert_eq!(daily_rows[0].user_id, user_id);
    assert_eq!(daily_rows[0].source_kind, "openai_account");
    assert_eq!(daily_rows[0].source_id, account_id);

    storage
        .rollup_request_token_stats_before(next_day)
        .expect("legacy rollup deletes history stats");
    let live_rows: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_token_stats", [], |row| {
            row.get(0)
        })
        .expect("count token stats");
    assert_eq!(live_rows, 1);

    let daily_usage = storage
        .summarize_request_token_stats_daily_mixed(day_start, day_after, 86_400, next_day)
        .expect("mixed daily usage");
    assert_eq!(daily_usage.len(), 2);
    assert_eq!(daily_usage[0].usage.total_tokens, 30);
    assert_eq!(daily_usage[1].usage.total_tokens, 20);

    let user_ranking = storage
        .summarize_request_token_stats_user_ranking_between_mixed(
            next_day, day_after, day_start, day_after, next_day, 10,
        )
        .expect("mixed user ranking");
    assert_eq!(user_ranking.len(), 1);
    assert_eq!(user_ranking[0].user_id, user_id);
    assert_eq!(user_ranking[0].today_usage.total_tokens, 20);
    assert_eq!(user_ranking[0].range_usage.total_tokens, 50);

    let source_ranking = storage
        .summarize_request_token_stats_source_ranking_between_mixed(
            "openai_account",
            next_day,
            day_after,
            day_start,
            day_after,
            next_day,
            10,
        )
        .expect("mixed source ranking");
    assert_eq!(source_ranking.len(), 1);
    assert_eq!(source_ranking[0].source_id, account_id);
    assert_eq!(source_ranking[0].today_usage.total_tokens, 20);
    assert_eq!(source_ranking[0].range_usage.total_tokens, 50);
}
#[test]
fn observability_prune_rolls_complete_local_days_before_deleting_retained_detail() {
    let _guard = env_guard();
    std::env::set_var("CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT", "100");
    std::env::set_var("CODEXMANAGER_REQUEST_TOKEN_STATS_RETENTION_DAYS", "1");
    std::env::set_var("CODEXMANAGER_REQUEST_LOG_RETENTION_DAYS", "1");

    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let today_start = 1_700_086_400;
    let previous_day = today_start - 86_400;
    let now = today_start + 3_600;
    let key_id = "gk-complete-day-rollup".to_string();
    let account_id = "acc-complete-day-rollup".to_string();

    for (trace_id, created_at, total_tokens) in [
        ("trace-retention-deleted", previous_day + 60, 10_i64),
        ("trace-retention-kept", previous_day + 7_200, 20_i64),
    ] {
        let log_id = storage
            .insert_request_log(&RequestLog {
                trace_id: Some(trace_id.to_string()),
                key_id: Some(key_id.clone()),
                account_id: Some(account_id.clone()),
                request_path: "/v1/responses".to_string(),
                method: "POST".to_string(),
                model: Some("gpt-5".to_string()),
                actual_source_kind: Some("openai_account".to_string()),
                actual_source_id: Some(account_id.clone()),
                status_code: Some(200),
                created_at,
                ..Default::default()
            })
            .expect("insert request log");
        storage
            .insert_request_token_stat(&RequestTokenStat {
                request_log_id: log_id,
                key_id: Some(key_id.clone()),
                account_id: Some(account_id.clone()),
                model: Some("gpt-5".to_string()),
                total_tokens: Some(total_tokens),
                estimated_cost_usd: Some(total_tokens as f64 / 100.0),
                created_at,
                ..Default::default()
            })
            .expect("insert token stat");
    }

    storage
        .prune_observability_history_with_daily_rollup_anchor(now, today_start)
        .expect("maintenance");

    let previous_rollups = storage
        .query_request_token_stat_daily_rollups(previous_day)
        .expect("query previous daily rollup");
    assert_eq!(previous_rollups.len(), 1);
    assert_eq!(previous_rollups[0].total_tokens, 30);

    let remaining_stats: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_token_stats", [], |row| {
            row.get(0)
        })
        .expect("count token stats");
    assert_eq!(
        remaining_stats, 1,
        "只应删除 retention cutoff 之前且已固化的明细"
    );

    let remaining_created_at: i64 = storage
        .conn
        .query_row("SELECT created_at FROM request_token_stats", [], |row| {
            row.get(0)
        })
        .expect("load remaining token stat");
    assert_eq!(remaining_created_at, previous_day + 7_200);

    let mixed_daily = storage
        .summarize_request_token_stats_daily_mixed(previous_day, today_start, 86_400, today_start)
        .expect("mixed daily usage");
    assert_eq!(mixed_daily.len(), 1);
    assert_eq!(mixed_daily[0].usage.total_tokens, 30);

    let request_logs: i64 = storage
        .conn
        .query_row("SELECT COUNT(1) FROM request_logs", [], |row| row.get(0))
        .expect("count request logs");
    assert_eq!(
        request_logs, 1,
        "已删除明细对应的旧请求日志可以按 retention 清理"
    );

    std::env::remove_var("CODEXMANAGER_OBSERVABILITY_MAINTENANCE_BATCH_LIMIT");
    std::env::remove_var("CODEXMANAGER_REQUEST_TOKEN_STATS_RETENTION_DAYS");
    std::env::remove_var("CODEXMANAGER_REQUEST_LOG_RETENTION_DAYS");
}
#[test]
fn daily_rollup_mixed_queries_use_unrolled_live_stats_when_rollup_is_not_ready() {
    let storage = Storage::open_in_memory().expect("open");
    storage.init().expect("init");
    let day_start = 1_700_000_000;
    let next_day = day_start + 86_400;
    let key_id = "gk-unrolled-fallback".to_string();
    let account_id = "acc-unrolled-fallback".to_string();
    let user_id = "user-unrolled-fallback".to_string();

    storage
        .insert_api_key(&test_api_key(&key_id, day_start))
        .expect("insert api key");
    storage
        .insert_app_user(&AppUser {
            id: user_id.clone(),
            username: user_id.clone(),
            display_name: None,
            password_hash: "test-hash".to_string(),
            role: "member".to_string(),
            status: "active".to_string(),
            created_at: day_start,
            updated_at: day_start,
            last_login_at: None,
        })
        .expect("insert user");
    storage
        .upsert_api_key_owner(&ApiKeyOwner {
            key_id: key_id.clone(),
            owner_kind: "user".to_string(),
            owner_user_id: Some(user_id.clone()),
            project_id: None,
            updated_at: day_start,
        })
        .expect("insert owner");

    let log_id = storage
        .insert_request_log(&RequestLog {
            trace_id: Some("trace-unrolled-fallback".to_string()),
            key_id: Some(key_id.clone()),
            account_id: Some(account_id.clone()),
            request_path: "/v1/responses".to_string(),
            method: "POST".to_string(),
            model: Some("gpt-5".to_string()),
            actual_source_kind: Some("openai_account".to_string()),
            actual_source_id: Some(account_id.clone()),
            status_code: Some(200),
            created_at: day_start + 60,
            ..Default::default()
        })
        .expect("insert request log");
    storage
        .insert_request_token_stat(&RequestTokenStat {
            request_log_id: log_id,
            key_id: Some(key_id),
            account_id: Some(account_id.clone()),
            model: Some("gpt-5".to_string()),
            total_tokens: Some(30),
            estimated_cost_usd: Some(0.3),
            created_at: day_start + 60,
            ..Default::default()
        })
        .expect("insert token stat");

    let daily_usage = storage
        .summarize_request_token_stats_daily_mixed(day_start, next_day, 86_400, next_day)
        .expect("mixed daily usage");
    assert_eq!(daily_usage.len(), 1);
    assert_eq!(daily_usage[0].usage.total_tokens, 30);

    let user_usage = storage
        .summarize_request_token_stats_by_user_between_mixed(day_start, next_day, next_day)
        .expect("mixed user usage");
    assert_eq!(user_usage.len(), 1);
    assert_eq!(user_usage[0].user_id, user_id);
    assert_eq!(user_usage[0].usage.total_tokens, 30);

    let source_usage = storage
        .summarize_request_token_stats_by_source_between_mixed(
            "openai_account",
            day_start,
            next_day,
            next_day,
        )
        .expect("mixed source usage");
    assert_eq!(source_usage.len(), 1);
    assert_eq!(source_usage[0].source_id, account_id);
    assert_eq!(source_usage[0].usage.total_tokens, 30);
}

#[test]
fn america_new_york_dst_days_keep_daily_rollups_before_and_after_detail_prune() {
    // America/New_York：2024-03-10 为 23 小时，2024-11-03 为 25 小时。
    for (case_name, day_start, day_end) in [
        ("spring-forward", 1_710_046_800_i64, 1_710_129_600_i64),
        ("fall-back", 1_730_606_400_i64, 1_730_696_400_i64),
    ] {
        let storage = Storage::open_in_memory().expect("open");
        storage.init().expect("init");
        let key_id = format!("gk-dst-{case_name}");
        let account_id = format!("acc-dst-{case_name}");

        for (offset, total_tokens) in [(60_i64, 10_i64), (day_end - day_start - 60, 20)] {
            let created_at = day_start + offset;
            let log_id = storage
                .insert_request_log(&RequestLog {
                    trace_id: Some(format!("trace-{case_name}-{offset}")),
                    key_id: Some(key_id.clone()),
                    account_id: Some(account_id.clone()),
                    request_path: "/v1/responses".to_string(),
                    method: "POST".to_string(),
                    model: Some("gpt-5".to_string()),
                    actual_source_kind: Some("openai_account".to_string()),
                    actual_source_id: Some(account_id.clone()),
                    status_code: Some(200),
                    created_at,
                    ..Default::default()
                })
                .expect("insert request log");
            storage
                .insert_request_token_stat(&RequestTokenStat {
                    request_log_id: log_id,
                    key_id: Some(key_id.clone()),
                    account_id: Some(account_id.clone()),
                    model: Some("gpt-5".to_string()),
                    total_tokens: Some(total_tokens),
                    created_at,
                    ..Default::default()
                })
                .expect("insert token stat");
        }

        assert_eq!(
            storage
                .rollup_request_token_stats_daily_before_limited(
                    day_end,
                    day_end,
                    day_end - day_start,
                    100,
                )
                .expect("roll up DST day"),
            2
        );
        let ranges = [(day_start, day_end)];
        let before_prune = storage
            .summarize_request_token_stats_daily_mixed_ranges(&ranges, &ranges)
            .expect("mixed summary before detail prune");
        assert_eq!(before_prune.len(), 1, "{case_name}");
        assert_eq!(before_prune[0].day_start_ts, day_start, "{case_name}");
        assert_eq!(before_prune[0].day_end_ts, day_end, "{case_name}");
        assert_eq!(before_prune[0].usage.total_tokens, 30, "{case_name}");

        storage
            .rollup_request_token_stats_before(day_end)
            .expect("prune rolled detail");
        let remaining_stats: i64 = storage
            .conn
            .query_row("SELECT COUNT(1) FROM request_token_stats", [], |row| {
                row.get(0)
            })
            .expect("count remaining stats");
        assert_eq!(remaining_stats, 0, "{case_name}");

        let after_prune = storage
            .summarize_request_token_stats_daily_mixed_ranges(&ranges, &ranges)
            .expect("mixed summary after detail prune");
        assert_eq!(after_prune.len(), 1, "{case_name}");
        assert_eq!(after_prune[0].usage.total_tokens, 30, "{case_name}");
    }
}
