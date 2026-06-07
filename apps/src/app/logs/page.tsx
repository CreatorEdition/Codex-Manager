"use client";

import { Suspense, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Database } from "lucide-react";
import { toast } from "sonner";
import { ConfirmDialog } from "@/components/modals/confirm-dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { accountClient } from "@/lib/api/account-client";
import {
  buildStartupSnapshotQueryKey,
  STARTUP_SNAPSHOT_REQUEST_LOG_LIMIT,
} from "@/lib/api/startup-snapshot";
import { serviceClient } from "@/lib/api/service-client";
import { useDesktopPageActive } from "@/hooks/useDesktopPageActive";
import { useDeferredDesktopActivation } from "@/hooks/useDeferredDesktopActivation";
import {
  isAdminRole,
  resolveSessionRole,
  useAppSession,
} from "@/hooks/useAppSession";
import { useLocalDayRange } from "@/hooks/useLocalDayRange";
import { usePageTransitionReady } from "@/hooks/usePageTransitionReady";
import { useRuntimeCapabilities } from "@/hooks/useRuntimeCapabilities";
import { useCodexProfileModeStatus } from "@/hooks/useCodexProfileModeStatus";
import { useI18n } from "@/lib/i18n/provider";
import { useAppStore } from "@/lib/store/useAppStore";
import { RequestLogsTabContent } from "./page-sections";
import {
<<<<<<< HEAD
  AggregateApi,
  ApiKey,
  RequestLog,
  RequestLogFilterSummary,
  RequestLogListResult,
  StartupSnapshot,
} from "@/types";

type StatusFilter = "all" | "2xx" | "4xx" | "5xx";
type LogsTab = "requests";
type TimeRangePreset = "all" | "30m" | "2h" | "24h" | "today" | "custom";
type TranslateFn = (message: string, values?: Record<string, string | number>) => string;
const REQUEST_LOG_LIST_REFETCH_INTERVAL_MS = 30_000;

function padDateTimeSegment(value: number): string {
  return String(value).padStart(2, "0");
}

function toDateTimeLocalValue(timestampSeconds: number | null | undefined): string {
  if (!timestampSeconds) return "";
  const date = new Date(timestampSeconds * 1000);
  if (Number.isNaN(date.getTime())) return "";
  const year = date.getFullYear();
  const month = padDateTimeSegment(date.getMonth() + 1);
  const day = padDateTimeSegment(date.getDate());
  const hour = padDateTimeSegment(date.getHours());
  const minute = padDateTimeSegment(date.getMinutes());
  return `${year}-${month}-${day}T${hour}:${minute}`;
}

function fromDateTimeLocalValue(value: string): number | null {
  const normalized = String(value || "").trim();
  if (!normalized) return null;
  const parsed = new Date(normalized);
  if (Number.isNaN(parsed.getTime())) {
    return null;
  }
  return Math.floor(parsed.getTime() / 1000);
}

function buildFixedTimePreset(
  preset: Exclude<TimeRangePreset, "all" | "custom">,
  localDayStartTs: number,
  localDayEndTs: number,
): { startInput: string; endInput: string } {
  if (preset === "today") {
    return {
      startInput: toDateTimeLocalValue(localDayStartTs),
      endInput: toDateTimeLocalValue(localDayEndTs),
    };
  }

  const nowTs = Math.floor(Date.now() / 1000);
  const durationSeconds =
    preset === "30m" ? 30 * 60 : preset === "2h" ? 2 * 60 * 60 : 24 * 60 * 60;
  return {
    startInput: toDateTimeLocalValue(nowTs - durationSeconds),
    endInput: toDateTimeLocalValue(nowTs),
  };
}

/**
 * 函数 `getStatusBadge`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - statusCode: 参数 statusCode
 *
 * # 返回
 * 返回函数执行结果
 */
function getStatusBadge(statusCode: number | null) {
  if (statusCode == null) {
    return <Badge variant="secondary">-</Badge>;
  }
  if (statusCode >= 200 && statusCode < 300) {
    return (
      <Badge className="border-green-500/20 bg-green-500/10 text-green-500">
        {statusCode}
      </Badge>
    );
  }
  if (statusCode >= 400 && statusCode < 500) {
    return (
      <Badge className="border-yellow-500/20 bg-yellow-500/10 text-yellow-500">
        {statusCode}
      </Badge>
    );
  }
  return (
    <Badge className="border-red-500/20 bg-red-500/10 text-red-500">
      {statusCode}
    </Badge>
  );
}

/**
 * 函数 `SummaryCard`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function SummaryCard({
  title,
  value,
  description,
  icon: Icon,
  toneClass,
}: {
  title: string;
  value: string;
  description: string;
  icon: LucideIcon;
  toneClass: string;
}) {
  return (
    <Card
      size="sm"
      className="glass-card shadow-sm transition-all"
    >
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-1.5">
        <CardTitle className="text-[13px] font-medium text-muted-foreground">
          {title}
        </CardTitle>
        <div
          className={cn(
            "flex h-8 w-8 items-center justify-center rounded-xl",
            toneClass,
          )}
        >
          <Icon className="h-3.5 w-3.5" />
        </div>
      </CardHeader>
      <CardContent className="space-y-0.5">
        <div className="text-[2rem] leading-none font-semibold tracking-tight">
          {value}
        </div>
        <p className="text-[11px] text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  );
}

/**
 * 函数 `LogsPageSkeleton`
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
function LogsPageSkeleton() {
  return (
    <div className="space-y-5">
      <Skeleton className="h-28 w-full rounded-xl" />
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-32 w-full rounded-xl" />
        ))}
      </div>
      <Skeleton className="h-[420px] w-full rounded-xl" />
    </div>
  );
}

/**
 * 函数 `formatDuration`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - value: 参数 value
 *
 * # 返回
 * 返回函数执行结果
 */
function formatDuration(value: number | null): string {
  if (value == null) return "-";
  if (value >= 10_000) return `${Math.round(value / 1000)}s`;
  if (value >= 1000) return `${(value / 1000).toFixed(1).replace(/\.0$/, "")}s`;
  return `${Math.round(value)}ms`;
}

/**
 * 函数 `formatTokenAmount`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - value: 参数 value
 *
 * # 返回
 * 返回函数执行结果
 */
function formatTokenAmount(value: number | null | undefined): string {
  const normalized =
    typeof value === "number" && Number.isFinite(value) ? Math.max(0, value) : 0;
  return normalized.toLocaleString("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

/**
 * 函数 `formatCompactTokenAmount`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - value: 参数 value
 *
 * # 返回
 * 返回函数执行结果
 */
function formatCompactTokenAmount(value: number | null | undefined): string {
  const normalized =
    typeof value === "number" && Number.isFinite(value) ? Math.max(0, value) : 0;
  if (normalized < 1000) {
    return formatTokenAmount(normalized);
  }
  return formatCompactNumber(normalized, "0.00", 2, true);
}

/**
 * 函数 `formatTableTokenAmount`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - value: 参数 value
 *
 * # 返回
 * 返回函数执行结果
 */
function formatTableTokenAmount(value: number | null | undefined): string {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "-";
  }
  const normalized = Math.max(0, value);
  return Math.round(normalized).toLocaleString("zh-CN");
}

/**
 * 函数 `fallbackAccountNameFromId`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - accountId: 参数 accountId
 *
 * # 返回
 * 返回函数执行结果
 */
function fallbackAccountNameFromId(accountId: string): string {
  const raw = accountId.trim();
  if (!raw) return "";
  const sep = raw.indexOf("::");
  if (sep < 0) return "";
  return raw.slice(sep + 2).trim();
}

/**
 * 函数 `fallbackAccountDisplayFromKey`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - keyId: 参数 keyId
 *
 * # 返回
 * 返回函数执行结果
 */
function fallbackAccountDisplayFromKey(keyId: string): string {
  const raw = keyId.trim();
  if (!raw) return "";
  return `Key ${raw.slice(0, 10)}`;
}

/**
 * 函数 `formatCompactKeyLabel`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - keyId: 参数 keyId
 *
 * # 返回
 * 返回函数执行结果
 */
function formatCompactKeyLabel(keyId: string): string {
  if (!keyId) return "-";
  if (keyId.length <= 12) return keyId;
  return `${keyId.slice(0, 8)}...`;
}

/**
 * 函数 `resolveDisplayRequestPath`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveDisplayRequestPath(log: RequestLog): string {
  const originalPath = String(log.originalPath || "").trim();
  if (originalPath) {
    return originalPath;
  }
  return String(log.path || log.requestPath || "").trim();
}

/**
 * 函数 `resolveFriendlyRequestPathLabel`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-14
 *
 * # 参数
 * - path: 参数 path
 * - t: 参数 t
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveFriendlyRequestPathLabel(
  path: string,
  t: TranslateFn,
): string {
  const normalized = String(path || "").trim();
  switch (normalized) {
    case "/v1/responses/compact":
      return t("上下文压缩");
    case "/internal/account/warmup":
      return t("账号预热");
    default:
      return normalized;
  }
}

/**
 * 函数 `resolveUpstreamDisplay`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - upstreamUrl: 参数 upstreamUrl
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveUpstreamDisplay(upstreamUrl: string, t: TranslateFn): string {
  const raw = String(upstreamUrl || "").trim();
  if (!raw) return "";
  if (raw === "默认" || raw === "本地" || raw === "自定义") {
    return t(raw);
  }
  try {
    const url = new URL(raw);
    const pathname = url.pathname.replace(/\/+$/, "");
    return pathname ? `${url.host}${pathname}` : url.host;
  } catch {
    return raw;
  }
}

/**
 * 函数 `resolveAccountDisplayName`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 * - accountNameMap: 参数 accountNameMap
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveAccountDisplayName(
  log: RequestLog,
  accountNameMap: Map<string, string>,
): string {
  if (log.accountId) {
    const label = accountNameMap.get(log.accountId);
    if (label) {
      return label;
    }
    const fallbackName = fallbackAccountNameFromId(log.accountId);
    if (fallbackName) {
      return fallbackName;
    }
  }
  return fallbackAccountDisplayFromKey(log.keyId);
}

/**
 * 函数 `resolveAccountDisplayNameById`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - accountId: 参数 accountId
 * - accountNameMap: 参数 accountNameMap
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveAccountDisplayNameById(
  accountId: string,
  accountNameMap: Map<string, string>,
): string {
  const normalized = String(accountId || "").trim();
  if (!normalized) return "";
  return (
    accountNameMap.get(normalized) ||
    fallbackAccountNameFromId(normalized) ||
    normalized
  );
}

/**
 * 函数 `resolveDisplayedStatusCode`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveDisplayedStatusCode(log: RequestLog): number | null {
  const statusCode = log.statusCode;
  const hasError = Boolean(String(log.error || "").trim());
  if (statusCode == null) {
    return hasError ? 502 : null;
  }
  if (hasError && statusCode < 400) {
    return 502;
  }
  return statusCode;
}

/**
 * 函数 `resolveAggregateApiDisplayName`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 * - aggregateApi: 参数 aggregateApi
 * - apiKey: 参数 apiKey
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveAggregateApiDisplayName(
  log: RequestLog,
  aggregateApi: AggregateApi | null,
  apiKey: ApiKey | null,
): string {
  if (log.aggregateApiSupplierName && log.aggregateApiSupplierName.trim()) {
    return log.aggregateApiSupplierName.trim();
  }
  if (aggregateApi?.supplierName && aggregateApi.supplierName.trim()) {
    return aggregateApi.supplierName.trim();
  }
  if (apiKey?.aggregateApiUrl) {
    return apiKey.aggregateApiUrl.trim();
  }
  return "-";
}

/**
 * 函数 `resolveAggregateApiTooltipUrl`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 * - aggregateApi: 参数 aggregateApi
 * - apiKey: 参数 apiKey
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveAggregateApiTooltipUrl(
  log: RequestLog,
  aggregateApi: AggregateApi | null,
  apiKey: ApiKey | null,
): string {
  if (log.aggregateApiUrl && log.aggregateApiUrl.trim()) {
    return log.aggregateApiUrl.trim();
  }
  if (aggregateApi?.url && aggregateApi.url.trim()) {
    return aggregateApi.url.trim();
  }
  if (apiKey?.aggregateApiUrl) {
    return apiKey.aggregateApiUrl.trim();
  }
  return "-";
}

/**
 * 函数 `resolveAggregateApiDisplayNameById`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - aggregateApiId: 参数 aggregateApiId
 * - aggregateApiMap: 参数 aggregateApiMap
 *
 * # 返回
 * 返回函数执行结果
 */
function resolveAggregateApiDisplayNameById(
  aggregateApiId: string,
  aggregateApiMap: Map<string, AggregateApi>,
): string {
  const normalized = String(aggregateApiId || "").trim();
  if (!normalized) return "";
  const aggregateApi = aggregateApiMap.get(normalized);
  if (aggregateApi?.supplierName && aggregateApi.supplierName.trim()) {
    return aggregateApi.supplierName.trim();
  }
  if (aggregateApi?.url && aggregateApi.url.trim()) {
    return aggregateApi.url.trim();
  }
  return normalized;
}

/**
 * 函数 `normalizeAggregateApiUrl`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - value: 参数 value
 *
 * # 返回
 * 返回函数执行结果
 */
function normalizeAggregateApiUrl(value: string): string {
  return String(value || "").trim().replace(/\/+$/, "");
}

/**
 * 函数 `formatModelEffortDisplay`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - log: 参数 log
 *
 * # 返回
 * 返回函数执行结果
 */
function formatModelEffortDisplay(log: RequestLog): string {
  const model = String(log.model || "").trim();
  const effort = String(log.reasoningEffort || "").trim();
  if (model && effort) {
    return `${model}/${effort}`;
  }
  return model || effort || "-";
}

function normalizeRequestType(value: string): "ws" | "http" {
  return String(value || "").trim().toLowerCase() === "ws" ? "ws" : "http";
}

function normalizeDisplayServiceTier(value: string | null | undefined): string {
  const normalized = String(value || "").trim().toLowerCase();
  if (!normalized || normalized === "auto") {
    return "";
  }
  if (normalized === "priority") {
    return "fast";
  }
  return normalized;
}

function resolveDisplayServiceTier(
  requestServiceTier: string | null | undefined,
): string {
  const direct = normalizeDisplayServiceTier(requestServiceTier);
  if (direct) {
    return direct;
  }
  return "auto";
}

function RequestTypeBadge({ requestType }: { requestType: string }) {
  const normalized = normalizeRequestType(requestType);
  const label = normalized.toUpperCase();
  const toneClass =
    normalized === "ws"
      ? "border-cyan-500/20 bg-cyan-500/10 text-cyan-500"
      : "border-slate-500/20 bg-slate-500/10 text-slate-500";
  return (
    <Badge className={cn("h-5 rounded-full px-1.5 text-[10px] font-medium", toneClass)}>
      {label}
    </Badge>
  );
}

function ServiceTierBadge({ serviceTier }: { serviceTier: string }) {
  const normalized = resolveDisplayServiceTier(serviceTier);
  const toneClass =
    normalized === "fast"
      ? "border-amber-500/20 bg-amber-500/10 text-amber-500"
      : "border-slate-500/20 bg-slate-500/10 text-slate-500";
  return (
    <Badge className={cn("h-5 rounded-full px-1.5 text-[10px] font-medium", toneClass)}>
      {normalized}
    </Badge>
  );
}

/**
 * 函数 `AccountKeyInfoCell`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function AccountKeyInfoCell({
  log,
  accountLabel,
  accountNameMap,
  apiKeyMap,
  aggregateApiMap,
}: {
  log: RequestLog;
  accountLabel: string;
  accountNameMap: Map<string, string>;
  apiKeyMap: Map<string, ApiKey>;
  aggregateApiMap: Map<string, AggregateApi>;
}) {
  const { t } = useI18n();
  const displayAccount = accountLabel || log.accountId || "-";
  const hasNamedAccount =
    Boolean(accountLabel) &&
    accountLabel.trim() !== "" &&
    accountLabel !== log.accountId;
  const attemptedAccountLabels = log.attemptedAccountIds
    .map((accountId) =>
      resolveAccountDisplayNameById(accountId, accountNameMap),
    )
    .filter((value) => value.trim().length > 0);
  const initialAccountLabel = resolveAccountDisplayNameById(
    log.initialAccountId,
    accountNameMap,
  );
  const attemptedAggregateApiLabels = log.attemptedAggregateApiIds
    .map((aggregateApiId) =>
      resolveAggregateApiDisplayNameById(aggregateApiId, aggregateApiMap),
    )
    .filter((value) => value.trim().length > 0);
  const initialAggregateApiLabel = resolveAggregateApiDisplayNameById(
    log.initialAggregateApiId,
    aggregateApiMap,
  );
  const apiKey = apiKeyMap.get(log.keyId) || null;
  const apiKeyName = String(apiKey?.name || "").trim();
  const apiKeyDisplayName = apiKeyName || formatCompactKeyLabel(log.keyId);
  const aggregateApiById = apiKey?.aggregateApiId
    ? aggregateApiMap.get(apiKey.aggregateApiId) || null
    : null;
  const actualAggregateApi =
    log.actualSourceKind === "aggregate_api" && log.actualSourceId
      ? aggregateApiMap.get(log.actualSourceId) || null
      : null;
  /**
   * 函数 `aggregateApiByUrl`
   *
   * 作者: gaohongshun
   *
   * 时间: 2026-04-02
   *
   * # 参数
   * - (): 参数 ()
   *
   * # 返回
   * 返回函数执行结果
   */
  const aggregateApiByUrl = (() => {
    const upstreamUrl = normalizeAggregateApiUrl(log.upstreamUrl);
    if (!upstreamUrl) return null;
    for (const aggregateApi of aggregateApiMap.values()) {
      if (normalizeAggregateApiUrl(aggregateApi.url) === upstreamUrl) {
        return aggregateApi;
      }
    }
    return null;
  })();
  const aggregateApi = actualAggregateApi || aggregateApiById || aggregateApiByUrl;
  const selectedAggregateApiId =
    log.actualSourceKind === "aggregate_api" && log.actualSourceId
      ? log.actualSourceId
      : aggregateApi?.id || "";
  const isAggregateApi = Boolean(
    log.actualSourceKind === "aggregate_api" ||
      log.aggregateApiSupplierName ||
      log.aggregateApiUrl ||
      aggregateApi,
  );
  const aggregateApiDisplayName = resolveAggregateApiDisplayName(
    log,
    aggregateApi,
    apiKey,
  );
  const aggregateApiDisplayUrl = resolveAggregateApiTooltipUrl(
    log,
    aggregateApi,
    apiKey,
  );
  const showAttemptHint =
    attemptedAccountLabels.length > 1 &&
    initialAccountLabel &&
    initialAccountLabel !== displayAccount;
  const showAggregateAttemptHint =
    attemptedAggregateApiLabels.length > 1 &&
    initialAggregateApiLabel &&
    String(log.initialAggregateApiId || "").trim() !== selectedAggregateApiId;

  if (isAggregateApi) {
    return (
      <Tooltip>
        <TooltipTrigger render={<div />} className="block text-left">
          <div className="flex max-w-[180px] flex-col gap-0.5 opacity-80">
            <div className="flex items-center gap-1">
              <Database className="h-3 w-3 text-primary" />
              <span className="truncate text-[11px] font-medium">
                {aggregateApiDisplayName}
              </span>
            </div>
            <div className="truncate font-mono text-[9px] text-muted-foreground">
              {aggregateApiDisplayUrl}
            </div>
            <div className="flex items-center gap-1 text-[9px] text-muted-foreground">
              <Shield className="h-2.5 w-2.5" />
              <span className={apiKeyName ? "truncate" : "font-mono"}>
                {apiKeyDisplayName}
              </span>
            </div>
            {showAggregateAttemptHint ? (
              <div className="text-[9px] text-amber-500">
                {t("先试")} {initialAggregateApiLabel}
              </div>
            ) : null}
          </div>
        </TooltipTrigger>
        <TooltipContent className="max-w-sm">
          <div className="flex min-w-[240px] flex-col gap-2">
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("供应商名称")}</div>
              <div className="break-all font-mono text-[11px]">
                {aggregateApiDisplayName}
              </div>
            </div>
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">URL</div>
              <div className="break-all font-mono text-[11px]">
                {aggregateApiDisplayUrl}
              </div>
            </div>
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("密钥")}</div>
              <div className="break-all text-[11px]">
                {apiKeyDisplayName || "-"}
              </div>
            </div>
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("密钥 ID")}</div>
              <div className="break-all font-mono text-[11px]">
                {log.keyId || "-"}
              </div>
            </div>
            {attemptedAggregateApiLabels.length > 1 ? (
              <div className="space-y-0.5">
                <div className="text-[10px] text-background/70">{t("尝试链路")}</div>
                <div className="break-all font-mono text-[11px]">
                  {attemptedAggregateApiLabels.join(" -> ")}
                </div>
              </div>
            ) : null}
            {initialAggregateApiLabel ? (
              <div className="space-y-0.5">
                <div className="text-[10px] text-background/70">{t("首尝试渠道")}</div>
                <div className="break-all font-mono text-[11px]">
                  {initialAggregateApiLabel}
                </div>
              </div>
            ) : null}
          </div>
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    <Tooltip>
      <TooltipTrigger render={<div />} className="block text-left">
        <div className="flex flex-col gap-0.5 opacity-80">
          <div className="flex items-center gap-1">
            <Zap className="h-3 w-3 text-yellow-500" />
            <span className="max-w-[140px] truncate">{displayAccount}</span>
          </div>
          <div className="flex items-center gap-1 text-[9px] text-muted-foreground">
            <Shield className="h-2.5 w-2.5" />
            <span className={apiKeyName ? "max-w-[140px] truncate" : "font-mono"}>
              {apiKeyDisplayName}
            </span>
          </div>
          {showAttemptHint ? (
            <div className="text-[9px] text-amber-500">
              {t("先试")} {initialAccountLabel}
            </div>
          ) : null}
        </div>
      </TooltipTrigger>
      <TooltipContent className="max-w-sm">
        <div className="flex min-w-[240px] flex-col gap-2">
          {initialAccountLabel ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("首尝试账号")}</div>
              <div className="break-all font-mono text-[11px]">
                {initialAccountLabel}
              </div>
            </div>
          ) : null}
          {attemptedAccountLabels.length > 1 ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("尝试链路")}</div>
              <div className="break-all font-mono text-[11px]">
                {attemptedAccountLabels.join(" -> ")}
              </div>
            </div>
          ) : null}
          {hasNamedAccount ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("邮箱 / 名称")}</div>
              <div className="break-all font-mono text-[11px]">
                {accountLabel}
              </div>
            </div>
          ) : null}
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("账号 ID")}</div>
            <div className="break-all font-mono text-[11px]">
              {log.accountId || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("密钥")}</div>
            <div className="break-all text-[11px]">
              {apiKeyDisplayName || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("密钥 ID")}</div>
            <div className="break-all font-mono text-[11px]">
              {log.keyId || "-"}
            </div>
          </div>
        </div>
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 函数 `RequestRouteInfoCell`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function RequestRouteInfoCell({ log }: { log: RequestLog }) {
  const { t } = useI18n();
  const displayPath = resolveDisplayRequestPath(log) || "-";
  const displayPathLabel = resolveFriendlyRequestPathLabel(displayPath, t) || "-";
  const recordedPath = String(log.path || log.requestPath || "").trim();
  const originalPath = String(log.originalPath || "").trim();
  const adaptedPath = String(log.adaptedPath || "").trim();
  const gatewayMode = String(log.gatewayMode || "").trim().toLowerCase();
  const isCompactGatewayMode = gatewayMode === "compact";
  const upstreamUrl = String(log.upstreamUrl || "").trim();
  const upstreamDisplay = resolveUpstreamDisplay(upstreamUrl, t);
  const forwardedPath = adaptedPath && adaptedPath !== displayPath ? adaptedPath : "";
  const friendlyDisplayPath =
    isCompactGatewayMode
      ? t("上下文压缩")
      : displayPathLabel && displayPathLabel !== displayPath
        ? displayPathLabel
        : "";
  const requestType = normalizeRequestType(log.requestType);
  const canonicalSource = String(log.canonicalSource || "native_codex").trim();
  const sizeRejectStage = String(log.sizeRejectStage || "-").trim();

  return (
    <Tooltip>
      <TooltipTrigger render={<div />} className="block text-left">
        <div className="flex flex-col gap-0.5">
          <div className="flex items-center gap-1.5">
            <RequestTypeBadge requestType={requestType} />
            {isCompactGatewayMode ? (
              <Badge className="h-5 rounded-full border-amber-500/20 bg-amber-500/10 px-1.5 text-[10px] font-medium text-amber-500">
                {t("压缩")}
              </Badge>
            ) : null}
            <span className="font-bold text-primary">{log.method || "-"}</span>
          </div>
          <span className="max-w-[220px] truncate font-mono text-[11px] text-foreground">
            {displayPath}
          </span>
          {friendlyDisplayPath ? (
            <span className="max-w-[220px] truncate text-[10px] text-muted-foreground">
              {friendlyDisplayPath}
            </span>
          ) : null}
          {forwardedPath ? (
            <span className="max-w-[220px] truncate font-mono text-[10px] text-amber-500">
              -&gt; {forwardedPath}
            </span>
          ) : null}
          {upstreamDisplay ? (
            <span className="max-w-[220px] truncate font-mono text-[10px] text-cyan-500">
              =&gt; {upstreamDisplay}
            </span>
          ) : null}
        </div>
      </TooltipTrigger>
      <TooltipContent className="max-w-md">
        <div className="flex min-w-[280px] flex-col gap-2">
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("请求类型")}</div>
            <div className="font-mono text-[11px] uppercase">{requestType}</div>
          </div>
          {gatewayMode ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("网关模式")}</div>
              <div className="font-mono text-[11px]">{gatewayMode}</div>
            </div>
          ) : null}
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">
              {t("规范来源")}
            </div>
            <div className="font-mono text-[11px]">{canonicalSource}</div>
          </div>
          {sizeRejectStage && sizeRejectStage !== "-" ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">
                {t("大小拒绝阶段")}
              </div>
              <div className="font-mono text-[11px]">{sizeRejectStage}</div>
            </div>
          ) : null}
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("方法")}</div>
            <div className="font-mono text-[11px]">{log.method || "-"}</div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("显示名称")}</div>
            <div className="break-all text-[11px]">{displayPathLabel}</div>
          </div>
          {displayPath && displayPathLabel !== displayPath ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("原始路径")}</div>
              <div className="break-all font-mono text-[11px]">{displayPath}</div>
            </div>
          ) : null}
          {recordedPath && recordedPath !== displayPath ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("记录地址")}</div>
              <div className="break-all font-mono text-[11px]">
                {recordedPath}
              </div>
            </div>
          ) : null}
          {originalPath && originalPath !== displayPath ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("原始地址")}</div>
              <div className="break-all font-mono text-[11px]">
                {originalPath}
              </div>
            </div>
          ) : null}
          {forwardedPath ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("转发路径")}</div>
              <div className="break-all font-mono text-[11px]">
                {forwardedPath}
              </div>
            </div>
          ) : null}
          {log.responseAdapter ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("适配器")}</div>
              <div className="break-all font-mono text-[11px]">
                {log.responseAdapter}
              </div>
            </div>
          ) : null}
          {upstreamDisplay ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("上游")}</div>
              <div className="break-all font-mono text-[11px]">
                {upstreamDisplay}
              </div>
            </div>
          ) : null}
          {upstreamUrl ? (
            <div className="space-y-0.5">
              <div className="text-[10px] text-background/70">{t("上游地址")}</div>
              <div className="break-all font-mono text-[11px]">
                {upstreamUrl}
              </div>
            </div>
          ) : null}
        </div>
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 函数 `ErrorInfoCell`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function ErrorInfoCell({ error }: { error: string }) {
  const text = String(error || "").trim();
  if (!text) {
    return <span className="text-muted-foreground">-</span>;
  }

  return (
    <Tooltip>
      <TooltipTrigger render={<div />} className="block text-left">
        <span className="block max-w-[220px] truncate font-medium text-red-400">
          {text}
        </span>
      </TooltipTrigger>
      <TooltipContent className="max-w-md">
        <div className="max-w-[360px] break-all font-mono text-[11px]">
          {text}
        </div>
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 函数 `GatewayTooltipCell`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-04
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function GatewayTooltipCell({
  preview,
  content,
  triggerClassName,
  contentClassName,
}: {
  preview: ReactNode;
  content: ReactNode;
  triggerClassName?: string;
  contentClassName?: string;
}) {
  return (
    <Tooltip>
      <TooltipTrigger render={<div />} className="block w-full text-left">
        <div className={cn("w-full", triggerClassName)}>{preview}</div>
      </TooltipTrigger>
      <TooltipContent
        className={cn("max-w-md whitespace-pre-wrap break-all", contentClassName)}
      >
        {content}
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 函数 `ModelEffortCell`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - params: 参数 params
 *
 * # 返回
 * 返回函数执行结果
 */
function ModelEffortCell({
  log,
}: {
  log: RequestLog;
}) {
  const { t } = useI18n();
  const model = String(log.model || "").trim();
  const upstreamModel = String(log.upstreamModel || "").trim();
  const actualSourceKind = String(log.actualSourceKind || "").trim();
  const actualSourceId = String(log.actualSourceId || "").trim();
  const effort = String(log.reasoningEffort || "").trim();
  const clientServiceTier = resolveDisplayServiceTier(log.serviceTier);
  const effectiveServiceTier = resolveDisplayServiceTier(
    log.effectiveServiceTier || log.serviceTier,
  );
  const badgeServiceTier =
    effectiveServiceTier !== "auto" ? effectiveServiceTier : clientServiceTier;
  const display = formatModelEffortDisplay(log);
  const forwardedModel = upstreamModel && upstreamModel !== model ? upstreamModel : "";

  return (
    <Tooltip>
      <TooltipTrigger render={<div />} className="block text-left">
        <div className="flex flex-col gap-1">
          <span className="block max-w-[200px] truncate font-medium text-foreground">
            {display}
          </span>
          {forwardedModel ? (
            <span className="block max-w-[200px] truncate font-mono text-[10px] text-amber-500">
              {t("转发")} {forwardedModel}
            </span>
          ) : null}
          <ServiceTierBadge serviceTier={badgeServiceTier} />
        </div>
      </TooltipTrigger>
      <TooltipContent className="max-w-sm">
        <div className="flex min-w-[220px] flex-col gap-2">
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("平台模型")}</div>
            <div className="break-all font-mono text-[11px]">
              {model || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("上游模型")}</div>
            <div className="break-all font-mono text-[11px]">
              {upstreamModel || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("实际来源")}</div>
            <div className="break-all font-mono text-[11px]">
              {actualSourceKind && actualSourceId
                ? `${actualSourceKind}:${actualSourceId}`
                : actualSourceKind || actualSourceId || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">{t("推理")}</div>
            <div className="break-all font-mono text-[11px]">
              {effort || "-"}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">
              {t("客户端显式服务等级")}
            </div>
            <div className="break-all font-mono text-[11px]">
              {clientServiceTier}
            </div>
          </div>
          <div className="space-y-0.5">
            <div className="text-[10px] text-background/70">
              {t("最终生效服务等级")}
            </div>
            <div className="break-all font-mono text-[11px]">
              {effectiveServiceTier}
            </div>
          </div>
        </div>
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * 函数 `buildSummaryPlaceholder`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - logs: 参数 logs
 *
 * # 返回
 * 返回函数执行结果
 */
function buildSummaryPlaceholder(logs: RequestLog[]): RequestLogFilterSummary {
  const successCount = logs.filter((item) => {
    const statusCode = item.statusCode ?? 0;
    return statusCode >= 200 && statusCode < 300 && !String(item.error || "").trim();
  }).length;
  const errorCount = logs.filter((item) => {
    const statusCode = item.statusCode;
    return Boolean(String(item.error || "").trim()) || (statusCode != null && statusCode >= 400);
  }).length;
  const totalTokens = logs.reduce(
    (sum, item) => sum + Math.max(0, item.totalTokens || 0),
    0
  );
  const totalCostUsd = logs.reduce(
    (sum, item) => sum + Math.max(0, item.estimatedCostUsd || 0),
    0
  );

  return {
    totalCount: logs.length,
    filteredCount: logs.length,
    successCount,
    errorCount,
    totalTokens,
    totalCostUsd,
  };
}

/**
 * 函数 `LogsPageContent`
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
=======
  buildFixedTimePreset,
  LogsPageSkeleton,
  type LogsTab,
  type StatusFilter,
  type TimeRangePreset,
  fromDateTimeLocalValue,
} from "./page-helpers";
import { buildSummaryPlaceholder } from "./page-cells";
import { AccountListResult, ApiKey, RequestLogListResult, StartupSnapshot } from "@/types";

>>>>>>> fccf5a63 (refactor frontend structure and gateway endpoint handling)
function LogsPageContent() {
  const { t } = useI18n();
  const localDayRange = useLocalDayRange();
  const searchParams = useSearchParams();
  const { serviceStatus } = useAppStore();
  const { isDesktopRuntime } = useRuntimeCapabilities();
  const { data: session, isLoading: isSessionLoading } = useAppSession();
  const role = resolveSessionRole(session, isSessionLoading, isDesktopRuntime);
  const isAdminMode = isAdminRole(role);
  const isPageActive = useDesktopPageActive("/logs/");
  const { isDirectAccountMode } = useCodexProfileModeStatus({
    enabled: isAdminMode && isPageActive,
    refetchIntervalMs: 10_000,
  });
  const queryClient = useQueryClient();
  const areLogQueriesEnabled = useDeferredDesktopActivation(serviceStatus.connected);
  const routeQuery = searchParams.get("query") || "";
  const [search, setSearch] = useState(routeQuery);
  const [filter, setFilter] = useState<StatusFilter>("all");
  const [timePreset, setTimePreset] = useState<TimeRangePreset>("all");
  const [startTimeInput, setStartTimeInput] = useState("");
  const [endTimeInput, setEndTimeInput] = useState("");
  const [pageSize, setPageSize] = useState("10");
  const [page, setPage] = useState(1);
  const [clearConfirmOpen, setClearConfirmOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<LogsTab>("requests");
  const pageSizeNumber = Number(pageSize) || 10;
  const startTs = useMemo(
    () => fromDateTimeLocalValue(startTimeInput),
    [startTimeInput],
  );
  const endTs = useMemo(() => fromDateTimeLocalValue(endTimeInput), [endTimeInput]);
  const hasActiveTimeRange = startTs != null || endTs != null;
  const startupSnapshot = queryClient.getQueryData<StartupSnapshot>(
    buildStartupSnapshotQueryKey(
      serviceStatus.addr,
      STARTUP_SNAPSHOT_REQUEST_LOG_LIMIT,
      localDayRange.dayStartTs,
    )
  );
  const startupAccounts = startupSnapshot?.accounts || [];
  const startupApiKeys = startupSnapshot?.apiKeys || [];
  const startupRequestLogs = startupSnapshot?.requestLogs || [];
  const canUseStartupLogsPlaceholder =
    !routeQuery.trim() &&
    !search.trim() &&
    filter === "all" &&
    page === 1 &&
    !hasActiveTimeRange;
  const hasStartupLogsSnapshot =
    canUseStartupLogsPlaceholder && startupRequestLogs.length > 0;

  const { data: logsResult, isLoading, isError: isLogsError } = useQuery({
    queryKey: ["logs", "list", search, filter, startTs, endTs, page, pageSizeNumber],
    queryFn: () =>
      serviceClient.listRequestLogs({
        query: search,
        statusFilter: filter,
        startTs,
        endTs,
        page,
        pageSize: pageSizeNumber,
      }),
    enabled: areLogQueriesEnabled && isPageActive,
    refetchInterval: REQUEST_LOG_LIST_REFETCH_INTERVAL_MS,
    refetchIntervalInBackground: false,
    retry: 1,
    placeholderData: (previousData): RequestLogListResult | undefined =>
      previousData ||
      (hasStartupLogsSnapshot
        ? {
            items: startupRequestLogs,
            total: startupRequestLogs.length,
            page: 1,
            pageSize: pageSizeNumber,
          }
        : undefined),
  });

  const { data: summaryResult, isError: isSummaryError } = useQuery({
    queryKey: ["logs", "summary", search, filter, startTs, endTs],
    queryFn: () =>
      serviceClient.getRequestLogSummary({
        query: search,
        statusFilter: filter,
        startTs,
        endTs,
      }),
    enabled: areLogQueriesEnabled && isPageActive,
    staleTime: 30_000,
    retry: 1,
    placeholderData: (previousData) =>
      previousData ||
      (canUseStartupLogsPlaceholder
        ? buildSummaryPlaceholder(startupRequestLogs)
        : undefined),
  });

  const clearMutation = useMutation({
    mutationFn: () => serviceClient.clearRequestLogs(),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["logs"] }),
        queryClient.invalidateQueries({ queryKey: ["today-summary"] }),
        queryClient.invalidateQueries({ queryKey: ["startup-snapshot"] }),
      ]);
      toast.success(t("日志已清空"));
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : String(error));
    },
  });

  const logs = logsResult?.items || [];
  const apiKeyLookupIds = useMemo(() => {
    const ids = logs
      .map((item) => String(item.keyId || "").trim())
      .filter(Boolean);
    return Array.from(new Set(ids)).sort();
  }, [logs]);

  const accountLookupIds = useMemo(() => {
    const ids = logs
      .map((item) => String(item.accountId || "").trim())
      .filter(Boolean);
    return Array.from(new Set(ids)).sort();
  }, [logs]);

  const { data: accountsResult } = useQuery({
    queryKey: ["accounts", "lookup", accountLookupIds],
    queryFn: () => accountClient.lookupAccounts(accountLookupIds),
    enabled:
      areLogQueriesEnabled &&
      isPageActive &&
      isAdminMode &&
      accountLookupIds.length > 0,
    staleTime: 60_000,
    retry: 1,
    placeholderData: (previousData) => {
      if (previousData) {
        return previousData;
      }
      if (startupAccounts.length === 0 || accountLookupIds.length === 0) {
        return undefined;
      }
      const lookupIdSet = new Set(accountLookupIds);
      return startupAccounts.filter((account) => lookupIdSet.has(account.id));
    },
  });

  const { data: apiKeysResult } = useQuery({
    queryKey: ["apikeys", "lookup", apiKeyLookupIds],
    queryFn: () => accountClient.lookupApiKeys(apiKeyLookupIds),
    enabled: areLogQueriesEnabled && isPageActive && apiKeyLookupIds.length > 0,
    staleTime: 60_000,
    retry: 1,
    placeholderData: (previousData): ApiKey[] | undefined => {
      if (previousData) {
        return previousData;
      }
      if (startupApiKeys.length === 0 || apiKeyLookupIds.length === 0) {
        return undefined;
      }
      const lookupIdSet = new Set(apiKeyLookupIds);
      return startupApiKeys.filter((apiKey) => lookupIdSet.has(apiKey.id));
    },
  });

  const aggregateApiLookupIds = useMemo(() => {
    const ids: string[] = [];
    for (const log of logs) {
      if (log.initialAggregateApiId) {
        ids.push(log.initialAggregateApiId);
      }
      if (log.actualSourceKind === "aggregate_api" && log.actualSourceId) {
        ids.push(log.actualSourceId);
      }
      ids.push(...log.attemptedAggregateApiIds);
    }
    for (const apiKey of apiKeysResult || []) {
      if (apiKey.aggregateApiId) {
        ids.push(apiKey.aggregateApiId);
      }
    }
    return Array.from(
      new Set(ids.map((id) => String(id || "").trim()).filter(Boolean)),
    ).sort();
  }, [apiKeysResult, logs]);

  const { data: aggregateApisResult } = useQuery({
    queryKey: ["aggregate-apis", "lookup", aggregateApiLookupIds],
    queryFn: () => accountClient.lookupAggregateApis(aggregateApiLookupIds),
    enabled:
      areLogQueriesEnabled &&
      isPageActive &&
      isAdminMode &&
      aggregateApiLookupIds.length > 0,
    staleTime: 60_000,
    retry: 1,
  });

  const accountNameMap = useMemo(() => {
    return new Map(
      (accountsResult || []).map((account) => [
        account.id,
        account.label || account.name || account.id,
      ]),
    );
  }, [accountsResult]);

  const apiKeyMap = useMemo(() => {
    return new Map((apiKeysResult || []).map((apiKey) => [apiKey.id, apiKey]));
  }, [apiKeysResult]);

  const aggregateApiMap = useMemo(() => {
    return new Map(
      (aggregateApisResult || []).map((aggregateApi) => [
        aggregateApi.id,
        aggregateApi,
      ]),
    );
  }, [aggregateApisResult]);

  const isLogsLoading =
    serviceStatus.connected &&
    !hasStartupLogsSnapshot &&
    (!areLogQueriesEnabled || isLoading);
  usePageTransitionReady(
    "/logs/",
    !serviceStatus.connected ||
      (!isLogsLoading &&
        (Boolean(summaryResult) || isLogsError || isSummaryError)),
  );
  const currentPage = logsResult?.page || page;
  const summary = summaryResult || {
    totalCount: logsResult?.total || 0,
    filteredCount: logsResult?.total || 0,
    successCount: 0,
    errorCount: 0,
    totalTokens: 0,
    totalCostUsd: 0,
  };
  const totalPages = Math.max(
    1,
    Math.ceil((logsResult?.total || 0) / pageSizeNumber),
  );

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    const frameId = window.requestAnimationFrame(() => {
      setSearch((current) => (current === routeQuery ? current : routeQuery));
      setPage(1);
    });
    return () => {
      window.cancelAnimationFrame(frameId);
    };
  }, [routeQuery]);

  useEffect(() => {
    if (isPageActive) {
      return;
    }
    if (typeof window === "undefined") {
      return;
    }
    const frameId = window.requestAnimationFrame(() => {
      setClearConfirmOpen(false);
    });
    return () => {
      window.cancelAnimationFrame(frameId);
    };
  }, [isPageActive]);

  useEffect(() => {
    if (timePreset !== "today") {
      return;
    }
    const frameId = window.requestAnimationFrame(() => {
      const todayRange = buildFixedTimePreset(
        "today",
        localDayRange.dayStartTs,
        localDayRange.dayEndTs,
      );
      setStartTimeInput((current) =>
        current === todayRange.startInput ? current : todayRange.startInput,
      );
      setEndTimeInput((current) =>
        current === todayRange.endInput ? current : todayRange.endInput,
      );
    });
    return () => {
      window.cancelAnimationFrame(frameId);
    };
  }, [localDayRange.dayEndTs, localDayRange.dayStartTs, timePreset]);

  const currentFilterLabel =
    filter === "all"
      ? t("全部状态")
      : filter === "2xx"
        ? t("成功请求")
        : filter === "4xx"
          ? t("客户端错误")
          : t("服务端错误");
  const currentTimeRangeLabel =
    timePreset === "30m"
      ? t("最近30分钟")
      : timePreset === "2h"
        ? t("最近2小时")
        : timePreset === "24h"
          ? t("最近24小时")
          : timePreset === "today"
            ? t("今天")
            : hasActiveTimeRange
              ? t("自定义时间")
              : t("全部时间");
  const compactMetaText = `${summary.filteredCount}/${summary.totalCount} ${t("条")} · ${currentFilterLabel} · ${currentTimeRangeLabel} · ${
    serviceStatus.connected ? t("5 秒刷新") : t("服务未连接")
  }`;

  const applyTimePreset = (preset: TimeRangePreset) => {
    setTimePreset(preset);
    setPage(1);
    if (preset === "all") {
      setStartTimeInput("");
      setEndTimeInput("");
      return;
    }
    if (preset === "custom") {
      return;
    }
    const nextRange = buildFixedTimePreset(
      preset,
      localDayRange.dayStartTs,
      localDayRange.dayEndTs,
    );
    setStartTimeInput(nextRange.startInput);
    setEndTimeInput(nextRange.endInput);
  };

  return (
    <div className="animate-in space-y-5 fade-in duration-500">
      <Tabs
        value={activeTab}
        onValueChange={(value) => {
          if (value === "requests") {
            setActiveTab("requests");
          }
        }}
        className="w-full"
      >
        <TabsList className="glass-card flex h-11 w-full justify-start overflow-x-auto rounded-xl p-1 no-scrollbar lg:w-fit">
          <TabsTrigger value="requests" className="gap-2 px-5 shrink-0">
            <Database className="h-4 w-4" /> {t("请求日志")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="requests" className="space-y-5">
          <RequestLogsTabContent
            t={t}
            isDirectAccountMode={isDirectAccountMode}
            isAdminMode={isAdminMode}
            serviceConnected={serviceStatus.connected}
            search={search}
            filter={filter}
            timePreset={timePreset}
            startTimeInput={startTimeInput}
            endTimeInput={endTimeInput}
            compactMetaText={compactMetaText}
            hasActiveTimeRange={hasActiveTimeRange}
            pageSize={pageSize}
            currentFilterLabel={currentFilterLabel}
            summary={summary}
            logs={logs}
            isLogsLoading={isLogsLoading}
            currentPage={currentPage}
            totalPages={totalPages}
            accountNameMap={accountNameMap}
            apiKeyMap={apiKeyMap}
            aggregateApiMap={aggregateApiMap}
            clearMutationPending={clearMutation.isPending}
            onSearchChange={(value) => {
              setSearch(value);
              setPage(1);
            }}
            onFilterChange={(value) => {
              setFilter(value);
              setPage(1);
            }}
            onRefresh={() => {
              void queryClient.invalidateQueries({ queryKey: ["logs"] });
            }}
            onOpenClearConfirm={() => setClearConfirmOpen(true)}
            onApplyTimePreset={applyTimePreset}
            onStartTimeChange={(value) => {
              setTimePreset("custom");
              setStartTimeInput(value);
              setPage(1);
            }}
            onEndTimeChange={(value) => {
              setTimePreset("custom");
              setEndTimeInput(value);
              setPage(1);
            }}
            onClearTimeRange={() => applyTimePreset("all")}
            onPageSizeChange={(value) => {
              setPageSize(value || "10");
              setPage(1);
            }}
            onPreviousPage={() => setPage(Math.max(1, currentPage - 1))}
            onNextPage={() => setPage(Math.min(totalPages, currentPage + 1))}
          />
        </TabsContent>

      </Tabs>

      {isAdminMode ? (
        <ConfirmDialog
          open={clearConfirmOpen}
          onOpenChange={setClearConfirmOpen}
          title={t("清空请求日志")}
          description={t("确定清空全部请求日志吗？该操作不可恢复。")}
          confirmText={t("清空")}
          confirmVariant="destructive"
          onConfirm={() => clearMutation.mutate()}
        />
      ) : null}
    </div>
  );
}

export default function LogsPage() {
  return (
    <Suspense fallback={<LogsPageSkeleton />}>
      <LogsPageContent />
    </Suspense>
  );
}
