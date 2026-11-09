//! Parse the view DSL and validate names before generating Rust identifiers.
use proc_macro2::Span;
use std::collections::HashSet;
use syn::{
    Expr, Ident, LitStr, Token, braced,
    parse::{Parse, ParseStream},
};

/// An author-written attribute with its original token span.
pub(crate) struct Attribute {
    pub(crate) name: String,
    pub(crate) value: Expr,
    pub(crate) span: Span,
}
/// One component/primitive node with scoped child nodes.
pub(crate) struct Element {
    pub(crate) tag: Ident,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) children: Vec<Element>,
}
impl Parse for Element {
    /// Start parsing at the root; nesting is bounded to avoid exhausting the stack.
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Self::parse_at(input, 0)
    }
}
impl Element {
    /// Parse one balanced element, retaining attribute spans through lowering.
    fn parse_at(input: ParseStream<'_>, depth: usize) -> syn::Result<Self> {
        if depth >= 128 {
            return Err(input.error("View nesting exceeds the supported depth of 128"));
        }
        input.parse::<Token![<]>()?;
        let tag: Ident = input.parse()?;
        let mut attrs = Vec::new();
        let mut seen = HashSet::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let span = input.span();
            let prefix = if input.peek(Token![@]) {
                input.parse::<Token![@]>()?;
                "@"
            } else if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                ":"
            } else {
                ""
            };
            let name: Ident = input.parse()?;
            let mut name = format!("{prefix}{name}");
            if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                let suffix: Ident = input.parse()?;
                name.push(':');
                name.push_str(&suffix.to_string());
            }
            if !seen.insert(name.clone()) {
                return Err(syn::Error::new(
                    span,
                    format!("Duplicate attribute `{name}`"),
                ));
            }
            input.parse::<Token![=]>()?;
            let value = if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                let expression = content.parse()?;
                if !content.is_empty() {
                    return Err(content.error("Expected one Rust expression"));
                }
                expression
            } else {
                Expr::Lit(syn::ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Str(input.parse::<LitStr>()?),
                })
            };
            attrs.push(Attribute { name, value, span });
        }
        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Self {
                tag,
                attrs,
                children: vec![],
            });
        }
        input.parse::<Token![>]>()?;
        let mut children = Vec::new();
        loop {
            if input.is_empty() {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("Unclosed <{tag}> element"),
                ));
            }
            let fork = input.fork();
            fork.parse::<Token![<]>()?;
            if fork.peek(Token![/]) {
                break;
            }
            children.push(Self::parse_at(input, depth + 1)?);
        }
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close: Ident = input.parse()?;
        if close != tag {
            return Err(syn::Error::new(close.span(), format!("Expected </{tag}>")));
        }
        input.parse::<Token![>]>()?;
        Ok(Self {
            tag,
            attrs,
            children,
        })
    }

    /// Anchor lowering failures to the source tag instead of generated Rust.
    pub(crate) fn error(&self, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new(self.tag.span(), message)
    }

    /// Locate a property-level failure in the original template.
    pub(crate) fn attribute_error(
        &self,
        name: &str,
        message: impl std::fmt::Display,
    ) -> syn::Error {
        let span = self
            .attrs
            .iter()
            .find(|attr| attr.name == name || attr.name == format!(":{name}"))
            .map_or(self.tag.span(), |attr| attr.span);
        syn::Error::new(span, message)
    }

    /// Framework-owned tags do not require application imports.
    pub(crate) fn is_builtin(&self) -> bool {
        crate::schema::BUILTINS.contains(&self.tag.to_string().as_str())
    }

    /// Model update listeners augment the generated setter instead of defining a second event prop.
    pub(crate) fn model_update_name<'a>(&self, name: &'a str) -> Option<&'a str> {
        let field = name.strip_prefix("@update:")?;
        self.attrs
            .iter()
            .any(|attr| attr.name == format!("model:{field}"))
            .then_some(field)
    }

    /// Normalize emitted property names before checking semantic duplicates and identifiers.
    pub(crate) fn validate(&self) -> syn::Result<()> {
        let builtin = self.is_builtin();
        if builtin
            && !matches!(
                self.tag.to_string().as_str(),
                "Flex"
                    | "ScrollArea"
                    | "Card"
                    | "Grid"
                    | "AspectRatio"
                    | "Box"
                    | "Container"
                    | "Center"
                    | "Column"
                    | "Row"
                    | "If"
                    | "ElseIf"
                    | "Else"
                    | "For"
                    | "Dialog"
                    | "Provide"
            )
            && !self.children.is_empty()
        {
            return Err(self.error(format!("<{}> does not accept children", self.tag)));
        }
        if !builtin {
            syn::parse_str::<Ident>(&self.tag.to_string().to_lowercase())
                .map_err(|_| self.error(format!("Invalid component name `{}`", self.tag)))?;
        }
        let accepted = crate::schema::attributes(&self.tag.to_string());
        let mut names = HashSet::new();
        for attr in &self.attrs {
            if builtin && !accepted.contains(&attr.name.trim_start_matches(':')) {
                return Err(syn::Error::new(
                    attr.span,
                    format!("Unsupported attribute `{}` on <{}>", attr.name, self.tag),
                ));
            }
            if !builtin && self.model_update_name(&attr.name).is_some() {
                continue;
            }
            let name = if let Some(model) = attr.name.strip_prefix("model:").filter(|_| !builtin) {
                model.to_owned()
            } else if let Some(event) = attr.name.strip_prefix('@') {
                if builtin {
                    attr.name.clone()
                } else {
                    format!("on_{}", event.replace(':', "_"))
                }
            } else {
                attr.name.trim_start_matches(':').to_owned()
            };
            if !builtin {
                syn::parse_str::<Ident>(&name).map_err(|_| {
                    syn::Error::new(
                        attr.span,
                        format!("Invalid component property or event `{}`", attr.name),
                    )
                })?;
            }
            if let Some(model) = attr.name.strip_prefix("model:").filter(|_| !builtin)
                && !names.insert(format!("on_update_{}", model.trim_start_matches("r#")))
            {
                return Err(syn::Error::new(
                    attr.span,
                    format!("Duplicate model update event for `{model}`"),
                ));
            }
            if !names.insert(name.clone()) {
                return Err(syn::Error::new(
                    attr.span,
                    format!("Duplicate property `{name}`"),
                ));
            }
        }
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}
