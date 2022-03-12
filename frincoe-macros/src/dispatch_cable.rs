use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, LitStr, ReturnType, Signature, TraitItem, TraitItemMethod};

use crate::helpers::is_self;



pub fn dispatch_cable_impl(args: TokenStream) -> TokenStream {
    // Try to parse the item as a header, report other elements as errors
    let TraitItemMethod {
        attrs: _,
        sig:
            Signature {
                constness,
                asyncness,
                unsafety,
                abi,
                fn_token: _,
                ident,
                generics,
                paren_token: _,
                inputs,
                variadic: _,
                output,
            },
        default: _,
        semi_token: _,
    } = match syn::parse2::<TraitItem>(args) {
        Ok(v) => match v {
            TraitItem::Method(v) => v,
            _ => return quote! {},
        },
        Err(e) => return e.to_compile_error(),
    };

    // Process the modifiers
    let mut modifiers = quote! {};
    if constness.is_some() {
        modifiers.extend_one(quote! { const });
    }
    if asyncness.is_some() {
        modifiers.extend_one(quote! { async });
    }
    if unsafety.is_some() {
        modifiers.extend_one(quote! { unsafe });
    }
    if let Some(abi) = abi {
        let span = abi.span();
        let name = abi.name.unwrap_or_else(|| LitStr::new("", span));
        modifiers.extend_one(quote! { extern #name });
    }
    let args = match inputs.first() {
        Some(car) if is_self(car) => inputs.iter().skip(1).map(|x| match x {
            FnArg::Receiver(_) => unreachable!(),
            FnArg::Typed(val) => val.pat.to_owned(),
        }),
        _ => {
            return quote! {
                compile_error!("Cable methods must be object method to iterate over the clients");
            }
        }
    };
    let typespec = match output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, ref ty) => quote! { -> #ty where #ty: Extend<#ty> + Default },
    };
    let body = match output {
        ReturnType::Default => quote! {
            for it in self.iter() {
                it.#ident(#(#args),*);
            }
        },
        ReturnType::Type(_, ty) => quote! {
            let mut res: #ty = Default::default();
            for it in self.iter() {
                res.extend(it.#ident(#(#args),*));
            }
            res
        },
    };

    quote! { #modifiers fn #ident #generics (#inputs) #typespec { #body } }
}



#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::dispatch_cable_impl;



    #[test]
    fn extend_applicable() {
        let mut i = 0;
        let mut provide = || {
            i += 1;
            vec![i]
        };
        // Code used in dispatch_cable_impl
        let mut res: Vec<i32> = Default::default();
        for _ in 0..5 {
            res.extend(provide());
        }
        assert_eq!(res, vec![1, 2, 3, 4, 5])
    }

    #[test]
    fn different_decls() {
        // Modifiers, complex return type, many args, and complex form of self
        assert_eq!(
            dispatch_cable_impl(
                quote! { const async unsafe extern "C" fn f(mut self: Pin<Self>, x: i32, y: i32, z: i32) -> Vec<i32>; }
            )
            .to_string(),
            quote! {
                const async unsafe extern "C" fn f(mut self: Pin<Self>, x: i32, y: i32, z: i32) -> Vec<i32>
                    where Vec<i32>: Extend<Vec<i32> > + Default {
                    let mut res: Vec<i32> = Default::default();
                    for it in self.iter() {
                        res.extend(it.f(x, y, z));
                    }
                    res
                }
            }
            .to_string()
        );
        // Simplified form of self
        assert_eq!(
            dispatch_cable_impl(quote! { fn f(&mut self) -> T; }).to_string(),
            quote! {
                fn f(&mut self) -> T where T: Extend<T> + Default {
                    let mut res: T = Default::default();
                    for it in self.iter() {
                        res.extend(it.f());
                    }
                    res
                }
            }
            .to_string(),
        );
        // Void result
        assert_eq!(
            dispatch_cable_impl(quote! { fn f(self); }).to_string(),
            quote! {
                fn f(self) {
                    for it in self.iter() {
                        it.f();
                    }
                }
            }
            .to_string(),
        );
    }

    #[test]
    fn errornous() {
        assert_eq!(
            dispatch_cable_impl(quote! { fn f(s: i32); }).to_string(),
            quote! {
                compile_error!("Cable methods must be object method to iterate over the clients");
            }
            .to_string(),
        );
        assert_eq!(
            dispatch_cable_impl(quote! { fn f(); }).to_string(),
            quote! {
                compile_error!("Cable methods must be object method to iterate over the clients");
            }
            .to_string(),
        );
    }
}
