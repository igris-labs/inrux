//! Explicit Rust imports shared by template validation and build-graph discovery.
use crate::view::Element;
use std::collections::BTreeMap;
use syn::UseTree;

/// Flatten explicit imports without treating wildcard prelude imports as component declarations.
pub(crate) fn imports(rust: &syn::File) -> BTreeMap<String, Vec<String>> {
    let mut result = BTreeMap::new();
    for item in &rust.items {
        if let syn::Item::Use(item) = item {
            collect(&item.tree, &[], &mut result);
        }
    }
    result
}

/// Retain aliases exactly as Rust resolves them, including grouped imports.
fn collect(tree: &UseTree, parent: &[String], result: &mut BTreeMap<String, Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            let mut parent = parent.to_vec();
            parent.push(path.ident.to_string());
            collect(&path.tree, &parent, result);
        }
        UseTree::Name(name) => {
            let mut path = parent.to_vec();
            if name.ident != "self" {
                path.push(name.ident.to_string());
            }
            if let Some(binding) = path.last() {
                result.insert(binding.clone(), path);
            }
        }
        UseTree::Rename(rename) => {
            let mut path = parent.to_vec();
            if rename.ident != "self" {
                path.push(rename.ident.to_string());
            }
            result.insert(rename.rename.to_string(), path);
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect(tree, parent, result);
            }
        }
        UseTree::Glob(_) => {}
    }
}

/// Require every custom tag to have an explicit binding in this component's Rust block.
pub(crate) fn validate(tree: &Element, imports: &BTreeMap<String, Vec<String>>) -> syn::Result<()> {
    let name = tree.tag.to_string();
    if !tree.is_builtin() && !imports.contains_key(&name) {
        return Err(tree.error(format!(
            "Cannot find component <{name}> in this file; try importing it in <rust>, for example `use crate::components::{} as {name};` (from {name}.iru)",
            module_name(&name)
        )));
    }
    for child in &tree.children {
        validate(child, imports)?;
    }
    Ok(())
}

/// Convert conventional PascalCase filenames to predictable Rust module names.
pub(crate) fn module_name(name: &str) -> String {
    let mut result = String::new();
    let chars: Vec<_> = name.chars().collect();
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_uppercase()
            && index > 0
            && (chars[index - 1].is_lowercase()
                || chars.get(index + 1).is_some_and(|next| next.is_lowercase()))
        {
            result.push('_');
        }
        result.extend(ch.to_lowercase());
    }
    result
}
