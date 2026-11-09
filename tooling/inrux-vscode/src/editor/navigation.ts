import * as vscode from "vscode";
import * as path from "node:path";
import { LanguageClient } from "vscode-languageclient/node";
import { documentInfo, range } from "./protocol";
import { viewTags } from "./markup";

/** Locate a tag name without matching tag-like strings inside Rust attributes. */
export function tagNameAt(
  doc: vscode.TextDocument,
  position: vscode.Position,
  body: vscode.Range,
): string | undefined {
  const offset = doc.offsetAt(position) - doc.offsetAt(body.start);
  try {
    for (const tag of viewTags(doc.getText(body))) {
      const match = /^<\/?([A-Za-z_][\w]*)/.exec(tag.text);
      if (!match) continue;
      const start = tag.start + match[0].indexOf(match[1]);
      if (offset >= start && offset <= start + match[1].length) return match[1];
    }
  } catch {
    /* Keep syntax diagnostics authoritative for incomplete attributes. */
  }
  return undefined;
}

/** Resolve only explicitly imported component modules, including grouped and aliased Rust imports. */
export function registerComponentNavigation(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): void {
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider("inrux", {
      async provideDefinition(doc, position) {
        const info = await documentInfo(client, doc);
        if (!range(info.view).contains(position)) return;
        const tag = tagNameAt(doc, position, range(info.view));
        if (!tag) return;
        const imported = info.imports[tag];
        if (!imported) return;
        let sourceRoot = path.dirname(doc.uri.fsPath);
        while (sourceRoot !== path.dirname(sourceRoot)) {
          try {
            await vscode.workspace.fs.stat(
              vscode.Uri.file(path.join(sourceRoot, "Cargo.toml")),
            );
            break;
          } catch {
            sourceRoot = path.dirname(sourceRoot);
          }
        }
        sourceRoot = path.join(sourceRoot, "src");
        let parts = path
          .relative(sourceRoot, doc.uri.fsPath)
          .replace(/\.iru$/, "")
          .split(path.sep)
          .map((name) =>
            name.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase(),
          );
        const rest = [...imported];
        if (rest[0] === "crate") {
          parts = [];
          rest.shift();
        } else if (rest[0] === "self") rest.shift();
        else if (rest[0] === "super") {
          while (rest[0] === "super") {
            parts.pop();
            rest.shift();
          }
        } else return;
        parts.push(...rest);
        let directory = vscode.Uri.file(sourceRoot);
        for (let index = 0; index < parts.length; index++) {
          const entries = await vscode.workspace.fs.readDirectory(directory);
          const component = entries.find(
            ([name, type]) =>
              type === vscode.FileType.File &&
              name.endsWith(".iru") &&
              moduleName(name.slice(0, -4)) === parts[index],
          );
          if (index === parts.length - 1 && component)
            return new vscode.Location(
              vscode.Uri.joinPath(directory, component[0]),
              new vscode.Position(0, 0),
            );
          const folder = entries.find(
            ([name, type]) =>
              type === vscode.FileType.Directory &&
              moduleName(name) === parts[index],
          );
          if (!folder) return;
          directory = vscode.Uri.joinPath(directory, folder[0]);
        }
      },
    }),
  );
}
/** Match the compiler's conventional PascalCase/acronym-to-module conversion. */
export function moduleName(name: string): string {
  return name
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase();
}
