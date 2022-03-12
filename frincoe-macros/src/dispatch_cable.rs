use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ReturnType, TraitItem, TraitItemMethod};

use crate::helpers::{extract_signature, is_self, ExtractedSignature};



pub fn dispatch_cable_impl(args: TokenStream) -> TokenStream {
    // Try to parse the item as a header, report other elements as errors
    let TraitItemMethod {
        attrs,
        sig,
        default: _,
        semi_token: _,
    } = match syn::parse2::<TraitItem>(args) {
        Ok(v) => match v {
            TraitItem::Method(v) => v,
            _ => return quote! {},
        },
        Err(e) => return e.to_compile_error(),
    };

    // Process the modifiers and extract the signature
    let ExtractedSignature {
        modifiers,
        ident,
        generics,
        inputs,
        output,
    } = extract_signature(attrs, sig);

    // Process the arguments, extract to names
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

    // Process the return type and function body
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
