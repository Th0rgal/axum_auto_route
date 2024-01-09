extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;

    if args.len() != 2 {
        panic!("Expected two arguments: HTTP method and route path");
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

    // Generate a unique identifier for the register function
    let register_function_name = format_ident!("register_{}", function_name);

    // Determine the Axum function to use based on the HTTP method
    let axum_method = match http_method.as_str() {
        "get" => quote! { get },
        "post" => quote! { post },
        "put" => quote! { put },
        "delete" => quote! { delete },
        // Add other HTTP methods here if needed
        _ => panic!("Unsupported HTTP method: {}", http_method),
    };

    let expanded = quote! {
        #function

        #[ctor::ctor]
        fn #register_function_name() {
            use axum::Router;
            let route = Router::new().route(#route_path, axum::routing::#axum_method(crate::#function_name));
            crate::ROUTE_REGISTRY.lock().unwrap().push(route);
        }
    };

    TokenStream::from(expanded)
}
