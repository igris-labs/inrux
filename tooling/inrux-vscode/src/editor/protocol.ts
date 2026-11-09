import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
export interface Position {
  line: number;
  character: number;
}
export interface Region {
  start: Position;
  end: Position;
}
export interface PropInfo {
  type: string;
  documentation: string;
  range: Region;
}
export interface DocumentInfo {
  rust: Region;
  view: Region;
  styles: Region[];
  imports: Record<string, string[]>;
  builtins: Record<string, string[]>;
  props: Record<string, PropInfo>;
  bindings: Record<string, Region>;
}
export const range = (r: Region): vscode.Range =>
  new vscode.Range(
    r.start.line,
    r.start.character,
    r.end.line,
    r.end.character,
  );
/** Use compiler block boundaries, including unsaved/incomplete Rust bodies. */
export async function documentInfo(
  client: LanguageClient,
  doc: vscode.TextDocument,
): Promise<DocumentInfo> {
  return client.sendRequest("inrux/documentInfo", {
    textDocument: { uri: doc.uri.toString() },
    text: doc.getText(),
  });
}
