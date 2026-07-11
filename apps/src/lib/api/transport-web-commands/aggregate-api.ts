import type { WebCommandDescriptor } from "./shared";
import {
  WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
  asRecord,
  noRetryTimeoutOptions,
} from "./shared";

export function createAggregateApiWebCommands(): Record<string, WebCommandDescriptor> {
  return {
    service_aggregate_api_list: { rpcMethod: "aggregateApi/list" },
    service_aggregate_api_lookup: { rpcMethod: "aggregateApi/lookup" },
    service_aggregate_api_create: { rpcMethod: "aggregateApi/create" },
    service_aggregate_api_update: { rpcMethod: "aggregateApi/update" },
    service_aggregate_api_delete: { rpcMethod: "aggregateApi/delete" },
    service_aggregate_api_read_secret: { rpcMethod: "aggregateApi/readSecret" },
    service_aggregate_api_test_connection: { rpcMethod: "aggregateApi/testConnection" },
    service_aggregate_api_refresh_balance: { rpcMethod: "aggregateApi/refreshBalance" },
    service_aggregate_api_supplier_models_list: { rpcMethod: "aggregateApi/supplierModels/list" },
    service_aggregate_api_supplier_model_save: { rpcMethod: "aggregateApi/supplierModels/save", mapParams: (params) => asRecord(asRecord(params)?.payload) ?? {} },
    service_aggregate_api_supplier_model_delete: { rpcMethod: "aggregateApi/supplierModels/delete" },
    service_aggregate_api_supplier_models_import: {
      rpcMethod: "aggregateApi/sourceModels/importSupplier",
      requestOptions: noRetryTimeoutOptions(
        WEB_RPC_LONG_OPERATION_TIMEOUT_MS,
        "RPC aggregateApi/sourceModels/importSupplier 超时：供应商模型导入超过 120 秒",
      ),
    },
  };
}
