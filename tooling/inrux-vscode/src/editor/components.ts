import * as vscode from "vscode";
import * as path from "node:path";

/** Resolve the compiler's crate/self/super module conventions without a built graph. */
export async function componentUri(
  doc: vscode.TextDocument,
  imported: readonly string[],
): Promise<vscode.Uri | undefined> {
  let root = path.dirname(doc.uri.fsPath);
  while (true) {
    try {
      await vscode.workspace.fs.stat(
        vscode.Uri.file(path.join(root, "Cargo.toml")),
      );
      break;
    } catch {
      const parent = path.dirname(root);
      if (parent === root) return;
      root = parent;
    }
  }
  const source = path.join(root, "src");
  const current = path
    .relative(source, doc.uri.fsPath)
    .replace(/\.iru$/, "")
    .split(path.sep)
    .map(moduleName);
  const rest = [...imported];
  let parts: string[] = [];
  if (rest[0] === "crate") rest.shift();
  else if (rest[0] === "self" || rest[0] === "super") {
    parts = current;
    if (rest[0] === "self") rest.shift();
    while (rest[0] === "super") {
      parts.pop();
      rest.shift();
    }
  }
  parts.push(...rest);
  let directory = vscode.Uri.file(source);
  for (let index = 0; index < parts.length; index++) {
    let entries: [string, vscode.FileType][];
    try {
      entries = await vscode.workspace.fs.readDirectory(directory);
    } catch {
      return;
    }
    if (index === parts.length - 1) {
      const file = entries.find(
        ([name, type]) =>
          (type & vscode.FileType.File) !== 0 &&
          name.endsWith(".iru") &&
          moduleName(name.slice(0, -4)) === parts[index],
      );
      return file ? vscode.Uri.joinPath(directory, file[0]) : undefined;
    }
    const folder = entries.find(
      ([name, type]) =>
        (type & vscode.FileType.Directory) !== 0 &&
        moduleName(name) === parts[index],
    );
    if (!folder) return;
    directory = vscode.Uri.joinPath(directory, folder[0]);
  }
}

/** Match the compiler's conventional PascalCase/acronym-to-module conversion. */
export function moduleName(name: string): string {
  return name
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase();
}
