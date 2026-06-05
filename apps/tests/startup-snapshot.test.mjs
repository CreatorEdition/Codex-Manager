import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");
const serviceClientPath = path.join(
  appsRoot,
  "src",
  "lib",
  "api",
  "service-client.ts"
);
const dashboardStatsPath = path.join(
  appsRoot,
  "src",
  "hooks",
  "useDashboardStats.ts"
);

function getStartupSnapshotCallBody(source) {
  const marker = "async getStartupSnapshot(";
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, "serviceClient.getStartupSnapshot should exist");
  const end = source.indexOf("return normalizeStartupSnapshot", start);
  assert.notEqual(end, -1, "startup snapshot call should normalize response");
  return source.slice(start, end);
}

test("serviceClient.getStartupSnapshot 默认保持轻量快照", async () => {
  const source = await fs.readFile(serviceClientPath, "utf8");
  const callBody = getStartupSnapshotCallBody(source);

  for (const includeName of [
    "includeUsageAggregate",
    "includeTodaySummary",
    "includeRecentLogs",
    "includeApiModels",
  ]) {
    assert.doesNotMatch(
      callBody,
      new RegExp(`${includeName}:\\s*true`),
      `${includeName} should not default to true in serviceClient`
    );
  }
});

test("首页仪表盘显式声明需要完整启动快照", async () => {
  const source = await fs.readFile(dashboardStatsPath, "utf8");

  for (const includeName of [
    "includeUsageAggregate",
    "includeTodaySummary",
    "includeRecentLogs",
    "includeApiModels",
  ]) {
    assert.match(
      source,
      new RegExp(`${includeName}:\\s*true`),
      `${includeName} should be opt-in on dashboard`
    );
  }
});
