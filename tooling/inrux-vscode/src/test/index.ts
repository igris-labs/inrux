import * as vscode from "vscode";
import assert from "node:assert/strict";
import * as path from "node:path";

/** Poll actual providers while rust-analyzer loads Cargo; fail with the last observed provider result. */
async function eventually<T>(
  name: string,
  operation: () => Thenable<T> | Promise<T>,
  predicate: (value: T) => boolean,
  attempts = 90,
): Promise<T> {
  let last: unknown;
  for (let index = 0; index < attempts; index++) {
    try {
      const value = await operation();
      last = value;
      if (predicate(value)) return value;
    } catch (error) {
      last = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`${name}: ${JSON.stringify(last)}`);
}
const target = (location: vscode.Location | vscode.LocationLink) =>
  "targetUri" in location ? location.targetUri : location.uri;
export async function run(): Promise<void> {
  const root = process.env.INRUX_TEST_WORKSPACE!;
  const uri = vscode.Uri.file(path.join(root, "src/App.iru"));
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(doc);
  assert.equal(doc.languageId, "inrux");
  assert.equal(
    vscode.workspace
      .getConfiguration("editor", { uri: doc.uri, languageId: doc.languageId })
      .get("defaultFormatter"),
    "inrux-local.inrux-vscode",
  );
  await vscode.extensions.getExtension("inrux-local.inrux-vscode")!.activate();
  const find = (text: string, last = false) =>
    doc.positionAt(
      last ? doc.getText().lastIndexOf(text) : doc.getText().indexOf(text),
    );
  console.log("TEST: extension activated");
  const components = await eventually(
    "component definition",
    () =>
      vscode.commands.executeCommand<(vscode.Location | vscode.LocationLink)[]>(
        "vscode.executeDefinitionProvider",
        uri,
        find("Example title").translate(0, 2),
      ),
    (result) =>
      !!result?.some((item) => target(item).fsPath.endsWith("Example.iru")),
  );
  assert(components.length > 0);
  assert(
    components.every((item) => target(item).fsPath.endsWith("Example.iru")),
    "component click must not offer generated Rust targets",
  );
  // Exercise actual completion/navigation providers against unsaved partial markup.
  const original = doc.getText();
  const label = (item: vscode.CompletionItem) =>
    typeof item.label === "string" ? item.label : item.label.label;
  const snapshot = async (text: string) => {
    const edit = new vscode.WorkspaceEdit();
    edit.replace(
      uri,
      new vscode.Range(doc.positionAt(0), doc.positionAt(doc.getText().length)),
      text,
    );
    await vscode.workspace.applyEdit(edit);
  };
  const suggestions = async (text: string, needle: string) => {
    await snapshot(text);
    return vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      doc.positionAt(doc.getText().indexOf(needle) + needle.length),
    );
  };
  let items = await suggestions(
    original.replace('<Example title="Child" />', "<Ex"),
    "<Ex",
  );
  assert(
    items?.items.some((item) => label(item) === "Example"),
    "imported component names complete in unfinished tags",
  );
  items = await suggestions(
    original.replace('<Example title="Child" />', "<Example "),
    "<Example ",
  );
  assert(
    items?.items.some(
      (item) => label(item) === "title" && item.detail?.includes("str"),
    ),
    "public Props include Rust types",
  );
  items = await suggestions(
    original.replace('<Example title="Child" />', "<Button "),
    "<Button ",
  );
  assert(
    items?.items.some((item) => label(item) === "@click"),
    "built-in event completion",
  );
  assert(
    items?.items.some((item) => label(item) === "disabled"),
    "built-in prop completion",
  );
  items = await suggestions(
    original.replace('<Example title="Child" />', "<Input model:"),
    "<Input model:",
  );
  assert(
    items?.items.some((item) => label(item) === "model:value"),
    "native model completion",
  );
  await snapshot(
    original.replace("</Column>", "<Input placeholder={</Column>"),
  );
  const incompleteDefinitions = await vscode.commands.executeCommand<
    (vscode.Location | vscode.LocationLink)[]
  >(
    "vscode.executeDefinitionProvider",
    uri,
    find("Example title").translate(0, 2),
  );
  assert(
    incompleteDefinitions?.some((item) =>
      target(item).fsPath.endsWith("Example.iru"),
    ),
    "unfinished later markup must not break component navigation",
  );
  await snapshot(original);
  const aliasDefinitions = await vscode.commands.executeCommand<
    (vscode.Location | vscode.LocationLink)[]
  >(
    "vscode.executeDefinitionProvider",
    uri,
    find("as Example").translate(0, 5),
  );
  assert(
    aliasDefinitions?.some((item) =>
      target(item).fsPath.endsWith("Example.iru"),
    ),
    "import alias navigation resolves the source component",
  );
  const midAttribute =
    await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      uri,
      find('title="Child"').translate(0, 2),
    );
  const titleSuggestion = midAttribute?.items.find(
    (item) => label(item) === "title",
  );
  assert(titleSuggestion, "completion inside an existing attribute name");
  assert.equal(
    titleSuggestion.insertText,
    "title",
    "an existing attribute value must be preserved",
  );
  const titleRange = titleSuggestion.range as vscode.Range;
  assert.equal(
    doc.getText(titleRange),
    "title",
    "replace the whole name, including the suffix after the cursor",
  );
  const language = vscode.extensions.getExtension("inrux-local.inrux-vscode")!
    .packageJSON.contributes.languages[0];
  assert(
    language.icon.light.endsWith("inrux-light.svg") &&
      language.icon.dark.endsWith("inrux-dark.svg"),
  );
  const child = await vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(root, "src/components/Example.iru")),
  );
  const originalChild = child.getText();
  const childEdit = new vscode.WorkspaceEdit();
  childEdit.replace(
    child.uri,
    new vscode.Range(
      child.positionAt(0),
      child.positionAt(originalChild.length),
    ),
    "<view>\n<Text />\n</view>\n<rust>\nuse inrux::prelude::*;\npub struct Props { pub title: String, pub value: String, pub on_update_value: Emit<String>, pub on_save: Emit<()>, private: bool }\n</rust>\n",
  );
  await vscode.workspace.applyEdit(childEdit);
  items = await suggestions(
    original.replace('<Example title="Child" />', "<Example "),
    "<Example ",
  );
  for (const name of ["@save", "@update:value", "model:value"])
    assert(
      items?.items.some((item) => label(item) === name),
      `unsaved child Props complete ${name}`,
    );
  assert(
    !items?.items.some((item) => label(item) === "private"),
    "private fields are not public props",
  );
  const restoreChild = new vscode.WorkspaceEdit();
  restoreChild.replace(
    child.uri,
    new vscode.Range(
      child.positionAt(0),
      child.positionAt(child.getText().length),
    ),
    originalChild,
  );
  await vscode.workspace.applyEdit(restoreChild);
  await snapshot(original);
  console.log(
    "TEST: component/prop/event/model completion, icons and partial-markup navigation passed",
  );
  const edits = await eventually(
    "full formatter",
    () =>
      vscode.commands.executeCommand<vscode.TextEdit[]>(
        "vscode.executeFormatDocumentProvider",
        uri,
        { tabSize: 4, insertSpaces: true },
      ),
    (value) => !!value?.length,
  );
  const change = new vscode.WorkspaceEdit();
  change.set(uri, edits);
  await vscode.workspace.applyEdit(change);
  assert(doc.getText().includes("    <Column>\n        <Example"));
  assert(doc.getText().includes("pub struct State {"));
  assert(doc.getText().includes("padding: 12px;"));
  const second = await vscode.commands.executeCommand<vscode.TextEdit[]>(
    "vscode.executeFormatDocumentProvider",
    uri,
    { tabSize: 4, insertSpaces: true },
  );
  assert.equal(second?.length ?? 0, 0, "formatting must be idempotent");
  const tokens = await vscode.commands.executeCommand<vscode.SemanticTokens>(
    "vscode.provideDocumentSemanticTokens",
    uri,
  );
  assert(tokens && tokens.data.length > 0, "component semantic tag tokens");
  const types = Array.from(tokens.data).filter((_, index) => index % 5 === 3);
  assert(
    types.includes(0) && types.includes(1),
    "lowercase and PascalCase tags have distinct semantic types",
  );
  console.log("TEST: formatting and tag tokens passed");
  const local = await eventually(
    "Rust local definition maps to .iru",
    () =>
      vscode.commands.executeCommand<(vscode.Location | vscode.LocationLink)[]>(
        "vscode.executeDefinitionProvider",
        uri,
        find('caption("Inrux")').translate(0, 2),
      ),
    (value) =>
      !!value?.some((item) => target(item).toString() === uri.toString()),
  );
  assert(local.length > 0);
  await eventually(
    "Rust crate import definition",
    () =>
      vscode.commands.executeCommand<(vscode.Location | vscode.LocationLink)[]>(
        "vscode.executeDefinitionProvider",
        uri,
        find("label(name)").translate(0, 2),
      ),
    (value) =>
      !!value?.some((item) => target(item).fsPath.endsWith("helpers.rs")),
  );
  console.log("TEST: Rust definitions passed");
  await eventually(
    "view handler definition",
    () =>
      vscode.commands.executeCommand<(vscode.Location | vscode.LocationLink)[]>(
        "vscode.executeDefinitionProvider",
        uri,
        find("change_name}").translate(0, 2),
      ),
    (value) =>
      !!value?.some((item) => target(item).toString() === uri.toString()),
  );
  await eventually(
    "Rust signature help",
    () =>
      vscode.commands.executeCommand<vscode.SignatureHelp>(
        "vscode.executeSignatureHelpProvider",
        uri,
        find('caption("Inrux")').translate(0, 8),
      ),
    (value) =>
      !!value?.signatures.some((signature) =>
        signature.label.includes("caption"),
      ),
  );
  const editPartial = new vscode.WorkspaceEdit();
  const at = find("message.len()");
  editPartial.replace(
    uri,
    new vscode.Range(at, at.translate(0, 13)),
    "message.le",
  );
  await vscode.workspace.applyEdit(editPartial);
  const completion = await eventually(
    "Rust completion",
    () =>
      vscode.commands.executeCommand<vscode.CompletionList>(
        "vscode.executeCompletionItemProvider",
        uri,
        find("message.le").translate(0, 10),
      ),
    (value) =>
      !!value?.items.some((item) =>
        (typeof item.label === "string"
          ? item.label
          : item.label.label
        ).startsWith("len"),
      ),
    15,
  );
  const suggestion = completion.items.find((item) =>
    (typeof item.label === "string" ? item.label : item.label.label).startsWith(
      "len",
    ),
  )!;
  const editRange =
    suggestion.range instanceof vscode.Range
      ? suggestion.range
      : suggestion.range?.replacing;
  if (editRange)
    assert.equal(
      doc.getText(editRange),
      "le",
      "completion replacement must map to authored Rust",
    );
  const moveView = new vscode.WorkspaceEdit();
  moveView.insert(uri, doc.positionAt(0), "\n");
  await vscode.workspace.applyEdit(moveView);
  const hover = await eventually(
    "Rust hover",
    () =>
      vscode.commands.executeCommand<vscode.Hover[]>(
        "vscode.executeHoverProvider",
        uri,
        find("message.le").translate(0, 2),
      ),
    (value) => !!value?.length,
  );
  assert(
    hover.some((item) =>
      item.range?.contains(find("message.le").translate(0, 2)),
    ),
    "hover range follows view-only offset changes",
  );
  const missing = new vscode.WorkspaceEdit(),
    start = find("Example title");
  missing.replace(
    uri,
    new vscode.Range(start, start.translate(0, 7)),
    "Missing",
  );
  await vscode.workspace.applyEdit(missing);
  await eventually(
    "unsaved framework diagnostic",
    async () => vscode.languages.getDiagnostics(uri),
    (value) => value.some((item) => item.code === "INRUX203"),
  );
  const correction = new vscode.WorkspaceEdit(),
    bad = find("Missing title");
  correction.replace(
    uri,
    new vscode.Range(bad, bad.translate(0, 7)),
    "Example",
  );
  await vscode.workspace.applyEdit(correction);
  await eventually(
    "cleared framework diagnostic",
    async () => vscode.languages.getDiagnostics(uri),
    (value) => !value.some((item) => item.code === "INRUX203"),
  );
  console.log(
    "INRUX EXTENSION HOST PASSED: .iru recognition, component definition, full/idempotent formatting, tag tokens, Rust local/crate definitions, completion and hover.",
  );
}
