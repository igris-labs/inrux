/** The existing Indocio highlighter supplies escaping and each language grammar. */
export type CodeHighlighter = (
  code: string,
  language: string,
) => Promise<string>;

/** Highlight line-delimited SFC bodies in their own languages without changing source bytes. */
export async function highlightIru(
  source: string,
  highlight: CodeHighlighter,
): Promise<string> {
  const blocks =
    /^([ \t]*<(rust|style)(?:[ \t]+(?:explicit|scoped))?[ \t]*>[ \t]*\r?\n)([\s\S]*?)(^[ \t]*<\/\2>[ \t]*$)/gm;
  const pieces: Promise<string>[] = [];
  let cursor = 0;
  for (const block of source.matchAll(blocks)) {
    pieces.push(highlight(source.slice(cursor, block.index), "vue"));
    pieces.push(highlight(block[1]!, "vue"));
    pieces.push(highlight(block[3]!, block[2] === "rust" ? "rs" : "css"));
    pieces.push(highlight(block[4]!, "vue"));
    cursor = block.index + block[0].length;
  }
  pieces.push(highlight(source.slice(cursor), "vue"));
  return (await Promise.all(pieces)).join("");
}
