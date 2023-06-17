use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{token, Expr, Token, Variant};

use crate::struct_field::{StructField, TypedFields};
use crate::{new_ident, TryCollectStable, ARG_CONSUMER, ATTR, PROP_ANY_ADDED};

pub struct ParsedVariant {
    pub ident: Ident,
    pub fields: TypedFields,
    pub unit_variant_value: Option<Expr>,
}

impl ToTokens for ParsedVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        tokens.append_all(quote! { Self::#ident });

        match self.fields {
            TypedFields::Unit => tokens.append_all({
                let body = self.unit_variant_value.as_ref().map(move |val| {
                    let consumer = new_ident(ARG_CONSUMER);
                    let any_added = new_ident(PROP_ANY_ADDED);

                    quote! {
                        if ::argley::Arg::add_unnamed_to(#val, #consumer) {
                            #any_added = true;
                        }
                    }
                });

                quote! { => {
                    #body
                }}
            }),
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

impl TryFrom<Variant> for ParsedVariant {
    type Error = syn::Error;

    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        if let Some((_, disc)) = variant.discriminant {
            return Err(syn::Error::new_spanned(disc, "Discriminants not supported"));
        }

        let fields = StructField::collect_from_fields(variant.fields, false)?;

        Ok(Self {
            ident: variant.ident,
            fields,
            unit_variant_value: variant
                .attrs
                .into_iter()
                .filter_map(move |attr| {
                    if !attr.path().is_ident(ATTR) {
                        return None;
                    }

                    let mut unit_variant_value = None;
                    let result = attr.parse_nested_meta(|meta| {
                        if !meta.path.is_ident("value") {
                            return Err(syn::Error::new_spanned(meta.path, "Unrecognised option"));
                        }

                        meta.input.parse::<Token![=]>()?;
                        unit_variant_value = Some(meta.input.parse::<Expr>()?);

                        Ok(())
                    });

                    Some(result.map(move |_| unit_variant_value))
                })
                .try_collect()?
                .into_iter()
                .fold(None, move |acc, val| if val.is_some() { val } else { acc }),
        })
    }
}
