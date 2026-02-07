use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Expr, Field, Ident, ItemStruct, Type};

// ==================== Constants ====================

const MARKER_EPH: &str = "eph";
const MARKER_SPONSOR: &str = "sponsor";
const ATTR_ACCOUNT: &str = "account";
const ATTR_SEEDS: &str = "seeds";
const ATTR_INIT: &str = "init";

// ==================== Parsing Helpers ====================

/// Returns the `#[account(...)]` attribute's inner tokens as a string.
fn get_account_attr(field: &Field) -> Option<String> {
    field
        .attrs
        .iter()
        .find(|a| a.path.is_ident(ATTR_ACCOUNT))
        .map(|a| a.tokens.to_string())
}

/// Extracts the bracketed content after a keyword (e.g., `seeds = [...]`).
fn extract_bracketed_after(s: &str, keyword: &str) -> Option<String> {
    let start = s.find(keyword)?;
    let after = &s[start..];
    let bracket_start = after.find('[')?;
    let content = &after[bracket_start..];

    let mut depth = 0;
    for (i, c) in content.char_indices() {
        match c {
            '[' => depth += 1,
            ']' if depth == 1 => return Some(content[..=i].to_string()),
            ']' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Extracts `seeds = [...]` expression from a field's `#[account(...)]` attribute.
fn extract_seeds(field: &Field) -> Option<TokenStream2> {
    let attr_str = get_account_attr(field)?;
    let seeds_str = extract_bracketed_after(&attr_str, ATTR_SEEDS)?;
    syn::parse_str::<Expr>(&seeds_str)
        .ok()
        .map(|e| e.to_token_stream())
}

/// Checks if a field has a specific marker in its account attribute.
fn has_marker(field: &Field, marker: &str) -> bool {
    get_account_attr(field).is_some_and(|s| s.contains(marker))
}

/// Returns true if the type is `Signer<'info>`.
fn is_signer_type(ty: &Type) -> bool {
    ty.to_token_stream().to_string().contains("Signer")
}

// ==================== Seed Transformation ====================

/// Transforms seed expressions to handle `.key()` lifetime issues.
///
/// Problem: `[b"seed", payer.key().as_ref()]` - the `Pubkey` from `key()` is
/// temporary and won't live long enough for the CPI call.
///
/// Solution: Extract `.key()` calls into let bindings that extend their lifetime.
fn transform_seeds(seeds: &TokenStream2, field_names: &[Ident]) -> (TokenStream2, TokenStream2) {
    let original = seeds.to_string();
    let mut result = original.clone();
    let mut bindings = Vec::new();

    // Add self. prefix to bare field references
    for name in field_names.iter().map(ToString::to_string) {
        for suffix in [".", ",", "]", " "] {
            let pattern = format!("{name}{suffix}");
            let replacement = format!("self.{name}{suffix}");
            result = result.replace(&pattern, &replacement);
        }
    }
    result = result.replace("self.self.", "self."); // Fix double-prefix

    // Extract .key() calls into let bindings to extend Pubkey lifetime
    for name in field_names.iter().map(ToString::to_string) {
        let key_call = format!("{name}.key()");
        if original.contains(&key_call) {
            let var = Ident::new(&format!("__{name}_key"), Span::call_site());
            let field = Ident::new(&name, Span::call_site());
            bindings.push(quote! { let #var = self.#field.key(); });
            // Replace both prefixed and unprefixed versions
            result = result.replace(&format!("self.{key_call}"), &format!("__{name}_key"));
        }
    }

    let bindings = quote! { #(#bindings)* };
    let transformed = syn::parse_str(&result).unwrap_or_else(|_| seeds.clone());
    (bindings, transformed)
}

// ==================== Code Generation ====================

/// Generates PDA signer seeds computation with bump derivation.
fn gen_pda_seeds(
    seeds: &TokenStream2,
    field_names: &[Ident],
    prefix: &str,
) -> (TokenStream2, Ident) {
    let (bindings, transformed) = transform_seeds(seeds, field_names);

    let raw = Ident::new(&format!("{prefix}_seeds_raw"), Span::call_site());
    let vec = Ident::new(&format!("{prefix}_seeds_vec"), Span::call_site());
    let bump = Ident::new(&format!("{prefix}_bump"), Span::call_site());
    let bump_arr = Ident::new(&format!("{prefix}_bump_arr"), Span::call_site());
    let with_bump = Ident::new(&format!("{prefix}_seeds_with_bump"), Span::call_site());

    let code = quote! {
        #bindings
        let #raw = #transformed;
        let #vec: Vec<&[u8]> = #raw.iter().map(|s| s.as_ref()).collect();
        let (_, #bump) = anchor_lang::prelude::Pubkey::find_program_address(&#vec, &crate::id());
        let #bump_arr: [u8; 1] = [#bump];
        let mut #with_bump = #vec.clone();
        #with_bump.push(&#bump_arr);
    };

    (code, with_bump)
}

/// Generates the `EphemeralAccount` builder call.
fn gen_builder(sponsor: &Ident, ephemeral: &Ident) -> TokenStream2 {
    quote! {
        ephemeral_rollups_sdk::ephemeral_accounts::EphemeralAccount::new(
            &self.#sponsor.to_account_info(),
            &self.#ephemeral.to_account_info(),
            &self.vault,
        )
    }
}

/// Generates code that optionally wraps builder with signer seeds.
fn gen_builder_with_seeds(
    sponsor: &Ident,
    ephemeral: &Ident,
    seeds_code: Option<(TokenStream2, Vec<Ident>)>,
) -> TokenStream2 {
    let builder = gen_builder(sponsor, ephemeral);

    match seeds_code {
        Some((setup, seed_vars)) => {
            let seeds_array = quote! { [#(#seed_vars.as_slice()),*] };
            quote! {
                #setup
                let signer_seeds = #seeds_array;
                #builder.with_signer_seeds(&signer_seeds)
            }
        }
        None => builder,
    }
}

/// Information about an ephemeral field for code generation.
struct EphFieldInfo<'a> {
    name: &'a Ident,
    seeds: Option<TokenStream2>,
}

/// Information about the sponsor for code generation.
struct SponsorInfo {
    name: Ident,
    is_signer: bool,
    seeds: Option<TokenStream2>,
}

/// Generates all four methods for an ephemeral field.
fn gen_ephemeral_methods(
    eph: &EphFieldInfo,
    sponsor: &SponsorInfo,
    field_names: &[Ident],
    span: proc_macro2::Span,
) -> Vec<TokenStream2> {
    let eph_name = eph.name;
    let spon_name = &sponsor.name;

    // Method names
    let create = Ident::new(&format!("create_ephemeral_{eph_name}"), span);
    let init_if_needed = Ident::new(&format!("init_if_needed_ephemeral_{eph_name}"), span);
    let resize = Ident::new(&format!("resize_ephemeral_{eph_name}"), span);
    let close = Ident::new(&format!("close_ephemeral_{eph_name}"), span);

    // Build signer seeds for create (may need both sponsor and ephemeral)
    let create_seeds = match (sponsor.is_signer, &eph.seeds, &sponsor.seeds) {
        // Wallet sponsor, PDA ephemeral: only ephemeral seeds
        (true, Some(s), _) => {
            let (code, var) = gen_pda_seeds(s, field_names, "eph");
            Some((code, vec![var]))
        }
        // Wallet sponsor, wallet ephemeral: no seeds
        (true, None, _) => None,
        // PDA sponsor, PDA ephemeral: both seeds
        (false, Some(eph_s), Some(spon_s)) => {
            let (spon_code, spon_var) = gen_pda_seeds(spon_s, field_names, "sponsor");
            let (eph_code, eph_var) = gen_pda_seeds(eph_s, field_names, "eph");
            Some((quote! { #spon_code #eph_code }, vec![spon_var, eph_var]))
        }
        // PDA sponsor, wallet ephemeral: only sponsor seeds
        (false, None, Some(s)) => {
            let (code, var) = gen_pda_seeds(s, field_names, "sponsor");
            Some((code, vec![var]))
        }
        _ => None,
    };

    // Build signer seeds for modify (only sponsor needs to sign)
    let modify_seeds = if sponsor.is_signer {
        None
    } else {
        sponsor.seeds.as_ref().map(|s| {
            let (code, var) = gen_pda_seeds(s, field_names, "sponsor");
            (code, vec![var])
        })
    };

    let create_builder = gen_builder_with_seeds(spon_name, eph_name, create_seeds);
    let modify_builder = gen_builder_with_seeds(spon_name, eph_name, modify_seeds);

    vec![
        quote! {
            /// Creates an ephemeral account with the specified data length.
            pub fn #create(&self, data_len: u32) -> anchor_lang::Result<()> {
                #create_builder.create(data_len)?;
                Ok(())
            }
        },
        quote! {
            /// Creates an ephemeral account only if it doesn't exist (data_len == 0).
            pub fn #init_if_needed(&self, data_len: u32) -> anchor_lang::Result<()> {
                if self.#eph_name.data_len() == 0 {
                    self.#create(data_len)?;
                }
                Ok(())
            }
        },
        quote! {
            /// Resizes an ephemeral account. Sponsor pays/receives rent difference.
            pub fn #resize(&self, new_data_len: u32) -> anchor_lang::Result<()> {
                #modify_builder.resize(new_data_len)?;
                Ok(())
            }
        },
        quote! {
            /// Closes an ephemeral account. Rent is refunded to sponsor.
            pub fn #close(&self) -> anchor_lang::Result<()> {
                #modify_builder.close()?;
                Ok(())
            }
        },
    ]
}

// ==================== Attribute Processing ====================

/// Removes custom markers from an attribute token string.
fn strip_markers(tokens: &str, markers: &[&str]) -> String {
    let mut result = tokens.to_string();
    for marker in markers {
        // Handle: ", marker" | "marker, " | "marker"
        result = result
            .replace(&format!(", {marker}"), "")
            .replace(&format!("{marker}, "), "")
            .replace(marker, "");
    }
    result
}

/// Cleans custom markers from a field's account attribute.
fn clean_field_attrs(attrs: &mut [Attribute], markers: &[&str]) {
    for attr in attrs.iter_mut() {
        if attr.path.is_ident(ATTR_ACCOUNT) {
            let original = attr.tokens.to_string();
            let cleaned = strip_markers(&original, markers);
            if cleaned != original {
                attr.tokens =
                    syn::parse_str(&cleaned).expect("internal: failed to parse cleaned attribute");
            }
        }
    }
}

// ==================== Validation ====================

/// Macro context extracted during validation.
struct MacroContext {
    field_names: Vec<Ident>,
    sponsor: Option<SponsorInfo>,
    has_vault: bool,
    has_magic_program: bool,
    has_eph: bool,
}

fn validate(input: &ItemStruct) -> Result<MacroContext, TokenStream> {
    let fields = &input.fields;
    let field_names: Vec<Ident> = fields.iter().filter_map(|f| f.ident.clone()).collect();

    let mut ctx = MacroContext {
        field_names,
        sponsor: None,
        has_vault: false,
        has_magic_program: false,
        has_eph: false,
    };

    for field in fields.iter() {
        let name = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(field, "Unnamed fields not supported").to_compile_error()
        })?;

        let attr = get_account_attr(field).unwrap_or_default();

        if attr.contains(MARKER_SPONSOR) {
            ctx.sponsor = Some(SponsorInfo {
                name: name.clone(),
                is_signer: is_signer_type(&field.ty),
                seeds: extract_seeds(field),
            });
        }

        if attr.contains(MARKER_EPH) {
            ctx.has_eph = true;
            if attr.contains(ATTR_INIT) {
                return Err(syn::Error::new_spanned(
                    field,
                    "'eph' cannot be combined with 'init'. Use create_ephemeral_*() instead.",
                )
                .to_compile_error()
                .into());
            }
        }

        if *name == "vault" {
            ctx.has_vault = true;
        }
        if *name == "magic_program" {
            ctx.has_magic_program = true;
        }
    }

    // Validation rules
    if ctx.has_eph && ctx.sponsor.is_none() {
        return Err(syn::Error::new(
            Span::call_site(),
            "Ephemeral accounts require a sponsor. Add #[account(mut, sponsor)].",
        )
        .to_compile_error()
        .into());
    }

    if let Some(ref s) = ctx.sponsor {
        if !s.is_signer && s.seeds.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Sponsor '{}' must have seeds for PDA signing.", s.name),
            )
            .to_compile_error()
            .into());
        }
    }

    Ok(ctx)
}

// ==================== Main Macro ====================

/// Generates ephemeral account helper methods for Anchor programs.
///
/// # Markers
/// - `sponsor` - The account paying rent
/// - `eph` - An ephemeral account
///
/// # Generated Methods (per `eph` field)
/// - `create_ephemeral_<field>(data_len)` - Creates the account
/// - `init_if_needed_ephemeral_<field>(data_len)` - Creates if not exists
/// - `resize_ephemeral_<field>(new_len)` - Resizes the account
/// - `close_ephemeral_<field>()` - Closes and refunds rent
///
/// # Example
/// ```ignore
/// #[ephemeral_accounts]
/// #[derive(Accounts)]
/// pub struct CreateGame<'info> {
///     #[account(mut, sponsor)]
///     pub payer: Signer<'info>,
///
///     /// CHECK: Ephemeral PDA
///     #[account(mut, eph, seeds = [b"game", payer.key().as_ref()], bump)]
///     pub game_state: AccountInfo<'info>,
/// }
/// ```
#[proc_macro_attribute]
pub fn ephemeral_accounts(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let ctx = match validate(&input) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let struct_name = &input.ident;
    let original_attrs = &input.attrs;
    let sponsor = ctx.sponsor.as_ref();

    let mut new_fields = Vec::new();
    let mut methods = Vec::new();

    for field in input.fields.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let mut attrs = field.attrs.clone();

        let is_eph = has_marker(field, MARKER_EPH);
        let is_sponsor = has_marker(field, MARKER_SPONSOR);

        // Generate methods for ephemeral fields
        if is_eph {
            if let Some(sponsor) = sponsor {
                let info = EphFieldInfo {
                    name,
                    seeds: extract_seeds(field),
                };
                methods.extend(gen_ephemeral_methods(
                    &info,
                    sponsor,
                    &ctx.field_names,
                    field.span(),
                ));
            }
        }

        // Clean markers from attributes
        if is_eph || is_sponsor {
            clean_field_attrs(&mut attrs, &[MARKER_EPH, MARKER_SPONSOR]);
        }

        new_fields.push(quote! {
            #(#attrs)*
            pub #name: #ty,
        });
    }

    // Auto-inject required fields
    if ctx.has_eph {
        if !ctx.has_vault {
            new_fields.push(quote! {
                /// CHECK: Ephemeral rent vault
                #[account(mut, address = ephemeral_rollups_sdk::consts::EPHEMERAL_VAULT_ID)]
                pub vault: AccountInfo<'info>,
            });
        }
        if !ctx.has_magic_program {
            new_fields.push(quote! {
                /// CHECK: Magic program for CPI
                #[account(address = ephemeral_rollups_sdk::consts::MAGIC_PROGRAM_ID)]
                pub magic_program: AccountInfo<'info>,
            });
        }
    }

    let expanded = quote! {
        #(#original_attrs)*
        pub struct #struct_name<'info> {
            #(#new_fields)*
        }

        impl<'info> #struct_name<'info> {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}
