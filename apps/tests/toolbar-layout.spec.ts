import { expect, test, type Locator, type Page } from "@playwright/test";

const SETTINGS_SNAPSHOT = {
  updateAutoCheck: true,
  closeToTrayOnClose: false,
  closeToTraySupported: false,
  lowTransparency: false,
  lightweightModeOnCloseToTray: false,
  codexCliGuideDismissed: true,
  webAccessPasswordConfigured: false,
  locale: "zh-CN",
  localeOptions: ["zh-CN", "en"],
  serviceAddr: "localhost:48760",
  serviceListenMode: "loopback",
  serviceListenModeOptions: ["loopback", "all_interfaces"],
  routeStrategy: "ordered",
  routeStrategyOptions: ["ordered", "balanced"],
  freeAccountMaxModel: "auto",
  freeAccountMaxModelOptions: ["auto", "gpt-5"],
  modelForwardRules: "",
  accountMaxInflight: 1,
  gatewayOriginator: "codex-cli",
  gatewayOriginatorDefault: "codex-cli",
  gatewayUserAgentVersion: "1.0.0",
  gatewayUserAgentVersionDefault: "1.0.0",
  gatewayResidencyRequirement: "",
  gatewayResidencyRequirementOptions: ["", "us"],
  pluginMarketMode: "builtin",
  pluginMarketSourceUrl: "",
  upstreamProxyUrl: "",
  upstreamStreamTimeoutMs: 600000,
  sseKeepaliveIntervalMs: 15000,
  backgroundTasks: {
    usagePollingEnabled: true,
    usagePollIntervalSecs: 600,
    gatewayKeepaliveEnabled: true,
    gatewayKeepaliveIntervalSecs: 180,
    tokenRefreshPollingEnabled: true,
    tokenRefreshPollIntervalSecs: 60,
    usageRefreshWorkers: 4,
    httpWorkerFactor: 4,
    httpWorkerMin: 8,
    httpStreamWorkerFactor: 1,
    httpStreamWorkerMin: 2,
  },
  envOverrides: {},
  envOverrideCatalog: [],
  envOverrideReservedKeys: [],
  envOverrideUnsupportedKeys: [],
  theme: "tech",
  appearancePreset: "classic",
};

async function mockRuntimeAndRpc(page: Page) {
  await page.route("**/api/runtime**", async (route) => {
    await route.fulfill({
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        mode: "web-gateway",
        rpcBaseUrl: "/api/rpc",
        canManageService: false,
        canSelfUpdate: false,
        canCloseToTray: false,
        canOpenLocalDir: false,
        canUseBrowserFileImport: true,
        canUseBrowserDownloadExport: true,
      }),
    });
  });

  await page.route("**/api/rpc**", async (route) => {
    const payload = route.request().postDataJSON() as Record<string, unknown>;
    const method = typeof payload?.method === "string" ? payload.method : "";
    const id = payload?.id ?? 1;

    const ok = (result: unknown) =>
      route.fulfill({
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({
          jsonrpc: "2.0",
          id,
          result,
        }),
      });

    if (method === "appSettings/get") {
      await ok(SETTINGS_SNAPSHOT);
      return;
    }
    if (method === "initialize") {
      await ok({
        userAgent: "codex_cli_rs/0.1.19",
        codexHome: "C:/Users/Test/.codex",
        platformFamily: "windows",
        platformOs: "windows",
      });
      return;
    }
    if (method === "gateway/concurrencyRecommendation/get") {
      await ok({
        usageRefreshWorkers: 4,
        httpWorkerFactor: 4,
        httpWorkerMin: 8,
        httpStreamWorkerFactor: 1,
        httpStreamWorkerMin: 2,
        accountMaxInflight: 1,
      });
      return;
    }
    if (method === "account/list") {
      await ok({ items: [], total: 0, page: 1, pageSize: 20 });
      return;
    }
    if (method === "account/usage/list") {
      await ok([]);
      return;
    }
    if (method === "apikey/list") {
      await ok({ items: [], total: 0, page: 1, pageSize: 20 });
      return;
    }
    if (method === "apikey/models") {
      await ok({ models: [] });
      return;
    }
    if (method === "apikey/usageStats") {
      await ok([]);
      return;
    }
    if (method === "apikey/modelCatalogList") {
      await ok({
        items: [
          {
            slug: "gpt-5.4",
            display_name: "GPT-5.4",
            description: "Layout smoke test model",
            supported_in_api: true,
            sourceKind: "remote",
            userEdited: false,
            sortIndex: 0,
            updatedAt: 1_770_000_000,
            input_modalities: ["text"],
          },
        ],
      });
      return;
    }
    if (method === "apikey/modelRouting") {
      await ok({ platformModels: [], sourceModels: [] });
      return;
    }
    if (method === "aggregateApi/list") {
      await ok({ items: [], total: 0, page: 1, pageSize: 20 });
      return;
    }
    if (method === "quota/modelPoolSources") {
      await ok({ items: [], total: 0, page: 1, pageSize: 20 });
      return;
    }
    if (method === "requestlog/list") {
      await ok({ items: [], total: 0, page: 1, pageSize: 10 });
      return;
    }
    if (method === "requestlog/summary") {
      await ok({
        totalCount: 0,
        filteredCount: 0,
        successCount: 0,
        errorCount: 0,
        totalTokens: 0,
        totalCostUsd: 0,
      });
      return;
    }

    await route.fulfill({
      status: 500,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify({
        jsonrpc: "2.0",
        id,
        error: {
          code: -32000,
          message: `Unhandled RPC method in toolbar layout test: ${method}`,
        },
      }),
    });
  });
}

async function expectInsideViewport(locator: Locator, page: Page, minWidth = 0) {
  await expect(locator).toBeVisible();
  const box = await locator.boundingBox();
  expect(box).not.toBeNull();
  const viewport = page.viewportSize();
  expect(viewport).not.toBeNull();
  expect(box!.x).toBeGreaterThanOrEqual(0);
  expect(box!.x + box!.width).toBeLessThanOrEqual(viewport!.width + 1);
  if (minWidth > 0) {
    expect(box!.width).toBeGreaterThan(minWidth);
  }
}

test("core management toolbars keep search and actions inside viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1100, height: 800 });
  await mockRuntimeAndRpc(page);

  await page.goto("/apikeys/");
  await expectInsideViewport(
    page.getByPlaceholder("搜索名称、ID、模型或上游地址"),
    page,
    220,
  );
  await expectInsideViewport(
    page.getByRole("button", { name: "创建密钥" }),
    page,
  );

  await page.goto("/aggregate-api/");
  await expectInsideViewport(
    page.getByRole("button", { name: "测试当前页" }),
    page,
  );
  await expectInsideViewport(
    page.getByRole("button", { name: "刷新当前页余额" }),
    page,
  );
  await expectInsideViewport(
    page.getByRole("button", { name: "新建聚合 API" }),
    page,
  );

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/logs/");
  await expectInsideViewport(
    page.getByPlaceholder("搜索路径、账号或密钥 ID..."),
    page,
    220,
  );
  await expectInsideViewport(page.getByRole("button", { name: "2XX" }), page);
  await expectInsideViewport(page.getByRole("button", { name: "刷新" }), page);

  await page.setViewportSize({ width: 1100, height: 800 });
  await page.goto("/models/");
  await expectInsideViewport(
    page.getByPlaceholder("搜索 slug、显示名称或描述"),
    page,
    220,
  );
  await expectInsideViewport(
    page.getByRole("button", { name: "新增自定义模型" }),
    page,
  );
});
