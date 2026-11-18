import { readFile, readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve, relative } from "node:path";
import assert from "node:assert/strict";
import { test } from "node:test";

const root = fileURLToPath(new URL("../", import.meta.url));
const content = resolve(root, "content");
async function pages(directory = content) {
  const entries = await readdir(directory, { withFileTypes: true });
  const children = await Promise.all(
    entries.map(async (entry) => {
      const path = resolve(directory, entry.name);
      return entry.isDirectory()
        ? pages(path)
        : entry.name.endsWith(".md")
          ? [{ path, source: await readFile(path, "utf8") }]
          : [];
    }),
  );
  return children.flat();
}
const documents = await pages();
const routes = new Map(
  documents.map((page) => [
    "/" +
      relative(content, page.path)
        .replaceAll("\\", "/")
        .replace(/\.md$/, "")
        .split("/")
        .map((segment) => segment.replace(/^\d+\./, ""))
        .join("/"),
    page,
  ]),
);
const prose = (source) => source.replace(/```[^]*?```/g, "");

test("content has stable unique routes and explicit page metadata", () => {
  assert.equal(routes.size, documents.length);
  assert(
    documents.length >= 50,
    "retain detailed reference and beginner guides",
  );
  for (const { path, source } of documents) {
    assert.match(
      source,
      /^---\ntitle: .+\ndescription: .+\nnavigation:\n  icon: "tabler:[a-z0-9-]+"\n---\n/,
      path,
    );
    assert(
      !/^# /m.test(prose(source)),
      `${path}: the page header supplies the only h1`,
    );
    assert(!/\/Users\/|~\/Projects\/|(?:\.\.\/)+plans\//.test(source), path);
  }
});

test("component usage examples are complete App.iru files", () => {
  const components = documents.filter(
    ({ path }) =>
      /[/\\](?:\d+\.)?components[/\\]/.test(path) &&
      !path.endsWith("/overview.md"),
  );
  assert(
    components.length >= 30,
    "retain complete component and reference examples",
  );
  for (const { path, source } of components) {
    const example =
      /## Usage\n[\s\S]*?```xml \[src\/App\.iru\]\n([\s\S]*?)\n```/.exec(
        source,
      )?.[1];
    assert(example, `${path}: missing copyable App.iru`);
    for (const [open, close] of [
      ["<view>", "</view>"],
      ["<rust>", "</rust>"],
      ["<style scoped>", "</style>"],
    ]) {
      assert(
        example.includes(open) && example.includes(close),
        `${path}: missing ${open}`,
      );
    }
    assert(
      /\blet \w+: [^;]+ =/.test(example) && !/\bfn setup\s*\(/.test(example),
      `${path}: use typed declarations with implicit initialization`,
    );
    const view = example.split("</view>")[0];
    assert(
      !/\b(?:if|match)\s/.test(view),
      `${path}: put derived branches in the Rust block`,
    );
    assert(
      !example.includes("Size::Auto"),
      `${path}: use the exported Size::Content variant`,
    );
  }
});

test("internal Markdown links resolve to pages or public assets", async () => {
  for (const { path, source } of documents) {
    for (const match of prose(source).matchAll(/\[[^\]]+\]\((\/[^)]+)\)/g)) {
      const [route, fragment] = match[1].split("#");
      const target = routes.get(route);
      if (!target) {
        const asset = resolve(root, "public", `.${route}`);
        assert(
          !relative(resolve(root, "public"), asset).startsWith(".."),
          route,
        );
        await readFile(asset);
        continue;
      }
      if (fragment) {
        const slugs = [...target.source.matchAll(/^#{2,6}\s+(.+)$/gm)].map(
          (m) =>
            m[1]
              .toLowerCase()
              .replace(/[^\w -]/g, "")
              .replaceAll(" ", "-"),
        );
        assert(
          slugs.includes(fragment),
          `${path}: missing ${route}#${fragment}`,
        );
      }
    }
  }
});

test("format rationale and current limits remain explicit", () => {
  const iru = routes.get("/authoring/iru-format").source;
  for (const text of [
    "business rules",
    "not standard XML",
    "ordinary `.rs`",
    "tradeoffs",
    "not implemented",
  ])
    assert(iru.includes(text), text);
  const status = routes.get("/getting-started/status").source;
  assert(status.includes("egui-only"));
  assert(status.includes("not implemented"));
  assert(
    routes
      .get("/internals/architecture")
      .source.includes(":architecture-diagram"),
  );
});

test("copied site assets match their repository sources", async () => {
  for (const [copy, original] of [
    ["public/diagrams/inrux-architecture.svg", "../inrux-architecture.svg"],
    ["public/brand/mark-light.svg", "../assets/brand/inrux-mark-light.svg"],
    ["public/brand/mark-dark.svg", "../assets/brand/inrux-mark-dark.svg"],
    [
      "public/fonts/NotoSans-Regular.ttf",
      "../packages/inrux-text/assets/NotoSans-Regular.ttf",
    ],
    [
      "public/fonts/NotoSansMono.ttf",
      "../playground/egui/src/assets/fonts/NotoSansMono.ttf",
    ],
  ])
    assert.deepEqual(
      await readFile(resolve(root, copy)),
      await readFile(resolve(root, original)),
      copy,
    );
});

test("component API references separate props and emitted payloads", () => {
  for (const { path, source } of documents.filter(
    ({ path }) =>
      /[/\\](?:\d+\.)?components[/\\]/.test(path) &&
      !path.endsWith("/overview.md"),
  )) {
    const api = source.split("## API Reference\n")[1];
    assert(api, `${path}: missing API Reference`);
    assert.match(api, /### Props\n/);
    assert.match(api, /\| Prop \| Type \| Default \| Description \|/);
    assert.match(api, /### Emits\n/);
    assert.match(api, /\| Event \| Payload \| Description \|/);
    assert(!source.includes("## Props and events"), path);
  }
});

test("installation has three OS tabs and current setup instructions", () => {
  const source = routes.get("/getting-started/installation").source;
  for (const os of ["macOS", "Linux", "Windows"]) {
    assert(source.includes(`title="${os}"`), os);
  }
  for (const required of [
    "xcode-select --install",
    "sudo apt-get install",
    "Desktop development with C++",
    "Windows 10 or Windows 11 SDK",
    "rustup default stable-msvc",
  ]) {
    assert(source.includes(required), required);
  }
  assert(source.includes('::tabs{label="Installation operating system"'));
});
