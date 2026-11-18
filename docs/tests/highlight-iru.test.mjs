import { test } from "node:test";
import assert from "node:assert/strict";
import { highlightIru } from "../app/utils/highlight-iru.ts";

async function highlighted(source) {
  const calls = [];
  const html = await highlightIru(source, async (code, language) => {
    calls.push({ code, language });
    return `[${language}]${code}`;
  });
  assert.equal(calls.map(({ code }) => code).join(""), source);
  return { calls, html };
}

test("complete .iru examples use Rust and CSS grammars inside markup", async () => {
  const source =
    '<view>\n<Button @click={save} />\n</view>\n<rust>\npub fn save() { println!("Saved"); }\n</rust>\n<style scoped>\nButton { color: red; }\n</style>\n';
  const { calls } = await highlighted(source);
  assert.equal(
    calls.find((call) => call.language === "rs").code,
    'pub fn save() { println!("Saved"); }\n',
  );
  assert.equal(
    calls.find((call) => call.language === "css").code,
    "Button { color: red; }\n",
  );
  assert(
    calls
      .filter((call) => call.language === "vue")
      .map((call) => call.code)
      .join("")
      .includes("<Button @click={save} />"),
  );
});

test("CRLF, indented delimiters, empty bodies and multiple style blocks retain source", async () => {
  const source =
    '  <rust>\r\nlet value = "<style>";\r\n  </rust>\r\n<style>\r\n\r\n</style>\r\n<style scoped>\r\n.item { color: blue; }\r\n</style>\r\n';
  const { calls } = await highlighted(source);
  assert.equal(calls.filter((call) => call.language === "rs").length, 1);
  assert.equal(calls.filter((call) => call.language === "css").length, 2);
  const empty = await highlighted("<rust>\n</rust>");
  assert.equal(empty.calls.find((call) => call.language === "rs").code, "");
});

test("view-only snippets and unfinished blocks stay intact", async () => {
  for (const source of [
    '<Button text="Example" @click={save} />',
    "<view>\n<Text />\n</view>\n<rust>\nfn incomplete(",
  ]) {
    const { calls } = await highlighted(source);
    assert(calls.every((call) => call.language === "vue"));
  }
});

test("explicit Rust setup uses the Rust grammar and preserves its tag", async () => {
  const source = "<rust explicit>\nfn setup() -> State { State }\n</rust>\n";
  const calls = [];
  const result = await highlightIru(source, async (code, language) => {
    calls.push([code, language]);
    return code;
  });
  assert.equal(result, source);
  assert(
    calls.some(
      ([code, language]) => code.includes("fn setup") && language === "rs",
    ),
  );
});
