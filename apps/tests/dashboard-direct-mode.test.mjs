import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readDashboardSource() {
  return fs.readFile(path.join(appsRoot, "src/app/page.tsx"), "utf8");
}

async function readSource(relativePath) {
  return fs.readFile(path.join(appsRoot, relativePath), "utf8");
}

test("账号直连与混合模式均展示已经写入 CodexManager 的统计数据", async () => {
  const source = await readDashboardSource();
  assert.doesNotMatch(source, /useCodexProfileModeStatus/);
  assert.doesNotMatch(source, /DirectModeUnavailable/);
  assert.doesNotMatch(source, /未经过本地网关的请求不可统计/);
  assert.match(source, /<AdminUsageAnalyticsCard\s+summary=\{adminUsageSummary\}/s);
  assert.match(source, /title: t\("今日Token"\)/);
  assert.match(source, /<CardTitle className="text-base font-semibold">\{t\("当前活跃账号"\)\}<\/CardTitle>/);
});

test("日志页不按当前路由模式隐藏已记录的历史日志", async () => {
  const source = await readSource("src/app/logs/page.tsx");
  const sectionsSource = await readSource("src/app/logs/page-sections.tsx");
  assert.doesNotMatch(source, /useCodexProfileModeStatus/);
  assert.match(source, /<RequestLogsTabContent/);
  assert.doesNotMatch(sectionsSource, /未经过本地网关的请求不会产生新的 CodexManager 请求日志/);
  assert.doesNotMatch(sectionsSource, /本地网关或包含本地网关的混合路由才会记录/);
  assert.doesNotMatch(sectionsSource, /DirectModeUnavailable/);
});

test("开发态 Next 配置忽略重型目录监听", async () => {
  const source = await readSource("next.config.ts");
  const gitignore = await fs.readFile(path.join(appsRoot, "..", ".gitignore"), "utf8");

  assert.match(source, /PHASE_DEVELOPMENT_SERVER/);
  assert.match(source, /configureDevWebpack/);
  assert.match(source, /\*\*\/src-tauri\/target\/\*\*/);
  assert.match(source, /\*\*\/\.pnpm-store\/\*\*/);
  assert.match(source, /poll: 1000/);
  assert.match(gitignore, /\/\.pnpm-store\//);
});

test("应用 store 会跳过无变化更新", async () => {
  const source = await readSource("src/lib/store/useAppStore.ts");

  assert.match(source, /function hasPartialStateChanges/);
  assert.match(source, /function areTabsEqual/);
  assert.match(source, /hasPartialStateChanges\(state\.serviceStatus, status\)/);
  assert.match(source, /hasPartialStateChanges\(state\.appSettings, settings\)/);
  assert.match(source, /Object\.is\(state\.runtimeCapabilities, runtimeCapabilities\)/);
  assert.match(source, /state\.isSidebarOpen === open \? state/);
  assert.match(source, /state\.isCodexCliGuideOpen \? state/);
  assert.match(source, /areTabsEqual\(state\.openShellTabs, normalizedTabs\)/);
});

test("AppBootstrap 使用 store selector 降低重渲染范围", async () => {
  const source = await readSource("src/components/layout/app-bootstrap.tsx");

  assert.match(source, /useAppStore\(\(state\) => state\.serviceStatus\)/);
  assert.match(source, /useAppStore\(\(state\) => state\.appSettings\)/);
  assert.match(source, /useAppStore\(\(state\) => state\.runtimeCapabilities\)/);
  assert.doesNotMatch(source, /const \{\s*setServiceStatus,[\s\S]*runtimeCapabilities,\s*\} = useAppStore\(\)/);
});

test("常驻布局组件使用 store selector 降低重渲染范围", async () => {
  const headerSource = await readSource("src/components/layout/header.tsx");
  const sidebarSource = await readSource("src/components/layout/sidebar.tsx");
  const logsSource = await readSource("src/app/logs/page.tsx");

  assert.match(headerSource, /useAppStore\(\(state\) => state\.appSettings\)/);
  assert.match(headerSource, /useAppStore\(\(state\) => state\.serviceStatus\)/);
  assert.match(headerSource, /useAppStore\(\(state\) => state\.currentShellPath\)/);
  assert.match(sidebarSource, /useAppStore\(\(state\) => state\.isSidebarOpen\)/);
  assert.match(sidebarSource, /useAppStore\(\(state\) => state\.currentShellPath\)/);
  assert.match(sidebarSource, /useAppStore\(\(state\) => state\.navigateShellPath\)/);
  assert.match(logsSource, /useAppStore\(\(state\) => state\.serviceStatus\)/);
  assert.doesNotMatch(headerSource, /} = useAppStore\(\)/);
  assert.doesNotMatch(sidebarSource, /} = useAppStore\(\)/);
});

test("托盘预览使用较轻的启动快照", async () => {
  const source = await readSource("src/app/tray-preview/page.tsx");
  const hookSource = await readSource("src/hooks/useDashboardStats.ts");

  assert.match(source, /requestLogLimit: TRAY_PREVIEW_REQUEST_LOG_LIMIT/);
  assert.match(source, /includeApiModels: false/);
  assert.match(hookSource, /includeApiModels\?: boolean/);
  assert.match(hookSource, /const includeApiModels = options\.includeApiModels \?\? true/);
  assert.match(hookSource, /includeApiModels,/);
});
