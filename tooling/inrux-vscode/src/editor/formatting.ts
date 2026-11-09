import * as vscode from "vscode";
import { createHash } from "node:crypto";
import { LanguageClient } from "vscode-languageclient/node";
import { documentInfo, range } from "./protocol";
import { formatView } from "./markup";

/** Apply edits against one immutable snapshot so earlier changes cannot shift later ranges. */
export function applyEdits(
  doc: vscode.TextDocument,
  edits: readonly vscode.TextEdit[],
): string {
  let text = doc.getText();
  for (const edit of [...edits].sort(
    (a, b) => doc.offsetAt(b.range.start) - doc.offsetAt(a.range.start),
  )) {
    text =
      text.slice(0, doc.offsetAt(edit.range.start)) +
      edit.newText +
      text.slice(doc.offsetAt(edit.range.end));
  }
  return text;
}

/** Rust uses rustfmt; CSS uses VS Code's CSS formatter; view attributes retain their original bytes. */
export function registerFormatting(
  context: vscode.ExtensionContext,
  client: LanguageClient,
): void {
  const css = new Map<string, string>();
  const changes = new vscode.EventEmitter<vscode.Uri>();
  context.subscriptions.push(
    changes,
    vscode.workspace.registerTextDocumentContentProvider("inrux-css", {
      onDidChange: changes.event,
      provideTextDocumentContent: (uri) => css.get(uri.toString()) ?? "",
    }),
  );
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("inrux", {
      async provideDocumentFormattingEdits(doc, options, token) {
        const version = doc.version;
        const info = await documentInfo(client, doc);
        const eol = doc.eol === vscode.EndOfLine.CRLF ? "\r\n" : "\n";
        const indent = options.insertSpaces
          ? " ".repeat(options.tabSize)
          : "\t";
        const rust = await client.sendRequest<
          {
            range: {
              start: { line: number; character: number };
              end: { line: number; character: number };
            };
            newText: string;
          }[]
        >("textDocument/formatting", {
          textDocument: { uri: doc.uri.toString() },
          text: doc.getText(),
          options,
        });
        const formatted = rust[0]?.newText ?? doc.getText();
        // Rust formatting changes only its body; isolate it before composing other original-range edits.
        const rustBefore = doc
          .getText()
          .slice(0, doc.offsetAt(range(info.rust).start));
        const rustAfter = doc
          .getText()
          .slice(doc.offsetAt(range(info.rust).end));
        const edits = [
          vscode.TextEdit.replace(
            range(info.rust),
            formatted.slice(
              rustBefore.length,
              formatted.length - rustAfter.length,
            ),
          ),
        ];
        edits.push(
          vscode.TextEdit.replace(
            range(info.view),
            formatView(doc.getText(range(info.view)), indent, eol),
          ),
        );
        if (info.styles.length)
          await vscode.extensions
            .getExtension("vscode.css-language-features")
            ?.activate();
        for (const [index, region] of info.styles.entries()) {
          const cssText = doc.getText(range(region));
          // Immutable virtual snapshots avoid asynchronous content-provider refresh races.
          const hash = createHash("sha256").update(cssText).digest("hex");
          const uri = vscode.Uri.parse(
            `inrux-css:/${encodeURIComponent(doc.uri.toString())}/${hash}/${index}.css`,
          );
          css.set(uri.toString(), cssText);
          const virtual = await vscode.workspace.openTextDocument(uri);
          let result = await vscode.commands.executeCommand<vscode.TextEdit[]>(
            "vscode.executeFormatDocumentProvider",
            uri,
            options,
          );
          for (
            let attempt = 0;
            result === undefined &&
            attempt < 10 &&
            !token.isCancellationRequested;
            attempt++
          ) {
            await new Promise((resolve) => setTimeout(resolve, 100));
            result = await vscode.commands.executeCommand<vscode.TextEdit[]>(
              "vscode.executeFormatDocumentProvider",
              uri,
              options,
            );
          }
          if (!result)
            throw new Error(
              "The built-in VS Code CSS formatter is unavailable",
            );
          const content = applyEdits(virtual, result)
            .trim()
            .replace(/\r?\n/g, eol);
          edits.push(
            vscode.TextEdit.replace(range(region), eol + content + eol),
          );
        }
        if (token.isCancellationRequested || version !== doc.version) return [];
        const output = applyEdits(doc, edits);
        return output === doc.getText()
          ? []
          : [
              vscode.TextEdit.replace(
                new vscode.Range(
                  doc.positionAt(0),
                  doc.positionAt(doc.getText().length),
                ),
                output,
              ),
            ];
      },
    }),
  );
}
