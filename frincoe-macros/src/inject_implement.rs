use std::io::Read;

use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::quote;
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, parenthesized, Generics, Ident, Item, ItemTrait, LitStr, Path, Token, TraitItem, Type, WhereClause};



enum TraitProvider {
    File(LitStr),
    Raw(Vec<TraitItem>),
}

struct ClientArgs {
    pub generics: Option<Generics>,
    pub adapter: Ident,
    pub source: TraitProvider,
    pub srcpath: Path,
    pub modpath: Option<Path>,
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
        // ["path"::mod::trait | { code }] ...
        let lookahead = input.lookahead1();
        let (source, srcpath) = if lookahead.peek(LitStr) {
            let src = TraitProvider::File(input.parse()?);
            input.parse::<Token![::]>()?;
            (src, input.parse::<Path>()?)
        } else if lookahead.peek(Brace) {
            let content;
            braced!(content in input);
            let content: ItemTrait = content.parse()?;
            (TraitProvider::Raw(content.items), content.ident.into())
        } else {
            return Err(lookahead.error());
        };
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
            source,
            srcpath,
            modpath,
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
        source,
        srcpath,
        modpath,
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
    let content = match source {
        TraitProvider::File(filename) => match find_trait_from_file(filename, &srcpath) {
            Some(v) => v,
            None => return TokenStream::new(),
        },
        TraitProvider::Raw(content) => content,
    }
    .iter()
    .map(|item| match item {
        TraitItem::Macro(_) => panic!("macros in trait declaration are not currently supported"),
        value => quote! { #adapter !(#extargs #value); },
    })
    .collect::<Vec<_>>();
    let modpath = modpath.unwrap_or(srcpath);

    quote! {
        impl #generics #modpath for #target #predicates {
            #(#content)*
        }
    }
}

macro_rules! report_error (
    ( $var:ident, $msg:expr ) => {
        {
            Diagnostic::spanned($var.span().unwrap(), Level::Error, $msg).emit();
            return None;
        }
    }
);

fn find_trait_from_file(fname: LitStr, srcpath: &Path) -> Option<Vec<TraitItem>> {
    // Parse the file and locate the trait to be injected
    fn read_file(fname: &str) -> Result<syn::File, Box<dyn std::error::Error>> {
        let mut handle = std::fs::File::open(fname)?;
        let mut content = String::new();
        handle.read_to_string(&mut content)?;
        Ok(syn::parse_file(&content)?)
    }
    let mut content = match read_file(
        proc_macro::Span::call_site()
            .source_file()
            .path()
            .parent()
            .unwrap()
            .join(fname.value())
            .to_str()
            .unwrap(),
    ) {
        Ok(v) => v.items,
        Err(err) => report_error!(fname, format!("failed when parsing the trait source: {}", err)),
    };

    // Match the mod path...
    if srcpath.segments.is_empty() {
        report_error!(srcpath, "expected the trait name");
    }
    for curpath in srcpath.segments.iter().take(srcpath.segments.len() - 1) {
        let mut next: Option<Vec<Item>> = None;
        for item in content {
            match item {
                Item::Mod(item) if item.ident == curpath.ident => {
                    next = item.content.map(|x| x.1);
                    break;
                }
                _ => (),
            }
        }
        if let Some(next) = next {
            content = next;
        } else {
            report_error!(srcpath, "expected the trait name");
        }
    }
    // Find the trait in the mod...
    let name = &srcpath.segments.last().unwrap().ident;
    let mut next: Option<Vec<TraitItem>> = None;
    for item in content {
        match item {
            Item::Trait(item) if item.ident == *name => {
                next = Some(item.items);
                break;
            }
            _ => (),
        }
    }

    match next {
        Some(v) => Some(v),
        None => report_error!(srcpath, "not found the trait with the given path"),
    }
}



#[cfg(test)]
mod tests {
    use quote::{quote, ToTokens};
    use syn::Type;

    use super::{inject_implement_impl, TraitProvider};

    #[test]
    fn parsing() {
        use super::ClientArgs;
        let ClientArgs {
            generics: _,
            adapter,
            source,
            srcpath,
            modpath,
            target,
            extargs,
            predicates: _,
        } = syn::parse2::<ClientArgs>(quote! { impl "fname"::path::T as U::V for A::B<R<I = J>> in func(d) }).unwrap();
        assert_eq!(adapter, "func");
        assert_eq!(extargs.unwrap().to_string(), quote! { d }.to_string());
        match source {
            TraitProvider::Raw(_) => panic!("expected a filename"),
            TraitProvider::File(n) => assert_eq!(n.value(), "fname"),
        }
        assert_eq!(srcpath.to_token_stream().to_string(), quote! { path::T }.to_string());
        assert_eq!(modpath.to_token_stream().to_string(), quote! { U::V }.to_string());
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
