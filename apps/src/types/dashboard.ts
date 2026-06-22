import type { ModelInfo } from "@/types/model";
import type { RequestLog } from "@/types/request-log";
import type {
  Account,
  AccountUsage,
  UsageAggregateSummary,
} from "@/types/account";
import type { RequestLogTodaySummary } from "@/types/request-log";

export interface DashboardTokenUsage {
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningOutputTokens: number;
  totalTokens: number;
  estimatedCostUsd: number;
  requestCount: number;
  successCount: number;
  errorCount: number;
}

export interface DashboardDailyUsagePoint {
  dayStartTs: number;
  dayEndTs: number;
  usage: DashboardTokenUsage;
}

export interface DashboardUserUsageSummary {
  userId: string;
  username: string | null;
  displayName: string | null;
  role: string | null;
  status: string | null;
  walletAvailableCreditMicros: number | null;
  todayUsage: DashboardTokenUsage;
  rangeUsage: DashboardTokenUsage;
}

export interface DashboardSourceUsageSummary {
  sourceKind: string;
  sourceId: string;
  name: string | null;
  status: string | null;
  provider: string | null;
  todayUsage: DashboardTokenUsage;
  rangeUsage: DashboardTokenUsage;
}

export interface DashboardAdminUsageSummary {
  rangeStartTs: number;
  rangeEndTs: number;
  todayStartTs: number;
  todayEndTs: number;
  todayUsage: DashboardTokenUsage;
  dailyUsage: DashboardDailyUsagePoint[];
  users: DashboardUserUsageSummary[];
  openaiAccounts: DashboardSourceUsageSummary[];
  aggregateApis: DashboardSourceUsageSummary[];
}

/**
 * 合并的管理员 Dashboard 数据
 * 包含 StartupSnapshot 和 AdminUsageSummary 的所有字段
 * 避免首页同时调用多个 hook 导致的重复聚合查询
 */
export interface DashboardAdminOverview {
  // 来自 StartupSnapshot 的基础统计
  accountTotal: number;
  accountAvailable: number;
  apiKeyTotal: number;
  accounts: Account[];
  usageSnapshots: AccountUsage[];
  usageAggregateSummary: UsageAggregateSummary;
  manualPreferredAccountId: string | null;
  requestLogTodaySummary: RequestLogTodaySummary;
  requestLogs: RequestLog[];
  // 来自 AdminUsageSummary 的聚合数据
  rangeStartTs: number;
  rangeEndTs: number;
  todayStartTs: number;
  todayEndTs: number;
  dailyUsage: DashboardDailyUsagePoint[];
  users: DashboardUserUsageSummary[];
  openaiAccounts: DashboardSourceUsageSummary[];
  aggregateApis: DashboardSourceUsageSummary[];
}


export interface MemberDashboardWallet {
  id: string;
  balanceCreditMicros: number;
  frozenCreditMicros: number;
  availableCreditMicros: number;
  status: string;
  updatedAt: number;
}

export interface MemberDashboardApiKeySummary {
  totalCount: number;
  enabledCount: number;
  disabledCount: number;
  lastUsedAt: number | null;
}

export interface MemberDashboardUsageToday {
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningOutputTokens: number;
  totalTokens: number;
  estimatedCostUsd: number;
  totalCount: number;
  successCount: number;
  errorCount: number;
  successRate: number | null;
}

export interface MemberDashboardUsagePoint {
  dayStartTs: number;
  dayEndTs: number;
  totalTokens: number;
  estimatedCostUsd: number;
}

export interface MemberDashboardKeyUsage {
  keyId: string;
  name: string | null;
  modelSlug: string | null;
  status: string;
  todayTokens: number;
  todayCostUsd: number;
  totalTokens: number;
  totalCostUsd: number;
  lastUsedAt: number | null;
}

export interface MemberDashboardModelUsage {
  model: string;
  totalTokens: number;
  estimatedCostUsd: number;
}

export interface MemberDashboardAlert {
  kind: string;
  severity: "info" | "warning" | "critical" | string;
  title: string;
  message: string;
  actionLabel: string | null;
  actionHref: string | null;
}

export interface MemberDashboardSummary {
  userId: string | null;
  distributionEnabled: boolean;
  wallet: MemberDashboardWallet | null;
  apiKeySummary: MemberDashboardApiKeySummary;
  usageToday: MemberDashboardUsageToday;
  usageTrend7d: MemberDashboardUsagePoint[];
  topKeys: MemberDashboardKeyUsage[];
  topModels: MemberDashboardModelUsage[];
  availableModels: ModelInfo[];
  recentLogs: RequestLog[];
  alerts: MemberDashboardAlert[];
}
