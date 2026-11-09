//! Private GPUI native infrastructure behind the common Inrux adapter contract.
#![deny(missing_docs)]
mod app;
mod capabilities;
mod layout;
mod widgets;
pub use app::Gpui;

mod services;
