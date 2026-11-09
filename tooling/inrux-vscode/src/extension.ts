import * as vscode from "vscode";
import * as path from "node:path";
import { realpath } from "node:fs/promises";
import { LanguageClient, ServerOptions } from "vscode-languageclient/node";
import { registerComponentCompletion } from "./editor/completion";
import { registerFormatting } from "./editor/formatting";
import { registerComponentNavigation } from "./editor/navigation";
import { RustSupport } from "./editor/rust";
import { registerTagColors } from "./editor/tags";
let client: LanguageClient | undefined;

/** Start only in trusted workspaces; prefer the linked checkout's server when available. */
export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  if (!vscode.workspace.isTrusted) return;
  const output = vscode.window.createOutputChannel("Inrux");
  context.subscriptions.push(output);
  const configured = vscode.workspace
    .getConfiguration("inrux")
    .get<string>("server.path", "");
  const local = path.resolve(
    await realpath(context.extensionPath),
    "../../target/debug",
    process.platform === "win32"
      ? "inrux-language-server.exe"
      : "inrux-language-server",
  );
  let command = configured;
  if (!command) {
    try {
      await vscode.workspace.fs.stat(vscode.Uri.file(local));
      command = local;
    } catch {
      command = "inrux-language-server";
    }
  }
  const server: ServerOptions = { command, args: [] };
  client = new LanguageClient("inrux", "Inrux", server, {
    documentSelector: [{ scheme: "file", language: "inrux" }],
    outputChannel: output,
  });
  context.subscriptions.push(client);
  await client.start();
  client.getFeature("textDocument/formatting").clear();
  registerFormatting(context, client);
  registerComponentNavigation(context, client);
  registerComponentCompletion(context, client);
  registerTagColors(context, client);
  new RustSupport(client, output).register(context);
}
/** Await protocol shutdown so the Rust process does not outlive the extension host. */
export async function deactivate(): Promise<void> {
  await client?.stop();
  client = undefined;
}
