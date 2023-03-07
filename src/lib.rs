#![no_std]

extern crate alloc;

use alloc::{
    string::ToString,
    vec::Vec
};

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse, Item, ImplItem, TraitItem, Attribute, Generics, GenericParam,
    TypeParamBound, TraitBound, Signature, WherePredicate, Meta,
    punctuated::Punctuated,
    token::Plus
};

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
            unconst_sig(&mut r#fn.sig);
            return quote!(#r#fn).into();
        }
        Item::Enum(mut r#enum) => {
            unconst_attrs(&mut r#enum.attrs);
            unconst_generics(&mut r#enum.generics);
            return quote!(#r#enum).into()
        }
        Item::Impl(mut r#impl) => {
            for item in r#impl.items.iter_mut() {
                match item {
                    ImplItem::Fn(r#fn) => unconst_sig(&mut r#fn.sig),
                    _ => continue
                };
            }
            unconst_generics(&mut r#impl.generics);
            return quote!(#r#impl).into()
        }
        Item::Struct(mut r#struct) => {
            unconst_attrs(&mut r#struct.attrs);
            unconst_generics(&mut r#struct.generics);
            return quote!(#r#struct).into()
        }
        Item::Trait(mut r#trait) => {
            unconst_attrs(&mut r#trait.attrs);
            for item in r#trait.items.iter_mut() {
                match item {
                    TraitItem::Fn(r#fn) => unconst_sig(&mut r#fn.sig),
                    _ => continue
                };
            }
            unconst_generics(&mut r#trait.generics);
            unconst_bounds(&mut r#trait.supertraits);
            return quote!(#r#trait).into()
        }
        Item::Verbatim(_tt) => unimplemented!(),
        _ => panic!("Input must be one of function/enum/struct/trait/impl")
    }
}

fn unconst_attrs(attrs: &mut Vec<Attribute>) {
    let mut srtta = Vec::new();
    while let Some(mut attr) = attrs.pop() {
        match &mut attr.meta {
            Meta::Path(path) => {
                if path.get_ident().unwrap().to_string() != "const_trait" {
                    srtta.push(attr);
                }
            }
            Meta::List(list) => {
                let mut segment = list.path.segments.first_mut().unwrap();
                if segment.ident.to_string() == "derive_const" {
                    segment.ident = Ident::new("derive", segment.ident.span());
                }
                srtta.push(attr);
            }
            _ => { srtta.push(attr); }
        }
    }
    while let Some(attr) = srtta.pop() {
        attrs.push(attr);
    }
}

fn unconst_sig(signature: &mut Signature) {
    signature.constness = None;
    unconst_generics(&mut signature.generics);
}

fn unconst_generics(generics: &mut Generics) {
    for param in generics.params.iter_mut() {
        match param {
            GenericParam::Type(param) => unconst_bounds(&mut param.bounds),
            _ => continue
        }
    }
    if let Some(r#where) = generics.where_clause.as_mut() {
        for predicate in r#where.predicates.iter_mut() {
            match predicate {
                WherePredicate::Type(pred) => unconst_bounds(&mut pred.bounds),
                _ => continue
            }
        }
    }
}

fn unconst_bounds(bounds: &mut Punctuated<TypeParamBound, Plus>) {
    for bound in bounds.iter_mut() {
        match bound {
            TypeParamBound::Trait(bound) => unconst_trait_bound(bound),
            TypeParamBound::Verbatim(tt) => {
                *tt = core::mem::take(tt).into_iter().skip(2).collect();
            },
            _ => continue
        }
    }
}

fn unconst_trait_bound(bound: &mut TraitBound) {
    let mut segments = Punctuated::new();
    let mut pairs = core::mem::take(&mut bound.path.segments).into_pairs();
    if let Some(pair) = pairs.next() {
        let (segment, punct) = pair.into_tuple();
        if segment.ident.to_string() != "const" {
            segments.push_value(segment);
            if let Some(punct) = punct {
                segments.push_punct(punct);
            }
        }
    } 
    while let Some(pair) = pairs.next() {
        let (segment, punct) = pair.into_tuple();
        segments.push_value(segment);
        if let Some(punct) = punct {
            segments.push_punct(punct);
        }
    }
    bound.path.segments = segments;
}
