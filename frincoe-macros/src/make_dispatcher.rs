use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{FnArg, Generics, Ident, ReturnType, Token, TraitItem, Type, TypePath, WhereClause};

use crate::helpers::{is_self, TraitSpec};



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
        let prefix = match target {
            Type::Path(ref path) => path
                .path
                .segments
                .last()
                .expect("the implementing target shouldn't be empty...")
                .ident
                .to_string(),
            _ => panic!("this type is not supported currently"),
        };
        let mut path = spec.name.clone();
        let item = path
            .segments
            .last_mut()
            .expect("the implementing trait shouldn't be empty...");
        item.ident = Ident::new(&format!("{}{}{}", prefix, item.ident, suffix), item.ident.span());
        TypePath { qself: None, path }.into()
    };
    let request = request.unwrap_or_else(|| mkname("Request"));
    let response = response.unwrap_or_else(|| mkname("Response"));

    // Process the methods and arguments
    let methods = spec.items.iter().filter_map(|item| match item {
        TraitItem::Method(func) => Some(&func.sig.ident),
        _ => None,
    });
    let variants = methods
        .clone()
        .map(|id| Ident::new(&id.to_string().to_class_case(), id.span()))
        .collect::<Vec<_>>();
    let (inputs, res_types): (Vec<_>, Vec<_>) = spec
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Method(func) => Some((
                &func.sig.inputs,
                match func.sig.output {
                    ReturnType::Default => quote! {},
                    ReturnType::Type(_, ref ty) => quote! { #ty },
                },
            )),
            _ => None,
        })
        .unzip();
    let selfs = inputs.iter().map(|args| {
        if args.first().map(is_self).unwrap_or(false) {
            quote! { self. }
        } else {
            quote! { Self:: }
        }
    });
    let (req_types, req_args): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|args| {
            let args = args
                .iter()
                .skip(args.first().map(|x| if is_self(x) { 1 } else { 0 }).unwrap_or(0));
            let (types, args): (Vec<_>, Vec<_>) = args
                .map(|item| match item {
                    FnArg::Typed(pat) => (&pat.ty, &pat.pat),
                    FnArg::Receiver(_) => unreachable!(),
                })
                .unzip();
            (quote! { #(#types),* }, quote! { #(#args),* })
        })
        .unzip();

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
                    #(#request :: #variants(#req_args) => #selfs #methods(#req_args),)*
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
                enum StructTRequest {
                    F1(i32, i64),
                    F2(i32),
                    F3Snake(),
                }
                enum StructTResponse {
                    F1(Vec<i32>),
                    F2(),
                    F3Snake(Box<i32>),
                }
                impl frincoe_rpc::Dispatcher for pathed::Struct
                {
                    type Request = StructTRequest;
                    type Response = StructTResponse;
                    fn dispatch(&mut self, request: Self::Request) -> Self::Response {
                        match request {
                            StructTRequest::F1(a, b) => self.f1(a, b),
                            StructTRequest::F2(u) => self.f2(u),
                            StructTRequest::F3Snake() => Self::f3_snake(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
