//! Derives the [`Abilitylike`] trait
//
//! This derive macro was inspired by the `strum` crate's `EnumIter` macro.
//! Original source: https://github.com/Peternator7/strum,
//! Copyright (c) 2019 Peter Glotfelty under the MIT License

extern crate proc_macro;
mod abilitylike;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Abilitylike)]
pub fn abilitylike(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    crate::abilitylike::abilitylike_inner(&ast).into()
}
