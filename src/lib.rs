// The 'proc_macro' crate is special and contains the APIs needed to
// hook into the compiler.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident, Type,};

struct FieldInfo{
    name: Ident,
    ty: Type,
    is_input: bool,
}

#[proc_macro_derive(zkcircuit, attributes(circuit))]
pub fn zkcircuit_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match get_field_info(&ast.data) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };
      let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let allocations = field.iter().map(|f|{
        let field_name = &f.name;
        let field_name_str = field.name_to_string();
        if f.is_input {
            quote!{
                let #field_name = cs.alloc_input(
                    || #field_name_str,
                    || self.#field_name.ok_or(bellman::Error::SynthesisError::AssignmentMissing)?
                )
            } 
        }else {

            quote! {
                let #field_name = cs.alloc(
                    || #field_name_str,
                    || self.#field_name.ok_or(bellman::SynthesisError::AssignmentMissing)
                )?;
            }
        }
    });
    let gen = quote! {
        impl #impl_generics bellman::Circuit<F> for #struct_name #ty_generics #where_clause {
            fn synthesis<CS: bellman::ConstraintSystem<F>>(
                self, cs: &mut CS
            ) -> Result<(), bellman::SynthesisError> {
                // Allocate the fields
                #(#allocations)*

                // Return Ok if all allocations were successful
                Ok(())
            }
        }
    };
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

            let mut is_input = false;
            for attrs in &f.attrs {
                if attr.path().is_ident("circuit"){
                    let meta_list = attr.meta.require_list()?;
                    if meta_list.tokens.to_string() == "input" {
                        is_input = true;
                    break;
                    }
                }
            }
            Ok(FieldInfo { name, ty, is_input })
        }).collect())
    } else {
        Err(syn::Error::new_spanned(data,  
            "zkcircuit derive macro can only be used on structs with named fields."))
    }

}