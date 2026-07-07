import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readSource(...segments) {
  return fs.readFile(path.join(appsRoot, ...segments), "utf8");
}

test("models page search field exposes focus border feedback", async () => {
  const source = await readSource("src", "app", "models", "page.tsx");

  assert.match(source, /focus-within:border-ring/);
  assert.match(source, /focus-within:ring-3/);
  assert.match(source, /border border-input/);
  assert.match(source, /focus-visible:border-transparent focus-visible:ring-0/);
  assert.doesNotMatch(
    source,
    /flex h-10 items-center gap-2 rounded-xl border border-border\/60 bg-background\/35 px-3/,
  );
});
