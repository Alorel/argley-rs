use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use crate::{new_ident, PROP_ANY_ADDED};

pub struct AnyAddedWrapper<'a, T>(pub &'a T);

impl<T: ToTokens> ToTokens for AnyAddedWrapper<'_, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(new_ident("if"));
        self.0.to_tokens(tokens);

        let inner = {
            let any_added = new_ident(PROP_ANY_ADDED);
            quote! { #any_added = true; }
        };

        tokens.append(Group::new(Delimiter::Brace, inner));
    }
}
