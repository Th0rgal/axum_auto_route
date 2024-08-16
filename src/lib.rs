extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;

    if args.len() < 2 {
        panic!(
            "Expected at least two arguments: HTTP method, route path, and optionally a middleware function"
        );
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

    let middleware_functions: Vec<_> = args
        .iter()
        .skip(2)
        .map(|arg| match arg {
            NestedMeta::Meta(Meta::Path(path)) => quote! { #path },
            _ => panic!("Expected a middleware function path"),
        })
        .rev()
        .collect();

    let full_function_path = quote! { #function_name };

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
            use axum::{Router, middleware::from_fn};

            let mut route = Router::new().route(
                #route_path,
                axum::routing::#axum_method(#full_function_path)
            );

            // Apply each middleware function in sequence
            #(route = route.layer(from_fn(#middleware_functions));)*

            crate::ROUTE_REGISTRY.lock().unwrap().push(Box::new(route));
        }
    };

    TokenStream::from(expanded)
}
