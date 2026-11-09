import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { componentUri } from "./components";
import { completionContext } from "./completion-context";
import { documentInfo, PropInfo, range } from "./protocol";

interface Attribute {
  name: string;
  type: string;
  documentation?: string;
}

/** Offer public props, emitted events and paired model contracts from real Rust fields. */
function propsAttributes(props: Record<string, PropInfo>): Attribute[] {
  return Object.entries(props)
    .filter(([name]) => name !== "children")
    .flatMap(([name, prop]) => {
      const event = name.startsWith("on_") && /\bEmit\s*</.test(prop.type);
      const attribute = event
        ? "@" + name.slice(3).replace(/^update_/, "update:")
        : name;
      const result: Attribute[] = [
        { name: attribute, type: prop.type, documentation: prop.documentation },
      ];
      if (!event && props[`on_update_${name}`])
        result.push({
          name: `model:${name}`,
          type: prop.type,
          documentation: `Two-way binding for ${name}.`,
        });
      return result;
    });
}

/** Complete markup only; embedded Rust remains owned by rust-analyzer. */
export function registerComponentCompletion(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): void {
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      "inrux",
      {
        async provideCompletionItems(doc, position, token) {
          const info = await documentInfo(client, doc);
          const view = range(info.view);
          if (!view.contains(position) || token.isCancellationRequested) return;
          const bodyStart = doc.offsetAt(view.start);
          const at = completionContext(
            doc.getText(view),
            doc.offsetAt(position) - bodyStart,
          );
          if (!at) return;
          const cursor = doc.offsetAt(position);
          const suffix = /^[\w:@-]*/.exec(doc.getText().slice(cursor))![0];
          const replacement = new vscode.Range(
            doc.positionAt(bodyStart + at.start),
            doc.positionAt(cursor + suffix.length),
          );
          const builtins = info.builtins ?? {};
          if (!at.tag) {
            const names = new Set(Object.keys(builtins));
            // Resolve only explicit imports; ordinary Rust helpers are not component tags.
            for (const [name, imported] of Object.entries(info.imports)) {
              if (await componentUri(doc, imported)) names.add(name);
            }
            return [...names]
              .filter((name) => name.startsWith(at.prefix))
              .map((name) => {
                const item = new vscode.CompletionItem(
                  name,
                  vscode.CompletionItemKind.Class,
                );
                item.range = replacement;
                item.detail = builtins[name]
                  ? "Inrux built-in component"
                  : "Imported .iru component";
                return item;
              });
          }
          const common = [
            "id",
            "class",
            "style",
            "gap",
            "padding",
            "width",
            "height",
            "key",
          ];
          let attributes: Attribute[];
          if (builtins[at.tag]) {
            attributes = builtins[at.tag].map((name) => ({
              name,
              type: "Rust expression",
            }));
          } else {
            const imported = info.imports[at.tag];
            const uri = imported && (await componentUri(doc, imported));
            if (!uri) return;
            const child = await vscode.workspace.openTextDocument(uri);
            const childInfo = await documentInfo(client, child);
            attributes = common.map((name) => ({
              name,
              type: "Root styling / identity",
            }));
            attributes.push(...propsAttributes(childInfo.props ?? {}));
          }
          if (token.isCancellationRequested) return;
          return attributes
            .filter(
              ({ name }) =>
                !at.attributes.includes(name) &&
                !at.attributes.includes(`model:${name}`) &&
                !(
                  name.startsWith("model:") &&
                  at.attributes.includes(name.slice(6))
                ),
            )
            .map(({ name, type, documentation }) => {
              const label =
                at.prefix.startsWith(":") &&
                !name.startsWith("@") &&
                !name.startsWith("model:")
                  ? ":" + name
                  : name;
              const item = new vscode.CompletionItem(
                label,
                name.startsWith("@")
                  ? vscode.CompletionItemKind.Event
                  : vscode.CompletionItemKind.Property,
              );
              item.range = replacement;
              item.detail = type;
              item.documentation = documentation;
              const literal =
                !name.startsWith("@") &&
                !name.startsWith("model:") &&
                !label.startsWith(":") &&
                ([
                  "id",
                  "class",
                  "text",
                  "label",
                  "placeholder",
                  "item",
                ].includes(name) ||
                  /\bString\b|&\s*(?:'\w+\s+)?str\b/.test(type));
              // Replace the name only when the existing attribute already has a value.
              const after = doc.getText(
                new vscode.Range(
                  replacement.end,
                  doc.lineAt(position.line).range.end,
                ),
              );
              item.insertText = /^\s*=/.test(after)
                ? label
                : new vscode.SnippetString(
                    literal ? `${label}="\${1}"` : `${label}={\${1}}`,
                  );
              return item;
            });
        },
      },
      "<",
      " ",
      ":",
      "@",
    ),
  );
}
