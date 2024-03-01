// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod prism;
use prism::expand_prism;

#[proc_macro_derive(Prism)]
pub fn prism(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_prism(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
