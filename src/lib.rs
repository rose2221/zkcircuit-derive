// The 'proc_macro' crate is special and contains the APIs needed to
// hook into the compiler.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident, Type,};

struct FieldInfo{
    name: Ident,
    ty: Type,
}

#[proc_macro_derive(zkcircuit, attributes(circuit))]
pub fn zkcircuit_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match get_field_info(&ast.data) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };
    let gen = quote! {};
    gen.into()
}

fn get_field_info(data: &Data) -> Result<Vec<FieldInfo>, syn::Error> {
    if let Data::Struct(DataStruct {
        fields: Field::Named(fields),
        ..
    }) = data
    {
        Ok(fields.named.iter().map(|f|{
            let name = f.ident.clone().unwrap();
            let ty = f.ty.clone();
            FieldInfo { name, ty }
        }).collect())
    } else {
        Err(syn::Error::new_spanned(data,  
            "zkcircuit derive macro can only be used on structs with named fields."))
    }

}