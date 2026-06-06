use codexmanager_core::rpc::types::{JsonRpcRequest, JsonRpcResponse};

use crate::quota::read::{
    self, BillingRuleUpsertInput, QuotaModelPoolSourcesInput, QuotaModelPoolsInput,
    QuotaRefreshSourcesInput, QuotaSourceListInput,
};

fn string_array_param(req: &JsonRpcRequest, key: &str) -> Vec<String> {
    req.params
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn try_handle(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "quota/overview" => super::value_or_error(read::read_quota_overview()),
        "quota/modelUsage" => {
            let start_ts = super::i64_param(req, "startTs");
            let end_ts = super::i64_param(req, "endTs");
            super::value_or_error(read::read_quota_model_usage(start_ts, end_ts))
        }
        "quota/apiKeyUsage" => super::value_or_error(read::read_quota_api_key_usage()),
        "quota/sourceList" => {
            let source_ids = req
                .params
                .as_ref()
                .and_then(|value| value.get("sourceIds"))
                .and_then(|value| value.as_array())
                .map(|_| string_array_param(req, "sourceIds"));
            super::value_or_error(read::read_quota_source_list(QuotaSourceListInput {
                source_kind: super::string_param(req, "sourceKind"),
                source_ids,
                page: super::i64_param(req, "page"),
                page_size: super::i64_param(req, "pageSize"),
            }))
        }
        "quota/modelPools" => {
            super::value_or_error(read::read_quota_model_pools(QuotaModelPoolsInput {
                include_sources: super::bool_param(req, "includeSources").unwrap_or(false),
                include_config: super::bool_param(req, "includeConfig").unwrap_or(false),
                source_kind: super::string_param(req, "sourceKind"),
            }))
        }
        "quota/modelPoolSources" => {
            let source_ids = req
                .params
                .as_ref()
                .and_then(|value| value.get("sourceIds"))
                .and_then(|value| value.as_array())
                .map(|_| string_array_param(req, "sourceIds"));
            super::value_or_error(read::read_quota_model_pool_sources(
                QuotaModelPoolSourcesInput {
                    source_kind: super::string_param(req, "sourceKind"),
                    source_ids,
                    page: super::i64_param(req, "page"),
                    page_size: super::i64_param(req, "pageSize"),
                },
            ))
        }
        "quota/systemPool" => {
            let reference_model = super::string_param(req, "referenceModel");
            super::value_or_error(read::read_quota_system_pool(reference_model))
        }
        "quota/capacityConfig" => super::value_or_error(read::read_quota_capacity_config()),
        "quota/billingRules" => super::value_or_error(read::read_billing_rules()),
        "quota/billingRule/upsert" => {
            super::value_or_error(read::upsert_billing_rule(BillingRuleUpsertInput {
                id: super::string_param(req, "id"),
                name: super::str_param(req, "name").unwrap_or("").to_string(),
                status: super::string_param(req, "status"),
                priority: super::i64_param(req, "priority"),
                multiplier_millis: super::i64_param(req, "multiplierMillis").unwrap_or(1_000),
                model_pattern: super::string_param(req, "modelPattern"),
                service_tier: super::string_param(req, "serviceTier"),
                user_id: super::string_param(req, "userId"),
                project_id: super::string_param(req, "projectId"),
                api_key_id: super::string_param(req, "apiKeyId"),
                starts_at: super::i64_param(req, "startsAt"),
                ends_at: super::i64_param(req, "endsAt"),
            }))
        }
        "quota/billingRule/delete" => {
            let id = super::str_param(req, "id").unwrap_or("");
            super::value_or_error(read::delete_billing_rule(id))
        }
        "quota/sourceModels/set" => {
            let source_kind = super::str_param(req, "sourceKind").unwrap_or("");
            let source_id = super::str_param(req, "sourceId").unwrap_or("");
            super::value_or_error(read::set_quota_source_models(
                source_kind,
                source_id,
                string_array_param(req, "modelSlugs"),
            ))
        }
        "quota/capacityTemplate/update" => {
            let plan_type = super::str_param(req, "planType").unwrap_or("");
            super::value_or_error(read::update_account_quota_capacity_template(
                plan_type,
                super::i64_param(req, "primaryWindowTokens"),
                super::i64_param(req, "secondaryWindowTokens"),
            ))
        }
        "quota/accountCapacityOverride/update" => {
            let account_id = super::str_param(req, "accountId").unwrap_or("");
            super::value_or_error(read::update_account_quota_capacity_override(
                account_id,
                super::i64_param(req, "primaryWindowTokens"),
                super::i64_param(req, "secondaryWindowTokens"),
            ))
        }
        "quota/refreshSources" => {
            super::value_or_error(read::refresh_quota_sources(QuotaRefreshSourcesInput {
                kinds: string_array_param(req, "kinds"),
                source_ids: string_array_param(req, "sourceIds"),
                refresh_all: super::bool_param(req, "refreshAll").unwrap_or(false),
            }))
        }
        _ => return None,
    };

    Some(super::response(req, result))
}
