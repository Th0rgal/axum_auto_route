extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

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
    let requires_state = function.sig.inputs.iter().any(|arg| {
        matches!(arg, syn::FnArg::Typed(pat) if matches!(*pat.ty, syn::Type::Path(ref path) if path.path.segments.iter().any(|seg| seg.ident == "State")))
    });

    let route_registration = if requires_state {
        quote! {
            use axum::{Router, extract::State};
            // We assume the state type is AppState and it's wrapped in an Arc
            fn generated_route_function(state: State<std::sync::Arc<models::AppState>>) -> Router<std::sync::Arc<models::AppState>, _> {
                Router::with_state(state).route(#route_path, axum::routing::#axum_method(#full_function_path))
            }
            // The state should be provided when calling this function during app initialization
        }
    } else {
        quote! {
            use axum::Router;
            Router::new().route(#route_path, axum::routing::#axum_method(#full_function_path))
        }
    };

    let expanded = quote! {
        #function
        #[ctor::ctor]
        fn #register_function_name() {
            // We lock the ROUTE_REGISTRY and push our route
            let mut registry = crate::ROUTE_REGISTRY.lock().unwrap();
            let route = #route_registration;
            registry.push(route);
        }
    };

    TokenStream::from(expanded)
}
