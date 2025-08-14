// SPDX-License-Identifier: MIT OR Apache-2.0

extern crate proc_macro;
use proc_macro::{TokenStream, Span};

use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn};

/**
A procedural macro that converts an async function into a test function.

On most platforms, the test function generates a stub function that uses the sleep_on runtime.

On wasm32 targets, this macro is equivalent to `#[wasm_bindgen_test::wasm_bindgen_test]`. This is because
it is generally not allowed to block the main thread in a browser environment.

# Example
```rust
use test_executors::async_test;

#[async_test]
async fn hello_world() {
    assert_eq!(1 + 1, 2);
}
```
*/
#[proc_macro_attribute]
pub fn async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Extract the async function's name
    let fn_name = &input.sig.ident;

    // Generate a new name for the test function by prefixing "async_test_"
    let test_fn_name = format_ident!("async_test_{}", fn_name);

    // Generate output for non-WASM targets (e.g., using `test_executors::sleep_on`)
    let non_wasm_output = quote! {
        // Original async function (not compiled into the test)
        #input

        // Generated synchronous test function with a new name
        #[test]
        fn #test_fn_name() {
            ::test_executors::sleep_on(#fn_name())
        }
    };

    // Figure out how wasm-bindgen-test is named in the caller.
    // In this way we can ship our version and not rely on the user to have it in their Cargo.toml.
    let wasm_crate = match crate_name("wasm_bindgen_test") {
        Ok(FoundCrate::Itself) | Err(_) => {
            // If the crate is itself wasm-bindgen-test, we can use it directly
            syn::Ident::new("wasm_bindgen_test", Span::call_site().into())
        }
        Ok(FoundCrate::Name(name)) => {
            syn::Ident::new(&name, Span::call_site().into())
        }
    };

    // Generate output for wasm32 targets (use `wasm_bindgen_test`)
    let wasm_output = quote! {
        #[#wasm_crate::wasm_bindgen_test]
        #input
    };

    // Use `cfg` attributes to conditionally compile the correct output
    let output = quote! {
        #[cfg(target_arch = "wasm32")]
        #wasm_output

        #[cfg(not(target_arch = "wasm32"))]
        #non_wasm_output
    };

    TokenStream::from(output)
}