extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Extract the async function's name and body
    let fn_name = &input.sig.ident;

    // Generate a new name for the test function by prefixing "async_test_"
    let test_fn_name = format_ident!("async_test_{}", fn_name);

    // Generate the output token stream
    let output = quote! {
        // Original async function (not compiled into the test)
        #input

        // Generated synchronous test function with a new name
        #[test]
        fn #test_fn_name() {
            ::test_executors::sleep_on(#fn_name())
        }
    };

    TokenStream::from(output)
}
