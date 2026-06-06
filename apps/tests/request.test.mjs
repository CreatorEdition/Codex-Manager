import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import ts from "../node_modules/typescript/lib/typescript.js";

const appsRoot = path.resolve(import.meta.dirname, "..");
const requestSourcePath = path.join(appsRoot, "src", "lib", "utils", "request.ts");
const timeoutSourcePath = path.join(appsRoot, "src", "lib", "utils", "timeout.ts");

async function loadRequestModule() {
  const [requestSource, timeoutSource] = await Promise.all([
    fs.readFile(requestSourcePath, "utf8"),
    fs.readFile(timeoutSourcePath, "utf8"),
  ]);
  const requestCompiled = ts.transpileModule(requestSource, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
    },
    fileName: requestSourcePath,
  });
  const timeoutCompiled = ts.transpileModule(timeoutSource, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
    },
    fileName: timeoutSourcePath,
  });

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "codexmanager-request-"));
  const requestFile = path.join(tempDir, "request.mjs");
  const timeoutFile = path.join(tempDir, "timeout.mjs");
  await fs.writeFile(
    requestFile,
    requestCompiled.outputText.replace(/from "\.\/timeout"/g, 'from "./timeout.mjs"'),
    "utf8"
  );
  await fs.writeFile(timeoutFile, timeoutCompiled.outputText, "utf8");
  return import(pathToFileURL(requestFile).href);
}

const request = await loadRequestModule();

test("fetchWithRetry 将自身超时转换为 TimeoutError 并按配置重试", async () => {
  const originalFetch = globalThis.fetch;
  let calls = 0;
  globalThis.fetch = async (_url, init) => {
    calls += 1;
    return await new Promise((_resolve, reject) => {
      init.signal.addEventListener("abort", () => {
        reject(new DOMException("Aborted", "AbortError"));
      });
    });
  };

  try {
    await assert.rejects(
      request.fetchWithRetry(
        "/api/rpc",
        { method: "POST" },
        {
          timeoutMs: 5,
          retries: 1,
          retryDelayMs: 1,
          maxRetryDelayMs: 1,
          timeoutMessage: "RPC startup/snapshot 超时",
        }
      ),
      {
        name: "TimeoutError",
        message: "RPC startup/snapshot 超时",
      }
    );
    assert.equal(calls, 2);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("fetchWithRetry 对调用方取消不伪装成超时", async () => {
  const originalFetch = globalThis.fetch;
  const callerController = new AbortController();
  globalThis.fetch = async (_url, init) => {
    return await new Promise((_resolve, reject) => {
      if (init.signal.aborted) {
        reject(new DOMException("Aborted", "AbortError"));
        return;
      }
      init.signal.addEventListener("abort", () => {
        reject(new DOMException("Aborted", "AbortError"));
      });
      queueMicrotask(() => callerController.abort());
    });
  };

  try {
    await assert.rejects(
      request.fetchWithRetry(
        "/api/rpc",
        { method: "POST" },
        {
          signal: callerController.signal,
          timeoutMs: 1000,
          retries: 1,
        }
      ),
      {
        name: "AbortError",
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});
