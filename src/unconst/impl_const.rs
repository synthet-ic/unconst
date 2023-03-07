extern crate alloc;

use alloc::{
    boxed::Box,
    vec::Vec
};

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Attribute, Generics, Path, Type, ImplItem, Token, Lifetime, Ident,
    TypePath,
    parse::{Parse, ParseStream, Result},
    token, braced, bracketed, AttrStyle,
};
pub struct ItemImplConst {
    pub attrs: Vec<Attribute>,
    pub defaultness: Option<Token![default]>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub constness: Option<Token![const]>,
    pub trait_: Option<(Option<Token![!]>, Path, Token![for])>,
    pub self_ty: Box<Type>,
    pub brace_token: token::Brace,
    pub items: Vec<ImplItem>,
}

impl Parse for ItemImplConst {
    fn parse(input: ParseStream) -> Result<Self> {
        parse_impl(input).map(Option::unwrap)
    }
}

fn parse_impl(input: ParseStream) -> Result<Option<ItemImplConst>> {
    let mut attrs = input.call(Attribute::parse_outer)?;
    // let has_visibility = input.parse::<Visibility>()?.is_some();
    let defaultness: Option<Token![default]> = input.parse()?;
    let unsafety: Option<Token![unsafe]> = input.parse()?;
    let impl_token: Token![impl] = input.parse()?;

    let has_generics = input.peek(Token![<])
        && (input.peek2(Token![>])
            || input.peek2(Token![#])
            || (input.peek2(Ident) || input.peek2(Lifetime))
                && (input.peek3(Token![:])
                    || input.peek3(Token![,])
                    || input.peek3(Token![>])
                    || input.peek3(Token![=]))
            || input.peek2(Token![const]));
    let mut generics: Generics = if has_generics {
        input.parse()?
    } else {
        Generics::default()
    };

    let is_const_impl =
        input.peek(Token![const]) || input.peek(Token![?]) && input.peek2(Token![const]);
    let constness: Option<Token![const]> = if is_const_impl {
        // input.parse::<Option<Token![?]>>()?
        input.parse()?
    } else {
        None
    };

    // let begin = input.fork();
    let polarity = if input.peek(Token![!]) && !input.peek2(token::Brace) {
        Some(input.parse::<Token![!]>()?)
    } else {
        None
    };

    let mut first_ty: Type = input.parse()?;
    let self_ty: Type;
    let trait_;

    let is_impl_for = input.peek(Token![for]);
    if is_impl_for {
        let for_token: Token![for] = input.parse()?;
        let mut first_ty_ref = &first_ty;
        while let Type::Group(ty) = first_ty_ref {
            first_ty_ref = &ty.elem;
        }
        if let Type::Path(TypePath { qself: None, .. }) = first_ty_ref {
            while let Type::Group(ty) = first_ty {
                first_ty = *ty.elem;
            }
            if let Type::Path(TypePath { qself: None, path }) = first_ty {
                trait_ = Some((polarity, path, for_token));
            } else {
                unreachable!();
            }
        } else {
            trait_ = None;
        }
        self_ty = input.parse()?;
    } else {
        // trait_ = None;
        // self_ty = if polarity.is_none() {
        //     first_ty
        // } else {
        //     Type::Verbatim(verbatim::between(begin, input))
        // };
        panic!();
    }

    generics.where_clause = input.parse()?;

    let content;
    let brace_token = braced!(content in input);
    parse_inner(&content, &mut attrs)?;

    let mut items = Vec::new();
    while !content.is_empty() {
        items.push(content.parse()?);
    }

    if is_impl_for && trait_.is_none() {
        Ok(None)
    } else {
        Ok(Some(ItemImplConst {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            constness,
            trait_,
            self_ty: Box::new(self_ty),
            brace_token,
            items,
        }))
    }
}

fn parse_inner(input: ParseStream, attrs: &mut Vec<Attribute>) -> Result<()> {
    while input.peek(Token![#]) && input.peek2(Token![!]) {
        attrs.push(input.call(single_parse_inner)?);
    }
    Ok(())
}

fn single_parse_inner(input: ParseStream) -> Result<Attribute> {
    let content;
    Ok(Attribute {
        pound_token: input.parse()?,
        style: AttrStyle::Inner(input.parse()?),
        bracket_token: bracketed!(content in input),
        meta: content.parse()?,
    })
}

impl ToTokens for ItemImplConst {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.attrs.outer());
        self.defaultness.to_tokens(tokens);
        self.unsafety.to_tokens(tokens);
        self.impl_token.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        if let Some((polarity, path, for_token)) = &self.trait_ {
            polarity.to_tokens(tokens);
            path.to_tokens(tokens);
            for_token.to_tokens(tokens);
        }
        self.self_ty.to_tokens(tokens);
        self.generics.where_clause.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            tokens.append_all(self.attrs.inner());
            tokens.append_all(&self.items);
        });
    }
}

trait FilterAttrs<'a> {
    type Ret: Iterator<Item = &'a Attribute>;

    fn outer(self) -> Self::Ret;
    fn inner(self) -> Self::Ret;
}

impl<'a> FilterAttrs<'a> for &'a [Attribute] {
    type Ret = core::iter::Filter<core::slice::Iter<'a, Attribute>, fn(&&Attribute) -> bool>;

    fn outer(self) -> Self::Ret {
        fn is_outer(attr: &&Attribute) -> bool {
            match attr.style {
                AttrStyle::Outer => true,
                AttrStyle::Inner(_) => false,
            }
        }
        self.iter().filter(is_outer)
    }

    fn inner(self) -> Self::Ret {
        fn is_inner(attr: &&Attribute) -> bool {
            match attr.style {
                AttrStyle::Inner(_) => true,
                AttrStyle::Outer => false,
            }
        }
        self.iter().filter(is_inner)
    }
}

impl core::fmt::Debug for ItemImplConst {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        impl ItemImplConst {
            fn debug(&self, formatter: &mut core::fmt::Formatter, name: &str) -> core::fmt::Result {
                let mut formatter = formatter.debug_struct(name);
                formatter.field("attrs", &self.attrs);
                formatter.field("defaultness", &self.defaultness);
                formatter.field("unsafety", &self.unsafety);
                formatter.field("impl_token", &self.impl_token);
                formatter.field("generics", &self.generics);
                formatter.field("constness", &self.constness);
                formatter.field("trait_", &self.trait_);
                formatter.field("self_ty", &self.self_ty);
                formatter.field("brace_token", &self.brace_token);
                formatter.field("items", &self.items);
                formatter.finish()
            }
        }
        self.debug(formatter, "ItemImpl")
    }
}
