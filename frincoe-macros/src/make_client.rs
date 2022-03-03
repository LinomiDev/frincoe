use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Path, Token, TraitItem, Type};



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
    fn verify_some() {
        let func = verify_input(
            quote! {
                trait TestTrait {
                    fn f1(self, a1: i32) -> i64;
                    fn f2(&mut self);
                }
            },
            "empty_trait".to_string(),
            quote! {Pathed::TestTrait},
        );
        assert_eq!(
            make_client_impl(
                quote! { impl "empty_trait"::Pathed::TestTrait for Pathed::TestStruct<U, R> in empty },
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
}
