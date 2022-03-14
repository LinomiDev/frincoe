use proc_macro2::TokenStream;
use quote::quote;
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Generics, Ident, Token, TraitItem, Type, WhereClause};

use crate::helpers::TraitSpec;



struct ClientArgs {
    pub generics: Option<Generics>,
    pub adapter: Ident,
    pub spec: TraitSpec,
    pub target: Type,
    pub extargs: Option<TokenStream>,
    pub predicates: Option<WhereClause>,
}

impl Parse for ClientArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
        // in adapter ...
        input.parse::<Token![in]>()?;
        let adapter: Ident = input.parse()?;
        // [(args)] ...
        let extargs = if input.peek(syn::token::Paren) {
            let argbuf;
            parenthesized!(argbuf in input);
            Some(argbuf.step(|cursor| Ok((cursor.token_stream(), Cursor::empty())))?)
        } else {
            None
        };
        // [where predicates]
        let predicates = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            generics,
            adapter,
            spec,
            target,
            extargs,
            predicates,
        })
    }
}

pub fn inject_implement_impl(args: TokenStream) -> TokenStream {
    let ClientArgs {
        generics,
        adapter,
        spec,
        target,
        extargs,
        predicates,
    } = match syn::parse2(args) {
        Ok(val) => val,
        Err(e) => return e.into_compile_error(),
    };

    let extargs = if extargs.is_some() {
        Some(quote! { #extargs ; })
    } else {
        None
    };

    // Generate the injecting content
    let spec = match spec.get_trait() {
        Some(content) => content,
        None => return quote! { compile_error!("") },
    };
    let content = spec
        .items
        .iter()
        .map(|item| match item {
            TraitItem::Macro(_) => panic!("macros in trait declaration are not currently supported"),
            value => quote! { #adapter !(#extargs #value); },
        })
        .collect::<Vec<_>>();
    let name = spec.name;

    quote! {
        impl #generics #name for #target #predicates {
            #(#content)*
        }
    }
}



#[cfg(test)]
mod tests {
    use quote::{quote, ToTokens};
    use syn::Type;

    use super::inject_implement_impl;

    #[test]
    fn parsing() {
        use super::ClientArgs;
        let ClientArgs {
            generics: _,
            adapter,
            spec: _,
            target,
            extargs,
            predicates: _,
        } = syn::parse2::<ClientArgs>(quote! { impl "fname"::path::T as U::V for A::B<R<I = J>> in func(d) }).unwrap();
        assert_eq!(adapter, "func");
        assert_eq!(extargs.unwrap().to_string(), quote! { d }.to_string());
        match target {
            Type::Path(target) => assert_eq!(
                target.to_token_stream().to_string(),
                quote! { A::B<R<I = J> > }.to_string()
            ),
            _ => panic!("expected a path"),
        }
        match syn::parse2::<ClientArgs>(quote! { impl "T"::T for T in qwq trailing }) {
            Ok(_) => panic!("Shouldn't pass compilation!"),
            Err(e) => assert_eq!(
                e.to_compile_error().to_string(),
                quote! { compile_error! { "unexpected token" } }.to_string()
            ),
        }
    }

    #[test]
    fn several() {
        macro_rules! verify {
            { $src:tt, $std:tt } => {
                assert_eq!(inject_implement_impl(quote! $src).to_string(), (quote! $std).to_string())
            }
        }
        verify! {
            {
                impl<'a, T: 'a + Orz> {
                    trait TestTrait {
                        fn f1(self, a1: i32) -> i64;
                        fn f2(&mut self);
                    }
                } as Pathed::TestTrait for Pathed::TestStruct<U, R> in empty
            },
            {
                impl<'a, T: 'a + Orz> Pathed::TestTrait for Pathed::TestStruct<U, R> {
                    empty!(fn f1(self, a1: i32) -> i64;);
                    empty!(fn f2(&mut self););
                }
            }
        }
        verify! {
            {
                impl {
                    trait T {
                        fn f();
                    }
                } as UR<Orz, P = R> for T in pr(a impl b) where T: QAQ
            },
            {
                impl UR<Orz, P = R> for T where T: QAQ {
                    pr!(a impl b; fn f(););
                }
            }
        }
    }
}
