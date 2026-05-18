use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemStruct};

fn unchecked_account_type() -> TokenStream2 {
    if cfg!(feature = "backward-compat") {
        quote! { AccountInfo<'info> }
    } else {
        quote! { UncheckedAccount<'info> }
    }
}

fn is_account_info_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(type_path) if type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "AccountInfo"))
}

fn is_option_account_info_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    args.args
        .iter()
        .any(|arg| matches!(arg, syn::GenericArgument::Type(inner) if is_account_info_type(inner)))
}

fn modernize_account_info_type(ty: &syn::Type) -> syn::Type {
    if cfg!(feature = "backward-compat") {
        return ty.clone();
    }
    if is_account_info_type(ty) {
        syn::parse_quote! { UncheckedAccount<'info> }
    } else if is_option_account_info_type(ty) {
        syn::parse_quote! { Option<UncheckedAccount<'info>> }
    } else {
        ty.clone()
    }
}

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
    let unchecked_account = unchecked_account_type();
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
                pub #buffer_field: #unchecked_account,
            });

            new_fields.push(quote! {
                /// CHECK: The delegation record account
                #[account(
                    mut, seeds = [ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG, #field_name.key().as_ref()],
                    bump, seeds::program = delegation_program.key()
                )]
                pub #delegation_record_field: #unchecked_account,
            });

            new_fields.push(quote! {
                /// CHECK: The delegation metadata account
                #[account(
                    mut, seeds = [ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG, #field_name.key().as_ref()],
                    bump, seeds::program = delegation_program.key()
                )]
                pub #delegation_metadata_field: #unchecked_account,
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
                    let payer = anchor_lang::ToAccountInfo::to_account_info(payer);
                    let pda = anchor_lang::ToAccountInfo::to_account_info(&self.#field_name);
                    let owner_program = anchor_lang::ToAccountInfo::to_account_info(&self.owner_program);
                    let buffer = anchor_lang::ToAccountInfo::to_account_info(&self.#buffer_field);
                    let delegation_record = anchor_lang::ToAccountInfo::to_account_info(&self.#delegation_record_field);
                    let delegation_metadata = anchor_lang::ToAccountInfo::to_account_info(&self.#delegation_metadata_field);
                    let delegation_program = anchor_lang::ToAccountInfo::to_account_info(&self.delegation_program);
                    let system_program = anchor_lang::ToAccountInfo::to_account_info(&self.system_program);
                    let del_accounts = ephemeral_rollups_sdk::cpi::DelegateAccounts {
                        payer: &payer,
                        pda: &pda,
                        owner_program: &owner_program,
                        buffer: &buffer,
                        delegation_record: &delegation_record,
                        delegation_metadata: &delegation_metadata,
                        delegation_program: &delegation_program,
                        system_program: &system_program,
                    };
                    ephemeral_rollups_sdk::cpi::delegate_account(del_accounts, seeds, config)
                }
            });
        }

        // Add the original field without `del`
        let field_type = modernize_account_info_type(&field.ty);
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
            pub owner_program: #unchecked_account,
        });
    }
    if !has_delegation_program {
        new_fields.push(quote! {
            /// CHECK: The delegation program
            #[account(address = ephemeral_rollups_sdk::id())]
            pub delegation_program: #unchecked_account,
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
