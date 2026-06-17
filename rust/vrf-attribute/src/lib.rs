use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn vrf(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let unchecked_account = generated_unchecked_account_type();
    let struct_name = &input.ident;
    let fields = &input.fields;
    let original_attrs = &input.attrs;
    let mut new_fields = Vec::new();
    let mut has_program_identity = false;
    let mut has_slot_hashes = false;
    let mut has_vrf_program = false;
    let mut has_system_program = false;

    for field in fields.iter() {
        let field_attrs = field.attrs.clone();

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

        let field_type = &field.ty;
        new_fields.push(quote! {
            #(#field_attrs)*
            pub #field_name: #field_type,
        });

        // Check for existing required fields
        if field_name.eq("program_identity") {
            has_program_identity = true;
        }
        if field_name.eq("vrf_program") {
            has_vrf_program = true;
        }
        if field_name.eq("slot_hashes") {
            has_slot_hashes = true;
        }
        if field_name.eq("system_program") {
            has_system_program = true;
        }
    }

    // Add missing required fields
    if !has_program_identity {
        new_fields.push(quote! {
            /// CHECK: Used to verify the identity of the program
            #[account(seeds = [b"identity"], bump)]
            pub program_identity: #unchecked_account,
        });
    }
    if !has_vrf_program {
        new_fields.push(quote! {
            pub vrf_program: Program<'info, ephemeral_rollups_sdk::anchor::VrfProgram>,
        });
    }
    if !has_slot_hashes {
        new_fields.push(quote! {
            /// CHECK: Slot hashes sysvar
            #[account(address = ephemeral_rollups_sdk::compat::slot_hashes::ID)]
            pub slot_hashes: #unchecked_account,
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
            fn invoke_signed_vrf<'a>(&self, payer: &'a AccountInfo<'info>, ix: &ephemeral_rollups_sdk::compat::Instruction) -> ephemeral_rollups_sdk::compat::anchor_lang::solana_program::entrypoint::ProgramResult {
                let bump = Pubkey::try_find_program_address(&[ephemeral_rollups_sdk::vrf::consts::IDENTITY], &crate::ID).ok_or(ephemeral_rollups_sdk::compat::anchor_lang::prelude::ProgramError::InvalidSeeds)?;
                // `#[vrf]` issues scoped randomness requests by default: the fulfillment signs
                // the callback with the per-program scoped identity PDA, which the callback
                // validates (see `#[vrf_callback]`). Map any legacy request discriminator to its
                // scoped equivalent (3/11 -> 11 high priority, else -> 10).
                let mut ix = ix.clone();
                if let Some(disc) = ix.data.first_mut() {
                    *disc = if *disc == 3 || *disc == 11 { 11 } else { 10 };
                }
                ephemeral_rollups_sdk::compat::anchor_lang::solana_program::program::invoke_signed(
                    &ix,
                    &[
                        payer.clone(),
                        self.program_identity.to_account_info(),
                        self.oracle_queue.to_account_info(),
                        self.system_program.to_account_info(),
                        self.slot_hashes.to_account_info(),
                    ],
                    &[&[ephemeral_rollups_sdk::vrf::consts::IDENTITY, &[bump.1]]],
                )
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro for a callback (consume) `#[derive(Accounts)]` struct.
///
/// Injects a `vrf_program_identity: Signer<'info>` constrained to the scoped per-program VRF
/// identity PDA (`scoped_vrf_identity(&crate::ID)`). This is the default way to authenticate
/// the VRF program in a callback; the identity is bound to this program. The legacy
/// global-identity check (`address = VRF_PROGRAM_IDENTITY`) is deprecated.
///
/// Place `#[vrf_callback]` ABOVE `#[derive(Accounts)]`.
#[proc_macro_attribute]
pub fn vrf_callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let original_attrs = &input.attrs;

    let mut new_fields = Vec::new();
    let mut has_identity = false;
    for field in input.fields.iter() {
        let field_attrs = field.attrs.clone();
        let field_name = match &field.ident {
            Some(name) => name,
            None => {
                return syn::Error::new_spanned(field, "Unnamed fields are not supported")
                    .to_compile_error()
                    .into();
            }
        };
        let field_type = &field.ty;
        new_fields.push(quote! {
            #(#field_attrs)*
            pub #field_name: #field_type,
        });
        if field_name.eq("vrf_program_identity") {
            has_identity = true;
        }
    }

    if !has_identity {
        new_fields.insert(
            0,
            quote! {
                /// Scoped VRF identity PDA, bound to this program. Its presence as a signer proves
                /// the callback was issued by the VRF program for this program.
                #[account(address = ephemeral_rollups_sdk::vrf::consts::scoped_vrf_identity(&crate::ID))]
                pub vrf_program_identity: Signer<'info>,
            },
        );
    }

    let expanded = quote! {
        #(#original_attrs)*
        pub struct #struct_name<'info> {
            #(#new_fields)*
        }
    };

    TokenStream::from(expanded)
}

fn generated_unchecked_account_type() -> proc_macro2::TokenStream {
    if cfg!(feature = "backward-compat") {
        quote! { AccountInfo<'info> }
    } else {
        quote! { UncheckedAccount<'info> }
    }
}
