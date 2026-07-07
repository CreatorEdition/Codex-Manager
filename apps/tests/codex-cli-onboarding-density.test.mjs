import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readSource(...segments) {
  return fs.readFile(path.join(appsRoot, ...segments), "utf8");
}

test("codex cli onboarding keeps full guide content while tightening dialog density", async () => {
  const source = await readSource(
    "src",
    "components",
    "layout",
    "codex-cli-onboarding-dialog.tsx",
  );

  assert.match(source, /Codex CLI 首次接入引导/);
  assert.match(source, /GUIDE_REMINDERS/);
  assert.match(source, /Rocket/);
  assert.match(source, /max-h-\[86dvh\]/);
  assert.match(source, /xl:!max-w-\[78rem\]/);
  assert.match(source, /max-h-\[14dvh\]/);
  assert.match(source, /max-h-\[24dvh\]/);
  assert.doesNotMatch(source, /mission-panel/);
  assert.doesNotMatch(source, /max-h-\[92vh\]/);
  assert.doesNotMatch(source, /2xl:!max-w-\[92rem\]/);
});
