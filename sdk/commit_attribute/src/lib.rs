extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, Field, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn commit(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let attrs = &input.attrs; // Capture all attributes
    let expanded = if let Fields::Named(fields_named) = &input.fields {
        let mut has_magic_program = false;
        let mut has_magic_context = false;

        for field in &fields_named.named {
            if let Some(ident) = &field.ident {
                if ident == "magic_program" {
                    has_magic_program = true;
                } else if ident == "magic_context" {
                    has_magic_context = true;
                }
            }
        }

        let mut new_fields = fields_named.named.clone();

        if !has_magic_program {
            new_fields.push(
                    Field::parse_named
                        .parse2(quote! {
                            pub magic_program: Program<'info, ::ephemeral_rollups_sdk_v2::anchor::MagicProgram>
                        })
                        .unwrap(),
                );
        }

        if !has_magic_context {
            new_fields.push(
                Field::parse_named
                    .parse2(quote! {
                        #[account(mut, address = ::ephemeral_rollups_sdk_v2::consts::MAGIC_CONTEXT_ID)]
                        /// CHECK:`
                        pub magic_context: AccountInfo<'info>
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
            compile_error!("Commit attribute can only be used with structs with named fields");
        }
    };

    TokenStream::from(expanded)
}
