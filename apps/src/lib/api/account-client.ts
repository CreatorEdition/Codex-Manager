import { invoke, withAddr } from "./transport";
import {
  normalizeAccountList,
  normalizeAggregateApiBalanceRefreshResult,
  normalizeAggregateApiCreateResult,
  normalizeAggregateApiList,
  normalizeAggregateApiListResult,
  normalizeAggregateApiSecretResult,
  normalizeAggregateApiSupplierModel,
  normalizeAggregateApiSupplierModelImportResult,
  normalizeAggregateApiSupplierModelList,
  normalizeAggregateApiTestResult,
  normalizeApiKeyCreateResult,
  normalizeApiKeyList,
  normalizeApiKeyListResult,
  normalizeApiKeyUsageStats,
  normalizeLoginStartResult,
  normalizeManagedModelCatalog,
  normalizeManagedModelInfo,
  normalizeManagedModelRouting,
  normalizeModelCatalog,
  normalizeUsageAggregateSummary,
  normalizeUsageList,
  normalizeUsageSnapshot,
} from "./normalize";
import {
  readChatgptAuthTokensRefreshAllResult,
  readChatgptAuthTokensRefreshResult,
  readCurrentAccessTokenAccountReadResult,
  readLoginStatusResult,
} from "./account-auth";
import {
  AccountExportResult,
  AccountImportResult,
  AccountWarmupResult,
  DeleteAccountsByStatusesResult,
  DeleteUnavailableFreeResult,
  readAccountExportResult,
  readAccountImportResult,
  readAccountWarmupResult,
  readDeleteAccountsByStatusesResult,
  readApiKeySecret,
  readDeleteUnavailableFreeResult,
  splitAccountImportContents,
} from "./account-maintenance";
import { serializeManagedModelForRpc } from "./model-catalog";
import { unwrapUsageSnapshotPayload } from "./usage-response";
import {
  AccountListResult,
  AccountUsage,
  AggregateApi,
  AggregateApiBalanceRefreshResult,
  AggregateApiCreateResult,
  AggregateApiListResult,
  AggregateApiSecretResult,
  AggregateApiSupplierModel,
  AggregateApiSupplierModelImportResult,
  AggregateApiTestResult,
  ApiKey,
  ApiKeyCreateResult,
  ApiKeyListResult,
  ApiKeyUsageStat,
  ChatgptAuthTokensRefreshAllResult,
  ChatgptAuthTokensRefreshResult,
  CurrentAccessTokenAccountReadResult,
  LoginStatusResult,
  LoginStartResult,
  ManagedModelCatalog,
  ManagedModelInfo,
  ManagedModelRouting,
  ManagedModelSourceMapping,
  ManagedModelSourceModel,
  ModelCatalog,
  ModelInfo,
  UsageAggregateSummary,
} from "../../types";

export interface AccountExportPayload {
  selectedAccountIds?: string[];
  exportMode?: "single" | "multiple";
}

export interface ApiKeyListParams {
  page?: number;
  pageSize?: number;
  query?: string | null;
  statusFilter?: string | null;
}

export interface AggregateApiListParams {
  page?: number;
  pageSize?: number;
  query?: string | null;
  providerType?: string | null;
  statusFilter?: string | null;
}

export interface AccountListParams {
  page?: number;
  pageSize?: number;
  query?: string | null;
  filter?: string | null;
  groupFilter?: string | null;
  planFilter?: string | null;
  statusFilter?: string | null;
  sortMode?: string | null;
  includePlanTypes?: boolean;
}

export interface AccountWarmupPayload {
  accountIds?: string[];
  message?: string;
}

export interface UsageListParams {
  accountIds?: string[];
}

export interface ApiKeyUsageStatsParams {
  keyIds?: string[];
}

export interface AccountDeleteByStatusesPayload {
  statuses: string[];
}

interface LoginStartPayload {
  loginType?: string;
  openBrowser?: boolean;
  note?: string | null;
  tags?: string[] | string | null;
  workspaceId?: string | null;
}

interface AccountUpdatePayload {
  sort?: number | null;
  preferred?: boolean | null;
  status?: string | null;
  label?: string | null;
  note?: string | null;
  tags?: string[] | string | null;
  modelSlugs?: string[] | null;
  quotaCapacityPrimaryWindowTokens?: number | null;
  quotaCapacitySecondaryWindowTokens?: number | null;
}

interface ChatgptAuthTokensLoginPayload {
  accessToken: string;
  refreshToken?: string | null;
  idToken?: string | null;
  chatgptAccountId?: string | null;
  workspaceId?: string | null;
  chatgptPlanType?: string | null;
}

interface ApiKeyPayload {
  name?: string | null;
  modelSlug?: string | null;
  reasoningEffort?: string | null;
  serviceTier?: string | null;
  protocolType?: string | null;
  upstreamBaseUrl?: string | null;
  staticHeadersJson?: string | null;
  rotationStrategy?: string | null;
  aggregateApiId?: string | null;
  accountPlanFilter?: string | null;
  quotaLimitTokens?: number | null;
  customKey?: string | null;
}

export interface ManagedModelPayload {
  previousSlug?: string | null;
  sourceKind?: string | null;
  userEdited?: boolean | null;
  sortIndex?: number | null;
  model: ManagedModelInfo | ModelInfo;
}

export interface ModelPriceRuleEntry {
  id: string;
  provider: string;
  modelPattern: string;
  matchType: string;
  billingMode: "standard" | "priority" | string;
  inputPricePer1m: number | null;
  cachedInputPricePer1m: number | null;
  outputPricePer1m: number | null;
  enabled: boolean;
  priority: number;
  source: string;
  createdAt: number;
  updatedAt: number;
}

export interface ModelPriceRuleUpsertPayload {
  id?: string | null;
  provider?: string | null;
  modelPattern: string;
  matchType?: string | null;
  billingMode?: "standard" | "priority" | string | null;
  inputPricePer1m?: number | null;
  cachedInputPricePer1m?: number | null;
  outputPricePer1m?: number | null;
  enabled?: boolean | null;
  priority?: number | null;
}

export interface ManagedModelSourceSyncPayload {
  sourceKind: string;
  sourceId?: string | null;
}

export interface ManagedModelSourceModelPayload {
  sourceKind: string;
  sourceId: string;
  upstreamModel: string;
  displayName?: string | null;
}

export interface ManagedModelSourceMappingPayload {
  id?: string | null;
  platformModelSlug: string;
  sourceKind: string;
  sourceId: string;
  upstreamModel: string;
  enabled?: boolean | null;
  priority?: number | null;
  weight?: number | null;
  billingModelSlug?: string | null;
}

export interface AggregateApiSupplierModelPayload {
  supplierKey: string;
  providerType: string;
  upstreamModel: string;
  displayName?: string | null;
  status?: string | null;
}

interface AggregateApiPayload {
  providerType?: string | null;
  supplierName?: string | null;
  sort?: number | null;
  status?: string | null;
  url?: string | null;
  key?: string | null;
  authType?: string | null;
  authCustomEnabled?: boolean | null;
  authParams?: Record<string, unknown> | null;
  actionCustomEnabled?: boolean | null;
  action?: string | null;
  modelOverride?: string | null;
  username?: string | null;
  password?: string | null;
  balanceQueryEnabled?: boolean | null;
  balanceQueryTemplate?: string | null;
  balanceQueryBaseUrl?: string | null;
  balanceQueryAccessToken?: string | null;
  balanceQueryUserId?: string | null;
  balanceQueryConfigJson?: string | null;
  modelSlugs?: string[] | null;
}

const MAX_IMPORT_ERROR_ITEMS = 50;

/**
 * 函数 `createEmptyImportResult`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * 无
 *
 * # 返回
 * 返回函数执行结果
 */
function createEmptyImportResult(): AccountImportResult {
  return {
    total: 0,
    created: 0,
    updated: 0,
    failed: 0,
    errors: [],
  };
}

/**
 * 函数 `mergeImportResult`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - target: 参数 target
 * - source: 参数 source
 * - indexOffset: 参数 indexOffset
 *
 * # 返回
 * 返回函数执行结果
 */
function mergeImportResult(
  target: AccountImportResult,
  source: AccountImportResult,
  indexOffset: number
) {
  target.total = (target.total || 0) + (source.total || 0);
  target.created = (target.created || 0) + (source.created || 0);
  target.updated = (target.updated || 0) + (source.updated || 0);
  target.failed = (target.failed || 0) + (source.failed || 0);

  const errors = source.errors || [];
  if (!target.errors) {
    target.errors = [];
  }
  for (const error of errors) {
    if (target.errors.length >= MAX_IMPORT_ERROR_ITEMS) {
      break;
    }
    target.errors.push({
      index: (error.index || 0) + indexOffset,
      message: error.message || "",
    });
  }
}

/**
 * 函数 `importAccountContents`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - contents: 参数 contents
 *
 * # 返回
 * 返回函数执行结果
 */
async function importAccountContents(contents: string[]): Promise<AccountImportResult> {
  const batches = splitAccountImportContents(contents);
  if (batches.length === 0) {
    return createEmptyImportResult();
  }

  const merged = createEmptyImportResult();
  let processed = 0;
  for (const batch of batches) {
    const imported = readAccountImportResult(
      await invoke<unknown>("service_account_import", withAddr({ contents: batch }))
    );
    mergeImportResult(merged, imported, processed);
    processed += batch.length;
  }

  return merged;
}

export const accountClient = {
  async list(params?: AccountListParams): Promise<AccountListResult> {
    const result = await invoke<unknown>(
      "service_account_list",
      withAddr(params ? { ...params } : {}),
    );
    return normalizeAccountList(result);
  },
  async lookupAccounts(ids: string[]) {
    const normalizedIds = Array.from(
      new Set(
        (Array.isArray(ids) ? ids : [])
          .map((id) => String(id || "").trim())
          .filter(Boolean),
      ),
    );
    if (normalizedIds.length === 0) {
      return [];
    }
    const result = await invoke<unknown>(
      "service_account_lookup",
      withAddr({ ids: normalizedIds }),
    );
    return normalizeAccountList(result).items;
  },
  delete: (accountId: string) =>
    invoke("service_account_delete", withAddr({ accountId })),
  deleteMany: (accountIds: string[]) =>
    invoke("service_account_delete_many", withAddr({ accountIds })),
  deleteUnavailableFree: async (): Promise<DeleteUnavailableFreeResult> =>
    readDeleteUnavailableFreeResult(
      await invoke<unknown>("service_account_delete_unavailable_free", withAddr())
    ),
  deleteByStatuses: async (
    params: AccountDeleteByStatusesPayload
  ): Promise<DeleteAccountsByStatusesResult> =>
    readDeleteAccountsByStatusesResult(
      await invoke<unknown>(
        "service_account_delete_by_statuses",
        withAddr({
          statuses: Array.isArray(params?.statuses) ? params.statuses : [],
        })
      )
    ),
  updateSort: (accountId: string, sort: number) =>
    invoke("service_account_update", withAddr({ accountId, sort })),
  updateProfile: (accountId: string, params: AccountUpdatePayload) =>
    invoke(
      "service_account_update",
      withAddr({
        accountId,
        sort: typeof params.sort === "number" ? params.sort : null,
        preferred: typeof params.preferred === "boolean" ? params.preferred : null,
        status: params.status || null,
        label: params.label ?? null,
        note: params.note ?? null,
        tags: Array.isArray(params.tags)
          ? params.tags
              .map((item: string) => String(item || "").trim())
              .filter(Boolean)
              .join(",")
          : params.tags ?? null,
        modelSlugs: Array.isArray(params.modelSlugs)
          ? params.modelSlugs
              .map((item) => String(item || "").trim())
              .filter(Boolean)
          : null,
        quotaCapacityPrimaryWindowTokens:
          typeof params.quotaCapacityPrimaryWindowTokens === "number"
            ? params.quotaCapacityPrimaryWindowTokens
            : null,
        quotaCapacitySecondaryWindowTokens:
          typeof params.quotaCapacitySecondaryWindowTokens === "number"
            ? params.quotaCapacitySecondaryWindowTokens
            : null,
      })
    ),
  setPreferred: (accountId: string) =>
    invoke("service_account_update", withAddr({ accountId, preferred: true })),
  clearPreferred: (accountId: string) =>
    invoke("service_account_update", withAddr({ accountId, preferred: false })),
  disableAccount: (accountId: string) =>
    invoke("service_account_update", withAddr({ accountId, status: "disabled" })),
  enableAccount: (accountId: string) =>
    invoke("service_account_update", withAddr({ accountId, status: "active" })),
  import: importAccountContents,
  async importByDirectory(): Promise<AccountImportResult> {
    const picked = readAccountImportResult(
      await invoke<unknown>("service_account_import_by_directory", withAddr())
    );
    if (picked?.canceled || !Array.isArray(picked?.contents) || picked.contents.length === 0) {
      return picked;
    }

    const imported = await importAccountContents(picked.contents);
    return {
      ...imported,
      canceled: false,
      directoryPath: picked.directoryPath || "",
      fileCount: picked.fileCount || picked.contents.length,
    };
  },
  async importByFile(): Promise<AccountImportResult> {
    const picked = readAccountImportResult(
      await invoke<unknown>("service_account_import_by_file", withAddr())
    );
    if (picked?.canceled || !Array.isArray(picked?.contents) || picked.contents.length === 0) {
      return picked;
    }

    const imported = await importAccountContents(picked.contents);
    return {
      ...imported,
      canceled: false,
      fileCount: picked.fileCount || picked.contents.length,
    };
  },
  export: async (params?: AccountExportPayload): Promise<AccountExportResult> =>
    readAccountExportResult(await invoke<unknown>(
      "service_account_export_by_account_files",
      withAddr({
        selectedAccountIds: Array.isArray(params?.selectedAccountIds)
          ? params?.selectedAccountIds
          : [],
        exportMode: params?.exportMode || "multiple",
      })
    )),
  warmup: async (params?: AccountWarmupPayload): Promise<AccountWarmupResult> =>
    readAccountWarmupResult(
      await invoke<unknown>(
        "service_account_warmup",
        withAddr({
          accountIds: Array.isArray(params?.accountIds) ? params.accountIds : [],
          message: params?.message || "hi",
        }),
      ),
    ),

  async getUsage(accountId: string): Promise<AccountUsage | null> {
    const result = await invoke<unknown>(
      "service_usage_read",
      withAddr({ accountId, account_id: accountId })
    );
    return normalizeUsageSnapshot(unwrapUsageSnapshotPayload(result));
  },
  async getLatestUsage(): Promise<AccountUsage | null> {
    const result = await invoke<unknown>("service_usage_read", withAddr());
    return normalizeUsageSnapshot(unwrapUsageSnapshotPayload(result));
  },
  async listUsage(params?: UsageListParams): Promise<AccountUsage[]> {
    const accountIds = Array.isArray(params?.accountIds)
      ? params.accountIds
          .map((id) => String(id || "").trim())
          .filter(Boolean)
      : [];
    const result = await invoke<unknown>(
      "service_usage_list",
      withAddr(params ? { accountIds } : {}),
    );
    return normalizeUsageList(result);
  },
  refreshUsage: (accountId?: string) => {
    const targetAccountId = accountId?.trim();
    return invoke(
      "service_usage_refresh",
      withAddr(
        targetAccountId
          ? { accountId: targetAccountId, account_id: targetAccountId }
          : {}
      )
    );
  },
  async aggregateUsage(): Promise<UsageAggregateSummary> {
    const result = await invoke<unknown>("service_usage_aggregate", withAddr());
    return normalizeUsageAggregateSummary(result);
  },

  async startLogin(params: LoginStartPayload): Promise<LoginStartResult> {
    const result = await invoke<unknown>(
      "service_login_start",
      withAddr({
        loginType: params?.loginType || "chatgpt",
        openBrowser: params?.openBrowser ?? true,
        note: params?.note || null,
        tags: Array.isArray(params?.tags)
          ? params.tags
              .map((item: string) => String(item || "").trim())
              .filter(Boolean)
              .join(",")
          : params?.tags || null,
        workspaceId: params?.workspaceId || null,
      })
    );
    return normalizeLoginStartResult(result);
  },
  async getLoginStatus(loginId: string): Promise<LoginStatusResult> {
    const result = await invoke<unknown>("service_login_status", withAddr({ loginId }));
    return readLoginStatusResult(result);
  },
  completeLogin: (state: string, code: string, redirectUri: string) =>
    invoke("service_login_complete", withAddr({ state, code, redirectUri })),
  loginWithChatgptAuthTokens: (params: ChatgptAuthTokensLoginPayload) =>
    invoke("service_login_chatgpt_auth_tokens", withAddr({
      accessToken: params.accessToken,
      refreshToken: params.refreshToken || null,
      idToken: params.idToken || null,
      chatgptAccountId: params.chatgptAccountId || null,
      workspaceId: params.workspaceId || null,
      chatgptPlanType: params.chatgptPlanType || null,
    })),
  async readCurrentAccessTokenAccount(
    refreshToken = false
  ): Promise<CurrentAccessTokenAccountReadResult> {
    const result = await invoke<unknown>(
      "service_account_read",
      withAddr({ refreshToken })
    );
    return readCurrentAccessTokenAccountReadResult(result);
  },
  logoutCurrentAccessTokenAccount: () =>
    invoke("service_account_logout", withAddr()),
  async refreshChatgptAuthTokens(
    accountId?: string
  ): Promise<ChatgptAuthTokensRefreshResult> {
    const targetAccountId = accountId?.trim() || null;
    const result = await invoke<unknown>(
      "service_chatgpt_auth_tokens_refresh",
      withAddr({
        accountId: targetAccountId,
        previousAccountId: targetAccountId,
      })
    );
    return readChatgptAuthTokensRefreshResult(result);
  },
  async refreshAllChatgptAuthTokens(): Promise<ChatgptAuthTokensRefreshAllResult> {
    const result = await invoke<unknown>(
      "service_chatgpt_auth_tokens_refresh_all",
      withAddr()
    );
    return readChatgptAuthTokensRefreshAllResult(result);
  },

  async listAggregateApis(): Promise<AggregateApi[]> {
    const result = await invoke<unknown>(
      "service_aggregate_api_list",
      withAddr({ page: 1, pageSize: 500 }),
    );
    return normalizeAggregateApiList(result);
  },
  async listAggregateApiPage(
    params: AggregateApiListParams,
  ): Promise<AggregateApiListResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_list",
      withAddr({
        page: params.page ?? 1,
        pageSize: params.pageSize ?? 20,
        page_size: params.pageSize ?? 20,
        query: params.query || null,
        providerType: params.providerType || null,
        provider_type: params.providerType || null,
        statusFilter: params.statusFilter || null,
        status_filter: params.statusFilter || null,
      }),
    );
    return normalizeAggregateApiListResult(result);
  },
  async lookupAggregateApis(ids: string[]): Promise<AggregateApi[]> {
    const normalizedIds = Array.from(
      new Set(
        (Array.isArray(ids) ? ids : [])
          .map((id) => String(id || "").trim())
          .filter(Boolean),
      ),
    );
    if (normalizedIds.length === 0) {
      return [];
    }
    const result = await invoke<unknown>(
      "service_aggregate_api_lookup",
      withAddr({ ids: normalizedIds }),
    );
    return normalizeAggregateApiList(result);
  },
  async createAggregateApi(params: AggregateApiPayload): Promise<AggregateApiCreateResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_create",
      withAddr({
        providerType: params.providerType || null,
        supplierName: params.supplierName || null,
        sort: typeof params.sort === "number" ? params.sort : null,
        status: params.status || null,
        url: params.url || null,
        key: params.key || null,
        authType: params.authType || null,
        authCustomEnabled:
          typeof params.authCustomEnabled === "boolean"
            ? params.authCustomEnabled
            : null,
        authParams: params.authParams || null,
        actionCustomEnabled:
          typeof params.actionCustomEnabled === "boolean"
            ? params.actionCustomEnabled
            : null,
        action: params.action ?? null,
        modelOverride:
          typeof params.modelOverride === "string" ? params.modelOverride : null,
        username: params.username || null,
        password: params.password || null,
        balanceQueryEnabled:
          typeof params.balanceQueryEnabled === "boolean"
            ? params.balanceQueryEnabled
            : null,
        balanceQueryTemplate: params.balanceQueryTemplate || null,
        balanceQueryBaseUrl:
          typeof params.balanceQueryBaseUrl === "string"
            ? params.balanceQueryBaseUrl
            : null,
        balanceQueryAccessToken: params.balanceQueryAccessToken || null,
        balanceQueryUserId:
          typeof params.balanceQueryUserId === "string"
            ? params.balanceQueryUserId
            : null,
        balanceQueryConfigJson:
          typeof params.balanceQueryConfigJson === "string"
            ? params.balanceQueryConfigJson
            : null,
        modelSlugs: Array.isArray(params.modelSlugs) ? params.modelSlugs : null,
      })
    );
    return normalizeAggregateApiCreateResult(result);
  },
  updateAggregateApi: (apiId: string, params: AggregateApiPayload) =>
    invoke(
      "service_aggregate_api_update",
      withAddr({
        id: apiId,
        providerType: params.providerType || null,
        supplierName: params.supplierName || null,
        sort: typeof params.sort === "number" ? params.sort : null,
        status: params.status || null,
        url: params.url || null,
        key: params.key || null,
        authType: params.authType || null,
        authCustomEnabled:
          typeof params.authCustomEnabled === "boolean"
            ? params.authCustomEnabled
            : null,
        authParams: params.authParams || null,
        actionCustomEnabled:
          typeof params.actionCustomEnabled === "boolean"
            ? params.actionCustomEnabled
            : null,
        action: params.action ?? null,
        modelOverride:
          typeof params.modelOverride === "string" ? params.modelOverride : null,
        username: params.username || null,
        password: params.password || null,
        balanceQueryEnabled:
          typeof params.balanceQueryEnabled === "boolean"
            ? params.balanceQueryEnabled
            : null,
        balanceQueryTemplate: params.balanceQueryTemplate || null,
        balanceQueryBaseUrl:
          typeof params.balanceQueryBaseUrl === "string"
            ? params.balanceQueryBaseUrl
            : null,
        balanceQueryAccessToken: params.balanceQueryAccessToken || null,
        balanceQueryUserId:
          typeof params.balanceQueryUserId === "string"
            ? params.balanceQueryUserId
            : null,
        balanceQueryConfigJson:
          typeof params.balanceQueryConfigJson === "string"
            ? params.balanceQueryConfigJson
            : null,
        modelSlugs: Array.isArray(params.modelSlugs) ? params.modelSlugs : null,
      })
    ),
  deleteAggregateApi: (apiId: string) =>
    invoke("service_aggregate_api_delete", withAddr({ id: apiId })),
  async readAggregateApiSecret(apiId: string): Promise<AggregateApiSecretResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_read_secret",
      withAddr({ id: apiId })
    );
    return normalizeAggregateApiSecretResult(result);
  },
  async testAggregateApiConnection(apiId: string): Promise<AggregateApiTestResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_test_connection",
      withAddr({ id: apiId })
    );
    return normalizeAggregateApiTestResult(result);
  },
  async refreshAggregateApiBalance(apiId: string): Promise<AggregateApiBalanceRefreshResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_refresh_balance",
      withAddr({ id: apiId })
    );
    return normalizeAggregateApiBalanceRefreshResult(result);
  },
  async listAggregateApiSupplierModels(params?: {
    supplierKey?: string | null;
    providerType?: string | null;
    page?: number | null;
    pageSize?: number | null;
  }): Promise<AggregateApiSupplierModel[]> {
    const result = await invoke<unknown>(
      "service_aggregate_api_supplier_models_list",
      withAddr({
        supplierKey: params?.supplierKey || null,
        providerType: params?.providerType || null,
        page: params?.page ?? null,
        pageSize: params?.pageSize ?? null,
      })
    );
    return normalizeAggregateApiSupplierModelList(result);
  },
  async saveAggregateApiSupplierModel(
    params: AggregateApiSupplierModelPayload,
  ): Promise<AggregateApiSupplierModel> {
    const result = await invoke<unknown>(
      "service_aggregate_api_supplier_model_save",
      withAddr({ payload: params }),
    );
    const item = normalizeAggregateApiSupplierModel(result);
    if (!item) throw new Error("供应商模型保存结果为空");
    return item;
  },
  deleteAggregateApiSupplierModel: (params: {
    supplierKey: string;
    providerType: string;
    upstreamModel: string;
  }) =>
    invoke(
      "service_aggregate_api_supplier_model_delete",
      withAddr({
        supplierKey: params.supplierKey,
        providerType: params.providerType,
        upstreamModel: params.upstreamModel,
      }),
    ),
  async importAggregateApiSupplierModels(params: {
    apiId: string;
    supplierKey?: string | null;
    providerType?: string | null;
  }): Promise<AggregateApiSupplierModelImportResult> {
    const result = await invoke<unknown>(
      "service_aggregate_api_supplier_models_import",
      withAddr({
        apiId: params.apiId,
        supplierKey: params.supplierKey || null,
        providerType: params.providerType || null,
      }),
    );
    return normalizeAggregateApiSupplierModelImportResult(result);
  },

  async listApiKeys(): Promise<ApiKey[]> {
    const result = await invoke<unknown>("service_apikey_list", withAddr());
    return normalizeApiKeyList(result);
  },
  async lookupApiKeys(ids: string[]): Promise<ApiKey[]> {
    const normalizedIds = Array.from(
      new Set(
        (Array.isArray(ids) ? ids : [])
          .map((id) => String(id || "").trim())
          .filter(Boolean),
      ),
    );
    if (normalizedIds.length === 0) {
      return [];
    }
    const result = await invoke<unknown>(
      "service_apikey_lookup",
      withAddr({ ids: normalizedIds }),
    );
    return normalizeApiKeyList(result);
  },
  async listApiKeyPage(params: ApiKeyListParams): Promise<ApiKeyListResult> {
    const result = await invoke<unknown>(
      "service_apikey_list",
      withAddr({
        page: params.page,
        pageSize: params.pageSize,
        query: params.query || null,
        statusFilter: params.statusFilter || null,
      }),
    );
    return normalizeApiKeyListResult(result);
  },
  async createApiKey(params: ApiKeyPayload): Promise<ApiKeyCreateResult> {
    const result = await invoke<unknown>(
      "service_apikey_create",
      withAddr({
        name: params.name || null,
        modelSlug: params.modelSlug || null,
        reasoningEffort: params.reasoningEffort || null,
        serviceTier: params.serviceTier || null,
        protocolType: params.protocolType || null,
        upstreamBaseUrl: params.upstreamBaseUrl || null,
        staticHeadersJson: params.staticHeadersJson || null,
        rotationStrategy: params.rotationStrategy || null,
        aggregateApiId: params.aggregateApiId || null,
        accountPlanFilter: params.accountPlanFilter || null,
        quotaLimitTokens: params.quotaLimitTokens ?? null,
        customKey: params.customKey || null,
      })
    );
    return normalizeApiKeyCreateResult(result);
  },
  async listApiKeyUsageStats(
    params?: ApiKeyUsageStatsParams,
  ): Promise<ApiKeyUsageStat[]> {
    const rawKeyIds = params?.keyIds;
    const hasKeyIds = Array.isArray(rawKeyIds);
    const keyIds = hasKeyIds
      ? rawKeyIds
          .map((id) => String(id || "").trim())
          .filter(Boolean)
      : [];
    const result = await invoke<unknown>(
      "service_apikey_usage_stats",
      withAddr(hasKeyIds ? { keyIds } : {}),
    );
    return normalizeApiKeyUsageStats(result);
  },
  deleteApiKey: (keyId: string) =>
    invoke("service_apikey_delete", withAddr({ keyId })),
  updateApiKey: (keyId: string, params: ApiKeyPayload) => {
    const payload: Record<string, unknown> = {
      keyId,
      name: params.name || null,
      modelSlug: params.modelSlug || null,
      reasoningEffort: params.reasoningEffort || null,
      serviceTier: params.serviceTier || null,
      protocolType: params.protocolType || null,
      upstreamBaseUrl: params.upstreamBaseUrl || null,
      staticHeadersJson: params.staticHeadersJson || null,
      rotationStrategy: params.rotationStrategy || null,
      aggregateApiId: params.aggregateApiId || null,
      accountPlanFilter: params.accountPlanFilter || null,
    };
    if ("quotaLimitTokens" in params) {
      payload.quotaLimitTokens = params.quotaLimitTokens ?? null;
    }
    return invoke("service_apikey_update_model", withAddr(payload));
  },
  disableApiKey: (keyId: string) =>
    invoke("service_apikey_disable", withAddr({ keyId })),
  enableApiKey: (keyId: string) =>
    invoke("service_apikey_enable", withAddr({ keyId })),
  async listModels(refreshRemote?: boolean): Promise<ModelCatalog> {
    const result = await invoke<unknown>(
      "service_apikey_models",
      withAddr({ refreshRemote })
    );
    return normalizeModelCatalog(result);
  },
  async listManagedModels(refreshRemote?: boolean): Promise<ManagedModelCatalog> {
    const result = await invoke<unknown>(
      "service_model_catalog_list",
      withAddr({ refreshRemote })
    );
    return normalizeManagedModelCatalog(result);
  },
  async listManagedModelRouting(): Promise<ManagedModelRouting> {
    const result = await invoke<unknown>("service_model_routing", withAddr());
    return normalizeManagedModelRouting(result);
  },
  async syncManagedModelSourceModels(
    params: ManagedModelSourceSyncPayload,
  ): Promise<ManagedModelRouting> {
    const result = await invoke<unknown>(
      "service_model_source_sync",
      withAddr({ payload: params }),
    );
    return normalizeManagedModelRouting(result);
  },
  async saveManagedModelSourceModel(
    params: ManagedModelSourceModelPayload,
  ): Promise<ManagedModelSourceModel> {
    const result = await invoke<unknown>(
      "service_model_source_model_save",
      withAddr({ payload: params }),
    );
    const routing = normalizeManagedModelRouting({ sourceModels: [result], mappings: [] });
    const item = routing.sourceModels[0];
    if (!item) throw new Error("来源模型保存结果为空");
    return item;
  },
  async saveManagedModelSourceMapping(
    params: ManagedModelSourceMappingPayload,
  ): Promise<ManagedModelSourceMapping> {
    const result = await invoke<unknown>(
      "service_model_source_mapping_save",
      withAddr({ payload: params }),
    );
    const routing = normalizeManagedModelRouting({ sourceModels: [], mappings: [result] });
    const item = routing.mappings[0];
    if (!item) throw new Error("模型映射保存结果为空");
    return item;
  },
  deleteManagedModelSourceMapping: (params: {
    id: string;
    sourceKind: string;
    sourceId: string;
    upstreamModel: string;
  }) =>
    invoke("service_model_source_mapping_delete", withAddr({ payload: params })),
  async saveManagedModel(params: ManagedModelPayload): Promise<ManagedModelInfo> {
    const payload = {
      previousSlug: params.previousSlug || null,
      sourceKind: params.sourceKind || null,
      userEdited:
        typeof params.userEdited === "boolean" ? params.userEdited : null,
      sortIndex: typeof params.sortIndex === "number" ? params.sortIndex : null,
      ...serializeManagedModelForRpc(params.model),
    };
    const result = await invoke<unknown>(
      "service_model_catalog_save",
      withAddr({ payload })
    );
    const normalized = normalizeManagedModelInfo(result);
    if (!normalized) {
      throw new Error("模型保存结果为空");
    }
    return normalized;
  },
  deleteManagedModel: (slug: string) =>
    invoke("service_model_catalog_delete", withAddr({ slug })),
  listModelPriceRules: async () => {
    const result = await invoke<{ items: ModelPriceRuleEntry[] }>(
      "service_model_price_rules_list",
      withAddr(),
    );
    return result.items;
  },
  readModelPriceRule: async (modelPattern: string, billingMode?: string | null) => {
    const result = await invoke<ModelPriceRuleEntry | null>(
      "service_model_price_rule_read",
      withAddr({ modelPattern, billingMode: billingMode || null }),
    );
    return result;
  },
  upsertModelPriceRule: async (payload: ModelPriceRuleUpsertPayload) => {
    const result = await invoke<ModelPriceRuleEntry>(
      "service_model_price_rule_upsert",
      withAddr({ payload }),
    );
    return result;
  },
  async pruneStaleRemoteManagedModels(): Promise<ManagedModelCatalog> {
    const result = await invoke<unknown>(
      "service_model_catalog_prune_stale_remote",
      withAddr()
    );
    return normalizeManagedModelCatalog(result);
  },
  async readApiKeySecret(keyId: string): Promise<string> {
    const result = await invoke<unknown>(
      "service_apikey_read_secret",
      withAddr({ keyId })
    );
    return readApiKeySecret(result);
  },
};
