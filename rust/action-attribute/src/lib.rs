extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, Field, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let attrs = &input.attrs; // Capture all attributes
    let expanded = if let Fields::Named(fields_named) = &input.fields {
        let mut escrow_auth = false;
        let mut escrow = false;

        for field in &fields_named.named {
            if let Some(ident) = &field.ident {
                if ident == "escrow_auth" {
                    has_escrow_auth = true;
                } else if ident == "escrow" {
                    has_escrow = true;
                }
            }
        }

        let mut new_fields = fields_named.named.clone();

        if !has_escrow_auth {
            new_fields.push(
                Field::parse_named
                    .parse2(quote! {
                        /// CHECK: the correct pda - this will be moved to the end in the future, meaning you can omit this unless needed
                        pub escrow_auth: UncheckedAccount<'info>
                    })
                    .unwrap(),
            );
        }

        if !has_has_escrow {
            new_fields.push(
                Field::parse_named
                    .parse2(quote! {
                        /// CHECK: the correct pda - this will be moved to the end in the future, meaning you can omit this unless needed
                        pub escrow: UncheckedAccount<'info>
                    })
                    .unwrap(),
            );
        }

        quote! {
            #(#attrs)*
            pub struct #name<'info> {
                #new_fields
            }
        }
    } else {
        quote! {
            compile_error!("Action attribute can only be used with structs with named fields");
        }
    };

    TokenStream::from(expanded)
}
