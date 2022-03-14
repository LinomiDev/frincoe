use std::io::Read;

use syn::parse::Parse;
use syn::{braced, Item, ItemTrait, LitStr, Path, Token, TraitItem};

#[allow(clippy::large_enum_variant, reason = "This is often used only once")]
pub enum TraitCode {
    File { name: LitStr, location: Path },
    Raw(ItemTrait),
}

pub struct TraitSpec {
    source: TraitCode,
    alias: Option<Path>,
}

pub struct TraitProvider {
    pub items: Vec<TraitItem>,
    pub name: Path,
}

impl Parse for TraitSpec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let source = if lookahead.peek(LitStr) {
            let name = input.parse()?;
            input.parse::<Token![::]>()?;
            TraitCode::File {
                name,
                location: input.parse()?,
            }
        } else if lookahead.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            TraitCode::Raw(content.parse()?)
        } else {
            return Err(lookahead.error());
        };
        let alias = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { source, alias })
    }
}



cfg_if::cfg_if! {
    if #[cfg(test)] {
        macro_rules! report_error {
            ( $var:ident, $msg:literal ) => {
                panic!($msg)
            };
            ( $var:ident, $msg:expr ) => {
                panic!("{}", $msg)
            }
        }
    }
    else {
        macro_rules! report_error {
            ( $var:ident, $msg:expr ) => {
                {
                    proc_macro::Diagnostic::spanned(
                        syn::spanned::Spanned::span(&$var).unwrap(),
                        proc_macro::Level::Error, $msg)
                    .emit();
                    return None;
                }
            }
        }
    }
}

impl TraitSpec {
    pub fn get_trait(self) -> Option<TraitProvider> {
        match self.source {
            TraitCode::File { name, location } => {
                let caller = proc_macro::Span::call_site().source_file().path();
                let dir = caller.parent().unwrap().join(name.value());
                let content = match read_file(dir.to_str().unwrap()) {
                    Ok(content) => content,
                    Err(err) => {
                        report_error!(name, format!("failed when parsing the trait source: {}", err));
                    }
                };
                Some(TraitProvider {
                    items: find_trait_from_file(content, &location)?,
                    name: self.alias.unwrap_or(location),
                })
            }
            TraitCode::Raw(ctnt) => Some(TraitProvider {
                items: ctnt.items,
                name: self.alias.unwrap_or_else(|| ctnt.ident.into()),
            }),
        }
    }
}

fn read_file(fname: &str) -> Result<syn::File, Box<dyn std::error::Error>> {
    let mut handle = std::fs::File::open(fname)?;
    let mut content = String::new();
    handle.read_to_string(&mut content)?;
    Ok(syn::parse_file(&content)?)
}

fn find_trait_from_file(content: syn::File, path: &Path) -> Option<Vec<TraitItem>> {
    let mut content = content.items;
    // Match the mod path...
    if path.segments.is_empty() {
        report_error!(path, "expected the trait name");
    }
    for curpath in path.segments.iter().take(path.segments.len() - 1) {
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
            report_error!(path, "expected the trait name");
        }
    }
    // Find the trait in the mod...
    let name = &path.segments.last().unwrap().ident;
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
        None => report_error!(path, "not found the trait with the given path"),
    }
}

#[cfg(test)]
mod tests {
    mod find_trait {
        use proc_macro2::TokenStream;
        use quote::{quote, ToTokens};

        use super::super::find_trait_from_file;

        macro_rules! try_find {
            ( $src:tt, $target:path, $std:tt ) => {
                let lhs = find_trait_from_file(syn::parse2(quote! $src)?, &syn::parse2(quote! {$target})?)
                    .unwrap()
                    .into_iter()
                    .map(|x| x.into_token_stream().to_string())
                    .collect::<Vec<_>>();
                let rhs = $std
                    .into_iter()
                    .map(|x: TokenStream| x.to_string())
                    .collect::<Vec<_>>();
                assert_eq!(lhs, rhs);
            };
        }

        #[test]
        fn several() -> syn::Result<()> {
            try_find! {
                {
                    trait R {
                        fn Rf();
                    }
                    trait T {
                        fn Tf(self);
                    }
                }, T, [ quote! { fn Tf(self); } ]
            };
            try_find! {
                {
                    trait T {
                        fn Tf();
                    }
                    mod Orz {
                        mod QWQ {
                            trait T {
                                fn Tf(&self);
                            }
                        }
                        mod QAQ {
                            trait T {
                                fn Tf(self);
                            }
                        }
                    }
                }, Orz::QAQ::T, [ quote! { fn Tf(self); } ]
            };
            Ok(())
        }

        fn not_exist_impl() -> syn::Result<()> {
            try_find! {
                {
                    mod Orz {
                        trait R {
                            fn Tf(self);
                        }
                    }
                }, Orz::T, []
            };
            Ok(())
        }
        #[test]
        #[should_panic]
        fn not_exist() {
            not_exist_impl().expect_err("Shouldn't be here");
        }
    }
}
