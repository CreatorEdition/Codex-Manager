import type { MessageCatalog } from "../types";

export const EN_API_KEYS_MESSAGES: MessageCatalog = {
  项目: "Project",
  "Token / 金额": "Token / Amount",
  已花费: "Spent",
  不限额: "Unlimited",
  已达上限: "Limit reached",
  "请选择平台 Key 归属成员": "Select the member owner for this platform key",
  账号组筛选: "Account group filter",
  自定义计划类型: "Custom plan type",
  "例如：k12": "e.g. k12",
  请填写自定义计划类型: "Enter a custom plan type",
  "用于匹配后端保留的原始计划类型，例如 k12；除 k12 外，未来新增计划也可按原值过滤。":
    "Matches the raw plan type preserved by the backend, such as k12. Future plan types can also be filtered by their raw value.",
  "额度分发开启时，平台 Key 必须归属到一个成员钱包。":
    "When quota distribution is enabled, the platform key must belong to a member wallet.",
  "未开启额度分发时可先不分配，开启后再补齐归属。":
    "When quota distribution is not enabled, you may leave this unassigned and fill in ownership later.",
  "总额度限制 (Token，可选)": "Total quota limit (tokens, optional)",
  不填表示不限制: "Leave blank for no limit",
  K: "K",
  M: "M",
  "达到上限后，这把平台密钥的新请求会被拒绝；已在途请求会按完成后的真实用量继续统计。":
    "After the limit is reached, new requests using this platform key will be rejected. In-flight requests continue to be counted by their final actual usage.",
  按: "By",
  参考估算: "Reference estimate",
};
