extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;
    let route_path = match &args[0] {
        syn::NestedMeta::Lit(syn::Lit::Str(lit_str)) => lit_str,
        _ => panic!("Expected a string literal for the route path"),
    };
    let handler_path = quote! { #function_name };
    let expanded = quote! {
        #function

        #[ctor::ctor]
        fn register_route() {
            use axum::{routing::get, Router};
            let route = Router::new().route(#route_path, get(#handler_path));
            crate::ROUTE_REGISTRY.lock().unwrap().push(route);
        }
    };

    TokenStream::from(expanded)
}
