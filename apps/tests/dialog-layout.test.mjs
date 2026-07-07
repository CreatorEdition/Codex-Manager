import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

async function readSource(...segments) {
  return fs.readFile(path.join(appsRoot, ...segments), "utf8");
}

test("Dialog content is positioned by a scrollable viewport", async () => {
  const source = await readSource("src", "components", "ui", "dialog.tsx");

  assert.match(source, /function DialogViewport\(/);
  assert.match(
    source,
    /DialogPrimitive\.Viewport[\s\S]*fixed inset-0 isolate z-50 flex min-h-dvh items-center justify-center overflow-y-auto p-4/,
  );
  assert.match(
    source,
    /<DialogViewport>[\s\S]*<DialogPrimitive\.Popup[\s\S]*relative z-50 grid[\s\S]*max-h-\[calc\(100dvh-2rem\)\][\s\S]*<\/DialogPrimitive\.Popup>[\s\S]*<\/DialogViewport>/,
  );
  assert.doesNotMatch(source, /-translate-x-1\/2 -translate-y-1\/2/);
});

test("Button appends caller classes after variant classes", async () => {
  const source = await readSource("src", "components", "ui", "button.tsx");

  assert.match(source, /className=\{cn\(buttonVariants\(\{ variant, size \}\), className\)\}/);
  assert.doesNotMatch(source, /buttonVariants\(\{ variant, size, className \}\)/);
});

test("ApiKeyModal keeps header and footer fixed while body scrolls", async () => {
  const source = await readSource(
    "src",
    "components",
    "modals",
    "api-key-modal.tsx",
  );

  assert.match(source, /DialogContent className="glass-card flex max-h-\[90dvh\]/);
  assert.match(source, /DialogHeader className="shrink-0 px-4 pb-4 pt-4/);
  assert.match(
    source,
    /className="grid min-h-0 flex-1 gap-5 overflow-y-auto px-4 py-4 sm:px-6"/,
  );
  assert.match(source, /DialogFooter className="mx-0 mb-0 shrink-0/);
});
