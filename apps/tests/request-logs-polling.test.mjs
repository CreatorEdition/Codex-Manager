import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");
const logsPagePath = path.join(appsRoot, "src", "app", "logs", "page.tsx");

test("日志页降低列表轮询频率并停止摘要轮询", async () => {
  const source = await fs.readFile(logsPagePath, "utf8");
  assert.match(
    source,
    /const REQUEST_LOG_LIST_REFETCH_INTERVAL_MS = 30_000;/,
  );
  assert.match(
    source,
    /refetchInterval: REQUEST_LOG_LIST_REFETCH_INTERVAL_MS,\s+refetchIntervalInBackground: false,/,
  );

  const summaryQueryMatch = source.match(
    /const \{ data: summaryResult[\s\S]*?\n  \}\);/,
  );
  assert.ok(summaryQueryMatch, "summary query block should exist");
  assert.doesNotMatch(summaryQueryMatch[0], /refetchInterval:/);
  assert.match(summaryQueryMatch[0], /staleTime: 30_000,/);
});
