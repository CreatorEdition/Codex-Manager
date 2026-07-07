import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readSwitchSource() {
  return fs.readFile(
    path.join(appsRoot, "src", "components", "ui", "switch.tsx"),
    "utf8",
  );
}

test("Switch 未选中态保留可见边框和更高对比度", async () => {
  const source = await readSwitchSource();

  assert.match(source, /data-unchecked:border-foreground\/20/);
  assert.match(source, /data-unchecked:bg-foreground\/18/);
  assert.match(source, /dark:data-unchecked:border-foreground\/30/);
  assert.match(source, /dark:data-unchecked:bg-foreground\/20/);
  assert.match(source, /data-unchecked:bg-white/);
  assert.match(source, /data-unchecked:ring-foreground\/20/);
  assert.doesNotMatch(source, /border border-transparent/);
  assert.doesNotMatch(source, /data-unchecked:bg-input/);
});
