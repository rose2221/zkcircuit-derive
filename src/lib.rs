// The 'proc_macro' crate is special and contains the APIs needed to
// hook into the compiler.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[prac_macro_derive(zkcircuit, attributes(circuit))]
pub fn zkcircuit_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = quote! {};
    gen.into()
}