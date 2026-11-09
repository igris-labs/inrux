import * as vscode from "vscode";
import { componentUri } from "./components";
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
    for (const tag of viewTags(doc.getText(body), true)) {
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
        const tag = range(info.view).contains(position)
          ? tagNameAt(doc, position, range(info.view))
          : Object.keys(info.imports).find((name) => {
              const binding = info.bindings[name];
              return binding && range(binding).contains(position);
            });
        if (!tag) return;
        const imported = info.imports[tag];
        if (!imported) return;
        const uri = await componentUri(doc, imported);
        if (uri) return new vscode.Location(uri, new vscode.Position(0, 0));
      },
    }),
  );
}
