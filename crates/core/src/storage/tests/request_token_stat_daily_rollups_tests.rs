use super::{now_ts, RequestTokenStatDailyRollup, Storage};

#[test]
fn test_daily_rollup_table_creation() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    // 验证表已创建
    let table_exists: i64 = storage
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='request_token_stat_daily_rollups'",
            [],
            |row| row.get(0),
        )
        .expect("query table existence");
    assert_eq!(table_exists, 1, "日级 rollup 表应该存在");

    // 验证索引已创建
    let index_count: i64 = storage
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='request_token_stat_daily_rollups'",
            [],
            |row| row.get(0),
        )
        .expect("query index count");
    assert!(index_count >= 3, "至少应该有 3 个索引");
}

#[test]
fn test_insert_and_query_daily_rollup() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    let day_start = 1719014400; // 2024-06-22 00:00:00 UTC
    let now = now_ts();

    let rollup = RequestTokenStatDailyRollup {
        day_start,
        key_id: "test-key".to_string(),
        account_id: "test-account".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "test-source".to_string(),
        user_id: "test-user".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 1000,
        cached_input_tokens: 100,
        output_tokens: 500,
        total_tokens: 1400,
        reasoning_output_tokens: 50,
        estimated_cost: 0.05,
        request_count: 10,
        success_count: 9,
        error_count: 1,
        source_rows: 10,
        updated_at: now,
    };

    storage
        .insert_request_token_stat_daily_rollup(&rollup)
        .expect("insert rollup");

    // 查询插入的记录
    let results = storage
        .query_request_token_stat_daily_rollups(day_start)
        .expect("query rollups");

    assert_eq!(results.len(), 1, "应该返回 1 条记录");
    let result = &results[0];
    assert_eq!(result.day_start, day_start);
    assert_eq!(result.key_id, "test-key");
    assert_eq!(result.account_id, "test-account");
    assert_eq!(result.source_kind, "openai_account");
    assert_eq!(result.source_id, "test-source");
    assert_eq!(result.user_id, "test-user");
    assert_eq!(result.model, "gpt-4");
    assert_eq!(result.status_bucket, "2xx");
    assert_eq!(result.input_tokens, 1000);
    assert_eq!(result.cached_input_tokens, 100);
    assert_eq!(result.output_tokens, 500);
    assert_eq!(result.total_tokens, 1400);
    assert_eq!(result.reasoning_output_tokens, 50);
    assert!((result.estimated_cost - 0.05).abs() < 1e-6);
    assert_eq!(result.request_count, 10);
    assert_eq!(result.success_count, 9);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.source_rows, 10);
}

#[test]
fn test_daily_rollup_upsert_conflict() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    let day_start = 1719014400;
    let now = now_ts();

    let rollup1 = RequestTokenStatDailyRollup {
        day_start,
        key_id: "key1".to_string(),
        account_id: "acc1".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "src1".to_string(),
        user_id: "user1".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 100,
        cached_input_tokens: 10,
        output_tokens: 50,
        total_tokens: 140,
        reasoning_output_tokens: 5,
        estimated_cost: 0.01,
        request_count: 1,
        success_count: 1,
        error_count: 0,
        source_rows: 1,
        updated_at: now,
    };

    storage
        .insert_request_token_stat_daily_rollup(&rollup1)
        .expect("insert rollup1");

    // 插入相同主键的记录，应该累加
    let rollup2 = RequestTokenStatDailyRollup {
        day_start,
        key_id: "key1".to_string(),
        account_id: "acc1".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "src1".to_string(),
        user_id: "user1".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 200,
        cached_input_tokens: 20,
        output_tokens: 100,
        total_tokens: 280,
        reasoning_output_tokens: 10,
        estimated_cost: 0.02,
        request_count: 2,
        success_count: 2,
        error_count: 0,
        source_rows: 2,
        updated_at: now + 100,
    };

    storage
        .insert_request_token_stat_daily_rollup(&rollup2)
        .expect("insert rollup2");

    let results = storage
        .query_request_token_stat_daily_rollups(day_start)
        .expect("query rollups");

    assert_eq!(results.len(), 1, "应该只有 1 条记录（冲突时累加）");
    let result = &results[0];
    assert_eq!(result.input_tokens, 300, "input_tokens 应该累加");
    assert_eq!(
        result.cached_input_tokens, 30,
        "cached_input_tokens 应该累加"
    );
    assert_eq!(result.output_tokens, 150, "output_tokens 应该累加");
    assert_eq!(result.total_tokens, 420, "total_tokens 应该累加");
    assert_eq!(
        result.reasoning_output_tokens, 15,
        "reasoning_output_tokens 应该累加"
    );
    assert!(
        (result.estimated_cost - 0.03).abs() < 1e-6,
        "estimated_cost 应该累加"
    );
    assert_eq!(result.request_count, 3, "request_count 应该累加");
    assert_eq!(result.success_count, 3, "success_count 应该累加");
    assert_eq!(result.error_count, 0, "error_count 应该累加");
    assert_eq!(result.source_rows, 3, "source_rows 应该累加");
    assert_eq!(result.updated_at, now + 100, "updated_at 应该更新为最新");
}

#[test]
fn test_daily_rollup_empty_string_normalization() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    let day_start = 1719014400;
    let now = now_ts();

    // 测试空字符串作为维度值（系统级聚合）
    let rollup = RequestTokenStatDailyRollup {
        day_start,
        key_id: "".to_string(),
        account_id: "".to_string(),
        source_kind: "".to_string(),
        source_id: "".to_string(),
        user_id: "".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 1000,
        cached_input_tokens: 100,
        output_tokens: 500,
        total_tokens: 1400,
        reasoning_output_tokens: 50,
        estimated_cost: 0.05,
        request_count: 10,
        success_count: 9,
        error_count: 1,
        source_rows: 10,
        updated_at: now,
    };

    storage
        .insert_request_token_stat_daily_rollup(&rollup)
        .expect("insert rollup with empty strings");

    let results = storage
        .query_request_token_stat_daily_rollups(day_start)
        .expect("query rollups");

    assert_eq!(results.len(), 1, "应该支持空字符串作为维度值");
    let result = &results[0];
    assert_eq!(result.key_id, "");
    assert_eq!(result.account_id, "");
    assert_eq!(result.source_kind, "");
    assert_eq!(result.source_id, "");
    assert_eq!(result.user_id, "");
}

#[test]
fn test_daily_rollup_multiple_dimensions() {
    let storage = Storage::open_in_memory().expect("open storage");
    storage.init().expect("init schema");

    let day_start = 1719014400;
    let now = now_ts();

    // 插入不同维度组合的记录
    let rollup1 = RequestTokenStatDailyRollup {
        day_start,
        key_id: "key1".to_string(),
        account_id: "acc1".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "src1".to_string(),
        user_id: "user1".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 100,
        cached_input_tokens: 10,
        output_tokens: 50,
        total_tokens: 140,
        reasoning_output_tokens: 5,
        estimated_cost: 0.01,
        request_count: 1,
        success_count: 1,
        error_count: 0,
        source_rows: 1,
        updated_at: now,
    };

    let rollup2 = RequestTokenStatDailyRollup {
        day_start,
        key_id: "key1".to_string(),
        account_id: "acc1".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "src1".to_string(),
        user_id: "user1".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "4xx".to_string(), // 不同的 status_bucket
        input_tokens: 50,
        cached_input_tokens: 5,
        output_tokens: 25,
        total_tokens: 70,
        reasoning_output_tokens: 2,
        estimated_cost: 0.005,
        request_count: 1,
        success_count: 0,
        error_count: 1,
        source_rows: 1,
        updated_at: now,
    };

    let rollup3 = RequestTokenStatDailyRollup {
        day_start,
        key_id: "key2".to_string(), // 不同的 key_id
        account_id: "acc1".to_string(),
        source_kind: "openai_account".to_string(),
        source_id: "src1".to_string(),
        user_id: "user1".to_string(),
        model: "gpt-4".to_string(),
        status_bucket: "2xx".to_string(),
        input_tokens: 200,
        cached_input_tokens: 20,
        output_tokens: 100,
        total_tokens: 280,
        reasoning_output_tokens: 10,
        estimated_cost: 0.02,
        request_count: 2,
        success_count: 2,
        error_count: 0,
        source_rows: 2,
        updated_at: now,
    };

    storage
        .insert_request_token_stat_daily_rollup(&rollup1)
        .expect("insert rollup1");
    storage
        .insert_request_token_stat_daily_rollup(&rollup2)
        .expect("insert rollup2");
    storage
        .insert_request_token_stat_daily_rollup(&rollup3)
        .expect("insert rollup3");

    let results = storage
        .query_request_token_stat_daily_rollups(day_start)
        .expect("query rollups");

    assert_eq!(results.len(), 3, "应该有 3 条不同维度组合的记录");
}
