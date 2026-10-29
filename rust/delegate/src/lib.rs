use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::ItemStruct;

fn generated_unchecked_account_type() -> TokenStream2 {
    if cfg!(feature = "backward-compat") {
        quote! { AccountInfo<'info> }
    } else {
        quote! { UncheckedAccount<'info> }
    }
}

/// Anchor re-serializes typed accounts after the handler returns. By then a delegated
/// account is owned by the delegation program, so that write fails with
/// `ExternalAccountDataModified` once direct mapping (SIMD-0460) is active.
/// Only `UncheckedAccount` and `AccountInfo` are exempt from that exit.
/// This is a syntactic guard: a proc macro cannot resolve type aliases, so it catches the
/// common mistake rather than a deliberate alias that shadows one of those names.
fn is_untyped_anchor_account(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| {
        matches!(
            segment.ident.to_string().as_str(),
            "UncheckedAccount" | "AccountInfo"
        )
    })
}

/// Splits `#[account(...)]` into its top-level comma-separated constraints.
/// Nested groups such as `seeds = [a, b]` stay intact because they are single token trees.
fn account_constraints(attr: &syn::Attribute) -> Option<Vec<TokenStream2>> {
    let mut trees = attr.tokens.clone().into_iter();
    let (Some(TokenTree::Group(group)), None) = (trees.next(), trees.next()) else {
        return None;
    };
    if group.delimiter() != Delimiter::Parenthesis {
        return None;
    }
    let mut constraints = Vec::new();
    let mut current = TokenStream2::new();
    for tree in group.stream() {
        match tree {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                constraints.push(std::mem::take(&mut current));
            }
            tree => current.extend(Some(tree)),
        }
    }
    constraints.push(current);
    constraints.retain(|constraint| !constraint.is_empty());
    Some(constraints)
}

/// A constraint is the `del` marker only when it is exactly that identifier, so
/// `seeds::program = delegation_program.key()` and the like are never mistaken for it.
fn is_del_marker(constraint: &TokenStream2) -> bool {
    let mut trees = constraint.clone().into_iter();
    matches!(
        (trees.next(), trees.next()),
        (Some(TokenTree::Ident(ident)), None) if ident == "del"
    )
}

/// Removes the `del` marker from an `#[account(...)]` attribute, reporting whether it was present.
fn strip_del_marker(attr: &mut syn::Attribute) -> bool {
    let Some(constraints) = account_constraints(attr) else {
        return false;
    };
    let (del, kept): (Vec<_>, Vec<_>) = constraints.into_iter().partition(is_del_marker);
    if del.is_empty() {
        return false;
    }
    attr.tokens = quote! { ( #(#kept),* ) };
    true
}

#[proc_macro_attribute]
pub fn delegate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand(item.into()).into()
}

fn expand(item: TokenStream2) -> TokenStream2 {
    let input = match syn::parse2::<ItemStruct>(item) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    // Extract the struct name and fields
    let struct_name = &input.ident;
    let fields = &input.fields;
    let original_attrs = &input.attrs;

    // Process fields to modify them according to the rules
    let mut new_fields = Vec::new();
    let mut delegate_methods = Vec::new();
    let unchecked_account = generated_unchecked_account_type();
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
                .to_compile_error();
            }
        };

        // Check if the field has the `del` marker, dropping it from the emitted attributes
        let mut has_del = false;
        for attr in &mut field_attrs {
            if attr.path.is_ident("account") {
                has_del |= strip_del_marker(attr);
            }
        }

        if has_del && !is_untyped_anchor_account(&field.ty) {
            return syn::Error::new_spanned(
                &field.ty,
                "`del` accounts must be `UncheckedAccount<'info>` or `AccountInfo<'info>`. \
                 Anchor re-serializes typed accounts after the handler returns, but by then the \
                 account is owned by the delegation program, so the write fails with \
                 ExternalAccountDataModified once direct mapping (SIMD-0460) is active. \
                 Load the account into a local typed wrapper (e.g. `Account::<T>::try_from`) \
                 inside the handler and call `.exit(&crate::ID)` on it before delegating.",
            )
            .to_compile_error();
        }

        if has_del {
            let buffer_field = syn::Ident::new(&format!("buffer_{field_name}"), field.span());
            let delegation_record_field =
                syn::Ident::new(&format!("delegation_record_{field_name}"), field.span());
            let delegation_metadata_field =
                syn::Ident::new(&format!("delegation_metadata_{field_name}"), field.span());

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

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand_str(item: &str) -> String {
        expand(item.parse().unwrap()).to_string()
    }

    #[test]
    fn del_marker_is_matched_as_whole_constraint() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]
        };
        let constraints = account_constraints(&attr).unwrap();
        assert_eq!(constraints.len(), 4);
        assert_eq!(constraints.iter().filter(|c| is_del_marker(c)).count(), 1);

        let attr: syn::Attribute = syn::parse_quote! {
            #[account(mut, seeds::program = delegation_program.key(), bump)]
        };
        assert!(!account_constraints(&attr)
            .unwrap()
            .iter()
            .any(is_del_marker));
    }

    #[test]
    fn strip_del_marker_keeps_other_constraints() {
        let mut attr: syn::Attribute = syn::parse_quote! {
            #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]
        };
        assert!(strip_del_marker(&mut attr));
        assert_eq!(
            attr.tokens.to_string(),
            quote! { (mut, seeds = [b"player", payer.key().as_ref()], bump) }.to_string()
        );

        let mut attr: syn::Attribute = syn::parse_quote! { #[account(del)] };
        assert!(strip_del_marker(&mut attr));
        assert_eq!(attr.tokens.to_string(), quote! { () }.to_string());
    }

    #[test]
    fn typed_field_referencing_delegation_program_is_not_a_del_field() {
        let output = expand_str(
            r#"
            pub struct Delegate<'info> {
                #[account(mut, seeds = [b"record"], bump, seeds::program = delegation_program.key())]
                pub record: Account<'info, Record>,
                /// CHECK: delegated PDA
                #[account(mut, del, seeds = [b"player"], bump)]
                pub player: UncheckedAccount<'info>,
            }
            "#,
        );
        assert!(!output.contains("compile_error"), "{output}");
        assert!(output.contains("delegate_player"));
        assert!(!output.contains("delegate_record"));
    }

    #[test]
    fn typed_del_field_is_rejected() {
        let output = expand_str(
            r#"
            pub struct Delegate<'info> {
                #[account(mut, del)]
                pub player: Account<'info, Player>,
            }
            "#,
        );
        assert!(output.contains("compile_error"), "{output}");
    }
}
