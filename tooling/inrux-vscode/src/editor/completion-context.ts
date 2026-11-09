import { viewTags } from "./markup";

export interface CompletionContext {
  tag?: string;
  prefix: string;
  start: number;
  attributes: string[];
  closing: boolean;
}

/** Locate the edited name, excluding quoted attributes and nested Rust values. */
export function completionContext(
  body: string,
  offset: number,
): CompletionContext | undefined {
  const tag = viewTags(body, true).find(
    (tag) => offset > tag.start && offset <= tag.start + tag.text.length,
  );
  if (!tag || tag.text.startsWith("<!--")) return;
  const text = body.slice(tag.start, offset);
  const name = /^<(\/?)([A-Za-z_][\w]*)?$/.exec(text);
  if (name)
    return {
      prefix: name[2] ?? "",
      start: tag.start + 1 + name[1].length,
      attributes: [],
      closing: !!name[1],
    };
  const opening = /^<([A-Za-z_][\w]*)\s+/.exec(text);
  if (!opening) return;
  const attributes: string[] = [];
  let cursor = opening[0].length;
  while (cursor < text.length) {
    while (/\s/.test(text[cursor] ?? "") && cursor < text.length) cursor++;
    if (cursor === text.length) break;
    const start = cursor;
    const attribute = /^[:@A-Za-z_][\w:@-]*/.exec(text.slice(cursor));
    if (!attribute) return;
    cursor += attribute[0].length;
    if (cursor === text.length)
      return {
        tag: opening[1],
        prefix: attribute[0],
        start: tag.start + start,
        attributes,
        closing: false,
      };
    attributes.push(attribute[0].replace(/^:/, ""));
    while (cursor < text.length && /\s/.test(text[cursor])) cursor++;
    if (text[cursor] !== "=") return;
    cursor++;
    while (cursor < text.length && /\s/.test(text[cursor])) cursor++;
    const end = valueEnd(text, cursor);
    if (end === undefined) return;
    cursor = end;
  }
  return {
    tag: opening[1],
    prefix: "",
    start: offset,
    attributes,
    closing: false,
  };
}

/** Skip Rust braces/comments/raw strings so markup completion never enters a value. */
function valueEnd(text: string, start: number): number | undefined {
  if (text[start] === '"') {
    for (let i = start + 1; i < text.length; i++) {
      if (text[i] === "\\") i++;
      else if (text[i] === '"') return i + 1;
    }
    return;
  }
  if (text[start] !== "{") return;
  let depth = 1,
    comments = 0,
    quote = "",
    raw = "";
  for (let i = start + 1; i < text.length; i++) {
    const ch = text[i],
      next = text[i + 1];
    if (raw) {
      if (text.startsWith(raw, i)) {
        i += raw.length - 1;
        raw = "";
      }
      continue;
    }
    if (quote) {
      if (ch === "\\") i++;
      else if (ch === quote) quote = "";
      continue;
    }
    if (comments) {
      if (ch === "/" && next === "*") {
        comments++;
        i++;
      } else if (ch === "*" && next === "/") {
        comments--;
        i++;
      }
      continue;
    }
    if (ch === "/" && next === "*") {
      comments++;
      i++;
      continue;
    }
    if (ch === "/" && next === "/") {
      const newline = text.indexOf("\n", i);
      if (newline < 0) return;
      i = newline;
      continue;
    }
    if (ch === "r") {
      const match = /^r(#{0,255})"/.exec(text.slice(i));
      if (match) {
        raw = '"' + match[1];
        i += match[0].length - 1;
        continue;
      }
    }
    if (
      ch === '"' ||
      (ch === "'" && /^'(?:\\.|[^'\r\n])'/.test(text.slice(i)))
    ) {
      quote = ch;
      continue;
    }
    if (ch === "{") depth++;
    if (ch === "}" && --depth === 0) return i + 1;
  }
}
