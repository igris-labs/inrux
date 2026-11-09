import { fileURLToPath } from "node:url";
import { dirname, resolve, join } from "node:path";
import { mkdir, lstat, realpath, symlink } from "node:fs/promises";
import { homedir } from "node:os";
import { spawnSync } from "node:child_process";
const extension = await realpath(
  resolve(dirname(fileURLToPath(import.meta.url)), ".."),
);
const workspace = resolve(extension, "../..");
/** Build the linked artifacts; failed builds leave the previous extension link untouched. */
for (const [command, args, cwd] of [
  ["cargo", ["build", "-p", "inrux-language-server", "--locked"], workspace],
  ["npm", ["run", "compile"], extension],
]) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
const directory =
  process.env.INRUX_VSCODE_EXTENSIONS_DIR ??
  join(homedir(), ".vscode", "extensions");
await mkdir(directory, { recursive: true });
const target = join(directory, "inrux-local.inrux-vscode");
let existing;
try {
  existing = await lstat(target);
} catch (error) {
  if (error.code !== "ENOENT") throw error;
}
if (existing) {
  if (!existing.isSymbolicLink() || (await realpath(target)) !== extension)
    throw new Error(`Refusing to replace existing extension at ${target}`);
} else
  await symlink(
    extension,
    target,
    process.platform === "win32" ? "junction" : "dir",
  );
console.log(
  `Linked ${target} -> ${extension}\nRun Developer: Reload Window once. Use npm run dev for rebuilds; no reinstall is needed.`,
);
