import type { NextConfig } from "next";
import { PHASE_DEVELOPMENT_SERVER } from "next/constants";

const baseNextConfig: NextConfig = {
  // 暂时禁用 Beta 版编译器以确保稳定性
  reactCompiler: false,
  experimental: {
    staleTimes: {
      dynamic: 30,
      static: 300,
    },
  },
  // 桌面开发态不展示右下角 Next 渲染指示器，避免用户误判为页面卡顿。
  devIndicators: false,
  // Tauri 开发态通过 127.0.0.1 加载 Next 资源，显式放行避免 dev 跨源告警。
  allowedDevOrigins: ["127.0.0.1", "[::1]"],
  output: 'export',
  // 中文注释：导出静态站点时强制 trailing slash，生成 /xxx/index.html，避免 Tauri 打包后导航丢失。
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

const configureDevWebpack: NonNullable<NextConfig["webpack"]> = (config) => {
  const ignored = config.watchOptions?.ignored;
  const ignoredPatterns = Array.isArray(ignored)
    ? ignored.filter(
        (pattern): pattern is string =>
          typeof pattern === "string" && pattern.length > 0,
      )
    : typeof ignored === "string" && ignored.length > 0
      ? [ignored]
      : [];

  config.watchOptions = {
    ...config.watchOptions,
    ignored: [
      ...ignoredPatterns,
      "**/node_modules/**",
      "**/src-tauri/target/**",
      "**/.pnpm-store/**",
    ],
    poll: 1000,
  };

  return config;
};

function normalizeDevWebOrigin(rawOrigin: string | undefined): string {
  const normalized = (rawOrigin ?? "").trim().replace(/\/+$/, "");
  return normalized || "http://localhost:48761";
}

const devWebRuntimeOrigin = normalizeDevWebOrigin(
  process.env.CODEXMANAGER_DEV_WEB_ORIGIN,
);

const configureDevWebRuntimeRewrites: NonNullable<NextConfig["rewrites"]> =
  async () => {
    const runtimePaths = [
      "/api/runtime",
      "/api/rpc",
      "/__auth_status",
      "/__login",
      "/__logout",
    ];

    return [
      ...runtimePaths.flatMap((source) => [
        { source, destination: `${devWebRuntimeOrigin}${source}` },
        { source: `${source}/`, destination: `${devWebRuntimeOrigin}${source}` },
      ]),
      {
        source: "/api/events/:path*",
        destination: `${devWebRuntimeOrigin}/api/events/:path*`,
      },
    ];
  };

const nextConfig = (phase: string): NextConfig =>
  phase === PHASE_DEVELOPMENT_SERVER
    ? {
        ...baseNextConfig,
        output: undefined,
        webpack: configureDevWebpack,
        rewrites: configureDevWebRuntimeRewrites,
      }
    : baseNextConfig;

export default nextConfig;
