import * as vscode from "vscode";
import * as path from "node:path";
import { realpath } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { LanguageClient } from "vscode-languageclient/node";
import * as protocol from "vscode-languageserver-protocol";
import { documentInfo, range } from "./protocol";
import { tagNameAt } from "./navigation";

interface SourceMap {
  generated?: string;
  source: string;
  sourceText: string;
  originalStart: number;
  originalEnd: number;
  generatedStart: number;
  generatedEnd: number;
}
interface BuildMap {
  version: number;
  generated: string;
  mappings: SourceMap[];
}
interface Segment {
  uri: vscode.Uri;
  sourceText: string;
  originalStart: number;
  generatedStart: number;
  generatedEnd: number;
}
interface Projection {
  uri: vscode.Uri;
  text: string;
  segments: Segment[];
  version: number;
}
interface RustApi {
  client?: {
    sendRequest<R>(
      method: string,
      params: unknown,
      token?: vscode.CancellationToken,
    ): Promise<R>;
    sendNotification(method: string, params: unknown): Promise<void>;
  };
}
const execute = promisify(execFile);
const byteOffset = (text: string, byte: number): number =>
  Buffer.from(text).subarray(0, byte).toString().length;
export const offsetAt = (
  text: string,
  p: { line: number; character: number },
): number => {
  let offset = 0;
  for (let i = 0; i < p.line; i++) {
    const end = text.indexOf("\n", offset);
    if (end < 0) return text.length;
    offset = end + 1;
  }
  return Math.min(text.length, offset + p.character);
};
export const positionAt = (text: string, offset: number): vscode.Position => {
  const before = text.slice(0, offset),
    lines = before.split("\n");
  return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
};

/** Forward into the real Cargo-generated module, keeping unsaved Rust solely in rust-analyzer's VFS. */
export class RustSupport implements vscode.Disposable {
  private projections = new Map<string, Projection>();
  private diagnostics =
    vscode.languages.createDiagnosticCollection("inrux-rust");
  private subscriptions: vscode.Disposable[] = [];
  private timer: ReturnType<typeof setTimeout> | undefined;
  private queue: Promise<unknown> = Promise.resolve();
  private targets = new Map<string, Promise<string>>();
  private warned = new Set<string>();
  constructor(
    private client: LanguageClient,
    private output: vscode.OutputChannel,
  ) {
    this.subscriptions.push(
      this.diagnostics,
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document.languageId !== "inrux") return;
        clearTimeout(this.timer);
        this.timer = setTimeout(
          () =>
            void this.prepare(event.document).catch((error) =>
              this.output.appendLine(String(error)),
            ),
          300,
        );
      }),
      vscode.languages.onDidChangeDiagnostics((event) => {
        for (const uri of event.uris) {
          const projection = this.projections.get(uri.toString());
          if (projection) this.publish(projection);
        }
      }),
      vscode.workspace.onDidCloseTextDocument((doc) => {
        if (doc.languageId !== "inrux") return;
        this.diagnostics.delete(doc.uri);
        this.warned.delete(doc.uri.toString());
        const extension = vscode.extensions.getExtension<RustApi>(
          "rust-lang.rust-analyzer",
        );
        for (const [key, projection] of this.projections) {
          if (
            !projection.segments.some(
              (segment) => segment.uri.toString() === doc.uri.toString(),
            )
          )
            continue;
          this.projections.delete(key);
          // Closing/discarding must restore the disk-backed Cargo module, not leave an unsaved overlay.
          if (extension?.isActive)
            void extension.exports.client?.sendNotification(
              "textDocument/didClose",
              { textDocument: { uri: key } },
            );
        }
      }),
    );
  }
  /** Serialize VFS updates so simultaneous completion/hover requests never publish older snapshots last. */
  private prepare(doc: vscode.TextDocument): Promise<Projection | undefined> {
    const next = this.queue.then(() => this.createProjection(doc));
    this.queue = next.catch(() => undefined);
    return next;
  }
  private async rustClient() {
    const extension = vscode.extensions.getExtension<RustApi>(
      "rust-lang.rust-analyzer",
    );
    if (!extension)
      throw new Error(
        "Install rust-lang.rust-analyzer for embedded Rust support",
      );
    const api = await extension.activate();
    if (!api.client)
      await vscode.commands.executeCommand("rust-analyzer.startServer");
    if (!api.client)
      throw new Error(
        "rust-analyzer has not started; open the application's Cargo workspace",
      );
    return api.client;
  }
  private async targetDirectory(doc: vscode.TextDocument): Promise<string> {
    const folder = vscode.workspace.getWorkspaceFolder(doc.uri);
    const root = folder?.uri.fsPath ?? path.dirname(doc.uri.fsPath);
    if (!this.targets.has(root))
      this.targets.set(
        root,
        execute("cargo", ["metadata", "--no-deps", "--format-version=1"], {
          cwd: root,
          maxBuffer: 8 * 1024 * 1024,
        }).then((result) => JSON.parse(result.stdout).target_directory),
      );
    return this.targets.get(root)!;
  }
  private async createProjection(
    doc: vscode.TextDocument,
  ): Promise<Projection | undefined> {
    if (doc.isClosed) return undefined;
    const directory = await this.targetDirectory(doc);
    const canonical = await realpath(doc.uri.fsPath);
    const opened = await Promise.all(
      vscode.workspace.textDocuments
        .filter(
          (item) => item.uri.scheme === "file" && item.languageId === "inrux",
        )
        .map(async (item) => ({
          doc: item,
          path: await realpath(item.uri.fsPath).catch(() => item.uri.fsPath),
        })),
    );
    const maps = await vscode.workspace.findFiles(
      new vscode.RelativePattern(directory, "**/inrux_app.map.json"),
      null,
    );
    const candidates = await Promise.all(
      maps.map(async (uri) => ({
        uri,
        modified: (await vscode.workspace.fs.stat(uri)).mtime,
      })),
    );
    candidates.sort((a, b) => b.modified - a.modified);
    for (const candidate of candidates) {
      const map: BuildMap = JSON.parse(
        Buffer.from(
          await vscode.workspace.fs.readFile(candidate.uri),
        ).toString(),
      );
      if (
        map.version !== 1 ||
        !map.mappings.some((item) => path.resolve(item.source) === canonical)
      )
        continue;
      const own = map.mappings.find(
        (item) => path.resolve(item.source) === canonical,
      )!;
      const target = own.generated ?? map.generated;
      const uri = vscode.Uri.file(target);
      if (
        vscode.workspace.textDocuments.some(
          (open) => open.uri.toString() === uri.toString(),
        )
      )
        throw new Error(
          "Close the generated Rust module editor before using embedded Rust support",
        );
      const base = Buffer.from(
        await vscode.workspace.fs.readFile(uri),
      ).toString();
      let text = "",
        cursor = 0;
      const segments: Segment[] = [];
      for (const item of map.mappings
        .filter((item) => (item.generated ?? map.generated) === target)
        .sort((a, b) => a.generatedStart - b.generatedStart)) {
        const start = byteOffset(base, item.generatedStart),
          end = byteOffset(base, item.generatedEnd);
        text += base.slice(cursor, start);
        const open = opened.find((open) => open.path === item.source)?.doc;
        const sourceUri = open?.uri ?? vscode.Uri.file(item.source);
        let sourceText = item.sourceText,
          originalStart = byteOffset(sourceText, item.originalStart),
          body = sourceText.slice(
            originalStart,
            byteOffset(sourceText, item.originalEnd),
          );
        if (open) {
          const info = await documentInfo(this.client, open);
          sourceText = open.getText();
          originalStart = open.offsetAt(range(info.rust).start);
          body = open.getText(range(info.rust));
        }
        const generatedStart = text.length;
        text += body;
        segments.push({
          uri: sourceUri,
          sourceText,
          originalStart,
          generatedStart,
          generatedEnd: text.length,
        });
        cursor = end;
      }
      text += base.slice(cursor);
      const previous = this.projections.get(uri.toString());
      if (doc.isClosed) return undefined;
      if (previous?.text === text) {
        previous.segments = segments;
        this.publish(previous);
        return previous;
      }
      const projection = {
        uri,
        text,
        segments,
        version: (previous?.version ?? 0) + 1,
      };
      const rust = await this.rustClient();
      if (previous)
        await rust.sendNotification("textDocument/didChange", {
          textDocument: { uri: uri.toString(), version: projection.version },
          contentChanges: [{ text }],
        });
      else
        await rust.sendNotification("textDocument/didOpen", {
          textDocument: {
            uri: uri.toString(),
            languageId: "rust",
            version: projection.version,
            text,
          },
        });
      this.projections.set(uri.toString(), projection);
      return projection;
    }
    if (!this.warned.has(doc.uri.toString())) {
      this.warned.add(doc.uri.toString());
      this.output.appendLine(
        `Embedded Rust: build the Cargo application containing ${doc.uri.fsPath} once to generate inrux_app.map.json.`,
      );
    }
    return undefined;
  }
  private mappedRange(
    projection: Projection,
    input: vscode.Range,
  ): { uri: vscode.Uri; range: vscode.Range } | undefined {
    const start = offsetAt(projection.text, input.start),
      end = offsetAt(projection.text, input.end);
    const segment = projection.segments.find(
      (item) => start >= item.generatedStart && end <= item.generatedEnd,
    );
    if (!segment) return;
    return {
      uri: segment.uri,
      range: new vscode.Range(
        positionAt(
          segment.sourceText,
          segment.originalStart + start - segment.generatedStart,
        ),
        positionAt(
          segment.sourceText,
          segment.originalStart + end - segment.generatedStart,
        ),
      ),
    };
  }
  private publish(projection: Projection): void {
    const grouped = new Map<string, vscode.Diagnostic[]>();
    for (const segment of projection.segments)
      grouped.set(segment.uri.toString(), []);
    for (const diagnostic of vscode.languages.getDiagnostics(projection.uri)) {
      const mapped = this.mappedRange(projection, diagnostic.range);
      if (!mapped) continue;
      const translated = new vscode.Diagnostic(
        mapped.range,
        diagnostic.message,
        diagnostic.severity,
      );
      translated.source = "rust-analyzer (Inrux)";
      translated.code = diagnostic.code;
      grouped.get(mapped.uri.toString())!.push(translated);
    }
    for (const [uri, diagnostics] of grouped)
      this.diagnostics.set(vscode.Uri.parse(uri), diagnostics);
  }
  private async request<R>(
    method: string,
    doc: vscode.TextDocument,
    pos: vscode.Position,
    token: vscode.CancellationToken,
    extra: object = {},
  ): Promise<{ result: R; projection: Projection } | undefined> {
    const version = doc.version;
    const info = await documentInfo(this.client, doc);
    let query = pos;
    if (!range(info.rust).contains(pos)) {
      if (
        method !== "textDocument/definition" ||
        !range(info.view).contains(pos)
      )
        return;
      if (tagNameAt(doc, pos, range(info.view))) return;
      const word = doc.getWordRangeAtPosition(pos),
        binding = word ? info.bindings?.[doc.getText(word)] : undefined;
      if (!binding) return;
      query = range(binding).start;
    }
    const projection = await this.prepare(doc);
    if (!projection || version !== doc.version || token.isCancellationRequested)
      return;
    const segment = projection.segments.find(
      (item) => item.uri.toString() === doc.uri.toString(),
    )!;
    const position = positionAt(
      projection.text,
      segment.generatedStart + doc.offsetAt(query) - segment.originalStart,
    );
    const rust = await this.rustClient();
    try {
      const result = await rust.sendRequest<R>(
        method,
        {
          textDocument: { uri: projection.uri.toString() },
          position,
          ...extra,
        },
        token,
      );
      return version === doc.version && !token.isCancellationRequested
        ? { result, projection }
        : undefined;
    } catch (error) {
      // Analyzer invalidation during typing is a canceled request, not an editor error.
      if (
        token.isCancellationRequested ||
        [-32800, -32801, -32802].includes(
          (error as { code?: number }).code ?? 0,
        )
      )
        return;
      throw error;
    }
  }
  /** Register just the requested language services; native Rust files stay with rust-analyzer. */
  register(context: vscode.ExtensionContext): void {
    context.subscriptions.push(
      this,
      vscode.languages.registerHoverProvider("inrux", {
        provideHover: async (doc, pos, token) => {
          const answer = await this.request<protocol.Hover | null>(
            "textDocument/hover",
            doc,
            pos,
            token,
          );
          if (!answer?.result) return;
          const hover = this.client.protocol2CodeConverter.asHover(
            answer.result,
          );
          if (hover.range)
            hover.range = this.mappedRange(
              answer.projection,
              hover.range,
            )?.range;
          return hover;
        },
      }),
      vscode.languages.registerDefinitionProvider("inrux", {
        provideDefinition: async (doc, pos, token) => {
          const answer = await this.request<
            protocol.Definition | protocol.DefinitionLink[] | null
          >("textDocument/definition", doc, pos, token);
          if (!answer?.result) return;
          const converted =
            await this.client.protocol2CodeConverter.asDefinitionResult(
              answer.result,
              token,
            );
          const locations = Array.isArray(converted)
            ? converted
            : converted
              ? [converted]
              : [];
          return locations.map((location) => {
            const target =
              "targetUri" in location
                ? new vscode.Location(
                    location.targetUri,
                    location.targetSelectionRange ?? location.targetRange,
                  )
                : location;
            const projection = this.projections.get(target.uri.toString());
            const mapped = projection
              ? this.mappedRange(projection, target.range)
              : undefined;
            return mapped
              ? new vscode.Location(mapped.uri, mapped.range)
              : target;
          });
        },
      }),
      vscode.languages.registerCompletionItemProvider(
        "inrux",
        {
          provideCompletionItems: async (doc, pos, token, context) => {
            const answer = await this.request<
              protocol.CompletionList | protocol.CompletionItem[] | null
            >("textDocument/completion", doc, pos, token, {
              context: {
                triggerKind: context.triggerKind + 1,
                triggerCharacter: context.triggerCharacter,
              },
            });
            if (!answer) return;
            const converted =
              await this.client.protocol2CodeConverter.asCompletionResult(
                answer.result,
                undefined,
                token,
              );
            if (!converted) return;
            const items = Array.isArray(converted)
              ? converted
              : converted.items;
            const valid = items.filter((item) => {
              if (item.range instanceof vscode.Range) {
                const mapped = this.mappedRange(answer.projection, item.range);
                if (!mapped) return false;
                item.range = mapped.range;
              } else if (item.range) {
                const inserting = this.mappedRange(
                    answer.projection,
                    item.range.inserting,
                  ),
                  replacing = this.mappedRange(
                    answer.projection,
                    item.range.replacing,
                  );
                if (!inserting || !replacing) return false;
                item.range = {
                  inserting: inserting.range,
                  replacing: replacing.range,
                };
              }
              if (item.additionalTextEdits) {
                const edits: vscode.TextEdit[] = [];
                for (const edit of item.additionalTextEdits) {
                  const mapped = this.mappedRange(
                    answer.projection,
                    edit.range,
                  );
                  if (!mapped || mapped.uri.toString() !== doc.uri.toString())
                    return false;
                  edits.push(
                    vscode.TextEdit.replace(mapped.range, edit.newText),
                  );
                }
                item.additionalTextEdits = edits;
              }
              // Commands can target generated files; only apply source-mapped text edits in this first bridge.
              item.command = undefined;
              return true;
            });
            return new vscode.CompletionList(
              valid,
              Array.isArray(converted) ? false : converted.isIncomplete,
            );
          },
        },
        ".",
        ":",
      ),
      vscode.languages.registerSignatureHelpProvider(
        "inrux",
        {
          provideSignatureHelp: async (doc, pos, token) => {
            const answer = await this.request<protocol.SignatureHelp | null>(
              "textDocument/signatureHelp",
              doc,
              pos,
              token,
            );
            return answer
              ? this.client.protocol2CodeConverter.asSignatureHelp(
                  answer.result,
                  token,
                )
              : undefined;
          },
        },
        "(",
        ",",
      ),
    );
    for (const doc of vscode.workspace.textDocuments)
      if (doc.languageId === "inrux")
        void this.prepare(doc).catch((error) =>
          this.output.appendLine(String(error)),
        );
    this.subscriptions.push(
      vscode.workspace.onDidOpenTextDocument((doc) => {
        if (doc.languageId === "inrux")
          void this.prepare(doc).catch((error) =>
            this.output.appendLine(String(error)),
          );
      }),
    );
  }
  dispose(): void {
    clearTimeout(this.timer);
    for (const subscription of this.subscriptions) subscription.dispose();
    const extension = vscode.extensions.getExtension<RustApi>(
      "rust-lang.rust-analyzer",
    );
    if (extension?.isActive)
      for (const projection of this.projections.values())
        void extension.exports.client?.sendNotification(
          "textDocument/didClose",
          { textDocument: { uri: projection.uri.toString() } },
        );
    this.projections.clear();
  }
}
