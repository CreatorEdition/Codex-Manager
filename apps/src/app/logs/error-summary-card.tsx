"use client";

import { useState } from "react";
import { AlertCircle, ChevronDown, ChevronRight } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { formatTsFromSeconds } from "@/lib/utils/usage";
import { cn } from "@/lib/utils";
import type { RequestLogErrorCodeSummaryItem } from "@/types";
import type { TranslateFn } from "./page-helpers";

interface ErrorSummaryCardProps {
  t: TranslateFn;
  items: RequestLogErrorCodeSummaryItem[];
  isLoading: boolean;
}

export function ErrorSummaryCard({ t, items, isLoading }: ErrorSummaryCardProps) {
  const [expandedCodes, setExpandedCodes] = useState<Set<string>>(new Set());

  if (isLoading) {
    return (
      <Card className="glass-card shadow-sm">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertCircle className="h-5 w-5 text-amber-500" />
            {t("错误摘要")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </CardContent>
      </Card>
    );
  }

  if (items.length === 0) {
    return null;
  }

  const totalErrors = items.reduce((sum, item) => sum + item.count, 0);
  const errorTypeCount = items.length;

  const toggleExpand = (errorCode: string) => {
    setExpandedCodes((prev) => {
      const next = new Set(prev);
      if (next.has(errorCode)) {
        next.delete(errorCode);
      } else {
        next.add(errorCode);
      }
      return next;
    });
  };

  return (
    <Card className="glass-card shadow-sm">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertCircle className="h-5 w-5 text-amber-500" />
            {t("错误摘要")}
          </CardTitle>
          <div className="text-sm text-muted-foreground">
            {errorTypeCount} {t("类错误")}，{t("共")} {totalErrors} {t("次")}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {items.map((item) => {
          const isExpanded = expandedCodes.has(item.errorCode);
          const percentage = ((item.count / totalErrors) * 100).toFixed(1);

          return (
            <div
              key={item.errorCode}
              className="rounded-lg border border-border/40 bg-muted/30 transition-colors hover:bg-muted/50"
            >
              <button
                type="button"
                onClick={() => toggleExpand(item.errorCode)}
                className="flex w-full items-center gap-3 p-3 text-left"
              >
                <div className="shrink-0">
                  {isExpanded ? (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  )}
                </div>
                <div className="flex min-w-0 flex-1 items-center justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-sm font-semibold text-foreground">
                        {item.errorCode}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {percentage}%
                      </span>
                    </div>
                    <div className="mt-0.5 text-xs text-muted-foreground">
                      {t("最后发生")}: {formatTsFromSeconds(item.lastSeen)}
                    </div>
                  </div>
                  <div className="shrink-0 text-right">
                    <div className="text-sm font-semibold text-foreground">
                      {item.count} {t("次")}
                    </div>
                  </div>
                </div>
              </button>
              {isExpanded && item.sampleMessage ? (
                <div className="border-t border-border/40 bg-background/50 p-3">
                  <div className="text-xs font-medium text-muted-foreground mb-1">
                    {t("代表样例")}:
                  </div>
                  <div className="rounded bg-muted/50 p-2 text-xs font-mono text-foreground break-all">
                    {item.sampleMessage}
                  </div>
                </div>
              ) : null}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}

export function ErrorSummaryCardSkeleton({ t }: { t: TranslateFn }) {
  return (
    <Card className="glass-card shadow-sm">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          <AlertCircle className="h-5 w-5 text-amber-500" />
          {t("错误摘要")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-16 w-full" />
        <Skeleton className="h-16 w-full" />
      </CardContent>
    </Card>
  );
}
