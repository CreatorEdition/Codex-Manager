import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readNextConfig() {
  return fs.readFile(path.join(appsRoot, "next.config.ts"), "utf8");
}

test("开发态 Next 配置代理 Web runtime 必需路由", async () => {
  const source = await readNextConfig();

  assert.match(source, /CODEXMANAGER_DEV_WEB_ORIGIN/);
  assert.match(source, /normalizeDevWebOrigin/);
  assert.match(source, /http:\/\/localhost:48761/);
  assert.match(source, /configureDevWebRuntimeRewrites/);
  assert.match(source, /output: undefined/);
  assert.match(source, /source: "\/api\/events\/:path\*"/);
  assert.match(source, /"\/api\/runtime"/);
  assert.match(source, /"\/api\/rpc"/);
  assert.match(source, /"\/__auth_status"/);
  assert.match(source, /"\/__login"/);
  assert.match(source, /"\/__logout"/);
});

test("开发态 Next rewrites 不恢复作者内容代理", async () => {
  const source = await readNextConfig();

  assert.doesNotMatch(source, /\/api\/author-content/);
  assert.doesNotMatch(source, /author\.qxnm\.top/);
});
