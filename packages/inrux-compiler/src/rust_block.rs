//! Lower component initialization statements into ordinary, statically typed Rust.
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::{Item, Pat, Stmt, parse::Parser, spanned::Spanned};

/// Lowered module plus an exact-byte projection for rust-analyzer.
pub(crate) struct RustBlock {
    pub file: syn::File,
    pub projection: serde_json::Value,
    pub bindings: Vec<syn::Ident>,
}

/// Parse without adding a wrapper, keeping parser spans relative to the authored block.
pub(crate) fn parse(source: &str, explicit: bool) -> syn::Result<RustBlock> {
    let file = syn::parse_file(source);
    if explicit {
        let file = file?;
        let state = file
            .items
            .iter()
            .any(|item| matches!(item, Item::Struct(item) if item.ident == "State"));
        let setup = file
            .items
            .iter()
            .any(|item| matches!(item, Item::Fn(item) if item.sig.ident == "setup"));
        if !state || !setup {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "<rust explicit> requires State and a setup function; use <rust> for automatic initialization",
            ));
        }
        return Ok(module(source, file));
    }
    if let Ok(file) = file {
        if let Some(setup) = file
            .items
            .iter()
            .find(|item| matches!(item, Item::Fn(item) if item.sig.ident == "setup"))
        {
            return Err(syn::Error::new_spanned(
                setup,
                "A manual setup function requires <rust explicit>; <rust> uses automatic setup",
            ));
        }
        if file.items.iter().any(|item| matches!(item, Item::Struct(item) if item.ident == "State" || item.ident == "Props")) {
            return Ok(module(source, file));
        }
    }
    let statements = syn::Block::parse_within.parse_str(source)?;
    lower(source, statements)
}

/// Explicit modules and stateless props preserve their original authored bytes.
fn module(source: &str, file: syn::File) -> RustBlock {
    RustBlock {
        file,
        projection: serde_json::json!({"code":source,"mappings":[{
            "originalStart":0,"originalEnd":source.len(),
            "generatedStart":0,"generatedEnd":source.len()
        }]}),
        bindings: vec![],
    }
}

/// Lift items into module scope; only typed top-level bindings become retained fields.
fn lower(source: &str, statements: Vec<Stmt>) -> syn::Result<RustBlock> {
    let mut items = Vec::new();
    let mut initialization = Vec::new();
    let mut fields = Vec::new();
    let mut names = Vec::new();
    let mut bindings = Vec::new();
    let mut seen = BTreeSet::new();
    for statement in statements {
        match statement {
            Stmt::Item(item) => items.push(item),
            Stmt::Local(local) => {
                let pattern = match &local.pat {
                    Pat::Type(pattern) => {
                        let Pat::Ident(binding) = &*pattern.pat else {
                            return Err(syn::Error::new_spanned(
                                &pattern.pat,
                                "A retained binding needs a single name: `let count: u32 = 0;`",
                            ));
                        };
                        if binding.by_ref.is_some()
                            || binding.subpat.is_some()
                            || local
                                .init
                                .as_ref()
                                .is_none_or(|init| init.diverge.is_some())
                        {
                            return Err(syn::Error::new_spanned(
                                &local,
                                "A retained binding needs an owned value and an initializer",
                            ));
                        }
                        let name = &binding.ident;
                        if !seen.insert(name.to_string()) || name == "props" {
                            return Err(syn::Error::new_spanned(
                                name,
                                "Retained names must be unique; `props` is reserved for component props",
                            ));
                        }
                        let ty = &pattern.ty;
                        let attrs = &local.attrs;
                        fields.push(quote!(#(#attrs)* pub #name: #ty));
                        names.push(quote!(#(#attrs)* #name));
                        Some(binding)
                    }
                    Pat::Ident(binding) => Some(binding),
                    _ => None,
                };
                if let Some(binding) = pattern {
                    bindings.push(binding.ident.clone());
                }
                initialization.push(Stmt::Local(local));
            }
            statement => initialization.push(statement),
        }
    }
    if let Some(item) = items
        .iter()
        .find(|item| matches!(item, Item::Fn(item) if item.sig.ident == "setup"))
    {
        return Err(syn::Error::new_spanned(
            item,
            "Do not combine a manual setup function with top-level initialization; use <rust explicit> with module items, or move the function body into <rust>",
        ));
    }
    let explicit_state = items
        .iter()
        .any(|item| matches!(item, Item::Struct(item) if item.ident == "State"));
    let has_props = items
        .iter()
        .any(|item| matches!(item, Item::Struct(item) if item.ident == "Props"));
    let parameters = if has_props {
        quote!(props: Props)
    } else {
        quote!()
    };
    let state = if explicit_state {
        if !matches!(initialization.last(), Some(Stmt::Expr(_, None))) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "An explicit State needs a final State value in <rust>; otherwise use typed let declarations",
            ));
        }
        quote!()
    } else {
        if let Some(Stmt::Expr(expression, None)) = initialization.last()
            && !matches!(
                expression,
                syn::Expr::If(_)
                    | syn::Expr::Match(_)
                    | syn::Expr::Block(_)
                    | syn::Expr::Loop(_)
                    | syn::Expr::While(_)
                    | syn::Expr::ForLoop(_)
                    | syn::Expr::Unsafe(_)
                    | syn::Expr::Const(_)
                    | syn::Expr::TryBlock(_)
            )
        {
            return Err(syn::Error::new_spanned(
                expression,
                "End initialization statements with `;`; retained values use `let name: Type = value;`",
            ));
        }
        let props_field = has_props.then(|| quote!(pub props: Props,));
        quote! {
            #[derive(Clone)]
            pub struct State { #props_field #(#fields,)* }
        }
    };
    let returned = if explicit_state {
        quote!()
    } else {
        let props_value = has_props.then(|| quote!(props,));
        quote!(State { #props_value #(#names,)* })
    };
    let file = syn::parse2(quote! {
        #(#items)*
        #state
        /// Initialize one component instance; generated from the authored Rust block.
        pub fn setup(#parameters) -> State { #(#initialization)* #returned }
    })?;
    let projection = project(source, &items, &initialization, state, parameters, returned);
    Ok(RustBlock {
        file,
        projection,
        bindings,
    })
}

/// Copy authored bytes into their new scopes so editor positions remain reversible.
fn project(
    source: &str,
    items: &[Item],
    initialization: &[Stmt],
    state: TokenStream,
    parameters: TokenStream,
    returned: TokenStream,
) -> serde_json::Value {
    let mut code = String::new();
    let mut mappings = Vec::new();
    for item in items {
        append(source, item.span(), &mut code, &mut mappings);
    }
    if !state.is_empty() {
        code.push_str("#[derive(Clone)]\npub struct State {\n");
        if !parameters.is_empty() {
            code.push_str("pub props: Props,\n");
        }
        // Map generated field names/types back to their typed let declarations too.
        for statement in initialization {
            if let Stmt::Local(local) = statement
                && let Pat::Type(pattern) = &local.pat
                && let Pat::Ident(binding) = &*pattern.pat
            {
                let attrs = &local.attrs;
                code.push_str(&quote!(#(#attrs)*).to_string());
                code.push_str("pub ");
                append(source, binding.ident.span(), &mut code, &mut mappings);
                code.push_str(": ");
                append(source, pattern.ty.span(), &mut code, &mut mappings);
                code.push_str(",\n");
            }
        }
        code.push_str("}\n");
    }
    code.push_str(&format!("\npub fn setup({parameters}) -> State {{\n"));
    for statement in initialization {
        append(source, statement.span(), &mut code, &mut mappings);
    }
    code.push_str(&returned.to_string());
    code.push_str("\n}\n");
    serde_json::json!({"code":code,"mappings":mappings})
}

/// Record UTF-8 byte offsets; consumers convert to their editor's coordinate system.
fn append(
    source: &str,
    span: proc_macro2::Span,
    code: &mut String,
    mappings: &mut Vec<serde_json::Value>,
) {
    let range = span.byte_range();
    let start = code.len();
    code.push_str(&source[range.clone()]);
    mappings.push(serde_json::json!({
        "originalStart":range.start,"originalEnd":range.end,
        "generatedStart":start,"generatedEnd":code.len()
    }));
    code.push('\n');
}
