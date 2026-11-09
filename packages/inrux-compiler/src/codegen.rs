//! Lower template nodes into type-checked Rust component implementations.
use crate::{style_codegen::rule_tokens, view::Element};
use inrux_style::Rule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident};

impl Element {
    /// Find an attribute using its literal or dynamic spelling.
    pub(crate) fn attr(&self, name: &str) -> Option<&Expr> {
        self.attrs
            .iter()
            .find(|a| a.name == name || a.name == format!(":{name}"))
            .map(|a| &a.value)
    }

    /// Emit a validated node and its descendants without running application code.
    pub(crate) fn generate(&self, stateful: bool, actions: bool) -> syn::Result<TokenStream> {
        self.generate_fallback(stateful, actions, quote!(::inrux::Node::empty()))
    }

    /// Lower a conditional chain without evaluating its conditions twice.
    fn generate_fallback(
        &self,
        stateful: bool,
        actions: bool,
        fallback: TokenStream,
    ) -> syn::Result<TokenStream> {
        let tag = self.tag.to_string();
        let children = generate_children(&self.children, stateful, actions)?;
        let mut node = match tag.as_str() {
            "Markdown" => {
                let document = self.required("document")?;
                quote!((#document).view())
            }
            "ScrollArea" => {
                let mut node = quote!(::inrux::Node::scroll_area(vec![#(#children),*]));
                if let Some(request) = self.attr("scroll_to") {
                    node = quote!((#node).scroll_to(#request));
                }
                if let Some(event) = self.attr("@scroll") {
                    let event = if stateful && actions {
                        return Err(self.attribute_error(
                            "@scroll",
                            "Scroll observations require a typed handler, not an action enum",
                        ));
                    } else if stateful {
                        quote!(events.listener(#event))
                    } else {
                        quote!(#event)
                    };
                    node = quote!((#node).on_scroll(#event));
                }
                node
            }
            "Card" => quote!(::inrux::Node::card(vec![#(#children),*])),
            "Badge" => {
                let text = self.text();
                quote!(::inrux::Node::badge((#text).to_string()))
            }
            "Separator" => {
                let vertical = self
                    .attr("vertical")
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(false));
                quote!(::inrux::Node::separator(#vertical))
            }
            "Flex" => {
                let mut node = quote!(::inrux::Node::flex(vec![#(#children),*]));
                for (attribute, method) in [("direction", "flex_direction"), ("wrap", "wrap")] {
                    if let Some(value) = self.attr(attribute) {
                        let method = format_ident!("{method}");
                        node = quote!((#node).styled(::inrux::Style::new().#method(#value)));
                    }
                }
                node
            }
            "Grid" => {
                let node = quote!(::inrux::Node::grid(vec![#(#children),*]));
                match self.attr("columns") {
                    Some(value) => {
                        quote!((#node).styled(::inrux::Style::new().grid_columns(#value)))
                    }
                    None => node,
                }
            }
            "AspectRatio" => {
                let node = quote!(::inrux::Node::aspect_ratio(vec![#(#children),*]));
                match self.attr("ratio") {
                    Some(value) => {
                        quote!((#node).styled(::inrux::Style::new().aspect_ratio(#value)))
                    }
                    None => node,
                }
            }
            "Box" => quote!(::inrux::Node::box_container(vec![#(#children),*])),
            "Container" => quote!(::inrux::Node::container(vec![#(#children),*])),
            "Center" => quote!(::inrux::Node::center(vec![#(#children),*])),
            "Spacer" => quote!(::inrux::Node::spacer()),
            "Column" => quote!(::inrux::Node::column(vec![#(#children),*])),
            "Row" => quote!(::inrux::Node::row(vec![#(#children),*])),
            "Text" => {
                let text = self.text();
                quote!(::inrux::Node::text((#text).to_string()))
            }
            "Button" => self.button(stateful, actions),
            "Input" | "TextInput" | "TextArea" => {
                let input = self.input(stateful, actions)?;
                let disabled = self
                    .attr("disabled")
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(false));
                quote!((#input).disabled(#disabled))
            }
            "Checkbox" | "Switch" => self.checkbox(stateful, actions)?,
            "Progress" | "Meter" => {
                let label = self.required("label")?;
                let value = self.required("value")?;
                let max = self.required("max")?;
                if tag == "Progress" {
                    quote!(::inrux::Node::progress((#label).to_string(),#value,#max))
                } else {
                    let min = self.required("min")?;
                    quote!(::inrux::Node::meter((#label).to_string(),#value,#min,#max))
                }
            }
            "Select" => self.select(stateful, actions)?,
            "Dialog" => {
                let open = self.required("open")?;
                let close = self.required("@close")?;
                let event = if stateful && actions {
                    quote!(events.action(#close))
                } else if stateful {
                    quote!(events.handler(#close))
                } else {
                    quote!(#close)
                };
                quote!(::inrux::Node::dialog(#open, ::inrux::Node::column(vec![#(#children),*]), #event))
            }
            "If" | "ElseIf" => {
                let condition = self.required("condition")?;
                quote!(if #condition { ::inrux::Node::column(vec![#(#children),*]) } else { #fallback })
            }
            "Else" => quote!(::inrux::Node::column(vec![#(#children),*])),
            "For" => {
                let each = self.required("each")?;
                let item = self.required("item")?;
                let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(item),
                    ..
                }) = item
                else {
                    return Err(
                        self.attribute_error("item", "For item must be a quoted Rust binding name")
                    );
                };
                let binding = syn::parse_str::<Ident>(&item.value())
                    .map_err(|_| self.attribute_error("item", "Invalid For binding name"))?;
                let key = self.required("key")?;
                quote!({
                    let mut __inrux_rows = Vec::new();
                    for #binding in #each {
                        let __inrux_key = (#key).to_string();
                        let __inrux_scope = ::inrux::component_scope(scope, &__inrux_key);
                        let scope = &__inrux_scope;
                        let _ = scope;
                        __inrux_rows.push(::inrux::Node::column(vec![#(#children),*]).key(__inrux_key));
                    }
                    ::inrux::Node::column(__inrux_rows)
                })
            }
            "Children" | "Slot" if !stateful => quote!(props.children.clone()),
            "Children" | "Slot" => quote!(state.props.children.clone()),
            "Provide" => {
                let value = self.required("value")?;
                quote!({ let __inrux_scope = scope.provide(#value); let scope = &__inrux_scope;
                    let _ = scope; ::inrux::Node::column(vec![#(#children),*]) })
            }
            "Component" => {
                let selected = self.required("is")?;
                let position = self.tag.span().start();
                let id = format!("dynamic:{}:{}", position.line, position.column);
                let keyed = self.attr("key").map(|key| quote!(let __inrux_key = (#key).to_string(); let __inrux_scope = ::inrux::component_scope(&__inrux_scope, &__inrux_key);));
                let node_key = if self.attr("key").is_some() {
                    quote!((#selected).key(__inrux_key))
                } else {
                    quote!(#selected)
                };
                quote!({
                    let __inrux_scope = ::inrux::component_scope(scope, concat!(module_path!(), ":", #id));
                    #keyed
                    let scope = &__inrux_scope;
                    let _ = scope;
                    #node_key
                })
            }
            "Heartbeat" => quote!(::inrux::Node::heartbeat()),
            _ => self.custom(stateful, actions)?,
        };
        if tag != "For"
            && self.is_builtin()
            && let Some(key) = self.attr("key")
        {
            node = quote!((#node).key(#key));
        }
        if let Some(class) = self.attr("class") {
            node = quote!((#node).classes((#class).as_ref()));
        }
        if self.is_builtin() && !matches!(tag.as_str(), "Children" | "Slot" | "Component") {
            node = quote!((#node).component_type(#tag));
        }
        if let Some(id) = self.attr("id") {
            node = quote!((#node).id(#id));
        }
        node = quote!((#node).with_styles(__inrux_rules().clone()));
        for name in ["gap", "padding", "width", "height"] {
            if let Some(value) = self.attr(name) {
                let method = format_ident!("{name}");
                node = quote!((#node).styled(::inrux::Style::new().#method(#value)));
            }
        }
        if let Some(style) = self.attr("style") {
            node = quote!((#node).styled(#style));
        }
        Ok(node)
    }

    /// Require an attribute without treating a missing property as an internal panic.
    pub(crate) fn required(&self, name: &str) -> syn::Result<&Expr> {
        self.attr(name)
            .ok_or_else(|| self.error(format!("<{}> requires `{name}`", self.tag)))
    }

    /// Supply the empty text default used by labels and buttons.
    fn text(&self) -> Expr {
        self.attr("text")
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(""))
    }

    /// Generate a typed button event; stateless children forward their provided event handle.
    fn button(&self, stateful: bool, actions: bool) -> TokenStream {
        let text = self.text();
        let callback = match self.attr("@click") {
            Some(expr) if stateful && actions => quote!(Some(events.action(#expr))),
            Some(expr) if stateful => quote!(Some(events.handler(#expr))),
            Some(expr) => quote!(Some(#expr)),
            None => quote!(None),
        };
        let disabled = self
            .attr("disabled")
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(false));
        quote!(::inrux::Node::button((#text).to_string(), #callback).disabled(#disabled))
    }

    /// Resolve custom tags through the exact explicit Rust import binding.
    fn custom(&self, stateful: bool, actions: bool) -> syn::Result<TokenStream> {
        let module = syn::parse_str::<Ident>(&self.tag.to_string())
            .map_err(|_| self.error("Invalid component module name"))?;
        let mut fields = Vec::new();
        for attr in &self.attrs {
            if self.model_update_name(&attr.name).is_some() {
                continue;
            }
            if [
                "key", "id", "class", "style", "gap", "padding", "width", "height",
            ]
            .contains(&attr.name.trim_start_matches(':'))
            {
                continue;
            }
            if let Some(model) = attr.name.strip_prefix("model:") {
                let binding = &attr.value;
                let invalid = || {
                    self.attribute_error(&attr.name, "Component model requires a direct state.field binding on a stateful parent")
                };
                let Expr::Field(field) = binding else {
                    return Err(invalid());
                };
                if !stateful
                    || !matches!(field.base.as_ref(), Expr::Path(base) if base.path.is_ident("state"))
                {
                    return Err(invalid());
                }
                let member = &field.member;
                let name = syn::parse_str::<Ident>(model)?;
                let event = syn::parse_str::<Ident>(&format!(
                    "on_update_{}",
                    model.trim_start_matches("r#")
                ))
                .map_err(|_| {
                    self.attribute_error(&attr.name, "Invalid component model event name")
                })?;
                fields.push(quote!(#name: ::inrux::ModelValue::model_value(&#binding)));
                let setter = quote!(|state: &mut State, value, _context: &::inrux::Context<State>| ::inrux::ModelValue::set_model_value(&mut state.#member, value));
                // The setter and update listener share one command, so a frame
                // cannot publish a new value with the old validation/dirty state.
                let event_value = match self.attr(&format!("@update:{model}")) {
                    Some(handler) if actions => quote!(events.model(|state, value, context| {
                        ::inrux::ModelValue::set_model_value(&mut state.#member, value);
                        <State as ::inrux::Component>::update(state, #handler, context);
                    })),
                    Some(handler) => quote!(events.model_with_handler(#setter, #handler)),
                    None => quote!(events.model(#setter)),
                };
                fields.push(quote!(#event: #event_value));
                continue;
            }
            let value = &attr.value;
            let (name, value) = if let Some(event) = attr.name.strip_prefix('@') {
                let name = syn::parse_str::<Ident>(&format!("on_{}", event.replace(':', "_")))
                    .map_err(|_| syn::Error::new(attr.span, "Invalid component event name"))?;
                let value = if stateful && actions {
                    quote!(events.action(#value))
                } else if stateful {
                    quote!(events.listener(#value))
                } else {
                    quote!(#value)
                };
                (name, value)
            } else {
                let name = syn::parse_str::<Ident>(attr.name.trim_start_matches(':'))
                    .map_err(|_| syn::Error::new(attr.span, "Invalid component property name"))?;
                (name, quote!(#value))
            };
            fields.push(quote!(#name: #value));
        }
        if !self.children.is_empty() {
            let children = generate_children(&self.children, stateful, actions)?;
            fields.push(quote!(children: ::inrux::Node::column(vec![#(#children),*])));
        }
        let position = self.tag.span().start();
        let identity = format!("{}:{}", position.line, position.column);
        let scoped_key = self.attr("key").map(|key| {
            quote!(
                let __inrux_key = (#key).to_string();
                let __inrux_scope = ::inrux::component_scope(&__inrux_scope, &__inrux_key);
            )
        });
        let node_key = if self.attr("key").is_some() {
            quote!(__inrux_node.key(__inrux_key))
        } else {
            quote!(if __inrux_node.key.is_some() {
                __inrux_node
            } else {
                __inrux_node.key(::inrux::component_key(&__inrux_scope))
            })
        };
        Ok(quote!({
            let __inrux_scope = ::inrux::component_scope(scope, concat!(module_path!(), ":", #identity));
            #scoped_key
            let __inrux_node = if ::inrux::component_depth(&__inrux_scope) > 128 {
                ::inrux::Node::text("Component nesting exceeds the supported depth of 128".into())
            } else { #module::view(#module::Props { #(#fields),* }, events, &__inrux_scope) };
            #node_key
        }))
    }
}

/// Group adjacent If/ElseIf/Else siblings into one Rust conditional expression.
fn generate_children(
    children: &[Element],
    stateful: bool,
    actions: bool,
) -> syn::Result<Vec<TokenStream>> {
    let mut output = vec![];
    let mut index = 0;
    while index < children.len() {
        let child = &children[index];
        if matches!(child.tag.to_string().as_str(), "ElseIf" | "Else") {
            return Err(child.error("ElseIf and Else must follow an If sibling; Else must be last"));
        }
        if child.tag == "If" {
            let mut end = index + 1;
            while let Some(next) = children.get(end) {
                if next.tag == "ElseIf" {
                    end += 1;
                } else if next.tag == "Else" {
                    end += 1;
                    break;
                } else {
                    break;
                }
            }
            let mut branch = quote!(::inrux::Node::empty());
            for item in children[index..end].iter().rev() {
                branch = item.generate_fallback(stateful, actions, branch)?;
            }
            output.push(branch);
            index = end;
        } else {
            output.push(child.generate(stateful, actions)?);
            index += 1;
        }
    }
    Ok(output)
}

/// Emit either a stateful component implementation or a stateless child view.
pub(crate) fn component(
    rust: &syn::File,
    tree: &Element,
    stateful: bool,
    rules: &[Rule],
    global_rules: &[Rule],
) -> syn::Result<String> {
    let actions = rust
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Enum(item) if item.ident == "Action"));
    if matches!(tree.tag.to_string().as_str(), "ElseIf" | "Else") {
        return Err(tree.error("ElseIf and Else must follow an If sibling"));
    }
    let tree = tree.generate(stateful, actions)?;
    let action_type = if actions { quote!(Action) } else { quote!(()) };
    let update = if actions {
        quote!(update(self, action, context);)
    } else {
        quote!(let _ = (action, context);)
    };
    let rule_factory = if rules.is_empty() {
        quote!(::std::vec::Vec::new())
    } else {
        let rules = rules.iter().map(rule_tokens);
        quote!(vec![#(#rules),*])
    };
    let global_rules = global_rules.iter().map(rule_tokens);
    let has_props = rust
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(item) if item.ident == "Props"));
    let child_view = if stateful && has_props {
        quote! {
            /// Compose a retained child on the parent's coordinator; prop changes preserve local state.
            pub fn view<P: ::inrux::Component>(props: Props, events: &::inrux::Events<P>, scope: &::inrux::Scope) -> ::inrux::Node {
                let initial_props = props.clone();
                ::inrux::render_child(events, scope, || setup(initial_props), |state: &mut State| { state.props = props; })
            }
        }
    } else {
        quote!()
    };
    let implementation = if stateful {
        quote! {
            impl ::inrux::Component for State {
                type Action = #action_type;
                fn update(&mut self, action: Self::Action, context: &::inrux::Context<Self>) { #update }
                fn view(&self, events: &::inrux::Events<Self>, scope: &::inrux::Scope) -> ::inrux::Node {
                    let state = self;
                    let _ = (state, events, scope);
                    #tree
                }
            }
        }
    } else {
        quote! { pub fn view<P: ::inrux::Component>(props: Props, events: &::inrux::Events<P>, scope: &::inrux::Scope) -> ::inrux::Node { let _ = (&props, events, scope); #tree } }
    };
    let output = quote! {
        #rust
        /// Global rules contributed by this reachable component.
        pub fn global_styles() -> Vec<::inrux::Rule> { vec![#(#global_rules),*] }
        fn __inrux_rules() -> &'static ::std::sync::Arc<::inrux::CompiledStyles> {
            static RULES: ::std::sync::OnceLock<::std::sync::Arc<::inrux::CompiledStyles>> = ::std::sync::OnceLock::new();
            RULES.get_or_init(|| ::std::sync::Arc::new(::inrux::CompiledStyles::new(#rule_factory)))
        }
        #implementation
        #child_view
    };
    Ok(prettyplease::unparse(&syn::parse2::<syn::File>(output)?))
}
