use std::rc::Rc;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::Data;

use crate::any_added_wrap::AnyAddedWrapper;
use crate::container_opts::ContainerOpts;
use crate::parsed_variant::ParsedVariant;
use crate::struct_field::{StructField, TypedFields};
use crate::{new_ident, TryCollectStable, ARG_CONSUMER, PROP_ANY_ADDED};

pub struct ParsedFields {
    inner: Inner,
    container_opts: Rc<ContainerOpts>,
}

impl ParsedFields {
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
            && match self.container_opts.static_args {
                Some(ref args) => args.elems.is_empty(),
                None => true,
            }
    }

    pub fn new_empty(opts: Rc<ContainerOpts>, data: &Data) -> syn::Result<Self> {
        Ok(Self {
            inner: Inner::new_empty(data)?,
            container_opts: opts,
        })
    }

    fn render_empty(tokens: &mut TokenStream) {
        FunctionSignature::without_consumer(true).to_tokens(tokens);
        Group::new(Delimiter::Brace, quote! { false }).to_tokens(tokens);
    }

    pub fn from_data(opts: Rc<ContainerOpts>, data: Data) -> syn::Result<Self> {
        Ok(Self {
            inner: data.try_into()?,
            container_opts: opts,
        })
    }
}

impl ToTokens for ParsedFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.is_empty() {
            return Self::render_empty(tokens);
        }

        FunctionSignature::with_consumer(false).to_tokens(tokens);

        let body = {
            let mut tokens = TokenStream::new();

            match self.container_opts.static_args {
                Some(ref args) if !args.elems.is_empty() => {
                    let consumer = new_ident(ARG_CONSUMER);

                    if args.elems.len() == 1 {
                        let first_arg = &args.elems[0];
                        tokens.append_all(quote! {
                            ::argley::ArgConsumer::add_arg(#consumer, #first_arg);
                        });
                    } else {
                        let group = args.to_token_stream();
                        tokens.append_all(quote! {
                            ::argley::ArgConsumer::add_args(#consumer, #group);
                        });
                    }

                    self.inner.to_tokens(Some(new_ident("true")), &mut tokens);
                }
                _ => {
                    self.inner.to_tokens(None, &mut tokens);
                }
            };

            tokens
        };

        Group::new(Delimiter::Brace, body).to_tokens(tokens);
    }
}

enum Inner {
    Struct(Vec<StructField>),
    Enum(Vec<ParsedVariant>),
}

impl Inner {
    fn is_empty(&self) -> bool {
        match self {
            Self::Struct(fields) => fields.is_empty(),
            Self::Enum(variants) => {
                variants.is_empty()
                    || variants.iter().all(move |v| {
                        matches!(v.fields, TypedFields::Unit) && v.unit_variant_value.is_none()
                    })
            }
        }
    }

    /// Like the [`TryFrom`] implementation, but always uses an empty vec for fields
    fn new_empty(data: &Data) -> syn::Result<Self> {
        Ok(match *data {
            Data::Struct(_) => Self::Struct(Vec::new()),
            Data::Enum(_) => Self::Enum(Vec::new()),
            Data::Union(ref un) => return Err(on_union(&un.union_token)),
        })
    }

    fn to_tokens(&self, return_value: Option<Ident>, tokens: &mut TokenStream) {
        if let Some(return_value) = return_value {
            self.to_tokens_base(false, tokens);
            tokens.append(return_value);
        } else {
            let any_added = new_ident(PROP_ANY_ADDED);
            tokens.append_all(quote! { let mut #any_added = false; });
            self.to_tokens_base(true, tokens);

            tokens.append(any_added);
        };
    }

    fn to_tokens_base(&self, use_any_added: bool, tokens: &mut TokenStream) {
        match self {
            Self::Struct(fields) => {
                if use_any_added {
                    tokens.append_all(fields.iter().map(AnyAddedWrapper));
                } else {
                    tokens.append_terminated(fields, Punct::new(';', Spacing::Joint));
                }
            }
            Self::Enum(variants) => {
                tokens.append_all(quote! { match *self });

                tokens.append({
                    let mut inner = TokenStream::new();
                    let variants = variants.iter().map(move |v| v.to_tokens(use_any_added));

                    inner.append_all(variants);
                    Group::new(Delimiter::Brace, inner)
                });

                tokens.append(Punct::new(';', Spacing::Alone));
            }
        }
    }
}

impl TryFrom<Data> for Inner {
    type Error = syn::Error;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        Ok(match data {
            Data::Struct(data) => {
                let data = StructField::collect_from_fields(data.fields, true)?;
                Self::Struct(data.into())
            }
            Data::Enum(data) => {
                let variants = data
                    .variants
                    .into_iter()
                    .map(ParsedVariant::try_from)
                    .try_collect()?;
                Self::Enum(variants)
            }
            Data::Union(un) => return Err(on_union(&un.union_token)),
        })
    }
}

fn on_union(un: &impl Spanned) -> syn::Error {
    syn::Error::new(un.span(), "Unions not supported")
}

pub struct FunctionSignature<'a> {
    inline: bool,
    consumer_arg: &'a str,
}

impl<'a> FunctionSignature<'a> {
    #[inline]
    pub fn with_consumer(inline: bool) -> Self {
        Self {
            inline,
            consumer_arg: ARG_CONSUMER,
        }
    }

    #[inline]
    pub fn without_consumer(inline: bool) -> Self {
        Self {
            inline,
            consumer_arg: "_",
        }
    }
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
