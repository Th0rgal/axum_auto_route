extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta, FnArg, PatType, Type};

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;

    if args.len() < 2 || args.len() > 3 {
        panic!("Expected two or three arguments: HTTP method, route path, and optionally the module path");
    }

    let http_method = match &args[0] {
        NestedMeta::Meta(Meta::Path(path)) => path
            .get_ident()
            .expect("Expected an HTTP method")
            .to_string(),
        _ => panic!("Expected an HTTP method (e.g., 'get', 'post')"),
    };
    let route_path = match &args[1] {
        NestedMeta::Lit(Lit::Str(lit_str)) => lit_str,
        _ => panic!("Expected a string literal for the route path"),
    };

    let module_path = args.get(2).and_then(|arg| {
        if let NestedMeta::Meta(Meta::Path(path)) = arg {
            Some(path)
        } else {
            None
        }
    });

    let full_function_path = if let Some(module_path) = module_path {
        quote! { #module_path::#function_name }
    } else {
        quote! { #function_name }
    };

    let register_function_name = format_ident!("register_{}", function_name);

    let axum_method = match http_method.as_str() {
        "get" => quote! { get },
        "post" => quote! { post },
        "put" => quote! { put },
        "delete" => quote! { delete },
        _ => panic!("Unsupported HTTP method: {}", http_method),
    };

    let expanded = quote! {
        #function

        #[ctor::ctor]
        fn #register_function_name() {
            use axum::{Router, routing::#axum_method};
            use std::sync::Arc;

            // Create a router and add the route
            let route = Router::new().route(#route_path, #axum_method(#full_function_path));

            // Add the route to the global registry
            crate::ROUTE_REGISTRY.lock().unwrap().push(route);
        }
    };

    TokenStream::from(expanded)
}
