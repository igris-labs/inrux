import { runTests } from "@vscode/test-electron";
import { mkdtemp, mkdir, writeFile, symlink, readdir } from "node:fs/promises";
import { tmpdir, homedir } from "node:os";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
const extension = resolve(dirname(fileURLToPath(import.meta.url)), ".."),
  root = resolve(extension, "../..");
const temporary = await mkdtemp(join(tmpdir(), "inrux vscode ")),
  fixture = join(temporary, "application");
await mkdir(join(fixture, "src", "components"), { recursive: true });
await mkdir(join(fixture, ".vscode"));
const q = (text) => JSON.stringify(text);
await writeFile(
  join(fixture, "Cargo.toml"),
  `[package]\nname = "inrux_editor_fixture"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\ninrux = { path = ${q(join(root, "packages/inrux"))}, default-features = false }\n[build-dependencies]\ninrux-compiler = { path = ${q(join(root, "packages/inrux-compiler"))} }\n`,
);
await writeFile(
  join(fixture, "build.rs"),
  'fn main() { inrux_compiler::build_app("src/App.iru", std::env::var("OUT_DIR").unwrap()).unwrap(); }',
);
await writeFile(
  join(fixture, "src/main.rs"),
  'mod helpers;\ninclude!(concat!(env!("OUT_DIR"), "/inrux_app.rs"));\nfn main() {}\n',
);
await writeFile(
  join(fixture, "src/helpers.rs"),
  'pub fn label(name: &str) -> String { format!("Hello {name}") }\n',
);
await writeFile(
  join(fixture, "src/components/Example.iru"),
  "<view>\n<Text text={props.title} />\n</view>\n<rust>\nuse inrux::prelude::*;\npub struct Props { pub title: &'static str }\n</rust>\n",
);
await writeFile(
  join(fixture, "src/App.iru"),
  `<view>\n<Column><Example title="Child" /><lower_panel title="Lowercase" /><Button text="Change" @click={change_name} /></Column>\n</view>\n<rust>\nuse inrux::prelude::*;\nuse crate::components::{example as Example, example as lower_panel};\n#[derive(Clone)]\npub struct State{ pub name:String }\npub fn setup()->State{State{name:String::new()}}\npub fn caption(name: &str) -> String { crate::helpers::label(name) }\npub fn change_name(state: &mut State, _: &Context<State>) { state.name = caption("Inrux"); }\npub fn example()->usize { let message=String::from("hello"); message.len() }\n</rust>\n<style scoped>\n.card{padding:12px;color:#123456;}\n</style>\n`,
);
await writeFile(
  join(fixture, ".vscode/settings.json"),
  JSON.stringify({
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "rust-analyzer.checkOnSave": false,
    "rust-analyzer.cachePriming.enable": false,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "inrux.server.path": join(root, "target/debug/inrux-language-server"),
  }),
);
for (const [args, cwd] of [
  [["build", "-p", "inrux-language-server", "--locked"], root],
  [["check"], fixture],
]) {
  const result = spawnSync("cargo", args, {
    cwd,
    stdio: "inherit",
    env: { ...process.env, CARGO_TARGET_DIR: join(root, "target") },
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
const extensions = join(temporary, "extensions");
await mkdir(extensions);
const installed = join(homedir(), ".vscode/extensions");
const rust = (await readdir(installed)).find((name) =>
  name.startsWith("rust-lang.rust-analyzer-"),
);
if (!rust)
  throw new Error(
    "Install rust-lang.rust-analyzer before the extension-host test",
  );
await symlink(join(installed, rust), join(extensions, rust), "dir");
console.log(`VS Code fixture: ${fixture}`);
delete process.env.ELECTRON_RUN_AS_NODE;
await runTests({
  vscodeExecutablePath:
    process.env.INRUX_VSCODE_EXECUTABLE ??
    "/Applications/Visual Studio Code.app/Contents/MacOS/Code",
  extensionDevelopmentPath: extension,
  extensionTestsPath: join(extension, "out/test/index.js"),
  extensionTestsEnv: {
    INRUX_TEST_WORKSPACE: fixture,
    CARGO_TARGET_DIR: join(root, "target"),
  },
  launchArgs: [
    fixture,
    "--user-data-dir",
    join(temporary, "user"),
    "--extensions-dir",
    extensions,
    "--skip-welcome",
    "--skip-release-notes",
    "--disable-workspace-trust",
    "--disable-gpu",
  ],
});
