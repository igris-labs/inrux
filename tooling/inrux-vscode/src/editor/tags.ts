import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { documentInfo, range } from "./protocol";
import { viewTags } from "./markup";

/** Theme-aware tag tokens complement the grammar without recoloring embedded Rust strings. */
export function registerTagColors(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): void {
  const legend = new vscode.SemanticTokensLegend(
    ["inruxComponent", "inruxLowercaseTag"],
    [],
  );
  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      "inrux",
      {
        async provideDocumentSemanticTokens(doc, token) {
          const info = await documentInfo(client, doc),
            builder = new vscode.SemanticTokensBuilder(legend);
          const start = doc.offsetAt(range(info.view).start);
          try {
            for (const tag of viewTags(doc.getText(range(info.view)))) {
              const match = /^<\/?([A-Za-z_][A-Za-z0-9_]*)/.exec(tag.text);
              if (!match) continue;
              const at = doc.positionAt(
                start + tag.start + match[0].indexOf(match[1]),
              );
              builder.push(
                at.line,
                at.character,
                match[1].length,
                /^[A-Z]/.test(match[1]) ? 0 : 1,
                0,
              );
            }
          } catch {
            /* Incomplete markup keeps TextMate colors until the tag closes. */
          }
          return token.isCancellationRequested ? null : builder.build();
        },
      },
      legend,
    ),
  );
}
