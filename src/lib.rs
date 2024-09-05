extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemFn, Lit, Token};

struct RouteArgs {
    method: syn::Ident,
    path: Lit,
    middleware: Vec<syn::Path>,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse()?;
        input.parse::<Token![,]>()?;
        let path = input.parse()?;
        let mut middleware = Vec::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            middleware.push(input.parse()?);
        }
        Ok(RouteArgs {
            method,
            path,
            middleware,
        })
    }
}

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as RouteArgs);
    let function = parse_macro_input!(input as ItemFn);
    let function_name = &function.sig.ident;

    let http_method = args.method.to_string();
    let route_path = &args.path;

    let middleware_functions: Vec<_> = args
        .middleware
        .iter()
        .map(|path| quote! { #path })
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
