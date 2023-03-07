#![no_std]

#[cfg(not(feature = "const"))]
mod unconst;

use proc_macro::TokenStream;

#[cfg(feature = "const")]
#[proc_macro_attribute]
pub fn unconst(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(not(feature = "const"))]
#[proc_macro_attribute]
pub fn unconst(_attr: TokenStream, item: TokenStream) -> TokenStream {
    unconst::unconst(_attr, item)
}
