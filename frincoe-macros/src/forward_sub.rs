use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{Attribute, FnArg, Ident, Signature, Token, TraitItem, TraitItemConst, TraitItemMethod, TraitItemType, Type};

use crate::helpers::{extract_signature, is_self, ExtractedSignature};



struct ForwardSubArgs {
    pub member: Ident,
    pub typename: Option<Type>,
    pub fwd_type: bool,
    pub fwd_const: bool,
    pub item: TraitItem,
}

impl Parse for ForwardSubArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // member [: Type] [type] [const]
        let member = input.parse()?;
        let typename = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        let mut fwd_type = false;
        let mut fwd_const = false;
        while !input.peek(Token![;]) {
            if input.peek(Token![type]) {
                input.parse::<Token![type]>()?;
                fwd_type = true;
            } else if input.peek(Token![const]) {
                input.parse::<Token![const]>()?;
                fwd_const = true;
            } else {
                return Err(input.error("expect `type`, `const` or `;`"));
            }
        }
        // ;
        input.parse::<Token![;]>()?;
        Ok(Self {
            member,
            typename,
            fwd_type,
            fwd_const,
            item: input.parse()?,
        })
    }
}

pub fn forward_sub_impl(args: TokenStream) -> TokenStream {
    let ForwardSubArgs {
        member,
        typename,
        fwd_type,
        fwd_const,
        item,
    } = match syn::parse2(args) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };
    if typename.is_none() && (fwd_type || fwd_const) {
        return quote! { compile_error!("Can't forward types and constants without a type name"); };
    }

    match item {
        TraitItem::Const(TraitItemConst {
            attrs,
            const_token: _,
            ident,
            colon_token: _,
            ty,
            default: _,
            semi_token: _,
        }) => {
            if fwd_const {
                quote! { #(#attrs)* const #ident : #ty = #typename :: #ident ; }
            } else {
                quote! {}
            }
        }
        TraitItem::Type(TraitItemType {
            attrs,
            type_token: _,
            ident,
            generics,
            colon_token: _,
            bounds: _,
            default: _,
            semi_token: _,
        }) => {
            if fwd_type {
                quote! { #(#attrs)* type #ident #generics = #typename :: #ident #generics ; }
            } else {
                quote! {}
            }
        }
        TraitItem::Method(TraitItemMethod {
            attrs,
            sig,
            default: _,
            semi_token: _,
        }) => proc_method(attrs, sig, member, typename),
        _ => quote! { compile_error!("not supported currently"); },
    }
}

fn proc_method(attrs: Vec<Attribute>, sig: Signature, member: Ident, typename: Option<Type>) -> TokenStream {
    let ExtractedSignature {
        modifiers,
        ident,
        generics,
        inputs,
        output,
    } = extract_signature(attrs, sig);

    let trans = |x: &FnArg| match x {
        FnArg::Receiver(_) => unreachable!(),
        FnArg::Typed(val) => val.pat.to_owned(),
    };
    let (prefix, args) = if inputs.first().map(|x| is_self(x)).unwrap_or(false) {
        (quote! { self.#member . }, inputs.iter().skip(1).map(trans))
    } else if typename.is_some() {
        (quote! { #typename :: }, inputs.iter().skip(0).map(trans))
    } else {
        return quote! {
            compile_error!("Don't know how to call the method since neither self nor the typename is given");
        };
    };

    quote! {
        #modifiers fn #ident #generics(#inputs) #output {
            #prefix #ident (#(#args),*)
        }
    }
}



#[cfg(test)]
mod tests {
    use quote::quote;

    use super::{forward_sub_impl, ForwardSubArgs};

    #[test]
    fn parsing() {
        syn::parse2::<ForwardSubArgs>(quote! { name; fn f(); }).expect("should be ok ><");
        syn::parse2::<ForwardSubArgs>(quote! { name; type T; }).expect("should be ok ><");
        syn::parse2::<ForwardSubArgs>(quote! { a: T type; const x: u8; }).expect("should be ok ><");
        let x = syn::parse2::<ForwardSubArgs>(quote! { name: Vec<i32>; fn f(); }).expect("should be ok ><");
        assert!(!x.fwd_type && !x.fwd_const);
        let x = syn::parse2::<ForwardSubArgs>(quote! { name: i32 type const; fn f(); }).expect("should be ok ><");
        assert!(x.fwd_type && x.fwd_const);
        assert!(syn::parse2::<ForwardSubArgs>(quote! { name: fn f(); }).is_err());
        assert!(syn::parse2::<ForwardSubArgs>(quote! { name; }).is_err());
    }

    #[test]
    fn various() {
        macro_rules! verify {
            { $src:tt, $std:tt } => {
                assert_eq!(
                    forward_sub_impl(quote! $src).to_string(),
                    (quote! $std).to_string()
                )
            };
        }
        // Types and constants
        verify! {{ a: T type; const x: u8; }, {}};
        verify! {{ a: T const; const x: u8; }, { const x: u8 = T::x; }};
        verify! {{ a: T const; type U<R>: Add = u8; }, {}};
        verify! {{ a: T type; type U<R>: Add = u8; }, { type U<R> = T::U<R>; }};
        verify! {{ a type; const x: u8; }, {
            compile_error!("Can't forward types and constants without a type name");
        }};
        // Functions
        verify! {{ x; fn f(); }, {
            compile_error!("Don't know how to call the method since neither self nor the typename is given");
        }};
        verify! {{ x; fn f(&mut self, a: i32, b: Vec<i32>) -> Ret; }, {
            fn f(&mut self, a: i32, b: Vec<i32>) -> Ret {
                self.x.f(a, b)
            }
        }};
        verify! {{ x: T; fn f(a: R); }, { fn f(a: R) { T::f(a) } }};
    }
}
