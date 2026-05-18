use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn delegate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // Extract the struct name and fields
    let struct_name = &input.ident;
    let fields = &input.fields;
    let original_attrs = &input.attrs;

    // Process fields to modify them according to the rules
    let mut new_fields = Vec::new();
    let mut delegate_methods = Vec::new();
    let mut has_owner_program = false;
    let mut has_delegation_program = false;
    let mut has_system_program = false;

    for field in fields.iter() {
        let mut field_attrs = field.attrs.clone();

        let field_name = match &field.ident {
            Some(name) => name,
            None => {
                return syn::Error::new_spanned(
                    field,
                    "Unnamed fields are not supported in this macro",
                )
                .to_compile_error()
                .into();
            }
        };

        // Check if the field has the `del` attribute
        let has_del = field_attrs
            .iter()
            .any(|attr| attr.path.is_ident("account") && attr.tokens.to_string().contains("del"));

        if has_del {
            let buffer_field = syn::Ident::new(&format!("buffer_{field_name}"), field.span());
            let delegation_record_field =
                syn::Ident::new(&format!("delegation_record_{field_name}"), field.span());
            let delegation_metadata_field =
                syn::Ident::new(&format!("delegation_metadata_{field_name}"), field.span());

            // Remove `del` from attributes
            for attr in &mut field_attrs {
                if attr.path.is_ident("account") {
                    let tokens = attr.tokens.to_string();
                    if tokens.contains("del") {
                        let new_tokens = tokens
                            .replace(", del", "")
                            .replace("del, ", "")
                            .replace("del", "");
                        attr.tokens = syn::parse_str(&new_tokens).unwrap();
                    }
                }
            }

            // Add new fields
            new_fields.push(quote! {
                /// CHECK: The buffer account
                #[account(
                    mut, seeds = [ephemeral_rollups_sdk::pda::DELEGATE_BUFFER_TAG, #field_name.key().as_ref()],
                    bump, seeds::program = crate::id()
                )]
                pub #buffer_field: AccountInfo<'info>,
            });

            new_fields.push(quote! {
                /// CHECK: The delegation record account
                #[account(
                    mut, seeds = [ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG, #field_name.key().as_ref()],
                    bump, seeds::program = delegation_program.key()
                )]
                pub #delegation_record_field: AccountInfo<'info>,
            });

            new_fields.push(quote! {
                /// CHECK: The delegation metadata account
                #[account(
                    mut, seeds = [ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG, #field_name.key().as_ref()],
                    bump, seeds::program = delegation_program.key()
                )]
                pub #delegation_metadata_field: AccountInfo<'info>,
            });

            // Add delegate method
            let delegate_method_name =
                syn::Ident::new(&format!("delegate_{field_name}"), field.span());
            delegate_methods.push(quote! {
                pub fn #delegate_method_name<'a>(
                    &'a self,
                    payer: &'a Signer<'info>,
                    seeds: &[&[u8]],
                    config: ephemeral_rollups_sdk::cpi::DelegateConfig,
                ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
                    let del_accounts = ephemeral_rollups_sdk::cpi::DelegateAccounts {
                        payer,
                        pda: &self.#field_name.to_account_info(),
                        owner_program: &self.owner_program,
                        buffer: &self.#buffer_field,
                        delegation_record: &self.#delegation_record_field,
                        delegation_metadata: &self.#delegation_metadata_field,
                        delegation_program: &self.delegation_program,
                        system_program: &self.system_program,
                    };
                    ephemeral_rollups_sdk::cpi::delegate_account(del_accounts, seeds, config)
                }
            });
        }

        // Add the original field without `del`
        let field_type = &field.ty;
        new_fields.push(quote! {
            #(#field_attrs)*
            pub #field_name: #field_type,
        });

        // Check for existing required fields
        if field_name.eq("owner_program") {
            has_owner_program = true;
        }
        if field_name.eq("delegation_program") {
            has_delegation_program = true;
        }
        if field_name.eq("system_program") {
            has_system_program = true;
        }
    }

    // Add missing required fields
    if !has_owner_program {
        new_fields.push(quote! {
            /// CHECK: The owner program of the pda
            #[account(address = crate::id())]
            pub owner_program: AccountInfo<'info>,
        });
    }
    if !has_delegation_program {
        new_fields.push(quote! {
            /// CHECK: The delegation program
            #[account(address = ephemeral_rollups_sdk::id())]
            pub delegation_program: AccountInfo<'info>,
        });
    }
    if !has_system_program {
        new_fields.push(quote! {
            pub system_program: Program<'info, System>,
        });
    }

    // Generate the new struct definition
    let expanded = quote! {
        #(#original_attrs)*
        pub struct #struct_name<'info> {
            #(#new_fields)*
        }

        impl<'info> #struct_name<'info> {
            #(#delegate_methods)*
        }
    };

    TokenStream::from(expanded)
}
