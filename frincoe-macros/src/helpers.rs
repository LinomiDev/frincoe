use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, FnArg, Generics, Ident, LitStr, Pat, PatType, ReturnType, Signature, Token};



pub fn is_self(arg: &FnArg) -> bool {
    match arg {
        FnArg::Receiver(_) => true,
        FnArg::Typed(PatType {
            attrs: _,
            pat,
            colon_token: _,
            ty: _,
        }) => {
            if let Pat::Ident(id) = &**pat {
                id.ident == "self"
            } else {
                false
            }
        }
    }
}



pub struct ExtractedSignature {
    pub modifiers: TokenStream,
    pub ident: Ident,
    pub generics: Generics,
    pub inputs: Punctuated<FnArg, Token![,]>,
    pub output: ReturnType,
}

pub fn extract_signature(attrs: Vec<Attribute>, sig: Signature) -> ExtractedSignature {
    let mut modifiers = quote! { #(#attrs)* };
    if sig.constness.is_some() {
        modifiers.extend_one(quote! { const });
    }
    if sig.asyncness.is_some() {
        modifiers.extend_one(quote! { async });
    }
    if sig.unsafety.is_some() {
        modifiers.extend_one(quote! { unsafe });
    }
    if let Some(abi) = sig.abi {
        let span = abi.span();
        let name = abi.name.unwrap_or_else(|| LitStr::new("", span));
        modifiers.extend_one(quote! { extern #name });
    }
    ExtractedSignature {
        modifiers,
        ident: sig.ident,
        generics: sig.generics,
        inputs: sig.inputs,
        output: sig.output,
    }
}
