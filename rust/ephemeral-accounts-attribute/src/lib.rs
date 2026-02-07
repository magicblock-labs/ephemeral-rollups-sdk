use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Expr, ExprArray, Field, Ident, ItemStruct, Type};

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

/// Splits parenthesized attribute content into top-level comma-separated segments.
///
/// Returns `(was_parenthesized, segments)`. Tracks bracket/paren depth so that
/// commas inside nested `[...]` or `(...)` are not treated as separators.
fn split_attr_tokens(tokens: &str) -> (bool, Vec<&str>) {
    let trimmed = tokens.trim();
    let inner = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        return (false, vec![trimmed]);
    };

    let mut segments = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                segments.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(&inner[start..]);
    (true, segments)
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
fn extract_seeds(field: &Field) -> Option<ExprArray> {
    let attr_str = get_account_attr(field)?;
    let seeds_str = extract_bracketed_after(&attr_str, ATTR_SEEDS)?;
    syn::parse_str::<ExprArray>(&seeds_str).ok()
}

/// Checks if a field has a specific marker in its account attribute (exact token match).
fn has_marker(field: &Field, marker: &str) -> bool {
    get_account_attr(field).is_some_and(|s| {
        let (_, segments) = split_attr_tokens(&s);
        segments.iter().any(|seg| seg.trim() == marker)
    })
}

/// Checks if any segment starts with the given prefix (for `init` / `init_if_needed`).
fn has_marker_prefix(field: &Field, prefix: &str) -> bool {
    get_account_attr(field).is_some_and(|s| {
        let (_, segments) = split_attr_tokens(&s);
        segments.iter().any(|seg| seg.trim().starts_with(prefix))
    })
}

/// Returns true if the type is `Signer<'info>` (AST-based, matches last path segment).
fn is_signer_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Signer")
    } else {
        false
    }
}

// ==================== Seed Transformation ====================

/// Transforms seed expressions to handle `.key()` lifetime issues (AST-based).
///
/// Problem: `[b"seed", payer.key().as_ref()]` - the `Pubkey` from `key()` is
/// temporary and won't live long enough for the CPI call.
///
/// Solution: Extract `.key()` calls into let bindings that extend their lifetime.
/// Also rewrites bare `field_name` paths to `self.field_name`.
fn transform_seeds(
    seeds: &ExprArray,
    field_names: &[Ident],
) -> Result<(TokenStream2, ExprArray), syn::Error> {
    let mut bindings = Vec::new();
    let mut new_elems = syn::punctuated::Punctuated::new();

    for elem in &seeds.elems {
        let transformed = transform_seed_expr(elem, field_names, &mut bindings)?;
        new_elems.push(transformed);
    }

    let mut result = seeds.clone();
    result.elems = new_elems;

    let bindings = quote! { #(#bindings)* };
    Ok((bindings, result))
}

/// Recursively transforms a single seed expression.
fn transform_seed_expr(
    expr: &Expr,
    field_names: &[Ident],
    bindings: &mut Vec<TokenStream2>,
) -> Result<Expr, syn::Error> {
    match expr {
        // Handle method calls like `field.key().as_ref()`
        Expr::MethodCall(mc) => {
            let method_name = mc.method.to_string();

            // Check for `field.key().as_ref()` pattern
            if method_name == "as_ref" {
                if let Expr::MethodCall(inner) = mc.receiver.as_ref() {
                    if inner.method == "key" {
                        if let Some(field_ident) = extract_bare_field(&inner.receiver, field_names)
                        {
                            let var =
                                Ident::new(&format!("__{field_ident}_key"), Span::call_site());
                            bindings.push(quote! { let #var = self.#field_ident.key(); });
                            return Ok(syn::parse_quote!(#var.as_ref()));
                        }
                    }
                }
            }

            // Generic method call: transform receiver
            let new_receiver = transform_seed_expr(&mc.receiver, field_names, bindings)?;
            let mut new_mc = mc.clone();
            new_mc.receiver = Box::new(new_receiver);
            Ok(Expr::MethodCall(new_mc))
        }

        // Handle bare field references like `field` → `self.field`
        Expr::Path(ep) => {
            if let Some(field_ident) = extract_bare_field(expr, field_names) {
                Ok(syn::parse_quote!(self.#field_ident))
            } else {
                Ok(Expr::Path(ep.clone()))
            }
        }

        // Handle field access like `field.something`
        Expr::Field(ef) => {
            let new_base = transform_seed_expr(&ef.base, field_names, bindings)?;
            let mut new_ef = ef.clone();
            new_ef.base = Box::new(new_base);
            Ok(Expr::Field(new_ef))
        }

        // Handle references like `&expr`
        Expr::Reference(er) => {
            let new_expr = transform_seed_expr(&er.expr, field_names, bindings)?;
            let mut new_er = er.clone();
            new_er.expr = Box::new(new_expr);
            Ok(Expr::Reference(new_er))
        }

        // Everything else (literals like b"seed") passes through unchanged
        _ => Ok(expr.clone()),
    }
}

/// Extracts a bare field identifier if the expression is a simple path matching a known field.
fn extract_bare_field<'a>(expr: &Expr, field_names: &'a [Ident]) -> Option<&'a Ident> {
    if let Expr::Path(ep) = expr {
        if ep.qself.is_none() && ep.path.segments.len() == 1 {
            let ident = &ep.path.segments[0].ident;
            return field_names.iter().find(|f| *f == ident);
        }
    }
    None
}

// ==================== Code Generation ====================

/// Generates PDA signer seeds computation with bump derivation.
///
/// Assumes `crate::id()` as the program owner for `find_program_address`.
/// Emits fixed-size arrays `[&[u8]; N]` and `[&[u8]; N+1]` instead of `Vec`.
fn gen_pda_seeds(
    seeds: &ExprArray,
    field_names: &[Ident],
    prefix: &str,
) -> Result<(TokenStream2, Ident), syn::Error> {
    let (bindings, transformed) = transform_seeds(seeds, field_names)?;
    let n = transformed.elems.len();

    let raw = Ident::new(&format!("{prefix}_seeds_raw"), Span::call_site());
    let seeds_arr = Ident::new(&format!("{prefix}_seeds_arr"), Span::call_site());
    let bump = Ident::new(&format!("{prefix}_bump"), Span::call_site());
    let bump_arr = Ident::new(&format!("{prefix}_bump_arr"), Span::call_site());
    let with_bump = Ident::new(&format!("{prefix}_seeds_with_bump"), Span::call_site());

    // Generate index literals for array copy: 0, 1, 2, ...
    let indices = (0..n).map(syn::Index::from);

    let code = quote! {
        #bindings
        let #raw = #transformed;
        let #seeds_arr: [&[u8]; #n] = [#(#raw[#indices].as_ref()),*];
        let (_, #bump) = anchor_lang::prelude::Pubkey::find_program_address(&#seeds_arr, &crate::id());
        let #bump_arr: [u8; 1] = [#bump];
        let #with_bump: [&[u8]; #n + 1] = {
            let mut arr: [&[u8]; #n + 1] = [&[0u8; 0]; #n + 1];
            let mut i = 0usize;
            while i < #n {
                arr[i] = #seeds_arr[i];
                i += 1;
            }
            arr[#n] = &#bump_arr;
            arr
        };
    };

    Ok((code, with_bump))
}

/// Generates the `EphemeralAccount` builder call.
fn gen_builder(sponsor: &Ident, ephemeral: &Ident) -> TokenStream2 {
    quote! {
        ephemeral_rollups_sdk::ephemeral_accounts::EphemeralAccount::new(
            &self.#sponsor.to_account_info(),
            &self.#ephemeral.to_account_info(),
            &self.vault.to_account_info(),
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
    seeds: Option<ExprArray>,
}

/// Information about the sponsor for code generation.
struct SponsorInfo {
    name: Ident,
    is_signer: bool,
    seeds: Option<ExprArray>,
}

/// Generates all four methods for an ephemeral field.
fn gen_ephemeral_methods(
    eph: &EphFieldInfo,
    sponsor: &SponsorInfo,
    field_names: &[Ident],
    span: proc_macro2::Span,
) -> Result<Vec<TokenStream2>, syn::Error> {
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
            let (code, var) = gen_pda_seeds(s, field_names, "eph")?;
            Some((code, vec![var]))
        }
        // Wallet sponsor, wallet ephemeral: no seeds
        (true, None, _) => None,
        // PDA sponsor, PDA ephemeral: both seeds
        (false, Some(eph_s), Some(spon_s)) => {
            let (spon_code, spon_var) = gen_pda_seeds(spon_s, field_names, "sponsor")?;
            let (eph_code, eph_var) = gen_pda_seeds(eph_s, field_names, "eph")?;
            Some((quote! { #spon_code #eph_code }, vec![spon_var, eph_var]))
        }
        // PDA sponsor, wallet ephemeral: only sponsor seeds
        (false, None, Some(s)) => {
            let (code, var) = gen_pda_seeds(s, field_names, "sponsor")?;
            Some((code, vec![var]))
        }
        // PDA sponsor without seeds is rejected during validation
        (false, Some(_), None) | (false, None, None) => {
            unreachable!("PDA sponsor without seeds is rejected during validation")
        }
    };

    // Build signer seeds for modify (only sponsor needs to sign)
    let modify_seeds = if sponsor.is_signer {
        None
    } else {
        sponsor
            .seeds
            .as_ref()
            .map(|s| gen_pda_seeds(s, field_names, "sponsor"))
            .transpose()?
            .map(|(code, var)| (code, vec![var]))
    };

    let create_builder = gen_builder_with_seeds(spon_name, eph_name, create_seeds);
    let modify_builder = gen_builder_with_seeds(spon_name, eph_name, modify_seeds);

    Ok(vec![
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
    ])
}

// ==================== Attribute Processing ====================

/// Removes custom markers from an attribute token string (boundary-aware).
fn strip_markers(tokens: &str, markers: &[&str]) -> String {
    let (was_paren, segments) = split_attr_tokens(tokens);
    let filtered: Vec<&str> = segments
        .into_iter()
        .filter(|seg| !markers.contains(&seg.trim()))
        .collect();
    let joined = filtered.join(",");
    if was_paren {
        format!("({joined})")
    } else {
        joined
    }
}

/// Cleans custom markers from a field's account attribute.
///
/// Returns `Err(compile_error)` if the cleaned attribute fails to parse.
fn clean_field_attrs(attrs: &mut [Attribute], markers: &[&str]) -> Result<(), TokenStream2> {
    for attr in attrs.iter_mut() {
        if attr.path.is_ident(ATTR_ACCOUNT) {
            let original = attr.tokens.to_string();
            let cleaned = strip_markers(&original, markers);
            if cleaned != original {
                attr.tokens = syn::parse_str(&cleaned).map_err(|_| {
                    syn::Error::new(
                        attr.span(),
                        format!("failed to parse cleaned attribute: {cleaned}"),
                    )
                    .to_compile_error()
                })?;
            }
        }
    }
    Ok(())
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

        if has_marker(field, MARKER_SPONSOR) {
            ctx.sponsor = Some(SponsorInfo {
                name: name.clone(),
                is_signer: is_signer_type(&field.ty),
                seeds: extract_seeds(field),
            });
        }

        if has_marker(field, MARKER_EPH) {
            ctx.has_eph = true;
            if has_marker(field, ATTR_INIT) || has_marker_prefix(field, ATTR_INIT) {
                return Err(syn::Error::new_spanned(
                    field,
                    "'eph' cannot be combined with 'init' or 'init_if_needed'. Use create_ephemeral_*() instead.",
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
                match gen_ephemeral_methods(&info, sponsor, &ctx.field_names, field.span()) {
                    Ok(m) => methods.extend(m),
                    Err(e) => return TokenStream::from(e.to_compile_error()),
                }
            }
        }

        // Clean markers from attributes
        if is_eph || is_sponsor {
            if let Err(err) = clean_field_attrs(&mut attrs, &[MARKER_EPH, MARKER_SPONSOR]) {
                return TokenStream::from(err);
            }
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
                pub magic_program: Program<'info, ephemeral_rollups_sdk::anchor::MagicProgram>,
            });
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let generics = &input.generics;

    let expanded = quote! {
        #(#original_attrs)*
        pub struct #struct_name #generics #where_clause {
            #(#new_fields)*
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}
