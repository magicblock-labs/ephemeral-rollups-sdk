extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, Field, Fields, ItemStruct};

fn generated_unchecked_account_type() -> TokenStream2 {
    if cfg!(feature = "backward-compat") {
        quote! { AccountInfo<'info> }
    } else {
        quote! { UncheckedAccount<'info> }
    }
}

#[proc_macro_attribute]
pub fn commit(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let attrs = &input.attrs; // Capture all attributes
    let unchecked_account = generated_unchecked_account_type();
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
                            pub magic_program: Program<'info, ephemeral_rollups_sdk::anchor::MagicProgram>
                        })
                        .unwrap(),
                );
        }

        if !has_magic_context {
            new_fields.push(
                Field::parse_named
                    .parse2(quote! {
                        #[account(mut, address = ephemeral_rollups_sdk::consts::MAGIC_CONTEXT_ID)]
                        /// CHECK:`
                        pub magic_context: #unchecked_account
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
