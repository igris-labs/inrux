//! Typed control lowering, with model setters applied atomically before handlers.
use crate::view::Element;
use proc_macro2::TokenStream;
use quote::quote;
impl Element {
    /// Generate controlled input wiring, keeping each model setter and input action atomic.
    pub(crate) fn input(&self, stateful: bool, actions: bool) -> syn::Result<TokenStream> {
        let (plain, model) = if self.tag == "TextArea" {
            (
                quote::format_ident!("text_area"),
                quote::format_ident!("model_text_area"),
            )
        } else {
            (
                quote::format_ident!("input"),
                quote::format_ident!("model_input"),
            )
        };
        let placeholder = self
            .attr("placeholder")
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(""));
        if let Some(binding) = self.attr("model:value") {
            let invalid = || {
                self.attribute_error(
                    "model:value",
                    "model:value requires a retained field such as name or counter.name on a stateful component",
                )
            };
            if !stateful {
                return Err(invalid());
            }
            if self.attr("value").is_some() {
                return Err(self.attribute_error("value", "Use either model:value or value"));
            }
            if !crate::bindings::model_target(binding) {
                return Err(invalid());
            }
            let action = self.attr("@input").map(|action| {
                if actions {
                    quote!(<State as ::inrux::Component>::update(state, #action, _context);)
                } else {
                    quote!((#action)(state, _context);)
                }
            });
            let position = self.tag.span().start();
            let id = format!("{}:{}", position.line, position.column);
            Ok(
                quote!(::inrux::Node::#model(::inrux::ModelValue::<String>::model_value(&#binding), (#placeholder).to_string(), events.model_versioned(concat!(module_path!(), ":", #id), |state, value, _context| { ::inrux::ModelValue::set_model_value(&mut #binding, value); #action }))),
            )
        } else {
            let value = self.required("value")?;
            let event = self.required("@input")?;
            let event = if stateful && actions {
                quote!(events.map_action(#event))
            } else if stateful {
                quote!(events.handler_payload(#event))
            } else {
                quote!(#event)
            };
            Ok(
                quote!(::inrux::Node::#plain((#value).to_string(), (#placeholder).to_string(), #event)),
            )
        }
    }

    /// Lower a boolean model or an explicit checked/change pair.
    pub(crate) fn checkbox(&self, stateful: bool, actions: bool) -> syn::Result<TokenStream> {
        let label = self.required("label")?;
        let (checked, event) =
            self.control_binding("checked", "model:checked", stateful, actions)?;
        let disabled = self
            .attr("disabled")
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(false));
        let constructor = if self.tag == "Switch" {
            quote!(switch)
        } else {
            quote!(checkbox)
        };
        Ok(
            quote!(::inrux::Node::#constructor((#label).to_string(), #checked, #event).disabled(#disabled)),
        )
    }
    /// Lower a controlled string selection; options remain Rust's typed owned values.
    pub(crate) fn select(&self, stateful: bool, actions: bool) -> syn::Result<TokenStream> {
        let options = self.required("options")?;
        let (value, event) = self.control_binding("value", "model:value", stateful, actions)?;
        let disabled = self
            .attr("disabled")
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(false));
        Ok(
            quote!(::inrux::Node::select((#value).to_string(), #options, #event).disabled(#disabled)),
        )
    }
    /// Share direct-field validation and ordinary/advanced handler wiring between controls.
    fn control_binding(
        &self,
        value_name: &str,
        model_name: &str,
        stateful: bool,
        actions: bool,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        if let Some(binding) = self.attr(model_name) {
            if self.attr(value_name).is_some() {
                return Err(self.attribute_error(value_name, "Use either a model or a value"));
            }
            if !stateful || !crate::bindings::model_target(binding) {
                return Err(self.attribute_error(
                    model_name,
                    "Model requires a retained state field; props are readonly",
                ));
            }
            let callback = self.attr("@change").map(|handler| {
                if actions {
                    quote!(<State as ::inrux::Component>::update(state, #handler, _context);)
                } else {
                    quote!((#handler)(state, _context);)
                }
            });
            Ok((
                if value_name == "checked" {
                    quote!(::inrux::ModelValue::<bool>::model_value(&#binding))
                } else {
                    quote!(::inrux::ModelValue::<String>::model_value(&#binding))
                },
                quote!(events.model(|state, value, _context| { ::inrux::ModelValue::set_model_value(&mut #binding, value); #callback })),
            ))
        } else {
            let value = self.required(value_name)?;
            let handler = self.required("@change")?;
            let event = if stateful && actions {
                quote!(events.map_action(#handler))
            } else if stateful {
                quote!(events.handler_payload(#handler))
            } else {
                quote!(#handler)
            };
            Ok((quote!(#value), event))
        }
    }
}
