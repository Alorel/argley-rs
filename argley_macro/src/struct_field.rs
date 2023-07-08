use std::cmp::Ordering;
use std::iter::Enumerate;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Attribute, Field, Fields};

use crate::field_ident::FieldIdent;
use crate::field_opts::FieldOpts;
use crate::{new_ident, TryCollectStable, ARG_CONSUMER, ATTR};

pub struct StructField {
    pub opts: FieldOpts,
    pub idx: usize,
    pub is_struct: bool,

    pub ident: FieldIdent,
}

pub enum TypedFields {
    Unit,
    Tuple(Vec<StructField>),
    Named {
        fields: Vec<StructField>,
        has_skips: bool,
    },
}

impl From<TypedFields> for Vec<StructField> {
    fn from(value: TypedFields) -> Self {
        match value {
            TypedFields::Tuple(fields) | TypedFields::Named { fields, .. } => fields,
            TypedFields::Unit => Vec::new(),
        }
    }
}

impl StructField {
    pub fn cmp(&self, other: &Self) -> Ordering {
        match self.opts.partial_cmp(&other.opts) {
            None | Some(Ordering::Equal) => self.idx.cmp(&other.idx),
            Some(or) => or,
        }
    }

    pub fn collect_from_fields(fields: Fields, is_struct: bool) -> syn::Result<TypedFields> {
        let (named, fields) = match fields {
            Fields::Named(f) => (true, f.named),
            Fields::Unnamed(f) => (false, f.unnamed),
            Fields::Unit => return Ok(TypedFields::Unit),
        };

        let (has_skips, mut fields) = if fields.is_empty() {
            (false, Vec::new())
        } else {
            let res = Self::collect_from_iter(fields, is_struct)?;
            (res.has_skips, res.fields)
        };

        Ok(if named {
            if !is_struct {
                fields.sort_by(StructField::cmp);
            }
            TypedFields::Named { has_skips, fields }
        } else {
            if has_skips && !is_struct {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Tuple struct fields may not be skipped",
                ));
            }
            TypedFields::Tuple(fields)
        })
    }

    pub fn collect_from_iter(
        fields: impl IntoIterator<Item = Field>,
        is_struct: bool,
    ) -> syn::Result<CollectFromIter> {
        let mut has_skips = false;

        let mut fields = FieldFilterMapper::new(is_struct, &mut has_skips, fields).try_collect()?;

        if is_struct {
            fields.sort_by(StructField::cmp);
        }

        Ok(CollectFromIter { fields, has_skips })
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ref opts,
            ref ident,
            is_struct,
            ..
        } = *self;

        tokens.append_all(quote! { ::argley::Arg:: });

        tokens.append_all({
            let field_expr = {
                let field_expr_base = if is_struct {
                    quote! { &self.#ident }
                } else {
                    ident.to_token_stream()
                };

                if let Some(fmt) = &opts.formatter {
                    quote! { #fmt(#field_expr_base) }
                } else {
                    field_expr_base
                }
            };
            let consumer = new_ident(ARG_CONSUMER);

            if opts.position.is_some() || opts.variadic.is_some() {
                quote! { add_unnamed_to(#field_expr, #consumer) }
            } else {
                let mut name = opts.name_prefix();
                let span = if let Some(rename) = &opts.rename {
                    let rename_str = rename.to_string();
                    name.push_str(&rename_str[1..rename_str.len() - 1]);
                    rename.span()
                } else {
                    name.push_str(&ident.to_string());
                    Span::call_site()
                };
                let mut name = Literal::string(&name);
                name.set_span(span);

                quote! { add_to(#field_expr, #name, #consumer) }
            }
        });
    }
}

pub struct CollectFromIter {
    pub fields: Vec<StructField>,
    pub has_skips: bool,
}

#[derive(Default)]
struct AttrCollector(Vec<FieldOpts>);
impl AttrCollector {
    fn next(&mut self, attrs: impl IntoIterator<Item = Attribute>) -> syn::Result<FieldOpts> {
        attrs
            .into_iter()
            .filter_map(move |attr| {
                if attr.path().is_ident(ATTR) {
                    Some(FieldOpts::try_from(attr))
                } else {
                    None
                }
            })
            .try_collect_to(&mut self.0)?;

        Ok(self.0.drain(..).sum::<FieldOpts>())
    }
}

struct FieldFilterMapper<'a, I> {
    src: Enumerate<I>,
    is_struct: bool,
    has_skips: &'a mut bool,
    has_variadic: bool,
    attr_collector: AttrCollector,
}

impl<'a, I: Iterator<Item = Field>> FieldFilterMapper<'a, I> {
    fn new(is_struct: bool, has_skips: &'a mut bool, src: impl IntoIterator<IntoIter = I>) -> Self {
        Self {
            is_struct,
            src: src.into_iter().enumerate(),
            has_skips,
            has_variadic: false,
            attr_collector: Default::default(),
        }
    }
}

impl<'a, I: Iterator<Item = Field>> Iterator for FieldFilterMapper<'a, I> {
    type Item = syn::Result<StructField>;

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, field) = self.src.next()?;

        let mut opts = match self.attr_collector.next(field.attrs) {
            Ok(opts) => opts,
            Err(err) => return Some(Err(err)),
        };

        if opts.skip {
            *self.has_skips = true;

            return self.next();
        }

        if let Some(ref variadic) = opts.variadic {
            if self.has_variadic {
                return Some(Err(syn::Error::new(
                    variadic.span(),
                    "Only one variadic field allowed",
                )));
            }

            self.has_variadic = true;
        }

        let ident = if let Some(ident) = field.ident {
            FieldIdent::Ident(ident)
        } else {
            // Make unnamed by default
            if opts.is_default_field_name() {
                opts.position = Some(idx.try_into().unwrap_or(u16::MAX));
            }

            let ident = FieldIdent::Idx(idx);
            if self.is_struct {
                ident
            } else {
                ident.with_prefix('f')
            }
        };

        Some(Ok(StructField {
            opts,
            idx,
            is_struct: self.is_struct,
            ident,
        }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.src.size_hint().1)
    }
}
