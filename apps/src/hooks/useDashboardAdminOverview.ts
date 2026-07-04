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
}

/**
 * 管理员 Dashboard 基础概览 hook
 *
 * 此 hook 只获取首屏基础数据：
 * - 账号统计和列表
 * - 今日用量摘要
 * - 最近请求日志
 * - 用量聚合信息
 *
 * 用户、账号、聚合 API 的排行和每日趋势由 useDashboardAdminUsageSummary 独立加载。
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
    ],
    queryFn: () =>
      dashboardClient.getAdminOverview({
        requestLogLimit,
        dayStartTs: localDayRange.dayStartTs,
        dayEndTs: localDayRange.dayEndTs,
        accountLimit,
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
