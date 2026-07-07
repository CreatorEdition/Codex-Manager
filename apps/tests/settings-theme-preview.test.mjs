import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readSource(...segments) {
  return fs.readFile(path.join(appsRoot, ...segments), "utf8");
}

test("settings appearance tab uses structured theme preview swatches", async () => {
  const source = await readSource(
    "src",
    "app",
    "settings",
    "components",
    "appearance-tab-content.tsx",
  );

  assert.match(source, /ThemePreviewSwatch/);
  assert.match(source, /grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4/);
  assert.match(source, /variant="outline"/);
  assert.match(source, /<ThemePreviewSwatch id=\{item\.id\} color=\{item\.color\} \/>/);
  assert.doesNotMatch(source, /h-10 w-10 rounded-full/);
});

test("theme preview swatch renders dark and light previews from theme metadata", async () => {
  const source = await readSource(
    "src",
    "app",
    "settings",
    "components",
    "theme-preview-swatch.tsx",
  );

  assert.match(source, /export function ThemePreviewSwatch/);
  assert.match(source, /DARK_THEME_IDS/);
  assert.match(source, /DARK_THEME_SURFACES/);
  assert.match(source, /linear-gradient\(135deg/);
  assert.match(source, /aria-hidden="true"/);
  assert.match(source, /backgroundColor: color, opacity: 0\.82/);
});
