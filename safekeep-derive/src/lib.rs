//! Derive macro for the `BackupType` trait.
//!
//! # Example
//! ```ignore
//! use safekeep::BackupType;
//! use serde::Serialize;
//!
//! #[derive(BackupType, Serialize)]
//! #[safekeep(format = "json")]
//! struct MyData {
//!     name: String,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, Expr, Ident, Lit, Meta, parse_macro_input};

/// Gets the path to the safekeep crate, handling both internal and external usage.
fn get_safekeep_crate() -> proc_macro2::TokenStream {
    match crate_name("safekeep") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
        Err(_) => quote! { crate },
    }
}

/// Derive macro for implementing `BackupType` trait.
///
/// Use `#[safekeep(format = "json")]` to specify the serialization format.
/// Supported formats: `json`, `yaml`, `toml`
#[proc_macro_derive(BackupType, attributes(safekeep))]
pub fn derive_backup_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let safekeep = get_safekeep_crate();

    // Extract generics for the impl block
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse #[safekeep(format = "...")] attribute
    let format = parse_safekeep_format(&input);

    let (serialize_impl, extension) = match format.as_str() {
        "json" => (
            quote! {
                ::serde_json::to_vec(self)
                    .map_err(|e| #safekeep::BackupError::Serialization(e.to_string()))
            },
            "json",
        ),
        "yaml" => (
            quote! {
                ::serde_yaml::to_string(self)
                    .map(|s| s.into_bytes())
                    .map_err(|e| #safekeep::BackupError::Serialization(e.to_string()))
            },
            "yaml",
        ),
        "toml" => (
            quote! {
                ::toml::to_string(self)
                    .map(|s| s.into_bytes())
                    .map_err(|e| #safekeep::BackupError::Serialization(e.to_string()))
            },
            "toml",
        ),
        _ => panic!(
            "Unsupported format '{}'. Use 'json', 'yaml', or 'toml'.",
            format
        ),
    };

    let expanded = quote! {
        impl #impl_generics #safekeep::BackupType for #name #ty_generics #where_clause {
            fn backup_bytes(&self) -> ::std::result::Result<::std::vec::Vec<u8>, #safekeep::BackupError> {
                #serialize_impl
            }

            fn name(&self) -> &'static str {
                stringify!(#name)
            }

            fn extension(&self) -> &'static str {
                #extension
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_safekeep_format(input: &DeriveInput) -> String {
    for attr in &input.attrs {
        if attr.path().is_ident("safekeep")
            && let Meta::List(meta_list) = &attr.meta
        {
            let tokens = meta_list.tokens.clone();
            if let Ok(meta) = syn::parse2::<Meta>(tokens)
                && let Meta::NameValue(nv) = meta
                && nv.path.is_ident("format")
                && let Expr::Lit(expr_lit) = &nv.value
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                return lit_str.value();
            }
        }
    }
    // Default to json if no format specified
    "json".to_string()
}
