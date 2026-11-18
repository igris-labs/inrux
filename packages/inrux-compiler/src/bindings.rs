//! Resolve template shorthand without changing authored Rust items or lexical locals.
use crate::view::Element;
use quote::{ToTokens, quote};
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    Expr, Item, Pat, Stmt,
    parse::Parser,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
};

/// Both generated and authored State structs expose the same template namespace.
pub(crate) fn resolve(tree: &mut Element, rust: &syn::File) {
    let mut values = BTreeMap::new();
    let mut handlers: BTreeSet<String> = crate::imports::imports(rust).into_keys().collect();
    for item in &rust.items {
        if let Item::Fn(function) = item {
            handlers.insert(function.sig.ident.to_string());
        }
    }
    // State fields take precedence over props with the same name.
    for owner in ["Props", "State"] {
        for item in &rust.items {
            if let Item::Struct(item) = item
                && item.ident == owner
            {
                for field in &item.fields {
                    if let Some(name) = &field.ident
                        && !["state", "props", "events", "scope"]
                            .iter()
                            .any(|reserved| name == reserved)
                    {
                        let value = if owner == "State" {
                            syn::parse_quote!(state.#name)
                        } else {
                            syn::parse_quote!(props.#name)
                        };
                        values.insert(name.to_string(), value);
                    }
                }
            }
        }
    }
    let stateful = rust
        .items
        .iter()
        .any(|item| matches!(item, Item::Struct(item) if item.ident == "State"));
    element(tree, &values, &handlers, &BTreeSet::new(), stateful);
}

/// A loop's item shadows component fields in its key and children, but not its source.
fn element(
    tree: &mut Element,
    values: &BTreeMap<String, Expr>,
    handlers: &BTreeSet<String>,
    outer: &BTreeSet<String>,
    stateful: bool,
) {
    let mut children = outer.clone();
    if tree.tag == "For"
        && let Some(Expr::Lit(item)) = tree.attr("item")
        && let syn::Lit::Str(item) = &item.lit
    {
        children.insert(item.value());
    }
    for attr in &mut tree.attrs {
        let locals = if tree.tag == "For" && attr.name.trim_start_matches(':') == "key" {
            &children
        } else {
            outer
        };
        let label = attr.value.to_token_stream().to_string().replace(" . ", ".");
        let event = attr.name.starts_with('@');
        let model = attr.name.starts_with("model:");
        let direct = path(&attr.value)
            .is_some_and(|root| values.contains_key(&root) || locals.contains(&root));
        let mut scope = locals.clone();
        if event {
            scope.extend(handlers.iter().cloned());
        }
        Resolver {
            values,
            locals: scope,
        }
        .visit_expr_mut(&mut attr.value);
        // Rust method calls remain Rust; only an entire direct value path is unwrapped.
        if direct && !model && (!event || !stateful) {
            let value = &attr.value;
            let text = attr.name.trim_start_matches(':') == "text"
                && matches!(tree.tag.to_string().as_str(), "Text" | "Badge" | "Button");
            let method = if text {
                quote!(__inrux_view_text)
            } else if tree.tag == "For" && attr.name.trim_start_matches(':') == "each" {
                quote!(__inrux_view_each)
            } else {
                quote!(__inrux_view_value)
            };
            attr.value = syn::parse_quote!({
                use ::inrux::ViewValue as _;
                match (&#value).#method() {
                    Ok(__inrux_value) => __inrux_value,
                    Err(__inrux_error) => return ::inrux::Node::text(format!("Cannot read binding `{}`: {}", #label, __inrux_error)),
                }
            });
            attr.owned_text = text;
        }
    }
    for child in &mut tree.children {
        element(child, values, handlers, &children, stateful);
    }
}

/// Only side-effect-free named paths participate in direct unwrapping/model writes.
fn path(value: &Expr) -> Option<String> {
    match value {
        Expr::Path(value) if value.qself.is_none() => {
            value.path.get_ident().map(ToString::to_string)
        }
        Expr::Field(value) if matches!(value.member, syn::Member::Named(_)) => path(&value.base),
        Expr::Paren(value) => path(&value.expr),
        Expr::Group(value) => path(&value.expr),
        _ => None,
    }
}

/// Models write retained fields, including nested composables; props remain readonly.
pub(crate) fn model_target(value: &Expr) -> bool {
    match value {
        Expr::Field(value) if matches!(value.member, syn::Member::Named(_)) => {
            if matches!(&*value.base, Expr::Path(base) if base.path.is_ident("state")) {
                value.member
                    != syn::Member::Named(syn::Ident::new("props", proc_macro2::Span::call_site()))
            } else {
                model_target(&value.base)
            }
        }
        Expr::Paren(value) => model_target(&value.expr),
        Expr::Group(value) => model_target(&value.expr),
        _ => false,
    }
}

struct Resolver<'a> {
    values: &'a BTreeMap<String, Expr>,
    locals: BTreeSet<String>,
}
impl VisitMut for Resolver<'_> {
    fn visit_expr_mut(&mut self, value: &mut Expr) {
        if let Expr::Path(path) = value
            && path.qself.is_none()
            && let Some(name) = path.path.get_ident()
            && !self.locals.contains(&name.to_string())
            && let Some(binding) = self.values.get(&name.to_string())
        {
            *value = binding.clone();
        } else {
            visit_mut::visit_expr_mut(self, value);
        }
    }
    fn visit_item_mut(&mut self, _: &mut Item) {
        // Local functions/types obey Rust item scope and cannot capture template fields.
    }
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let before = self.locals.clone();
        for statement in &block.stmts {
            match statement {
                Stmt::Item(Item::Fn(item)) => {
                    self.locals.insert(item.sig.ident.to_string());
                }
                Stmt::Item(Item::Const(item)) => {
                    self.locals.insert(item.ident.to_string());
                }
                Stmt::Item(Item::Static(item)) => {
                    self.locals.insert(item.ident.to_string());
                }
                Stmt::Item(Item::Use(item)) => {
                    let file = syn::File {
                        shebang: None,
                        attrs: vec![],
                        items: vec![Item::Use(item.clone())],
                    };
                    self.locals
                        .extend(crate::imports::imports(&file).into_keys());
                }
                _ => {}
            }
        }
        for statement in &mut block.stmts {
            if let Stmt::Local(local) = statement {
                if let Some(init) = &mut local.init {
                    self.visit_expr_mut(&mut init.expr);
                    if let Some((_, fallback)) = &mut init.diverge {
                        self.visit_expr_mut(fallback);
                    }
                }
                names(&local.pat, &mut self.locals);
            } else {
                self.visit_stmt_mut(statement);
            }
        }
        self.locals = before;
    }
    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        let before = self.locals.clone();
        for input in &closure.inputs {
            names(input, &mut self.locals);
        }
        self.visit_expr_mut(&mut closure.body);
        self.locals = before;
    }
    fn visit_expr_for_loop_mut(&mut self, expression: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut expression.expr);
        let before = self.locals.clone();
        names(&expression.pat, &mut self.locals);
        self.visit_block_mut(&mut expression.body);
        self.locals = before;
    }
    fn visit_expr_match_mut(&mut self, expression: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut expression.expr);
        for arm in &mut expression.arms {
            let before = self.locals.clone();
            names(&arm.pat, &mut self.locals);
            if let Some((_, guard)) = &mut arm.guard {
                self.visit_expr_mut(guard);
            }
            self.visit_expr_mut(&mut arm.body);
            self.locals = before;
        }
    }
    fn visit_expr_let_mut(&mut self, expression: &mut syn::ExprLet) {
        self.visit_expr_mut(&mut expression.expr);
        names(&expression.pat, &mut self.locals);
    }
    fn visit_expr_if_mut(&mut self, expression: &mut syn::ExprIf) {
        let before = self.locals.clone();
        self.visit_expr_mut(&mut expression.cond);
        self.visit_block_mut(&mut expression.then_branch);
        self.locals = before;
        if let Some((_, otherwise)) = &mut expression.else_branch {
            self.visit_expr_mut(otherwise);
        }
    }
    fn visit_expr_while_mut(&mut self, expression: &mut syn::ExprWhile) {
        let before = self.locals.clone();
        self.visit_expr_mut(&mut expression.cond);
        self.visit_block_mut(&mut expression.body);
        self.locals = before;
    }
    fn visit_field_value_mut(&mut self, value: &mut syn::FieldValue) {
        let before = value.expr.to_token_stream().to_string();
        self.visit_expr_mut(&mut value.expr);
        if value.colon_token.is_none() && before != value.expr.to_token_stream().to_string() {
            value.colon_token = Some(Default::default());
        }
    }
    fn visit_expr_macro_mut(&mut self, expression: &mut syn::ExprMacro) {
        let name = expression
            .mac
            .path
            .segments
            .last()
            .map(|part| part.ident.to_string())
            .unwrap_or_default();
        if name == "vec" {
            let tokens = &expression.mac.tokens;
            if let Ok(mut value) = syn::parse2::<Expr>(quote!([#tokens])) {
                self.visit_expr_mut(&mut value);
                expression.mac.tokens = match value {
                    Expr::Array(value) => value.elems.to_token_stream(),
                    Expr::Repeat(value) => {
                        let (value, count) = (value.expr, value.len);
                        quote!(#value; #count)
                    }
                    _ => return,
                };
            }
        } else if [
            "format",
            "format_args",
            "print",
            "println",
            "eprint",
            "eprintln",
            "dbg",
        ]
        .contains(&name.as_str())
            && let Ok(mut values) = Punctuated::<Expr, syn::Token![,]>::parse_terminated
                .parse2(expression.mac.tokens.clone())
        {
            for value in &mut values {
                if let Expr::Assign(named) = value {
                    self.visit_expr_mut(&mut named.right);
                } else {
                    self.visit_expr_mut(value);
                }
            }
            expression.mac.tokens = values.to_token_stream();
        }
    }
}

/// Gather Rust pattern bindings without interpreting their type or moving their values.
fn names(pattern: &Pat, output: &mut BTreeSet<String>) {
    match pattern {
        Pat::Ident(value) => {
            output.insert(value.ident.to_string());
            if let Some((_, pattern)) = &value.subpat {
                names(pattern, output);
            }
        }
        Pat::Tuple(value) => {
            for value in &value.elems {
                names(value, output);
            }
        }
        Pat::TupleStruct(value) => {
            for value in &value.elems {
                names(value, output);
            }
        }
        Pat::Struct(value) => {
            for value in &value.fields {
                names(&value.pat, output);
            }
        }
        Pat::Slice(value) => {
            for value in &value.elems {
                names(value, output);
            }
        }
        Pat::Reference(value) => names(&value.pat, output),
        Pat::Type(value) => names(&value.pat, output),
        Pat::Paren(value) => names(&value.pat, output),
        Pat::Or(value) => {
            for value in &value.cases {
                names(value, output);
            }
        }
        _ => {}
    }
}
