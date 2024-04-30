use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod prism;
use prism::expand_prism;
mod widget;
use widget::expand_widget;

#[proc_macro_derive(Prism)]
pub fn prism(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_prism(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Widget, attributes(widget))]
pub fn widget(input: TokenStream) -> TokenStream {
    expand_widget(input)
}
