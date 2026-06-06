use super::*;
use codexmanager_core::rpc::types::{JsonRpcMessage, JsonRpcResponse};
use codexmanager_core::storage::{ModelGroupModel, RequestLog, RequestTokenStat};

/// 函数 `response_result`
///
/// 作者: gaohongshun
///
/// 时间: 2026-04-02
///
/// # 参数
/// - resp: 参数 resp
///
/// # 返回
/// 返回函数执行结果
fn response_result(resp: JsonRpcMessage) -> JsonRpcResponse {
    match resp {
        JsonRpcMessage::Response(resp) => resp,
        JsonRpcMessage::Error(err) => panic!("unexpected rpc error: {}", err.error.message),
        JsonRpcMessage::Notification(_) => panic!("unexpected rpc notification"),
        JsonRpcMessage::Request(_) => panic!("unexpected rpc request"),
    }
}

/// 函数 `login_complete_requires_params`
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
fn login_complete_requires_params() {
    let req = JsonRpcRequest {
        id: 1.into(),
        method: "account/login/complete".to_string(),
        params: None,
        trace: None,
    };
    let resp = response_result(handle_request(req));
    let err = resp
        .result
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(err.contains("missing"));

    let req = JsonRpcRequest {
        id: 2.into(),
        method: "account/login/complete".to_string(),
        params: Some(serde_json::json!({ "code": "x" })),
        trace: None,
    };
    let resp = response_result(handle_request(req));
    let err = resp
        .result
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(err.contains("missing"));

    let req = JsonRpcRequest {
        id: 3.into(),
        method: "account/login/complete".to_string(),
        params: Some(serde_json::json!({ "state": "y" })),
        trace: None,
    };
    let resp = response_result(handle_request(req));
    let err = resp
        .result
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(err.contains("missing"));
}

/// 函数 `unknown_method_returns_jsonrpc_error`
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
fn unknown_method_returns_jsonrpc_error() {
    let req = JsonRpcRequest {
        id: 9.into(),
        method: "not/a/method".to_string(),
        params: None,
        trace: None,
    };

    match handle_request(req) {
        JsonRpcMessage::Error(err) => {
            assert_eq!(err.id, 9.into());
            assert_eq!(err.error.code, -32601);
            assert_eq!(err.error.message, "unknown_method");
        }
        other => panic!("expected rpc error, got {other:?}"),
    }
}

#[test]
fn member_actor_cannot_call_admin_only_rpc() {
    let req = JsonRpcRequest {
        id: 21.into(),
        method: "accountManager/users/list".to_string(),
        params: None,
        trace: None,
    };

    let resp = response_result(handle_request_with_actor(
        req,
        RpcActor::from_parts(Some(ROLE_MEMBER), Some("user-1")),
    ));
    let err = resp
        .result
        .get("error")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert!(err.contains("permission_denied"));
}

#[test]
fn password_mode_can_call_admin_and_model_source_rpcs() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-password-model-source-rpc");
    set_web_access_password(Some("password123")).expect("set web password");
    set_web_auth_mode("password").expect("enable password mode");
    let actor = RpcActor::from_parts(Some(ROLE_MEMBER), Some("password-mode-user"));

    let admin_resp = response_result(handle_request_with_actor(
        JsonRpcRequest {
            id: 22.into(),
            method: "accountManager/users/list".to_string(),
            params: None,
            trace: None,
        },
        actor.clone(),
    ));
    let admin_err = rpc_error(&admin_resp);
    assert!(
        !admin_err.contains("permission_denied"),
        "password mode unexpectedly denied accountManager/users/list: {admin_err}"
    );

    for (method, params) in [
        (
            "apikey/modelSourceSync",
            serde_json::json!({ "sourceKind": "aggregate_api" }),
        ),
        (
            "apikey/modelSourceModelSave",
            serde_json::json!({
                "sourceKind": "aggregate_api",
                "sourceId": "ag_test",
                "upstreamModel": "gpt-4o"
            }),
        ),
        (
            "apikey/modelSourceMappingSave",
            serde_json::json!({
                "platformModelSlug": "gpt-4o",
                "sourceKind": "aggregate_api",
                "sourceId": "ag_test",
                "upstreamModel": "gpt-4o"
            }),
        ),
        (
            "apikey/modelSourceMappingDelete",
            serde_json::json!({ "id": "map_test" }),
        ),
    ] {
        let resp = response_result(handle_request_with_actor(
            rpc_request(method, params),
            actor.clone(),
        ));
        let err = rpc_error(&resp);
        assert!(
            !err.contains("permission_denied"),
            "{method} unexpectedly denied: {err}"
        );
    }

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn admin_user_update_edits_member_and_protects_last_active_admin() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-user-update");
    let admin = create_app_user(AppUserCreateInput {
        username: "admin-update-one".to_string(),
        password: "password-one".to_string(),
        display_name: None,
        role: Some(ROLE_ADMIN.to_string()),
        initial_balance_credit_micros: None,
    })
    .expect("create admin");
    let member = create_test_member("member-update-one", Some(1_000_000));

    let updated = update_app_user(AppUserUpdateInput {
        id: member.id.clone(),
        display_name: Some("Updated Member".to_string()),
        role: Some(ROLE_MEMBER.to_string()),
        status: Some("disabled".to_string()),
        password: Some("new-password".to_string()),
    })
    .expect("update member");
    assert_eq!(updated.display_name.as_deref(), Some("Updated Member"));
    assert_eq!(updated.status, "disabled");
    assert_eq!(updated.role, ROLE_MEMBER);
    assert!(updated.wallet.is_some());

    let last_admin_error = update_app_user(AppUserUpdateInput {
        id: admin.id.clone(),
        display_name: None,
        role: Some(ROLE_ADMIN.to_string()),
        status: Some("disabled".to_string()),
        password: None,
    })
    .expect_err("last active admin should be protected");
    assert!(last_admin_error.contains("至少需要保留一个启用的管理员"));

    let _second_admin = create_app_user(AppUserCreateInput {
        username: "admin-update-two".to_string(),
        password: "password-two".to_string(),
        display_name: None,
        role: Some(ROLE_ADMIN.to_string()),
        initial_balance_credit_micros: None,
    })
    .expect("create second admin");
    let disabled_admin = update_app_user(AppUserUpdateInput {
        id: admin.id,
        display_name: Some("Disabled Admin".to_string()),
        role: Some(ROLE_ADMIN.to_string()),
        status: Some("disabled".to_string()),
        password: None,
    })
    .expect("disable admin when another admin exists");
    assert_eq!(disabled_admin.status, "disabled");
    assert!(disabled_admin.wallet.is_none());

    let _ = std::fs::remove_file(db_path);
}

fn setup_dashboard_test_db(name: &str) -> String {
    let db_path = std::env::temp_dir()
        .join(format!(
            "{name}-{}-{}.sqlite",
            std::process::id(),
            codexmanager_core::storage::now_ts()
        ))
        .to_string_lossy()
        .to_string();
    let _ = std::fs::remove_file(&db_path);
    std::env::set_var("CODEXMANAGER_DB_PATH", &db_path);
    storage_helpers::initialize_storage().expect("init storage");
    db_path
}

fn rpc_request(method: &str, params: serde_json::Value) -> JsonRpcRequest {
    JsonRpcRequest {
        id: 31.into(),
        method: method.to_string(),
        params: Some(params),
        trace: None,
    }
}

fn rpc_error(resp: &JsonRpcResponse) -> String {
    resp.result
        .get("error")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string()
}

fn create_test_member(
    username: &str,
    initial_balance_credit_micros: Option<i64>,
) -> AppUserPublicResult {
    create_app_user(AppUserCreateInput {
        username: username.to_string(),
        password: format!("{username}-password"),
        display_name: None,
        role: Some(ROLE_MEMBER.to_string()),
        initial_balance_credit_micros,
    })
    .expect("create member")
}

fn create_owned_test_api_key(user_id: &str, name: &str, model: &str) -> String {
    let created = apikey_create::create_api_key(
        Some(name.to_string()),
        Some(model.to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("create api key");
    set_api_key_owner(&created.id, "user", Some(user_id), None).expect("own api key");
    created.id
}

#[test]
fn quota_source_list_bare_call_defaults_to_first_page() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-quota-source-list-page");
    let user = create_test_member("quota-source-list-user", Some(1_000_000));
    for index in 0..125 {
        create_owned_test_api_key(
            &user.id,
            &format!("quota source key {index:03}"),
            "gpt-5-mini",
        );
    }

    let resp = response_result(handle_request(rpc_request(
        "quota/sourceList",
        serde_json::json!({}),
    )));
    assert!(
        rpc_error(&resp).is_empty(),
        "quota/sourceList failed: {:?}",
        resp.result
    );
    assert_eq!(resp.result["items"].as_array().unwrap().len(), 100);
    assert_eq!(resp.result["total"], 125);
    assert_eq!(resp.result["page"], 1);
    assert_eq!(resp.result["pageSize"], 100);

    let second_page = response_result(handle_request(rpc_request(
        "quota/sourceList",
        serde_json::json!({ "sourceKind": "api_key", "page": 2, "pageSize": 100 }),
    )));
    assert_eq!(second_page.result["items"].as_array().unwrap().len(), 25);
    assert_eq!(second_page.result["total"], 125);

    let ambiguous_ids = response_result(handle_request(rpc_request(
        "quota/sourceList",
        serde_json::json!({ "sourceIds": ["key_1"] }),
    )));
    assert!(rpc_error(&ambiguous_ids).contains("sourceKind"));

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn quota_api_key_usage_bare_call_defaults_to_first_page() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-quota-api-key-usage-page");
    let user = create_test_member("quota-api-key-usage-user", Some(1_000_000));
    let mut key_ids = Vec::new();
    for index in 0..125 {
        let key_id = create_owned_test_api_key(
            &user.id,
            &format!("quota usage key {index:03}"),
            "gpt-5-mini",
        );
        if index == 0 {
            insert_test_request_log(
                &key_id,
                "trace-quota-usage-key",
                "gpt-5-mini",
                200,
                120,
                20,
                30,
                0.012,
                codexmanager_core::storage::now_ts(),
            );
        }
        key_ids.push(key_id);
    }

    let resp = response_result(handle_request(rpc_request(
        "quota/apiKeyUsage",
        serde_json::json!({}),
    )));
    assert!(
        rpc_error(&resp).is_empty(),
        "quota/apiKeyUsage failed: {:?}",
        resp.result
    );
    assert_eq!(resp.result["items"].as_array().unwrap().len(), 100);
    assert_eq!(resp.result["total"], 125);
    assert_eq!(resp.result["page"], 1);
    assert_eq!(resp.result["pageSize"], 100);
    assert!(resp.result["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["models"].as_array().unwrap().is_empty()));

    let scoped = response_result(handle_request(rpc_request(
        "quota/apiKeyUsage",
        serde_json::json!({ "keyIds": [key_ids[0]], "includeModels": true }),
    )));
    assert_eq!(scoped.result["items"].as_array().unwrap().len(), 1);
    assert_eq!(scoped.result["total"], 1);
    assert_eq!(
        scoped.result["items"][0]["models"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let _ = std::fs::remove_file(db_path);
}

fn insert_test_request_log(
    key_id: &str,
    trace_id: &str,
    model: &str,
    status_code: i64,
    input_tokens: i64,
    cached_input_tokens: i64,
    output_tokens: i64,
    estimated_cost_usd: f64,
    created_at: i64,
) {
    let total_tokens = input_tokens + output_tokens;
    let storage = storage_helpers::open_storage().expect("open storage");
    storage
        .insert_request_log_with_token_stat(
            &RequestLog {
                trace_id: Some(trace_id.to_string()),
                key_id: Some(key_id.to_string()),
                request_path: "/v1/chat/completions".to_string(),
                method: "POST".to_string(),
                model: Some(model.to_string()),
                upstream_model: Some(format!("{model}-upstream")),
                actual_source_kind: Some("openai_account".to_string()),
                actual_source_id: Some("private-account-id".to_string()),
                status_code: Some(status_code),
                input_tokens: Some(input_tokens),
                cached_input_tokens: Some(cached_input_tokens),
                output_tokens: Some(output_tokens),
                total_tokens: Some(total_tokens),
                estimated_cost_usd: Some(estimated_cost_usd),
                created_at,
                ..RequestLog::default()
            },
            &RequestTokenStat {
                key_id: Some(key_id.to_string()),
                model: Some(model.to_string()),
                input_tokens: Some(input_tokens),
                cached_input_tokens: Some(cached_input_tokens),
                output_tokens: Some(output_tokens),
                total_tokens: Some(total_tokens),
                estimated_cost_usd: Some(estimated_cost_usd),
                created_at,
                ..RequestTokenStat::default()
            },
        )
        .expect("insert request log");
}

#[test]
fn wallet_charge_uses_model_group_billing_model_override() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-model-group-billing-override");
    set_web_auth_mode("accounts").expect("enable accounts mode");
    set_distribution_enabled(true).expect("enable distribution");
    let user = create_test_member("member-model-group-billing", Some(1_000_000));
    let key_id = create_owned_test_api_key(&user.id, "member model group key", "gpt-5-mini");
    let storage = storage_helpers::open_storage().expect("open storage");
    let group_id = storage
        .default_model_group_id()
        .expect("read default model group")
        .expect("default model group");
    let now = codexmanager_core::storage::now_ts();
    storage
        .replace_model_group_models(
            &group_id,
            &[ModelGroupModel {
                group_id: group_id.clone(),
                platform_model_slug: "gpt-5-mini".to_string(),
                enabled: true,
                rate_multiplier_millis: Some(1000),
                billing_model_slug: Some("gpt-5.5".to_string()),
                note: None,
                created_at: now,
                updated_at: now,
            }],
        )
        .expect("save model group models");

    let ledger = wallet_charge_for_request(
        &storage,
        Some(&key_id),
        42,
        0.00225,
        Some("gpt-5-mini"),
        None,
        Some(
            serde_json::json!({
                "inputTokens": 1000,
                "cachedInputTokens": 0,
                "outputTokens": 1000
            })
            .to_string(),
        ),
    )
    .expect("charge wallet")
    .expect("ledger entry");

    assert_eq!(ledger.amount_credit_micros, -35_000);
    let usage: serde_json::Value =
        serde_json::from_str(ledger.raw_usage_json.as_deref().unwrap()).expect("usage json");
    assert_eq!(usage["billingModelSlug"], "gpt-5.5");
    assert_eq!(usage["platformEstimatedCostUsd"], 0.00225);
    assert!((usage["baseEstimatedCostUsd"].as_f64().unwrap() - 0.035).abs() < 0.000_001);
    assert!((usage["chargedCostUsd"].as_f64().unwrap() - 0.035).abs() < 0.000_001);
    let wallet = storage
        .find_wallet_by_owner("user", &user.id)
        .expect("read wallet")
        .expect("wallet");
    assert_eq!(wallet.balance_credit_micros, 965_000);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_dashboard_filters_to_current_user_keys() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-dashboard-filter");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user_one = create_app_user(AppUserCreateInput {
        username: "member-one".to_string(),
        password: "password-one".to_string(),
        display_name: None,
        role: Some(ROLE_MEMBER.to_string()),
        initial_balance_credit_micros: Some(2_000_000),
    })
    .expect("create member one");
    let user_two = create_app_user(AppUserCreateInput {
        username: "member-two".to_string(),
        password: "password-two".to_string(),
        display_name: None,
        role: Some(ROLE_MEMBER.to_string()),
        initial_balance_credit_micros: Some(2_000_000),
    })
    .expect("create member two");
    let key_one = apikey_create::create_api_key(
        Some("member one key".to_string()),
        Some("gpt-5-mini".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("create key one");
    let key_two = apikey_create::create_api_key(
        Some("member two key".to_string()),
        Some("gpt-5-mini".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("create key two");
    set_api_key_owner(&key_one.id, "user", Some(&user_one.id), None).expect("own key one");
    set_api_key_owner(&key_two.id, "user", Some(&user_two.id), None).expect("own key two");

    let storage = storage_helpers::open_storage().expect("open storage");
    storage
        .insert_request_log_with_token_stat(
            &RequestLog {
                trace_id: Some("trace-one".to_string()),
                key_id: Some(key_one.id.clone()),
                request_path: "/v1/chat/completions".to_string(),
                method: "POST".to_string(),
                model: Some("gpt-5-mini".to_string()),
                status_code: Some(200),
                input_tokens: Some(40),
                cached_input_tokens: Some(10),
                output_tokens: Some(30),
                total_tokens: Some(70),
                estimated_cost_usd: Some(0.01),
                created_at: day_start + 10,
                ..RequestLog::default()
            },
            &RequestTokenStat {
                key_id: Some(key_one.id.clone()),
                model: Some("gpt-5-mini".to_string()),
                input_tokens: Some(40),
                cached_input_tokens: Some(10),
                output_tokens: Some(30),
                total_tokens: Some(70),
                estimated_cost_usd: Some(0.01),
                created_at: day_start + 10,
                ..RequestTokenStat::default()
            },
        )
        .expect("insert member one log");
    storage
        .insert_request_log_with_token_stat(
            &RequestLog {
                trace_id: Some("trace-two".to_string()),
                key_id: Some(key_two.id.clone()),
                request_path: "/v1/chat/completions".to_string(),
                method: "POST".to_string(),
                model: Some("gpt-5-mini".to_string()),
                status_code: Some(200),
                input_tokens: Some(400),
                cached_input_tokens: Some(0),
                output_tokens: Some(300),
                total_tokens: Some(700),
                estimated_cost_usd: Some(0.1),
                created_at: day_start + 20,
                ..RequestLog::default()
            },
            &RequestTokenStat {
                key_id: Some(key_two.id.clone()),
                model: Some("gpt-5-mini".to_string()),
                input_tokens: Some(400),
                output_tokens: Some(300),
                total_tokens: Some(700),
                estimated_cost_usd: Some(0.1),
                created_at: day_start + 20,
                ..RequestTokenStat::default()
            },
        )
        .expect("insert member two log");

    let resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/memberSummary",
            serde_json::json!({
                "dayStartTs": day_start,
                "dayEndTs": day_end
            }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id)),
    ));

    assert!(resp.result.get("error").is_none(), "{:?}", resp.result);
    assert_eq!(resp.result["apiKeySummary"]["totalCount"], 1);
    assert_eq!(resp.result["usageToday"]["totalTokens"], 70);
    assert_eq!(resp.result["recentLogs"][0]["keyId"], key_one.id);
    assert_eq!(resp.result["topKeys"][0]["keyId"], key_one.id);
    assert_eq!(resp.result["topKeys"][0]["todayTokens"], 70);
    assert_eq!(resp.result["topModels"][0]["model"], "gpt-5-mini");
    assert_eq!(resp.result["topModels"][0]["totalTokens"], 70);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_dashboard_counts_owned_keys_across_lookup_batches() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-dashboard-key-batches");
    let user = create_test_member("member-key-batches", Some(2_000_000));

    for index in 0..251 {
        create_owned_test_api_key(
            &user.id,
            &format!("member batch key {index:03}"),
            "gpt-5-mini",
        );
    }

    let resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/memberSummary",
            serde_json::json!({
                "dayStartTs": 1_700_000_000,
                "dayEndTs": 1_700_086_400
            }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user.id)),
    ));

    assert!(resp.result.get("error").is_none(), "{:?}", resp.result);
    assert_eq!(resp.result["apiKeySummary"]["totalCount"], 251);
    assert_eq!(resp.result["usageToday"]["totalTokens"], 0);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_dashboard_no_key_returns_alert() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-dashboard-empty");
    let user = create_app_user(AppUserCreateInput {
        username: "member-empty".to_string(),
        password: "password-empty".to_string(),
        display_name: None,
        role: Some(ROLE_MEMBER.to_string()),
        initial_balance_credit_micros: None,
    })
    .expect("create member");

    let resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/memberSummary",
            serde_json::json!({
                "dayStartTs": 1_700_000_000,
                "dayEndTs": 1_700_086_400
            }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user.id)),
    ));

    assert!(resp.result.get("error").is_none(), "{:?}", resp.result);
    assert_eq!(resp.result["apiKeySummary"]["totalCount"], 0);
    assert!(resp.result["alerts"]
        .as_array()
        .map(|items| items.iter().any(|item| item["kind"] == "no_api_key"))
        .unwrap_or(false));

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_dashboard_ignores_requested_user_id() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-dashboard-user-id-spoof");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user_one = create_test_member("member-spoof-one", Some(2_000_000));
    let user_two = create_test_member("member-spoof-two", Some(2_000_000));
    let key_one = create_owned_test_api_key(&user_one.id, "member spoof one key", "gpt-5-mini");
    let key_two = create_owned_test_api_key(&user_two.id, "member spoof two key", "gpt-5-mini");

    insert_test_request_log(
        &key_one,
        "trace-spoof-one",
        "gpt-5-mini",
        200,
        12,
        2,
        8,
        0.01,
        day_start + 10,
    );
    insert_test_request_log(
        &key_two,
        "trace-spoof-two",
        "gpt-5-mini",
        200,
        1200,
        0,
        800,
        1.0,
        day_start + 20,
    );

    let resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/memberSummary",
            serde_json::json!({
                "userId": user_two.id,
                "dayStartTs": day_start,
                "dayEndTs": day_end
            }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id)),
    ));

    assert!(resp.result.get("error").is_none(), "{:?}", resp.result);
    assert_eq!(resp.result["userId"], user_one.id);
    assert_eq!(resp.result["apiKeySummary"]["totalCount"], 1);
    assert_eq!(resp.result["usageToday"]["totalTokens"], 20);
    assert_eq!(resp.result["recentLogs"][0]["keyId"], key_one);
    assert_eq!(resp.result["topKeys"][0]["keyId"], key_one);
    assert_ne!(resp.result["recentLogs"][0]["keyId"], key_two);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn admin_member_dashboard_can_query_requested_user() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-admin-member-dashboard-debug");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user_one = create_test_member("admin-debug-one", Some(2_000_000));
    let user_two = create_test_member("admin-debug-two", Some(2_000_000));
    let key_one = create_owned_test_api_key(&user_one.id, "admin debug one key", "gpt-5-mini");
    let key_two = create_owned_test_api_key(&user_two.id, "admin debug two key", "gpt-5-mini");

    insert_test_request_log(
        &key_one,
        "trace-admin-debug-one",
        "gpt-5-mini",
        200,
        10,
        0,
        10,
        0.01,
        day_start + 10,
    );
    insert_test_request_log(
        &key_two,
        "trace-admin-debug-two",
        "gpt-5-mini",
        200,
        40,
        5,
        20,
        0.02,
        day_start + 20,
    );

    let resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/memberSummary",
            serde_json::json!({
                "userId": user_two.id,
                "dayStartTs": day_start,
                "dayEndTs": day_end
            }),
        ),
        RpcActor::system_admin(),
    ));

    assert!(resp.result.get("error").is_none(), "{:?}", resp.result);
    assert_eq!(resp.result["userId"], user_two.id);
    assert_eq!(resp.result["apiKeySummary"]["totalCount"], 1);
    assert_eq!(resp.result["usageToday"]["totalTokens"], 60);
    assert_eq!(resp.result["recentLogs"][0]["keyId"], key_two);
    assert_ne!(resp.result["recentLogs"][0]["keyId"], key_one);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn admin_usage_summary_requires_admin_and_returns_range_rollups() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-admin-usage-summary");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user = create_test_member("admin-usage-member", Some(2_000_000));
    let key_id = create_owned_test_api_key(&user.id, "admin usage key", "gpt-5-mini");

    insert_test_request_log(
        &key_id,
        "trace-admin-usage",
        "gpt-5-mini",
        200,
        20,
        5,
        10,
        0.03,
        day_start + 10,
    );

    let member_resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/adminUsageSummary",
            serde_json::json!({
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user.id)),
    ));
    assert!(
        rpc_error(&member_resp).contains("permission_denied"),
        "{:?}",
        member_resp.result
    );

    let admin_resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/adminUsageSummary",
            serde_json::json!({
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        RpcActor::system_admin(),
    ));
    assert!(
        admin_resp.result.get("error").is_none(),
        "{:?}",
        admin_resp.result
    );
    assert_eq!(admin_resp.result["rangeStartTs"], day_start);
    assert_eq!(admin_resp.result["rangeEndTs"], day_end);
    assert_eq!(admin_resp.result["dailyUsage"].as_array().unwrap().len(), 1);
    assert_eq!(
        admin_resp.result["dailyUsage"][0]["usage"]["totalTokens"],
        30
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][0]["usage"]["requestCount"],
        1
    );

    let user_item = admin_resp.result["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["userId"] == user.id)
        .expect("user usage item");
    assert_eq!(user_item["rangeUsage"]["totalTokens"], 30);

    let account_item = admin_resp.result["openaiAccounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["sourceId"] == "private-account-id")
        .expect("account usage item");
    assert_eq!(account_item["rangeUsage"]["totalTokens"], 30);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn admin_usage_summary_ranking_limit_bounds_top_results() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-admin-usage-ranking-limit");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user_low = create_test_member("admin-ranking-low", Some(2_000_000));
    let user_high = create_test_member("admin-ranking-high", Some(2_000_000));
    let key_low = create_owned_test_api_key(&user_low.id, "admin ranking low key", "gpt-5-mini");
    let key_high = create_owned_test_api_key(&user_high.id, "admin ranking high key", "gpt-5-mini");
    let storage = storage_helpers::open_storage().expect("open storage");
    let now = codexmanager_core::storage::now_ts();

    for (id, label) in [
        ("ranking-account-low", "Ranking Account Low"),
        ("ranking-account-high", "Ranking Account High"),
    ] {
        storage
            .insert_account(&codexmanager_core::storage::Account {
                id: id.to_string(),
                label: label.to_string(),
                issuer: "https://auth.openai.com".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: 0,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            })
            .expect("insert ranking account");
    }
    for (id, name) in [
        ("ranking-aggregate-low", "Ranking Aggregate Low"),
        ("ranking-aggregate-high", "Ranking Aggregate High"),
    ] {
        storage
            .insert_aggregate_api(&codexmanager_core::storage::AggregateApi {
                id: id.to_string(),
                provider_type: "codex".to_string(),
                supplier_name: Some(name.to_string()),
                sort: 0,
                url: format!("https://{id}.example.invalid/v1"),
                auth_type: "apikey".to_string(),
                auth_params_json: None,
                action: None,
                model_override: None,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
                last_test_at: None,
                last_test_status: None,
                last_test_error: None,
                balance_query_enabled: false,
                balance_query_template: None,
                balance_query_base_url: None,
                balance_query_user_id: None,
                balance_query_config_json: None,
                last_balance_at: None,
                last_balance_status: None,
                last_balance_error: None,
                last_balance_json: None,
            })
            .expect("insert ranking aggregate");
    }

    for (trace_id, key_id, account_id, aggregate_id, input_tokens, output_tokens) in [
        (
            "trace-ranking-low",
            key_low.as_str(),
            "ranking-account-low",
            "ranking-aggregate-low",
            10,
            5,
        ),
        (
            "trace-ranking-high",
            key_high.as_str(),
            "ranking-account-high",
            "ranking-aggregate-high",
            90,
            30,
        ),
    ] {
        let total_tokens = input_tokens + output_tokens;
        storage
            .insert_request_log_with_token_stat(
                &RequestLog {
                    trace_id: Some(trace_id.to_string()),
                    key_id: Some(key_id.to_string()),
                    account_id: Some(account_id.to_string()),
                    initial_aggregate_api_id: Some(aggregate_id.to_string()),
                    request_path: "/v1/responses".to_string(),
                    method: "POST".to_string(),
                    model: Some("gpt-5-mini".to_string()),
                    actual_source_kind: Some("openai_account".to_string()),
                    actual_source_id: Some(account_id.to_string()),
                    status_code: Some(200),
                    input_tokens: Some(input_tokens),
                    output_tokens: Some(output_tokens),
                    total_tokens: Some(total_tokens),
                    created_at: day_start + total_tokens,
                    ..RequestLog::default()
                },
                &RequestTokenStat {
                    key_id: Some(key_id.to_string()),
                    account_id: Some(account_id.to_string()),
                    model: Some("gpt-5-mini".to_string()),
                    input_tokens: Some(input_tokens),
                    cached_input_tokens: Some(0),
                    output_tokens: Some(output_tokens),
                    total_tokens: Some(total_tokens),
                    estimated_cost_usd: Some(total_tokens as f64 / 1000.0),
                    created_at: day_start + total_tokens,
                    ..RequestTokenStat::default()
                },
            )
            .expect("insert ranking account usage");
        storage
            .insert_request_log_with_token_stat(
                &RequestLog {
                    trace_id: Some(format!("{trace_id}-aggregate")),
                    key_id: Some(key_id.to_string()),
                    initial_aggregate_api_id: Some(aggregate_id.to_string()),
                    request_path: "/v1/responses".to_string(),
                    method: "POST".to_string(),
                    model: Some("gpt-5-mini".to_string()),
                    actual_source_kind: Some("aggregate_api".to_string()),
                    actual_source_id: Some(aggregate_id.to_string()),
                    status_code: Some(200),
                    input_tokens: Some(input_tokens),
                    output_tokens: Some(output_tokens),
                    total_tokens: Some(total_tokens),
                    created_at: day_start + total_tokens + 10,
                    ..RequestLog::default()
                },
                &RequestTokenStat {
                    key_id: Some(key_id.to_string()),
                    model: Some("gpt-5-mini".to_string()),
                    input_tokens: Some(input_tokens),
                    cached_input_tokens: Some(0),
                    output_tokens: Some(output_tokens),
                    total_tokens: Some(total_tokens),
                    estimated_cost_usd: Some(total_tokens as f64 / 1000.0),
                    created_at: day_start + total_tokens + 10,
                    ..RequestTokenStat::default()
                },
            )
            .expect("insert ranking aggregate usage");
    }

    let admin_resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/adminUsageSummary",
            serde_json::json!({
                "startTs": day_start,
                "endTs": day_end,
                "rankingLimit": 1
            }),
        ),
        RpcActor::system_admin(),
    ));
    assert!(
        admin_resp.result.get("error").is_none(),
        "{:?}",
        admin_resp.result
    );
    assert_eq!(admin_resp.result["dailyUsage"].as_array().unwrap().len(), 1);
    assert_eq!(
        admin_resp.result["dailyUsage"][0]["usage"]["totalTokens"],
        270
    );

    let users = admin_resp.result["users"].as_array().expect("users");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["userId"], user_high.id);
    assert_eq!(users[0]["rangeUsage"]["totalTokens"], 240);

    let openai_accounts = admin_resp.result["openaiAccounts"]
        .as_array()
        .expect("openai accounts");
    assert_eq!(openai_accounts.len(), 1);
    assert_eq!(openai_accounts[0]["sourceId"], "ranking-account-high");
    assert_eq!(openai_accounts[0]["name"], "Ranking Account High");

    let aggregate_apis = admin_resp.result["aggregateApis"]
        .as_array()
        .expect("aggregate apis");
    assert_eq!(aggregate_apis.len(), 1);
    assert_eq!(aggregate_apis[0]["sourceId"], "ranking-aggregate-high");
    assert_eq!(aggregate_apis[0]["name"], "Ranking Aggregate High");

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn admin_usage_summary_daily_trend_includes_token_stats_without_request_logs() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-admin-usage-orphan-stats");
    let day_start = 1_700_000_000;
    let day_end = day_start + 3 * 86_400;
    let user = create_test_member("admin-usage-orphan-member", Some(2_000_000));
    let key_id = create_owned_test_api_key(&user.id, "admin orphan key", "gpt-5-mini");
    let storage = storage_helpers::open_storage().expect("open storage");

    storage
        .insert_request_token_stat(&RequestTokenStat {
            request_log_id: 98_001,
            key_id: Some(key_id.clone()),
            account_id: Some("private-account-id".to_string()),
            model: Some("gpt-5-mini".to_string()),
            input_tokens: Some(400),
            cached_input_tokens: Some(0),
            output_tokens: Some(100),
            total_tokens: Some(500),
            estimated_cost_usd: Some(0.5),
            created_at: day_start + 120,
            ..RequestTokenStat::default()
        })
        .expect("insert orphan day one stat");
    storage
        .insert_request_token_stat(&RequestTokenStat {
            request_log_id: 98_002,
            key_id: Some(key_id.clone()),
            account_id: Some("private-account-id".to_string()),
            model: Some("gpt-5-mini".to_string()),
            input_tokens: Some(300),
            cached_input_tokens: Some(0),
            output_tokens: Some(100),
            total_tokens: Some(400),
            estimated_cost_usd: Some(0.4),
            created_at: day_start + 86_400 + 180,
            ..RequestTokenStat::default()
        })
        .expect("insert orphan day two stat");

    insert_test_request_log(
        &key_id,
        "trace-admin-usage-current-day",
        "gpt-5-mini",
        200,
        20,
        5,
        10,
        0.03,
        day_start + 2 * 86_400 + 240,
    );

    let admin_resp = response_result(handle_request_with_actor(
        rpc_request(
            "dashboard/adminUsageSummary",
            serde_json::json!({
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        RpcActor::system_admin(),
    ));
    assert!(
        admin_resp.result.get("error").is_none(),
        "{:?}",
        admin_resp.result
    );
    assert_eq!(admin_resp.result["dailyUsage"].as_array().unwrap().len(), 3);
    assert_eq!(
        admin_resp.result["dailyUsage"][0]["usage"]["totalTokens"],
        500
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][0]["usage"]["requestCount"],
        0
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][1]["usage"]["totalTokens"],
        400
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][1]["usage"]["requestCount"],
        0
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][2]["usage"]["totalTokens"],
        30
    );
    assert_eq!(
        admin_resp.result["dailyUsage"][2]["usage"]["requestCount"],
        1
    );

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_cannot_read_or_mutate_other_user_api_key() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-apikey-cross-user-deny");
    let user_one = create_test_member("apikey-deny-one", Some(2_000_000));
    let user_two = create_test_member("apikey-deny-two", Some(2_000_000));
    let key_one = create_owned_test_api_key(&user_one.id, "member one private key", "gpt-5-mini");
    let key_two = create_owned_test_api_key(&user_two.id, "member two private key", "gpt-5-mini");
    let actor_one = RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id));

    for (method, params) in [
        ("apikey/readSecret", serde_json::json!({ "id": key_two })),
        (
            "apikey/updateModel",
            serde_json::json!({ "id": key_two, "name": "stolen", "modelSlug": "gpt-5" }),
        ),
        ("apikey/disable", serde_json::json!({ "id": key_two })),
        ("apikey/delete", serde_json::json!({ "id": key_two })),
    ] {
        let resp = response_result(handle_request_with_actor(
            rpc_request(method, params),
            actor_one.clone(),
        ));
        assert!(
            rpc_error(&resp).contains("permission_denied"),
            "{method} should deny cross-user access: {:?}",
            resp.result
        );
    }

    let member_one_list = response_result(handle_request_with_actor(
        rpc_request("apikey/list", serde_json::json!({})),
        actor_one,
    ));
    assert_eq!(member_one_list.result["items"].as_array().unwrap().len(), 1);
    assert_eq!(member_one_list.result["items"][0]["id"], key_one);

    let member_two_list = response_result(handle_request_with_actor(
        rpc_request("apikey/list", serde_json::json!({})),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_two.id)),
    ));
    assert_eq!(member_two_list.result["items"].as_array().unwrap().len(), 1);
    assert_eq!(member_two_list.result["items"][0]["id"], key_two);
    assert_eq!(
        member_two_list.result["items"][0]["name"],
        "member two private key"
    );
    assert_eq!(member_two_list.result["items"][0]["status"], "active");

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_api_key_lookup_filters_to_owned_ids() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-apikey-lookup-filter");
    let user_one = create_test_member("apikey-lookup-one", Some(2_000_000));
    let user_two = create_test_member("apikey-lookup-two", Some(2_000_000));
    let key_one = create_owned_test_api_key(&user_one.id, "lookup owned key", "gpt-5-mini");
    let key_two = create_owned_test_api_key(&user_two.id, "lookup foreign key", "gpt-5-mini");
    let actor_one = RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id));

    let member_lookup = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/lookup",
            serde_json::json!({
                "ids": [key_two.clone(), key_one.clone(), "missing-key", key_one.clone()]
            }),
        ),
        actor_one,
    ));
    let member_items = member_lookup.result.as_array().expect("lookup items");
    assert_eq!(member_items.len(), 1);
    assert_eq!(member_items[0]["id"], key_one);
    assert_eq!(member_items[0]["name"], "lookup owned key");

    let admin_lookup = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/lookup",
            serde_json::json!({ "ids": [key_two.clone(), key_one.clone()] }),
        ),
        RpcActor::system_admin(),
    ));
    let admin_ids = admin_lookup
        .result
        .as_array()
        .expect("admin lookup items")
        .iter()
        .filter_map(|item| item.get("id").and_then(|value| value.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        admin_ids,
        std::collections::BTreeSet::from([key_one.as_str(), key_two.as_str()])
    );

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn account_lookup_is_admin_only_and_filters_requested_ids() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-account-lookup-admin-only");
    let storage = storage_helpers::open_storage().expect("open storage");
    let now = codexmanager_core::storage::now_ts();
    for (id, label, sort) in [("acc-a", "Account A", 2), ("acc-b", "Account B", 1)] {
        storage
            .insert_account(&codexmanager_core::storage::Account {
                id: id.to_string(),
                label: label.to_string(),
                issuer: "https://auth.openai.com".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort,
                status: "active".to_string(),
                created_at: now,
                updated_at: now + sort,
            })
            .expect("insert account");
    }

    let member = create_test_member("account-lookup-member", Some(2_000_000));
    let member_resp = response_result(handle_request_with_actor(
        rpc_request("account/lookup", serde_json::json!({ "ids": ["acc-a"] })),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&member.id)),
    ));
    assert!(rpc_error(&member_resp).contains("permission_denied"));

    let admin_resp = response_result(handle_request_with_actor(
        rpc_request(
            "account/lookup",
            serde_json::json!({ "ids": ["acc-a", "missing", "acc-a", "acc-b"] }),
        ),
        RpcActor::system_admin(),
    ));
    let items = admin_resp.result.as_array().expect("account lookup items");
    assert_eq!(items.len(), 2);
    let ids = items
        .iter()
        .filter_map(|item| item.get("id").and_then(|value| value.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids, std::collections::BTreeSet::from(["acc-a", "acc-b"]));

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn account_list_bare_rpc_defaults_to_first_page() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-account-list-default-page");
    let storage = storage_helpers::open_storage().expect("open storage");
    let now = codexmanager_core::storage::now_ts();
    for index in 0..6 {
        storage
            .insert_account(&codexmanager_core::storage::Account {
                id: format!("acc-default-page-{index}"),
                label: format!("Account Default Page {index}"),
                issuer: "https://auth.openai.com".to_string(),
                chatgpt_account_id: None,
                workspace_id: None,
                group_name: None,
                sort: index,
                status: "active".to_string(),
                created_at: now + index,
                updated_at: now + index,
            })
            .expect("insert account");
    }

    let listed = response_result(handle_request_with_actor(
        rpc_request("account/list", serde_json::json!({})),
        RpcActor::system_admin(),
    ));
    assert!(listed.result.get("error").is_none(), "{:?}", listed.result);
    assert_eq!(listed.result["total"], 6);
    assert_eq!(listed.result["page"], 1);
    assert_eq!(listed.result["pageSize"], 5);
    assert_eq!(listed.result["items"].as_array().unwrap().len(), 5);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn aggregate_api_lookup_is_admin_only_and_filters_requested_ids() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-aggregate-api-lookup-admin-only");
    let storage = storage_helpers::open_storage().expect("open storage");
    let now = codexmanager_core::storage::now_ts();
    for (id, supplier_name, sort) in [("agg-a", "Aggregate A", 2), ("agg-b", "Aggregate B", 1)] {
        storage
            .insert_aggregate_api(&codexmanager_core::storage::AggregateApi {
                id: id.to_string(),
                provider_type: "codex".to_string(),
                supplier_name: Some(supplier_name.to_string()),
                sort,
                url: format!("https://{id}.example.invalid/v1"),
                auth_type: "apikey".to_string(),
                auth_params_json: None,
                action: None,
                model_override: None,
                status: "active".to_string(),
                created_at: now,
                updated_at: now + sort,
                last_test_at: None,
                last_test_status: None,
                last_test_error: None,
                balance_query_enabled: false,
                balance_query_template: None,
                balance_query_base_url: None,
                balance_query_user_id: None,
                balance_query_config_json: None,
                last_balance_at: None,
                last_balance_status: None,
                last_balance_error: None,
                last_balance_json: None,
            })
            .expect("insert aggregate api");
    }

    let member = create_test_member("aggregate-lookup-member", Some(2_000_000));
    let member_resp = response_result(handle_request_with_actor(
        rpc_request(
            "aggregateApi/lookup",
            serde_json::json!({ "ids": ["agg-a"] }),
        ),
        RpcActor::from_parts(Some(ROLE_MEMBER), Some(&member.id)),
    ));
    assert!(rpc_error(&member_resp).contains("permission_denied"));

    let admin_resp = response_result(handle_request_with_actor(
        rpc_request(
            "aggregateApi/lookup",
            serde_json::json!({ "ids": ["agg-a", "missing", "agg-b", "agg-a"] }),
        ),
        RpcActor::system_admin(),
    ));
    let items = admin_resp
        .result
        .as_array()
        .expect("aggregate lookup items");
    assert_eq!(items.len(), 2);
    let ids = items
        .iter()
        .filter_map(|item| item.get("id").and_then(|value| value.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids, std::collections::BTreeSet::from(["agg-a", "agg-b"]));

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_created_api_key_ignores_admin_only_routing_fields() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-apikey-create-sanitizes");
    let user = create_test_member("apikey-create-sanitize", Some(2_000_000));
    let actor = RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user.id));

    let created = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/create",
            serde_json::json!({
                "name": "member safe key",
                "modelSlug": "gpt-5-mini",
                "rotationStrategy": "aggregate_api_rotation",
                "aggregateApiId": "agg-secret",
                "upstreamBaseUrl": "https://example.invalid/v1",
                "staticHeadersJson": "{\"x-admin\":\"secret\"}",
                "accountPlanFilter": "pro"
            }),
        ),
        actor.clone(),
    ));
    assert!(
        created.result.get("error").is_none(),
        "{:?}",
        created.result
    );

    let listed = response_result(handle_request_with_actor(
        rpc_request("apikey/list", serde_json::json!({})),
        actor,
    ));
    assert_eq!(listed.result["items"].as_array().unwrap().len(), 1);
    let item = &listed.result["items"][0];
    assert_eq!(item["id"], created.result["id"]);
    assert_eq!(item["rotationStrategy"], "account_rotation");
    assert!(item["aggregateApiId"].is_null());
    assert!(item["upstreamBaseUrl"].is_null());
    assert!(item["staticHeadersJson"].is_null());
    assert!(item["accountPlanFilter"].is_null());

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_api_key_list_supports_backend_pagination_and_filters() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-apikey-list-pagination");
    let user_one = create_test_member("apikey-page-one", Some(2_000_000));
    let user_two = create_test_member("apikey-page-two", Some(2_000_000));
    let key_alpha = create_owned_test_api_key(&user_one.id, "alpha owned key", "gpt-5-mini");
    let key_beta = create_owned_test_api_key(&user_one.id, "beta owned key", "gpt-5-mini");
    let key_disabled = create_owned_test_api_key(&user_one.id, "disabled owned key", "gpt-5-mini");
    let key_foreign = create_owned_test_api_key(&user_two.id, "alpha foreign key", "gpt-5-mini");
    let storage = storage_helpers::open_storage().expect("open storage");
    storage
        .update_api_key_status(&key_disabled, "disabled")
        .expect("disable owned key");
    let actor_one = RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id));

    let first_page = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/list",
            serde_json::json!({ "page": 1, "pageSize": 2 }),
        ),
        actor_one.clone(),
    ));
    assert_eq!(first_page.result["total"], 3);
    assert_eq!(first_page.result["page"], 1);
    assert_eq!(first_page.result["pageSize"], 2);
    let first_page_items = first_page.result["items"].as_array().unwrap();
    assert_eq!(first_page_items.len(), 2);
    assert!(first_page_items
        .iter()
        .all(|item| item["id"] != key_foreign));

    let alpha_page = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/list",
            serde_json::json!({ "page": 1, "pageSize": 20, "query": "alpha" }),
        ),
        actor_one.clone(),
    ));
    assert_eq!(alpha_page.result["total"], 1);
    assert_eq!(alpha_page.result["items"][0]["id"], key_alpha);

    let disabled_page = response_result(handle_request_with_actor(
        rpc_request(
            "apikey/list",
            serde_json::json!({ "page": 1, "pageSize": 20, "statusFilter": "disabled" }),
        ),
        actor_one.clone(),
    ));
    assert_eq!(disabled_page.result["total"], 1);
    assert_eq!(disabled_page.result["items"][0]["id"], key_disabled);

    let default_page = response_result(handle_request_with_actor(
        rpc_request("apikey/list", serde_json::json!({})),
        actor_one,
    ));
    assert_eq!(default_page.result["total"], 3);
    assert_eq!(default_page.result["page"], 1);
    assert_eq!(default_page.result["pageSize"], 20);
    let default_page_ids = default_page.result["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(default_page_ids.contains(&key_alpha));
    assert!(default_page_ids.contains(&key_beta));
    assert!(default_page_ids.contains(&key_disabled));
    assert!(!default_page_ids.contains(&key_foreign));

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn member_requestlog_queries_filter_to_owned_keys() {
    let _guard = test_env_guard();
    let db_path = setup_dashboard_test_db("codexmanager-member-requestlog-filter");
    let day_start = 1_700_000_000;
    let day_end = day_start + 86_400;
    let user_one = create_test_member("log-filter-one", Some(2_000_000));
    let user_two = create_test_member("log-filter-two", Some(2_000_000));
    let key_one = create_owned_test_api_key(&user_one.id, "log filter one key", "gpt-5-mini");
    let key_two = create_owned_test_api_key(&user_two.id, "log filter two key", "gpt-5-mini");
    let actor_one = RpcActor::from_parts(Some(ROLE_MEMBER), Some(&user_one.id));

    insert_test_request_log(
        &key_one,
        "trace-log-filter-one",
        "gpt-5-mini",
        200,
        30,
        10,
        20,
        0.03,
        day_start + 10,
    );
    insert_test_request_log(
        &key_two,
        "trace-log-filter-two",
        "gpt-5",
        500,
        300,
        0,
        200,
        0.3,
        day_start + 20,
    );

    let list = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/list",
            serde_json::json!({
                "page": 1,
                "pageSize": 20,
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        actor_one.clone(),
    ));
    assert!(list.result.get("error").is_none(), "{:?}", list.result);
    assert_eq!(list.result["total"], 1);
    assert_eq!(list.result["items"][0]["keyId"], key_one);
    assert_eq!(list.result["items"][0]["model"], "gpt-5-mini");
    assert!(list.result["items"][0]["upstreamModel"].is_null());
    assert!(list.result["items"][0]["actualSourceKind"].is_null());
    assert!(list.result["items"][0]["actualSourceId"].is_null());

    let hidden_model_query = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/list",
            serde_json::json!({
                "page": 1,
                "pageSize": 20,
                "query": "upstream_model:=gpt-5-mini-upstream",
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        actor_one.clone(),
    ));
    assert!(
        hidden_model_query.result.get("error").is_none(),
        "{:?}",
        hidden_model_query.result
    );
    assert_eq!(hidden_model_query.result["total"], 0);

    let hidden_model_global_query = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/list",
            serde_json::json!({
                "page": 1,
                "pageSize": 20,
                "query": "gpt-5-mini-upstream",
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        actor_one.clone(),
    ));
    assert!(
        hidden_model_global_query.result.get("error").is_none(),
        "{:?}",
        hidden_model_global_query.result
    );
    assert_eq!(hidden_model_global_query.result["total"], 0);

    let admin_list = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/list",
            serde_json::json!({
                "page": 1,
                "pageSize": 20,
                "query": "upstream_model:=gpt-5-mini-upstream",
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        RpcActor::system_admin(),
    ));
    assert!(
        admin_list.result.get("error").is_none(),
        "{:?}",
        admin_list.result
    );
    assert_eq!(admin_list.result["total"], 1);
    assert_eq!(
        admin_list.result["items"][0]["upstreamModel"],
        "gpt-5-mini-upstream"
    );
    assert_eq!(
        admin_list.result["items"][0]["actualSourceKind"],
        "openai_account"
    );
    assert_eq!(
        admin_list.result["items"][0]["actualSourceId"],
        "private-account-id"
    );

    let summary = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/summary",
            serde_json::json!({
                "page": 1,
                "pageSize": 20,
                "startTs": day_start,
                "endTs": day_end
            }),
        ),
        actor_one.clone(),
    ));
    assert!(
        summary.result.get("error").is_none(),
        "{:?}",
        summary.result
    );
    assert_eq!(summary.result["totalCount"], 1);
    assert_eq!(summary.result["filteredCount"], 1);
    assert_eq!(summary.result["successCount"], 1);
    assert_eq!(summary.result["errorCount"], 0);
    assert_eq!(summary.result["totalTokens"], 50);

    let today = response_result(handle_request_with_actor(
        rpc_request(
            "requestlog/today_summary",
            serde_json::json!({
                "dayStartTs": day_start,
                "dayEndTs": day_end
            }),
        ),
        actor_one.clone(),
    ));
    assert!(today.result.get("error").is_none(), "{:?}", today.result);
    assert_eq!(today.result["todayTokens"], 40);
    assert_eq!(today.result["estimatedCost"], 0.03);

    let clear = response_result(handle_request_with_actor(
        rpc_request("requestlog/clear", serde_json::json!({})),
        actor_one,
    ));
    assert!(
        rpc_error(&clear).contains("permission_denied"),
        "member must not clear global logs: {:?}",
        clear.result
    );

    let _ = std::fs::remove_file(db_path);
}
