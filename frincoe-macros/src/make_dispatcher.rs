use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{
    FnArg, Generics, Ident, Pat, ReturnType, Signature, Token, TraitItem, TraitItemMethod, Type, TypePath, WhereClause,
};

use crate::helpers::TraitSpec;



struct DispatcherArgs {
    pub generics: Option<Generics>,
    pub spec: TraitSpec,
    pub target: Type,
    pub request: Option<Type>,
    pub response: Option<Type>,
    pub predicates: Option<WhereClause>,
}

impl Parse for DispatcherArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // impl[<Generics>] ...
        input.parse::<Token![impl]>()?;
        let generics = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        // ["path"::mod::trait | { code }] [as mod::trait] ...
        let spec = input.parse()?;
        // for Type ...
        input.parse::<Token![for]>()?;
        let target: Type = input.parse()?;
        // [as RequestType -> ResponseType]
        let (request, response) = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let req = input.parse()?;
            input.parse::<Token![->]>()?;
            let res = input.parse()?;
            (Some(req), Some(res))
        } else {
            (None, None)
        };
        // [where predicates]
        let predicates = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            generics,
            spec,
            target,
            request,
            response,
            predicates,
        })
    }
}

pub fn make_dispatcher_impl(args: TokenStream) -> TokenStream {
    let DispatcherArgs {
        generics,
        spec,
        target,
        request,
        response,
        predicates,
    } = match syn::parse2(args) {
        Ok(val) => val,
        Err(e) => return e.into_compile_error(),
    };
    // Preprocess trait specification
    let spec = match spec.get_trait() {
        Some(content) => content,
        None => return quote! { compile_error!("not found the target trait") },
    };

    // Process the request and response type
    let mkname = |suffix| -> Type {
        let mut path = spec.name.clone();
        let item = path
            .segments
            .last_mut()
            .expect("the implementing target shouldn't be empty...");
        item.ident = Ident::new(&(item.ident.to_string().to_class_case() + suffix), item.ident.span());
        TypePath { qself: None, path }.into()
    };
    let request = request.unwrap_or_else(|| mkname("Request"));
    let response = response.unwrap_or_else(|| mkname("Response"));

    // Process the content
    let (names, types): (Vec<_>, Vec<_>) = spec
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Method(TraitItemMethod {
                attrs: _,
                sig:
                    Signature {
                        constness: _,
                        asyncness: _,
                        unsafety: _,
                        abi: _,
                        fn_token: _,
                        ident,
                        generics: _,
                        paren_token: _,
                        inputs,
                        variadic: _,
                        output,
                    },
                default: _,
                semi_token: _,
            }) => {
                let (req_types, req_args): (Vec<_>, Vec<_>) = inputs
                    .into_iter()
                    .map(|x| match x {
                        FnArg::Receiver(_) => (target.clone(), Ident::new("self", x.span())),
                        FnArg::Typed(ty) => (
                            *ty.ty.to_owned(),
                            match &*ty.pat {
                                Pat::Ident(id) => id.ident.to_owned(),
                                _ => panic!("not expected"),
                            },
                        ),
                    })
                    .unzip();
                let output = match output {
                    ReturnType::Default => quote! {},
                    ReturnType::Type(_, ty) => quote! { #ty },
                };
                Some((
                    (ident, Ident::new(&ident.to_string().to_class_case(), ident.span())),
                    ((quote! { #(#req_types),* }, quote! { #(#req_args),* }), output),
                ))
            }
            _ => None,
        })
        .unzip();
    let (methods, variants): (Vec<_>, Vec<_>) = names.into_iter().unzip();
    let (req_items, res_types): (Vec<_>, Vec<_>) = types.into_iter().unzip();
    let (req_types, req_args): (Vec<_>, Vec<_>) = req_items.into_iter().unzip();

    quote! {
        enum #generics #request {
            #(#variants(#req_types),)*
        }
        enum #generics #response {
            #(#variants(#res_types),)*
        }
        impl #generics frincoe_rpc::Dispatcher for #target #predicates {
            type Request = #request;
            type Response = #response;
            fn dispatch(&mut self, request: Self::Request) -> Self::Response {
                match request {
                    #(#variants(#req_args) => #methods(#req_args),)*
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use quote::quote;

    use super::make_dispatcher_impl;

    #[test]
    fn several() {
        assert_eq!(
            make_dispatcher_impl(quote! {
                impl {
                    trait T {
                        const T: i32;
                        fn f1(&mut self, a: i32, b: i64) -> Vec<i32>;
                        fn f2(mut self: Pin<Box<Self> >, u: i32);
                        fn f3_snake() -> Box<i32>;
                    }
                } for pathed::Struct
            })
            .to_string(),
            quote! {
                enum TRequest {
                    F1(pathed::Struct, i32, i64),
                    F2(Pin<Box<Self> >, i32),
                    F3Snake(),
                }
                enum TResponse {
                    F1(Vec<i32>),
                    F2(),
                    F3Snake(Box<i32>),
                }
                impl frincoe_rpc::Dispatcher for pathed::Struct
                {
                    type Request = TRequest;
                    type Response = TResponse;
                    fn dispatch(&mut self, request: Self::Request) -> Self::Response {
                        match request {
                            F1(self, a, b) => f1(self, a, b),
                            F2(self, u) => f2(self, u),
                            F3Snake() => f3_snake(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
