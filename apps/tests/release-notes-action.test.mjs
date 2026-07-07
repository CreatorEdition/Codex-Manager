import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");

async function readRepoFile(...segments) {
  return fs.readFile(path.join(repoRoot, ...segments), "utf8");
}

function extractChangelogSection(source, version) {
  const lines = source.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith(`## [${version}]`));
  assert.notEqual(start, -1, `missing changelog section for ${version}`);
  const end = lines.findIndex(
    (line, index) => index > start && line.startsWith("## ["),
  );
  return lines.slice(start, end === -1 ? undefined : end).join("\n");
}

test("publish release action syncs GitHub Release body from changelog", async () => {
  const source = await readRepoFile(
    ".github",
    "actions",
    "publish-github-release",
    "action.yml",
  );

  assert.match(source, /CHANGELOG_PATH="docs\/zh-CN\/CHANGELOG\.md"/);
  assert.match(source, /RELEASE_NOTES_FILE="\$\(mktemp\)"/);
  assert.match(source, /write_release_notes/);
  assert.match(source, /--notes-file "\$RELEASE_NOTES_FILE"/);
  assert.match(source, /body: \$body/);
  assert.doesNotMatch(source, /--generate-notes/);
});

test("0.3.11 release notes include CE upstream divergence and update summary", async () => {
  const changelog = await readRepoFile("docs", "zh-CN", "CHANGELOG.md");
  const section = extractChangelogSection(changelog, "0.3.11");

  assert.match(section, /### Fork \/ Upstream/);
  assert.match(section, /CE 不再直接 merge upstream/);
  assert.match(section, /upstream\/main = 6ac01a2a/);
  assert.match(section, /已语义移植网关模型转发规则/);
  assert.match(section, /明确不移植作者页、赞助导流、远程 author content、AtomGit 推广/);
  assert.doesNotMatch(section, /## \[Unreleased\]/);
});
