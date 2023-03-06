#![no_std]

extern crate alloc;

use alloc::string::ToString;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse, Item, ImplItem, TraitItem};

#[cfg(feature = "const")]
#[proc_macro_attribute]
pub fn unconst(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(not(feature = "const"))]
#[proc_macro_attribute]
pub fn unconst(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse::<Item>(item).unwrap() {
        Item::Fn(mut r#fn) => {
            r#fn.sig.constness = None;
            return quote!(#r#fn).into();
        }
        Item::Enum(mut r#enum) => {
            for attr in r#enum.attrs.iter_mut() {
                let mut segment = attr.path.segments.first_mut().unwrap();
                if segment.ident.to_string() == "derive_const" {
                    segment.ident = Ident::new("derive", segment.ident.span());
                }
            }
            return quote!(#r#enum).into()
        }
        Item::Impl(mut r#impl) => {
            for item in r#impl.items.iter_mut() {
                match item {
                    ImplItem::Method(method) => method.sig.constness = None,
                    _ => continue
                };
            }
            return quote!(#r#impl).into()
        }
        Item::Struct(mut r#struct) => {
            for attr in r#struct.attrs.iter_mut() {
                let mut segment = attr.path.segments.first_mut().unwrap();
                if segment.ident.to_string() == "derive_const" {
                    segment.ident = Ident::new("derive", segment.ident.span());
                }
            }
            return quote!(#r#struct).into()
        }
        Item::Trait(mut r#trait) => {
            for item in r#trait.items.iter_mut() {
                match item {
                    TraitItem::Method(method) => method.sig.constness = None,
                    _ => continue
                };
            }
            return quote!(#r#trait).into()
        }
        Item::Verbatim(_tt) => unimplemented!(),
        _ => panic!("Input is neither a function, a trait nor an impl")
    }
}
