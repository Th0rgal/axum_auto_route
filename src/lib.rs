extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta, Lit};

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;

    let route_path = match args.first().expect("Expected a route path argument") {
        NestedMeta::Lit(Lit::Str(lit_str)) => lit_str,
        _ => panic!("Expected a string literal for the route path"),
    };

    // Generate a unique identifier for the register function
    let register_function_name = format_ident!("register_{}", function_name);

    let expanded = quote! {
        #function

        #[ctor::ctor]
        fn #register_function_name() {
            use axum::{routing::get, Router};
            let route = Router::new().route(#route_path, get(crate::#function_name));
            crate::ROUTE_REGISTRY.lock().unwrap().push(route);
        }
    };

    TokenStream::from(expanded)
}
