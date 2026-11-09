/** Reindent tags while retaining all bytes inside attributes, expressions, strings and comments. */
export function viewTags(body: string): { start: number; text: string }[] {
  const tags: { start: number; text: string }[] = [];
  let offset = 0;
  while (offset < body.length) {
    if (/\s/.test(body[offset])) {
      offset++;
      continue;
    }
    const start = offset;
    if (body.startsWith("<!--", offset)) {
      const end = body.indexOf("-->", offset + 4);
      if (end < 0)
        throw new Error("Cannot format an unterminated view comment");
      offset = end + 3;
    } else {
      if (body[offset] !== "<")
        throw new Error("Cannot format view text outside a component tag");
      offset++;
      let braces = 0,
        quote = "",
        rawEnd = "",
        comments = 0;
      for (; offset < body.length; offset++) {
        const ch = body[offset],
          next = body[offset + 1];
        if (rawEnd) {
          if (body.startsWith(rawEnd, offset)) {
            offset += rawEnd.length - 1;
            rawEnd = "";
          }
          continue;
        }
        if (comments) {
          if (ch === "/" && next === "*") {
            comments++;
            offset++;
          } else if (ch === "*" && next === "/") {
            comments--;
            offset++;
          }
          continue;
        }
        if (quote) {
          if (ch === "\\") offset++;
          else if (ch === quote) quote = "";
          continue;
        }
        if (braces && ch === "/" && next === "/") {
          const end = body.indexOf("\n", offset);
          if (end < 0) throw new Error("Unterminated expression");
          offset = end;
          continue;
        }
        if (braces && ch === "/" && next === "*") {
          comments = 1;
          offset++;
          continue;
        }
        if (braces && ch === "r") {
          const raw = /^r(#{0,255})"/.exec(body.slice(offset));
          if (raw) {
            rawEnd = '"' + raw[1];
            offset += raw[0].length - 1;
            continue;
          }
        }
        if (
          ch === '"' ||
          (braces &&
            ch === "'" &&
            /^'(?:\\.|[^'\r\n])'/.test(body.slice(offset)))
        ) {
          quote = ch;
          continue;
        }
        if (ch === "{") braces++;
        else if (ch === "}") {
          if (--braces < 0) throw new Error("Unbalanced view expression");
        } else if (ch === ">" && braces === 0) {
          offset++;
          break;
        }
      }
      if (
        offset > body.length ||
        body[offset - 1] !== ">" ||
        braces ||
        quote ||
        rawEnd ||
        comments
      )
        throw new Error("Cannot format an incomplete view tag");
    }
    tags.push({ start, text: body.slice(start, offset) });
  }
  return tags;
}

/** Indent only physical tags; Rust expression and literal bytes remain untouched. */
export function formatView(body: string, indent: string, eol = "\n"): string {
  const tags = viewTags(body);
  let depth = 0;
  const lines = tags.map(({ text: tag }) => {
    if (tag.startsWith("</")) depth--;
    if (depth < 0) throw new Error("Unbalanced view tags");
    const line = indent.repeat(depth + 1) + tag;
    if (
      !tag.startsWith("</") &&
      !tag.startsWith("<!--") &&
      !/\/\s*>$/.test(tag)
    )
      depth++;
    return line;
  });
  if (depth !== 0) throw new Error("Unbalanced view tags");
  return eol + lines.join(eol) + eol;
}
