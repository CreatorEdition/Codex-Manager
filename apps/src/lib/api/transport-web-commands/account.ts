import type { WebCommandDescriptor, WebRpcCaller } from "./shared";
import {
  WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
  asRecord,
  noRetryTimeoutOptions,
} from "./shared";
import { exportAccountsViaBrowser, pickImportFilesFromBrowser } from "./browser-direct";

export function createAccountWebCommands(postWebRpc: WebRpcCaller): Record<string, WebCommandDescriptor> {
  return {
    service_account_list: { rpcMethod: "account/list" },
    service_account_delete: { rpcMethod: "account/delete" },
    service_account_delete_many: {
      rpcMethod: "account/deleteMany",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/deleteMany 超时：批量删除账号超过 120 秒",
      ),
    },
    service_account_delete_by_statuses: {
      rpcMethod: "account/deleteByStatuses",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/deleteByStatuses 超时：按状态清理账号超过 120 秒",
      ),
    },
    service_account_delete_unavailable_free: {
      rpcMethod: "account/deleteUnavailableFree",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/deleteUnavailableFree 超时：清理不可用免费账号超过 120 秒",
      ),
    },
    service_account_update: { rpcMethod: "account/update" },
    service_account_import: {
      rpcMethod: "account/import",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/import 超时：账号批量导入超过 120 秒",
      ),
    },
    service_account_import_by_file: { direct: () => pickImportFilesFromBrowser(false) },
    service_account_import_by_directory: { direct: () => pickImportFilesFromBrowser(true) },
    service_account_export_by_account_files: {
      direct: (params, options) => exportAccountsViaBrowser(postWebRpc, asRecord(params), options),
    },
    service_account_warmup: {
      rpcMethod: "account/warmup",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/warmup 超时：账号预热超过 120 秒",
      ),
    },
    service_account_manager_status: { rpcMethod: "accountManager/status" },
    service_account_manager_session_current: { rpcMethod: "accountManager/session/current" },
    service_account_manager_profile_update: { rpcMethod: "accountManager/profile/update" },
    service_account_manager_password_change: { rpcMethod: "accountManager/password/change" },
    service_account_manager_users_list: { rpcMethod: "accountManager/users/list" },
    service_account_manager_user_create: {
      rpcMethod: "accountManager/users/create",
      mapParams: (params) => asRecord(asRecord(params)?.payload) ?? {},
    },
    service_account_manager_user_update: {
      rpcMethod: "accountManager/users/update",
      mapParams: (params) => asRecord(asRecord(params)?.payload) ?? {},
    },
    service_account_manager_user_delete: { rpcMethod: "accountManager/users/delete" },
    service_account_manager_wallet_top_up: {
      rpcMethod: "accountManager/wallet/topUp",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          ownerKind: source.owner_kind ?? source.ownerKind,
          ownerId: source.owner_id ?? source.ownerId,
          amountCreditMicros: source.amount_credit_micros ?? source.amountCreditMicros,
          note: source.note,
        };
      },
    },
    service_account_manager_wallet_set_available: {
      rpcMethod: "accountManager/wallet/setAvailable",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          ownerKind: source.owner_kind ?? source.ownerKind,
          ownerId: source.owner_id ?? source.ownerId,
          availableCreditMicros: source.available_credit_micros ?? source.availableCreditMicros,
          note: source.note,
        };
      },
    },
    service_account_manager_api_key_owners_list: { rpcMethod: "accountManager/apiKeyOwners/list" },
    service_account_manager_api_key_owner_set: {
      rpcMethod: "accountManager/apiKeyOwners/set",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          keyId: source.key_id ?? source.keyId,
          ownerKind: source.owner_kind ?? source.ownerKind,
          ownerUserId: source.owner_user_id ?? source.ownerUserId,
          projectId: source.project_id ?? source.projectId,
        };
      },
    },
    service_model_groups_list: { rpcMethod: "modelGroups/list" },
    service_model_group_save: { rpcMethod: "modelGroups/save" },
    service_model_group_delete: { rpcMethod: "modelGroups/delete" },
    service_model_group_models_set: { rpcMethod: "modelGroups/setModels" },
    service_model_group_users_set: { rpcMethod: "modelGroups/setUsers" },
    service_model_price_rules_list: { rpcMethod: "modelPriceRules/list" },
    service_model_price_rule_read: {
      rpcMethod: "modelPriceRule/read",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          modelPattern: source.model_pattern ?? source.modelPattern,
        };
      },
    },
    service_model_price_rule_upsert: {
      rpcMethod: "modelPriceRule/upsert",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          payload: source.payload,
        };
      },
    },
    service_dashboard_admin_usage_summary: {
      rpcMethod: "dashboard/adminUsageSummary",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        const mapped: Record<string, unknown> = {
          startTs: source.start_ts ?? source.startTs,
          endTs: source.end_ts ?? source.endTs,
        };
        const rankingLimit = source.ranking_limit ?? source.rankingLimit;
        if (rankingLimit !== undefined) {
          mapped.rankingLimit = rankingLimit;
        }
        return mapped;
      },
    },
    service_dashboard_admin_overview: {
      rpcMethod: "dashboard/adminOverview",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          requestLogLimit: source.request_log_limit ?? source.requestLogLimit,
          dayStartTs: source.day_start_ts ?? source.dayStartTs,
          dayEndTs: source.day_end_ts ?? source.dayEndTs,
          accountLimit: source.account_limit ?? source.accountLimit,
        };
      },
    },
    service_dashboard_member_summary: {
      rpcMethod: "dashboard/memberSummary",
      mapParams: (params) => {
        const source = asRecord(params) ?? {};
        return {
          userId: source.user_id ?? source.userId,
          dayStartTs: source.day_start_ts ?? source.dayStartTs,
          dayEndTs: source.day_end_ts ?? source.dayEndTs,
        };
      },
    },
    service_usage_read: { rpcMethod: "account/usage/read" },
    service_usage_list: { rpcMethod: "account/usage/list" },
    service_usage_refresh: {
      rpcMethod: "account/usage/refresh",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC account/usage/refresh 超时：账号用量刷新超过 120 秒",
      ),
    },
    service_usage_aggregate: { rpcMethod: "account/usage/aggregate" },
  };
}
