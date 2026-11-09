//! Compiler-owned block/import metadata shared by editor integrations.
use crate::{CompileError, SourceFile, imports, sfc};
use std::{collections::BTreeMap, ops::Range, sync::Arc};

/// Original byte ranges; editors convert them to their own UTF-16 coordinates.
pub struct EditorDocument {
    /// Authored Rust body, excluding its delimiters.
    pub rust: Range<usize>,
    /// View body, including original expression/attribute bytes.
    pub view: Range<usize>,
    /// Global and scoped stylesheet bodies.
    pub styles: Vec<Range<usize>>,
    /// Explicit Rust import aliases; empty while Rust syntax is incomplete.
    pub imports: BTreeMap<String, Vec<String>>,
    /// Declaration/import binding ranges for navigation from view expressions.
    pub bindings: BTreeMap<String, Range<usize>>,
}

/// Extract editor regions even when the view or Rust code is temporarily incomplete.
///
/// # Errors
/// Returns the compiler's block diagnostic for missing or unbalanced SFC delimiters.
pub fn editor_document(source: SourceFile) -> Result<EditorDocument, CompileError> {
    let source = Arc::new(source);
    let blocks = sfc::parse(&source)?;
    let rust = syn::parse_file(&source.text()[blocks.rust.body.clone()]).ok();
    let imports = rust.as_ref().map(imports::imports).unwrap_or_default();
    let mut bindings = BTreeMap::new();
    if let Some(rust) = &rust {
        let mut identifiers = Vec::new();
        for item in &rust.items {
            match item {
                syn::Item::Fn(item) => identifiers.push(&item.sig.ident),
                syn::Item::Struct(item) => identifiers.push(&item.ident),
                syn::Item::Enum(item) => identifiers.push(&item.ident),
                syn::Item::Const(item) => identifiers.push(&item.ident),
                syn::Item::Type(item) => identifiers.push(&item.ident),
                syn::Item::Use(item) => use_bindings(&item.tree, &mut identifiers),
                _ => {}
            }
        }
        for ident in identifiers {
            bindings.insert(
                ident.to_string(),
                source.parser_range(blocks.rust.body.clone(), ident.span()),
            );
        }
    }
    Ok(EditorDocument {
        rust: blocks.rust.body,
        view: blocks.view.body,
        styles: [blocks.global_style, blocks.style]
            .into_iter()
            .flatten()
            .map(|block| block.body)
            .collect(),
        imports,
        bindings,
    })
}

/// Preserve the actual alias token, not a guessed offset in the import's string form.
fn use_bindings<'a>(tree: &'a syn::UseTree, output: &mut Vec<&'a syn::Ident>) {
    match tree {
        syn::UseTree::Path(path) => use_bindings(&path.tree, output),
        syn::UseTree::Name(name) => output.push(&name.ident),
        syn::UseTree::Rename(rename) => output.push(&rename.rename),
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                use_bindings(tree, output);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}
