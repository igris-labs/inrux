//! Optional native Markdown documents rendered with existing engine-independent components.
#![deny(missing_docs)]
mod parse;
mod view;
use std::sync::Arc;
/// A parsed, immutable Markdown document; cloning shares the owned block tree.
#[derive(Clone, Debug)]
pub struct Markdown(Arc<[Block]>);
/// Internal presentation blocks keep parser/native details out of application contracts.
#[derive(Clone, Debug)]
enum Block {
    Paragraph(String),
    Heading(u8, String),
    Code(String),
    Quote(Vec<Block>),
    List(Option<u64>, Vec<Block>),
    Item(Vec<Block>),
    Rule,
}
impl Markdown {
    /// Parse once in setup/load code; no HTML engine, image fetch or application callback is involved.
    /// Documents exceeding 128 nested presentation frames fall back to literal source text.
    pub fn parse(source: &str) -> Self {
        Self(parse::blocks(source).into())
    }
    /// Build portable component descriptions from the cached block tree.
    pub fn view(&self) -> inrux_components::Node {
        view::document(&self.0)
    }
}
#[cfg(test)]
mod tests;
