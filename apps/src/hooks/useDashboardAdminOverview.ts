"use client";

import { useQuery } from "@tanstack/react-query";
import { useDeferredDesktopActivation } from "@/hooks/useDeferredDesktopActivation";
import { useDesktopPageActive } from "@/hooks/useDesktopPageActive";
import { useLocalDayRange } from "@/hooks/useLocalDayRange";
import { dashboardClient } from "@/lib/api/dashboard-client";
import { useAppStore } from "@/lib/store/useAppStore";
import type { DashboardAdminOverview } from "@/types";

export const DASHBOARD_ADMIN_OVERVIEW_QUERY_KEY = [
  "dashboard",
  "admin-overview",
] as const;

export const DASHBOARD_ADMIN_OVERVIEW_STALE_TIME = 30_000;
export const DASHBOARD_ADMIN_OVERVIEW_REQUEST_LOG_LIMIT = 120;
export const DASHBOARD_ADMIN_OVERVIEW_ACCOUNT_LIMIT = 20;

interface UseDashboardAdminOverviewParams {
  requestLogLimit?: number;
  accountLimit?: number;
  startTs?: number | null;
  endTs?: number | null;
  rankingLimit?: number | null;
}

/**
 * 统一的管理员 Dashboard hook
 *
 * 合并了 useDashboardStats 和 useDashboardAdminUsageSummary 的功能，
 * 避免首页同时调用多个 hook 导致的重复聚合查询。
 *
 * 此 hook 一次性获取首页所需的所有数据：
 * - 账号统计和列表
 * - 今日用量摘要
 * - 最近请求日志
 * - 用量聚合信息
 * - 用户、账号、聚合 API 的排名数据
 * - 每日用量趋势
 */
export function useDashboardAdminOverview(
  params: UseDashboardAdminOverviewParams = {},
  enabled = true,
) {
  const serviceStatus = useAppStore((state) => state.serviceStatus);
  const localDayRange = useLocalDayRange();
  const isPageActive = useDesktopPageActive("/");
  const isServiceReady = serviceStatus.connected;
  const isQueryEnabled = useDeferredDesktopActivation(
    enabled && isServiceReady && isPageActive,
  );

  const requestLogLimit =
    params.requestLogLimit ?? DASHBOARD_ADMIN_OVERVIEW_REQUEST_LOG_LIMIT;
  const accountLimit = params.accountLimit ?? DASHBOARD_ADMIN_OVERVIEW_ACCOUNT_LIMIT;

  const query = useQuery<DashboardAdminOverview>({
    queryKey: [
      ...DASHBOARD_ADMIN_OVERVIEW_QUERY_KEY,
      serviceStatus.addr,
      requestLogLimit,
      localDayRange.dayStartTs,
      accountLimit,
      params.startTs ?? null,
      params.endTs ?? null,
      params.rankingLimit ?? null,
    ],
    queryFn: () =>
      dashboardClient.getAdminOverview({
        requestLogLimit,
        dayStartTs: localDayRange.dayStartTs,
        dayEndTs: localDayRange.dayEndTs,
        accountLimit,
        startTs: params.startTs ?? null,
        endTs: params.endTs ?? null,
        rankingLimit: params.rankingLimit ?? null,
      }),
    enabled: isQueryEnabled,
    retry: 1,
    staleTime: DASHBOARD_ADMIN_OVERVIEW_STALE_TIME,
  });

  return {
    ...query,
    isServiceReady,
  };
}
