//! crates/test-macro/src/lib.rs
//! Cargo.toml → syn = { version = "2", default-features = false, features = ["full"] }
//!              quote = "1" ; proc-macro2 = "1"

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    ItemFn, LitStr, Meta, Token,
};

use proc_macro_crate::{crate_name, FoundCrate};

/*──────────────────────── argument parsing ───────────────────────*/

enum Harness { Auto, Native, Browser(Option<LitStr>), Wasi }

struct Args {
    harness: Harness,
    forward: Vec<Meta>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metas: Punctuated<Meta, Token![,]> =
            input.parse_terminated(Meta::parse, Token![,])?;

        let mut iter = metas.iter();
        let first = iter.next();

        let harness = match first {
            None                                   => Harness::Auto,
            Some(Meta::Path(p)) if p.is_ident("native")  => Harness::Native,
            Some(Meta::Path(p)) if p.is_ident("wasi")    => Harness::Wasi,
            Some(Meta::Path(p)) if p.is_ident("browser") => Harness::Browser(None),
            Some(Meta::List(list)) if list.path.is_ident("browser") => {
                let cfg = syn::parse2::<LitStr>(list.tokens.clone()).ok();
                Harness::Browser(cfg)
            }
            _ => Harness::Auto,
        };

        let forward: Vec<Meta> = match harness {
            Harness::Auto => metas.into_iter().collect(),
            _             => iter.cloned().collect(),
        };

        Ok(Self { harness, forward })
    }
}

/*──────────────────────── the macro proper ──────────────────────*/

#[proc_macro_attribute]
pub fn cross_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let Args { harness, forward } = parse_macro_input!(attr as Args);
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);

    if sig.asyncness.is_none() {
        return syn::Error::new(sig.span(), "`cross_test` needs an async fn")
            .to_compile_error()
            .into();
    }

    /*── façade crate path ─────────────────────────────────────────*/
    let root: proc_macro2::TokenStream = match crate_name("everywhere_test") {
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
        _ => quote!(everywhere_test),
    };

    /*── attribute paths ───────────────────────────────────────────*/
    let tokio_attr     = quote!(#root::__rt::tokio::test);
    let wasm_attr      = quote!(#root::__rt::wasm_bindgen_test::wasm_bindgen_test);
    let async_std_attr = quote!(#root::__rt::async_std::test);

    /*── per-harness wrappers ─────────────────────────────────────*/
    let (cfg_wrap, harness_attr, glue_mod) = match harness {
        Harness::Native => (
            quote!(#[cfg(not(target_arch = "wasm32"))]),
            quote!(#[#tokio_attr]),
            quote!(),
        ),
        Harness::Wasi => (
            quote!(#[cfg(target_os = "wasi")]),
            quote!(#[#async_std_attr]),
            quote!(),
        ),
        Harness::Browser(cfg) => {
            let mod_name  = format_ident!("__ct_setup_{}", sig.ident);
            let cfg_token = cfg.unwrap_or_else(|| LitStr::new("run_in_browser", sig.span()));
            (
                quote!(#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]),
                quote!(#[#wasm_attr]),
                quote! {
                    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
                    mod #mod_name {
                        use super::*;
                        ::wasm_bindgen_test::wasm_bindgen_test_configure!(#cfg_token);
                    }
                },
            )
        }
        Harness::Auto => (
            quote!(),
            quote!(
                #[cfg_attr(not(target_arch = "wasm32"), #tokio_attr)]
                #[cfg_attr(all(target_arch = "wasm32", not(target_os = "wasi")), #wasm_attr)]
                #[cfg_attr(target_os = "wasi", #async_std_attr)]
            ),
            quote!(),
        ),
    };

    /*── re-emit the function ─────────────────────────────────────*/
    let extra_attrs = forward.iter().map(|m| quote!(#[#m]));

    quote! {
        #glue_mod
        #cfg_wrap
        #harness_attr
        #(#attrs)*
        #(#extra_attrs)*
        #vis #sig #block
    }
    .into()
}
