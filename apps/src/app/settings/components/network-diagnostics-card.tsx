"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Globe2, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  networkDiagnosticsClient,
  type NetworkDiagnosticsSnapshot,
} from "@/lib/api/network-diagnostics";
import { getAppErrorMessage } from "@/lib/api/transport";

const QUERY_KEY = ["network-diagnostics"] as const;

function formatCheckedAt(timestamp: number | null): string {
  if (!timestamp) return "--";
  return new Date(timestamp * 1000).toLocaleString();
}

export function NetworkDiagnosticsCard({
  t,
  enabled,
}: {
  t: (value: string) => string;
  enabled: boolean;
}) {
  const queryClient = useQueryClient();
  const diagnostics = useQuery({
    queryKey: QUERY_KEY,
    queryFn: () => networkDiagnosticsClient.get(),
    enabled,
    staleTime: 30_000,
    refetchInterval: (query) =>
      query.state.data?.refreshing ? 1_000 : false,
  });
  const refresh = useMutation({
    mutationFn: () => networkDiagnosticsClient.refresh(),
    onSuccess: (snapshot) => {
      queryClient.setQueryData<NetworkDiagnosticsSnapshot>(QUERY_KEY, snapshot);
      toast.success(
        snapshot.refreshScheduled
          ? t("已开始刷新出口诊断")
          : t("已使用最近的出口诊断结果"),
      );
    },
    onError: (error: unknown) => {
      toast.error(`${t("出口诊断刷新失败")}: ${getAppErrorMessage(error)}`);
    },
  });
  const snapshot = diagnostics.data;
  const isRefreshing = refresh.isPending || snapshot?.refreshing;
  const regionLabel = [snapshot?.country, snapshot?.countryCode]
    .filter(Boolean)
    .join(" / ") || "--";

  return (
    <Card className="glass-card shadow-sm">
      <CardHeader>
        <div className="flex items-center gap-2">
          <Globe2 className="h-4 w-4 text-primary" />
          <CardTitle className="text-base">{t("出口网络诊断")}</CardTitle>
          {snapshot?.enabled === false ? (
            <Badge variant="secondary">{t("已关闭")}</Badge>
          ) : null}
        </div>
        <CardDescription>
          {t("用于核对当前服务出口，不会根据 IP 结果自动改变账号状态。")}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 text-sm sm:grid-cols-2 lg:grid-cols-3">
          <div>
            <div className="text-xs text-muted-foreground">{t("出口 IP")}</div>
            <code className="break-all text-xs">{snapshot?.ip || "--"}</code>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">{t("国家或地区")}</div>
            <div>{regionLabel}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">ASN</div>
            <div>{snapshot?.asn ? `AS${snapshot.asn}` : "--"}</div>
          </div>
          <div className="sm:col-span-2">
            <div className="text-xs text-muted-foreground">{t("网络组织")}</div>
            <div className="break-words">{snapshot?.organization || "--"}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">{t("检测时间")}</div>
            <div>{formatCheckedAt(snapshot?.checkedAt || null)}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">{t("检测来源")}</div>
            <div>{snapshot?.source || "--"}</div>
          </div>
        </div>
        {snapshot?.error ? (
          <p className="text-xs text-destructive">{snapshot.error}</p>
        ) : null}
        <div className="flex items-center justify-between gap-3">
          <p className="text-[10px] text-muted-foreground">
            {t("查询沿用当前 OpenAI 上游代理；外部诊断服务失败不会影响账号刷新或网关请求。")}
          </p>
          <Button
            variant="outline"
            size="sm"
            className="shrink-0 gap-2"
            disabled={!enabled || snapshot?.enabled === false || Boolean(isRefreshing)}
            onClick={() => refresh.mutate()}
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
            {isRefreshing ? t("检测中...") : t("手动刷新")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
