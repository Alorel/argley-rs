use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::token;

use crate::struct_field::{StructField, TypedFields};

pub struct ParsedVariant {
    pub ident: Ident,
    pub fields: TypedFields,
}

impl ToTokens for ParsedVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        tokens.append_all(quote! { Self::#ident });

        match self.fields {
            TypedFields::Unit => tokens.append_all(quote! { => {}}),
            TypedFields::Tuple(ref fields) => {
                // Self::Variant(THIS_PART)
                tokens.append({
                    let inner_iter = fields.iter().map(move |f| &f.ident);

                    let mut inner_stream = TokenStream::new();
                    inner_stream.append_separated(inner_iter, Punct::new(',', Spacing::Joint));

                    Group::new(Delimiter::Parenthesis, inner_stream)
                });

                token::FatArrow::default().to_tokens(tokens);

                let mut inner = TokenStream::new();

                // Keep original ordering while rendering the header, sort for the body
                let mut fields = fields.iter().collect::<Vec<_>>();
                fields.sort_by(move |a, b| StructField::cmp(a, b));
                inner.append_all(fields);

                tokens.append(Group::new(Delimiter::Brace, inner));
            }
            TypedFields::Named {
                ref fields,
                has_skips,
            } => {
                // Self::Variant { THIS_PART }
                tokens.append({
                    let inner_iter = fields.iter().map(move |f| f.ident.to_token_stream());

                    let mut inner_stream = TokenStream::new();
                    let separator = Punct::new(',', Spacing::Joint);
                    if has_skips {
                        inner_stream.append_separated(
                            inner_iter.chain(iter::once(quote! { .. })),
                            separator,
                        );
                    } else {
                        inner_stream.append_separated(inner_iter, separator);
                    }

                    Group::new(Delimiter::Brace, inner_stream)
                });

                token::FatArrow::default().to_tokens(tokens);

                let mut inner = TokenStream::new();
                inner.append_all(fields);

                tokens.append(Group::new(Delimiter::Brace, inner));
            }
        };
    }
}
