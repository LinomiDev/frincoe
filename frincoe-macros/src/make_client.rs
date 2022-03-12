use proc_macro2::TokenStream;
use quote::quote;
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Ident, LitStr, Path, Token, TraitItem, Type};



struct ClientArgs {
    pub adapter: Ident,
    pub filename: LitStr,
    pub srcpath: Path,
    pub modpath: Option<Path>,
    pub target: Type,
    pub extargs: Option<TokenStream>,
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
        // [(args)]
        let extargs = if !input.is_empty() {
            let argbuf;
            parenthesized!(argbuf in input);
            Some(argbuf.step(|cursor| Ok((cursor.token_stream(), Cursor::empty())))?)
        } else {
            None
        };
        Ok(Self {
            adapter,
            filename,
            srcpath,
            modpath,
            target,
            extargs,
        })
    }
}

pub fn make_client_impl(
    args: TokenStream,
    read_trait: impl FnOnce(LitStr, &Path) -> Option<Vec<TraitItem>>,
) -> TokenStream {
    let ClientArgs {
        adapter,
        filename,
        srcpath,
        modpath,
        target,
        extargs,
    } = match syn::parse2(args) {
        Ok(v) => v,
        Err(err) => return err.into_compile_error(),
    };

    let extargs = if extargs.is_some() {
        Some(quote! { #extargs ; })
    } else {
        None
    };

    // Generate the injecting content
    let content = match read_trait(filename, &srcpath) {
        Some(v) => v,
        None => return TokenStream::new(),
    }
    .iter()
    .map(|item| match item {
        TraitItem::Macro(_) => panic!("macros in trait declaration are not currently supported"),
        value => quote! { #adapter !(#extargs #value); },
    })
    .collect::<Vec<_>>();
    let modpath = modpath.unwrap_or(srcpath);

    quote! {
        impl #modpath for #target {
            #(#content)*
        }
    }
}



#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{ItemTrait, LitStr, Path, TraitItem};

    use crate::make_client::make_client_impl;

    fn verify_input(
        src: TokenStream,
        name: String,
        path: TokenStream,
    ) -> impl FnOnce(LitStr, &Path) -> Option<Vec<TraitItem>> {
        let result: ItemTrait = syn::parse2(src).expect("Should be a vaild trait");
        let result = result.items;
        let path: Path = syn::parse2(path).expect("Should be a vaild path");
        move |inname, inpath| {
            assert_eq!(inname.value(), name);
            assert_eq!(
                inpath.segments.iter().map(|x| x.ident.to_string()).collect::<Vec<_>>(),
                path.segments.iter().map(|x| x.ident.to_string()).collect::<Vec<_>>()
            );
            Some(result)
        }
    }

    #[test]
    fn invoked_actually() {
        let func = verify_input(
            quote! {
                trait TestTrait {
                    fn f1(self, a1: i32) -> i64;
                    fn f2(&mut self);
                }
            },
            "emptied".to_string(),
            quote! {Pathed::TestTrait},
        );
        assert_eq!(
            make_client_impl(
                quote! { impl "emptied"::Pathed::TestTrait for Pathed::TestStruct<U, R> in empty },
                func
            )
            .to_string(),
            quote! {
                impl Pathed::TestTrait for Pathed::TestStruct<U, R> {
                    empty!(fn f1(self, a1: i32) -> i64;);
                    empty!(fn f2(&mut self););
                }
            }
            .to_string()
        );
    }

    #[test]
    fn extra_args() {
        let func = verify_input(
            quote! {
                trait T {
                    fn f();
                }
            },
            "T".to_string(),
            quote! {T},
        );
        assert_eq!(
            make_client_impl(quote! { impl "T"::T for T in pr(a impl b) }, func).to_string(),
            quote! {
                impl T for T {
                    pr!(a impl b; fn f(););
                }
            }
            .to_string()
        );
    }

    #[test]
    fn trailing() {
        use super::ClientArgs;
        match syn::parse2::<ClientArgs>(quote! { impl "T"::T for T in qwq trailing }) {
            Ok(_) => panic!("Shouldn't pass compilation!"),
            Err(e) => assert_eq!(
                e.to_compile_error().to_string(),
                quote! { compile_error! { "expected parentheses" } }.to_string()
            ),
        }
    }
}
