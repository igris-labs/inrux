import { watch } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
const extension = resolve(dirname(fileURLToPath(import.meta.url)), ".."),
  workspace = resolve(extension, "../..");
const typescript = spawn("npm", ["run", "watch"], {
  cwd: extension,
  stdio: "inherit",
  shell: process.platform === "win32",
});
let timer,
  running = false,
  pending = false;
/** Coalesce compiler/server edits without concurrent Cargo rebuilds. */
function build() {
  if (running) {
    pending = true;
    return;
  }
  running = true;
  const child = spawn(
    "cargo",
    ["build", "-p", "inrux-language-server", "--locked"],
    { cwd: workspace, stdio: "inherit" },
  );
  child.on("exit", (code) => {
    running = false;
    console.log(
      code === 0
        ? "Inrux server rebuilt. Reload the VS Code window to load changes."
        : "Inrux server build failed.",
    );
    if (pending) {
      pending = false;
      build();
    }
  });
}
const watchers = ["inrux-language-server", "inrux-compiler", "inrux-style"].map(
  (name) =>
    watch(
      resolve(workspace, "packages", name),
      { recursive: true },
      (_, file) => {
        if (!file || (!file.endsWith(".rs") && !file.endsWith(".toml"))) return;
        clearTimeout(timer);
        timer = setTimeout(build, 300);
      },
    ),
);
for (const signal of ["SIGINT", "SIGTERM"])
  process.on(signal, () => {
    clearTimeout(timer);
    watchers.forEach((w) => w.close());
    typescript.kill();
    process.exit();
  });
console.log(
  "Watching TypeScript and Rust tooling. Grammar/manifest changes need only Reload Window.",
);
