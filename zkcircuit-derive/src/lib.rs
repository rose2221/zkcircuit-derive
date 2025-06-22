// The 'proc_macro' crate is special and contains the APIs needed to
// hook into the compiler.
extern crate proc_macro;
use quote::ToTokens;
use proc_macro::TokenStream;
use quote::quote;
use quote::format_ident;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident, Type, Path,};
// pub mod witness;
struct FieldInfo{
    name: Ident,
    ty: Type,
    is_input: bool,
}

fn get_inner_type(ty: &Type) -> Option<&Path> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        if let Type::Path(inner_path) = inner_ty {
                            return Some(&inner_path.path);
                        }
                    }
                }
            }
        }
    }
    None
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

      let builder_name = format_ident!("{}Builder", struct_name);

    let allocations = fields.iter().map(|f|{
        let field_name = &f.name;
        let field_name_str = field_name.to_string();
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
    let builder_fields = fields.iter().map(|f|{
        let field_name = &f.name;
        let field_ty = &f.ty;
        quote!{
           pub #field_name: #field_ty
        }
    });
       let new_assignments = fields.iter().map(|f| {
        let field_name = &f.name;
        quote! { #field_name: None }
    });
    let build_setters = fields.iter().map(|f|{
        let field_name =&f.name;
        let inner_ty = get_inner_type(&f.ty).expect("Circuit fields must be Option<T> or T");

        quote! {
            pub fn #field_name(mut self, value: #inner_ty) -> Self {
                self.#field_name = Some(value);
                self
            }
        }
    });

    let build_checks = fields.iter().map(|f|{
        let field_name = &f.name;
        let error_msg = format!("{} field is missing", field_name);
        quote! {
            if self.#field_name.is_none() {
                return Err(#error_msg.to_string());
            }
        }
    });

    let assignments = fields.iter().map(|f| {
        let field_name = &f.name;
        quote! {
            #field_name : self.#field_name
        }
    });
    let build_assignments = fields.iter().map(|f| {
        let field_name = &f.name;
        quote! {
            #field_name: self.#field_name
        }
    });
    let witness_pushes = fields.iter().map(|f| {
        let field_name = &f.name;
        quote! {
            witness.push(self.#field_name.expect("Witness field should be set"));
        }
    });
    let build_assignments = assignments.clone();
    let gen = quote! {
               pub struct #builder_name #impl_generics #where_clause {
            #(#builder_fields),*
        }

        impl #impl_generics bellman::Circuit<F> for #struct_name #ty_generics #where_clause {
            fn synthesize<CS: bellman::ConstraintSystem<F>>(
                self, cs: &mut CS
            ) -> Result<(), bellman::SynthesisError> {
                // Allocate the fields
                #(#allocations)*

                // Return Ok if all allocations were successful
                Ok(())
            }
        }
 

        impl #impl_generics #builder_name #ty_generics #where_clause {
            pub fn new() -> Self {
                Self {
                    #(#new_assignments),*
                }
            }
        
        // impl #impl_generics #builder_name #ty_generics #where_clause {
        //     pub fn new() -> Self {
        //         Self {
        //             #(#assignments),*
        //         }
        //     }
        // }
            #(#build_setters)*
pub fn build(self) -> Result<#struct_name #ty_generics, String> {
                #(#build_checks)*
                Ok(#struct_name {
                    #(#build_assignments),*
                })
            }
        }
        // #(#build_setters)*
        // pub fn build(self) -> Result<#struct_name #ty_generics, &'static str>{
        //     #(#build_checks)*
        //     Ok(#struct_name {
        //         #(#build_assignments),*
        //     })
        // }

        impl #impl_generics crate::Witness<F> for #struct_name #ty_generics #where_clause {
            fn into_witness(self) -> Vec<F> {
                let mut witness = Vec::new();
                #(#witness_pushes)*
                witness
            }
        }
   
    };
   
    gen.into()
}

fn get_field_info(data: &Data) -> Result<Vec<FieldInfo>, syn::Error> {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = data
    {
        fields.named.iter().map(|f|{
            let name = f.ident.clone().unwrap();
            let ty = f.ty.clone();

            let mut is_input = false;
            for attrs in &f.attrs {
                if attrs.path().is_ident("circuit"){
                    let meta_list = attrs.meta.require_list()?;
                    if meta_list.tokens.to_string() == "input" {
                        is_input = true;
                    break;
                    }
                }
            }
            Ok(FieldInfo { name, ty, is_input })
        }).collect()
    } else {
         let span_target = match data {
            Data::Struct(s) => s.fields.to_token_stream(),
            Data::Enum(e) => e.enum_token.to_token_stream(),
            Data::Union(u) => u.union_token.to_token_stream(),
        };
        Err(syn::Error::new_spanned(span_target,  
            "zkcircuit derive macro can only be used on structs with named fields."))
    }

}

// #[cfg(test)]
// mod tests {

//     use crate::witness::Witness;

//     use bellman::{Circuit, ConstraintSystem, SynthesisError};
//     use ff::Field;
//     use bellman::groth16::create_random_proof;

//     struct TestSysmte<F: Field>{
//         _marker: std::marker::PhantomData<F>,
//     }
//     impl<F: Field> ConstraintSystem<F> for TestContraintSystem<F> {
//         type Root = Self;
//         fn alloc<F1, F2, F3>(&mut self, _: F1, _: F2 ) -> Result<bellman::Variable, SynthesisError> 
//         where F1: FnOnce() -> String,
//         F2: FnOnce() -> Result<F, SynthesisError>,{
//             // Mock implementation
//             Ok(bellman::Variable::new_unchecked(bellman::Indiex::Input(0)))
//         }
//         fn alloc_input<F1, F2, F3>(&mut self, _: F1, _: F2) -> Result<bellman::Variable, SynthesisError>
//         where F1: FnOnce() -> String,
//         F2: FnOnce() -> Refult<F, SynthesisError>,
//         {
//             Ok(bellman::Variable::new_unchecked(bellman::Index::Input(0)))

//         }
//         fn enforce<F1, F2, F3, F4>(&mut self, _: F1, _ :F2, _:F3, _:F4)

//     }
// }