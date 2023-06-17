use std::cmp::Ordering;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{Field, Fields};

use crate::field_ident::FieldIdent;
use crate::field_opts::FieldOpts;
use crate::{new_ident, parse_eq, TryCollectStable, ARG_CONSUMER, ATTR, OPT_SKIP, PROP_ANY_ADDED};

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

    #[allow(clippy::too_many_lines)]
    pub fn collect_from_iter(
        fields: impl IntoIterator<Item = Field>,
        is_struct: bool,
    ) -> syn::Result<CollectFromIter> {
        let mut has_variadic = false;
        let mut has_skips = false;
        let mut attr_collector = Vec::new();

        let mut fields = fields
            .into_iter()
            .enumerate()
            .filter_map(
                |(idx, field): (usize, Field)| -> Option<syn::Result<StructField>> {
                    let mut opts = {
                        let attrs = field
                            .attrs
                            .into_iter()
                            .filter_map(move |attr| {
                                if !attr.path().is_ident(ATTR) {
                                    return None;
                                }

                                let mut opts = FieldOpts::default();
                                let opts_result = attr.parse_nested_meta(|meta| {
                                    let ident = match meta.path.get_ident() {
                                        Some(ident) => ident,
                                        None => {
                                            return Err(syn::Error::new(
                                                meta.path.span(),
                                                "Expected `Ident`",
                                            ));
                                        }
                                    };

                                    match ident.to_string().as_str() {
                                        v if v == OPT_SKIP => {
                                            opts.skip = true;
                                        }
                                        "short" => {
                                            opts.short = true;
                                        }
                                        "variadic" => {
                                            opts.variadic = Some(ident.clone());
                                        }
                                        "position" => {
                                            let literal: Literal = parse_eq(meta.input)?;
                                            if let Ok(pos) = literal.to_string().parse() {
                                                opts.position = Some(pos);
                                            } else {
                                                return Err(syn::Error::new(
                                                    literal.span(),
                                                    "Position must be a u16",
                                                ));
                                            }
                                        }
                                        "formatter" => {
                                            opts.formatter = Some(parse_eq(meta.input)?);
                                        }
                                        "rename" => {
                                            opts.rename = Some(parse_eq(meta.input)?);
                                        }
                                        _ => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                "Unknown option",
                                            ))
                                        }
                                    };
                                    Ok(())
                                });

                                Some(if let Err(e) = opts_result {
                                    Err(e)
                                } else {
                                    Ok(opts)
                                })
                            })
                            .try_collect_to(&mut attr_collector);

                        if let Err(e) = attrs {
                            return Some(Err(e));
                        }

                        attr_collector.drain(..).sum::<FieldOpts>()
                    };

                    if opts.skip {
                        has_skips = true;
                        return None;
                    }

                    if let Some(variadic) = &opts.variadic {
                        if has_variadic {
                            return Some(Err(syn::Error::new(
                                variadic.span(),
                                "Only one variadic field allowed",
                            )));
                        }
                        has_variadic = true;
                    }

                    let ident = if let Some(ident) = field.ident {
                        FieldIdent::Ident(ident)
                    } else {
                        // Make unnamed by default
                        if opts.is_default_field_name() {
                            opts.position = Some(idx.try_into().unwrap_or(u16::MAX));
                        }

                        let ident = FieldIdent::Idx(idx);
                        if is_struct {
                            ident
                        } else {
                            ident.with_prefix('f')
                        }
                    };

                    Some(Ok(StructField {
                        opts,
                        idx,
                        is_struct,
                        ident,
                    }))
                },
            )
            .try_collect()?;

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

        tokens.append_all(quote! { if ::argley::Arg:: });

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

        let any_added = new_ident(PROP_ANY_ADDED);
        tokens.append_all(quote! {{ #any_added = true; } });
    }
}

pub struct CollectFromIter {
    pub fields: Vec<StructField>,
    pub has_skips: bool,
}
