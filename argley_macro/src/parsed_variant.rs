use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{token, Expr, Token, Variant};

use crate::any_added_wrap::AnyAddedWrapper;
use crate::struct_field::{StructField, TypedFields};
use crate::{new_ident, TryCollectStable, ARG_CONSUMER, ATTR};

pub struct ParsedVariant {
    pub ident: Ident,
    pub fields: TypedFields,
    pub unit_variant_value: Option<Expr>,
}

impl ParsedVariant {
    fn append_unit(&self, use_any_added: bool, tokens: &mut TokenStream) {
        let body = if let Some(ref val) = self.unit_variant_value {
            let mut base = {
                let consumer = new_ident(ARG_CONSUMER);

                quote! { ::argley::Arg::add_unnamed_to(#val, #consumer) }
            };

            if use_any_added {
                AnyAddedWrapper(&base).into_token_stream()
            } else {
                base.append(Punct::new(';', Spacing::Joint));
                base
            }
        } else {
            TokenStream::new()
        };

        Self::append_body(tokens, body);
    }

    fn append_tuple(fields: &[StructField], use_any_added: bool, tokens: &mut TokenStream) {
        // Self::Variant(THIS_PART) =>
        tokens.append({
            let inner_iter = fields.iter().map(move |f| {
                let id = &f.ident;
                quote! { ref #id }
            });

            let mut inner_stream = TokenStream::new();
            inner_stream.append_separated(inner_iter, Punct::new(',', Spacing::Joint));

            Group::new(Delimiter::Parenthesis, inner_stream)
        });

        let mut body = TokenStream::new();

        if use_any_added {
            let mut fields = fields.iter().map(AnyAddedWrapper).collect::<Vec<_>>();
            fields.sort_by(move |a, b| StructField::cmp(a.0, b.0));
            body.append_all(fields);
        } else {
            // Keep original ordering while rendering the header, sort for the body
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(move |a, b| StructField::cmp(a, b));

            body.append_terminated(fields, Punct::new(';', Spacing::Joint));
        }

        Self::append_body(tokens, body);
    }

    fn append_fields(
        fields: &Vec<StructField>,
        has_skips: bool,
        use_any_added: bool,
        tokens: &mut TokenStream,
    ) {
        // Self::Variant { THIS_PART } =>
        tokens.append({
            let inner_iter = fields.iter().map(move |f| {
                let id = &f.ident;
                quote! { ref #id }
            });

            let mut inner_stream = TokenStream::new();
            let separator = Punct::new(',', Spacing::Joint);
            if has_skips {
                inner_stream
                    .append_separated(inner_iter.chain(iter::once(quote! { .. })), separator);
            } else {
                inner_stream.append_separated(inner_iter, separator);
            }

            Group::new(Delimiter::Brace, inner_stream)
        });

        let mut body = TokenStream::new();

        if use_any_added {
            body.append_all(fields.iter().map(AnyAddedWrapper));
        } else {
            body.append_terminated(fields, Punct::new(';', Spacing::Joint));
        }

        Self::append_body(tokens, body);
    }

    fn append_body(tokens: &mut TokenStream, body: TokenStream) {
        token::FatArrow::default().to_tokens(tokens);
        tokens.append(Group::new(Delimiter::Brace, body));
    }

    pub fn to_tokens(&self, use_any_added: bool) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.append_all(quote! { Self:: });
        self.ident.to_tokens(&mut tokens);

        match self.fields {
            TypedFields::Unit => {
                self.append_unit(use_any_added, &mut tokens);
            }
            TypedFields::Tuple(ref fields) => {
                Self::append_tuple(fields, use_any_added, &mut tokens);
            }
            TypedFields::Named {
                ref fields,
                has_skips,
            } => {
                Self::append_fields(fields, has_skips, use_any_added, &mut tokens);
            }
        };

        tokens
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
