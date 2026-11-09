//! Convert pulldown-cmark's CommonMark event stream into owned native presentation blocks.
use crate::Block;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
/// Temporary block nesting; inline events append text without allocating native nodes per word.
struct Frame {
    kind: Kind,
    text: String,
    children: Vec<Block>,
}
/// Only block-level semantics become containers; emphasis and links remain readable inline text.
enum Kind {
    Root,
    Paragraph,
    Heading(u8),
    Code,
    Quote,
    List(Option<u64>),
    Item,
}
/// Parse blocks, preserving code whitespace and nested list/quote structure.
pub(crate) fn blocks(source: &str) -> Vec<Block> {
    let mut frames = vec![Frame {
        kind: Kind::Root,
        text: String::new(),
        children: vec![],
    }];
    for event in Parser::new(source) {
        match event {
            Event::Start(tag) => {
                let kind = match tag {
                    Tag::Paragraph => Some(Kind::Paragraph),
                    Tag::Heading { level, .. } => Some(Kind::Heading(level as u8)),
                    Tag::CodeBlock(_) => Some(Kind::Code),
                    Tag::BlockQuote(_) => Some(Kind::Quote),
                    Tag::List(start) => Some(Kind::List(start)),
                    Tag::Item => Some(Kind::Item),
                    _ => None,
                };
                if let Some(kind) = kind {
                    if frames.len() >= 128 {
                        return vec![Block::Paragraph(source.into())];
                    }
                    frames.push(Frame {
                        kind,
                        text: String::new(),
                        children: vec![],
                    });
                }
            }
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::CodeBlock
                | TagEnd::BlockQuote(_)
                | TagEnd::List(_)
                | TagEnd::Item,
            ) => {
                if frames.len() > 1
                    && let Some(frame) = frames.pop()
                    && let Some(parent) = frames.last_mut()
                {
                    let mut children = frame.children;
                    if !frame.text.is_empty()
                        && matches!(frame.kind, Kind::Quote | Kind::List(_) | Kind::Item)
                    {
                        children.insert(0, Block::Paragraph(frame.text.clone()));
                    }
                    let block = match frame.kind {
                        Kind::Heading(level) => Block::Heading(level, frame.text),
                        Kind::Code => {
                            Block::Code(frame.text.strip_suffix('\n').unwrap_or(&frame.text).into())
                        }
                        Kind::Quote => Block::Quote(children),
                        Kind::List(start) => Block::List(start, children),
                        Kind::Item => Block::Item(children),
                        _ => Block::Paragraph(frame.text),
                    };
                    parent.children.push(block);
                }
            }
            Event::Text(text) | Event::Code(text) | Event::InlineHtml(text) => {
                if let Some(frame) = frames.last_mut() {
                    frame.text.push_str(&text);
                }
            }
            Event::Html(text) => {
                if let Some(frame) = frames.last_mut() {
                    frame.children.push(Block::Paragraph(text.into_string()));
                }
            }
            Event::SoftBreak => {
                if let Some(frame) = frames.last_mut() {
                    frame.text.push(' ');
                }
            }
            Event::HardBreak => {
                if let Some(frame) = frames.last_mut() {
                    frame.text.push('\n');
                }
            }
            Event::Rule => {
                if let Some(frame) = frames.last_mut() {
                    frame.children.push(Block::Rule);
                }
            }
            _ => {}
        }
    }
    frames
        .into_iter()
        .next()
        .map_or_else(Vec::new, |frame| frame.children)
}
