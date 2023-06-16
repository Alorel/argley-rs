use std::fmt::Display;

use delegate_display::DelegateDisplay;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::ToTokens;

#[derive(DelegateDisplay)]
pub enum FieldIdent {
    Ident(Ident),
    Idx(usize),
}

impl FieldIdent {
    pub fn with_prefix(&self, prefix: impl Display) -> Self {
        Self::Ident(Ident::new(&format!("{prefix}{self}"), Span::call_site()))
    }
}

impl ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(i) => i.to_tokens(tokens),
            Self::Idx(i) => Literal::usize_unsuffixed(*i).to_tokens(tokens),
        }
    }
}
