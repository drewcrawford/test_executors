// SPDX-License-Identifier: MIT OR Apache-2.0

extern crate proc_macro;
use proc_macro::{Span, TokenStream};

use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/**
A procedural macro that converts an async function into a test function.

On most platforms, the test function generates a stub function that uses the sleep_on runtime.

On wasm32 targets it generates a `#[wasm_lite::wasm_lite_test]` entry point that hands the future to
`wasm_lite_std::async_doctest!`, which spawns it on the event loop and reports the verdict when it
settles. The sleep_on path is not usable there: blocking the browser main thread is forbidden
(`Atomics.wait` traps), which is why this is a different shape rather than the same one.

# Example
```rust
use test_executors::async_test;

#[async_test]
async fn hello_world() {
    assert_eq!(1 + 1, 2);
}
```
*/
/// A path to `crate_name` that resolves from the caller's crate.
///
/// Lets a consumer rename the dependency without the generated code breaking.
///
/// `FoundCrate::Itself` must not be folded in with the error case: it means the
/// caller *is* the crate being asked about, and an absolute `::wasm_lite_std`
/// path does not resolve inside `wasm_lite_std` itself. That collapse made
/// `#[async_test]` unusable in the crate's own tests.
fn resolve(crate_name_str: &str) -> syn::Path {
    match crate_name(crate_name_str) {
        Ok(FoundCrate::Itself) => syn::parse_quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, Span::call_site().into());
            syn::parse_quote!(::#ident)
        }
        Err(_) => {
            let ident = syn::Ident::new(crate_name_str, Span::call_site().into());
            syn::parse_quote!(::#ident)
        }
    }
}

#[proc_macro_attribute]
pub fn async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Extract the async function's name
    let fn_name = &input.sig.ident;

    // Generate a new name for the test function by prefixing "async_test_"
    let test_fn_name = format_ident!("async_test_{}", fn_name);

    // Figure out how the wasm_lite crates are named in the caller, so a rename
    // (or a re-export under another name) still resolves.
    let wl = resolve("wasm_lite");
    let wls = resolve("wasm_lite_std");

    // One `#[cfg]` per *item*. A `#[cfg]` in front of an interpolated block only
    // guards that block's **first** item, which is why the async fn is emitted
    // once here rather than inside each arm — previously the non-wasm entry
    // point was generated unconditionally and only looked gated.
    //
    // `#[cfg]` also has to come before the test attribute so the item is
    // stripped before that macro runs; otherwise the wasm arm's
    // `wasm_lite::wasm_lite_test` is resolved on native, where it does not
    // exist.
    let output = quote! {
        #input

        #[cfg(not(target_arch = "wasm32"))]
        #[test]
        fn #test_fn_name() {
            ::test_executors::sleep_on(#fn_name())
        }

        // `#[wasm_lite_test]` rejects an `async fn` on purpose — the future
        // would be built and dropped unpolled, so the test could never fail.
        // A sync entry point drives it instead, mirroring the arm above.
        // `async_doctest!` marks the test pending, spawns the future on the
        // event loop and passes when it settles; it does not block, which the
        // browser main thread would not permit.
        #[cfg(target_arch = "wasm32")]
        #[#wl::wasm_lite_test(crate = #wl)]
        fn #test_fn_name() {
            #wls::async_doctest!(#fn_name());
        }
    };

    TokenStream::from(output)
}
