use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Path, Token, TraitItem, Type};

use crate::helpers::read_trait;

struct ClientArgs {
    pub adapter: Ident,
    pub filename: LitStr,
    pub srcpath: Path,
    pub modpath: Option<Path>,
    pub target: Type,
}

impl Parse for ClientArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // impl "path"::mod::trait ...
        input.parse::<Token![impl]>()?;
        let filename: LitStr = input.parse()?;
        input.parse::<Token![::]>()?;
        let srcpath: Path = input.parse()?;
        // [as mod::trait] ...
        let lookahead = input.lookahead1();
        let modpath: Option<Path> = if lookahead.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        // for Type ...
        input.parse::<Token![for]>()?;
        let target: Type = input.parse()?;
        // in adapter
        input.parse::<Token![in]>()?;
        let adapter: Ident = input.parse()?;
        // )
        if !input.is_empty() {
            return Err(input.error("Wrong syntax for make_client: nothing should be following the struct"));
        }
        Ok(Self {
            adapter,
            filename,
            srcpath,
            modpath,
            target,
        })
    }
}

pub fn make_client_impl(args: TokenStream) -> TokenStream {
    let ClientArgs {
        adapter,
        filename,
        srcpath,
        modpath,
        target,
    } = match syn::parse2(args) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error(),
    };

    // Generate the injecting content
    let content = match read_trait(filename, &srcpath) {
        Some(v) => v,
        None => return TokenStream::new(),
    }
    .iter()
    .map(|item| match item {
        TraitItem::Macro(_) => panic!("macros in trait declaration are not currently supported"),
        value => quote! { #adapter !(#value); },
    })
    .collect::<Vec<_>>();
    let modpath = modpath.unwrap_or(srcpath);

    quote! {
        impl #modpath for #target {
            #(#content)*
        }
    }
}
