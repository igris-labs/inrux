const assert = require("node:assert/strict");
const { test } = require("node:test");
const fs = require("node:fs");
const path = require("node:path");
const { formatView, viewTags } = require("../out/editor/markup");

test("view formatting preserves Rust comparisons, nested blocks, raw strings, comments and lifetimes", () => {
  const body =
    '<Column><Text text={if n > 0 { r#"<Fake> }"# } else { "none" }} /><Button text="A > B" /></Column>';
  const formatted = formatView(body, "    ");
  assert.equal(
    formatted,
    '\n    <Column>\n        <Text text={if n > 0 { r#"<Fake> }"# } else { "none" }} />\n        <Button text="A > B" />\n    </Column>\n',
  );
  assert.equal(formatView(formatted, "    "), formatted);
  assert.equal(viewTags(body).length, 4);
  assert.throws(() => formatView('<Text text={"incomplete"', "    "));
});

test("TextMate distinguishes lowercase tags, PascalCase components and embedded Rust", async () => {
  const onig = require("vscode-oniguruma"),
    { Registry, parseRawGrammar } = require("vscode-textmate");
  await onig.loadWASM(
    fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"))
      .buffer,
  );
  const builtin =
    "/Applications/Visual Studio Code.app/Contents/Resources/app/extensions";
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (p) => new onig.OnigScanner(p),
      createOnigString: (s) => new onig.OnigString(s),
    }),
    loadGrammar: async (scope) => {
      const file =
        scope === "source.inrux"
          ? path.join(__dirname, "../syntaxes/inrux.tmLanguage.json")
          : scope === "source.rust"
            ? path.join(builtin, "rust/syntaxes/rust.tmLanguage.json")
            : scope === "source.css"
              ? path.join(builtin, "css/syntaxes/css.tmLanguage.json")
              : undefined;
      return file ? parseRawGrammar(fs.readFileSync(file, "utf8"), file) : null;
    },
  });
  const grammar = await registry.loadGrammar("source.inrux");
  let stack;
  const scopes = (line, index) => {
    const result = grammar.tokenizeLine(line, stack);
    stack = result.ruleStack;
    return result.tokens.find(
      (t) => t.startIndex <= index && t.endIndex > index,
    ).scopes;
  };
  assert(scopes("<view>", 2).includes("entity.name.tag.inrux"));
  assert(
    scopes('<MyComponent text={format!("{}", value)} />', 3).includes(
      "entity.name.type.class.inrux",
    ),
  );
  assert(scopes("<children />", 2).includes("entity.name.tag.inrux"));
  scopes("</view>", 2);
  scopes("<rust>", 2);
  const rust = scopes("pub fn answer() -> u32 { 42 }", 5);
  assert(rust.includes("source.rust"));
  assert(rust.some((s) => s.includes("storage.type") || s.includes("keyword")));
  scopes("</rust>", 2);
  scopes("<style scoped>", 2);
  assert(scopes(".card { color: #ffffff; }", 9).includes("source.css"));
});

const { completionContext } = require("../out/editor/completion-context");

test("navigation scanner retains complete tags before and after an unfinished tag", () => {
  const tags = viewTags(
    '<Column><Example title="ok" /><Input placeholder={',
    true,
  );
  assert.deepEqual(
    tags.map((t) => t.text),
    ["<Column>", '<Example title="ok" />', "<Input placeholder={"],
  );
  assert.throws(() => viewTags("<Column><Input placeholder={"));
});

test("markup completion distinguishes names, attributes and Rust/string/comment values", () => {
  const at = (text) => completionContext(text, text.length);
  assert.equal(at("<Bu").prefix, "Bu");
  assert.equal(at("</Ex").closing, true);
  assert.equal(at("<Button ").tag, "Button");
  assert.equal(at("<Input model:").prefix, "model:");
  assert.equal(at("<Button @cl").prefix, "@cl");
  assert.equal(at("<Example :ti").prefix, ":ti");
  assert.equal(at('<Button text="literal <Fake '), undefined);
  assert.equal(at('<Text text={format!("<Fake {}>", x) '), undefined);
  assert.equal(at("<!-- <Fake "), undefined);
  assert.deepEqual(
    at('<Text text={if a > 0 { r#"}"# } else { "x" }} class').attributes,
    ["text"],
  );
  assert.deepEqual(at('<Text text="ok" ').attributes, ["text"]);
});
