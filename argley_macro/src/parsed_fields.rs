use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Data;

use crate::parsed_variant::ParsedVariant;
use crate::struct_field::StructField;
use crate::{new_ident, try_collect, ARG_CONSUMER, PROP_ANY_ADDED};

pub enum ParsedFields {
    Struct(Vec<StructField>),
    Enum(Vec<ParsedVariant>),
}

impl ParsedFields {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Struct(fields) => fields.is_empty(),
            Self::Enum(variants) => variants.is_empty(),
        }
    }
}

impl ToTokens for ParsedFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.is_empty() {
            FunctionSignature {
                inline: true,
                consumer_arg: "_",
            }
            .to_tokens(tokens);

            tokens.append_all(quote! { { false } });
            return;
        }

        FunctionSignature {
            inline: false,
            consumer_arg: ARG_CONSUMER,
        }
        .to_tokens(tokens);

        let any_added = new_ident(PROP_ANY_ADDED);
        tokens.append_all(match self {
            Self::Struct(fields) => {
                quote! {{
                    let mut #any_added = false;
                    #(#fields)*
                    #any_added
                }}
            }
            Self::Enum(variants) => {
                let mut outer = TokenStream::new();
                outer.append_all(quote! {
                    let mut #any_added = false;
                    match self
                });

                outer.append({
                    let mut inner = TokenStream::new();
                    inner.append_terminated(variants, Punct::new(',', Spacing::Joint));
                    Group::new(Delimiter::Brace, inner)
                });

                Punct::new(';', Spacing::Joint).to_tokens(&mut outer);
                any_added.to_tokens(&mut outer);

                Group::new(Delimiter::Brace, outer).into_token_stream()
            }
        });
    }
}

impl TryFrom<Data> for ParsedFields {
    type Error = syn::Error;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        Ok(match data {
            Data::Struct(data) => {
                let data = StructField::collect_from_fields(data.fields, true)?;
                Self::Struct(data.into())
            }
            Data::Enum(data) => {
                let variants =
                    data.variants
                        .into_iter()
                        .map(|variant| -> syn::Result<ParsedVariant> {
                            if let Some((_, disc)) = variant.discriminant {
                                return Err(syn::Error::new_spanned(
                                    disc,
                                    "Discriminants not supported",
                                ));
                            }

                            let fields = StructField::collect_from_fields(variant.fields, false)?;

                            Ok(ParsedVariant {
                                ident: variant.ident,
                                fields,
                            })
                        });

                Self::Enum(try_collect(variants)?)
            }
            Data::Union(un) => {
                return Err(syn::Error::new_spanned(
                    un.union_token,
                    "Unions not supported",
                ))
            }
        })
    }
}

struct FunctionSignature<'a> {
    inline: bool,
    consumer_arg: &'a str,
}

impl ToTokens for FunctionSignature<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.inline {
            tokens.append_all(quote! { #[inline] });
        }

        let consumer = new_ident(self.consumer_arg);
        tokens.append_all(quote! {
            fn add_unnamed_to(&self, #consumer: &mut impl ::argley::ArgConsumer) -> bool
        });
    }
}