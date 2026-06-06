import { TimeoutError } from "./timeout";

export interface RequestOptions {
  signal?: AbortSignal;
  timeoutMs?: number;
  timeoutMessage?: string;
  retries?: number;
  retryDelayMs?: number;
  maxRetryDelayMs?: number;
  shouldRetry?: (error: unknown) => boolean;
  shouldRetryStatus?: (status: number) => boolean;
}

/**
 * 函数 `fetchWithRetry`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - url: 参数 url
 * - init?: 参数 init?
 * - options: 参数 options
 *
 * # 返回
 * 返回函数执行结果
 */
export async function fetchWithRetry(
  url: string,
  init?: RequestInit,
  options: RequestOptions = {}
): Promise<Response> {
  const {
    timeoutMs = 10000,
    timeoutMessage = `Request timed out after ${timeoutMs}ms`,
    retries = 3,
    retryDelayMs = 200,
    maxRetryDelayMs = 3000,
    shouldRetryStatus = (status) => status >= 500 || status === 429,
  } = options;

  let lastError: unknown;
  for (let i = 0; i <= retries; i++) {
    const controller = new AbortController();
    let timedOut = false;
    const id =
      Number.isFinite(timeoutMs) && timeoutMs > 0
        ? setTimeout(() => {
            timedOut = true;
            controller.abort();
          }, timeoutMs)
        : null;
    const abortFromCaller = () => controller.abort();
    if (options.signal) {
      if (options.signal.aborted) {
        abortFromCaller();
      } else {
        options.signal.addEventListener("abort", abortFromCaller, { once: true });
      }
    }

    try {
      const response = await fetch(url, {
        ...init,
        signal: controller.signal,
      });

      if (response.ok || !shouldRetryStatus(response.status) || i === retries) {
        return response;
      }
    } catch (err: unknown) {
      if (id !== null) {
        clearTimeout(id);
      }
      if (options.signal) {
        options.signal.removeEventListener("abort", abortFromCaller);
      }
      if (err instanceof Error && err.name === "AbortError" && timedOut) {
        lastError = new TimeoutError(timeoutMessage);
      } else {
        lastError = err;
      }
      if (
        !(err instanceof Error && err.name === "AbortError" && timedOut) ||
        i === retries
      ) {
        if (lastError instanceof Error) {
          throw lastError;
        }
        throw err;
      }
    } finally {
      if (id !== null) {
        clearTimeout(id);
      }
      if (options.signal) {
        options.signal.removeEventListener("abort", abortFromCaller);
      }
    }

    if (i === retries) {
      break;
    }
    const delay = Math.min(retryDelayMs * Math.pow(2, i), maxRetryDelayMs);
    await new Promise((resolve) => setTimeout(resolve, delay));
  }
  throw lastError || new Error("Fetch failed after retries");
}

/**
 * 函数 `runWithControl`
 *
 * 作者: gaohongshun
 *
 * 时间: 2026-04-02
 *
 * # 参数
 * - fn: 参数 fn
 * - options: 参数 options
 *
 * # 返回
 * 返回函数执行结果
 */
export async function runWithControl<T>(
  fn: () => Promise<T>,
  options: RequestOptions = {}
): Promise<T> {
  const {
    retries = 0,
    retryDelayMs = 200,
    maxRetryDelayMs = 3000,
    shouldRetry = () => true,
  } = options;

  let lastError: unknown;
  for (let i = 0; i <= retries; i++) {
    try {
      return await fn();
    } catch (err: unknown) {
      lastError = err;
      if (i === retries || !shouldRetry(err)) {
        throw err;
      }
    }
    const delay = Math.min(retryDelayMs * Math.pow(2, i), maxRetryDelayMs);
    await new Promise((resolve) => setTimeout(resolve, delay));
  }
  throw lastError;
}
