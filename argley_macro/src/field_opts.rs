use std::cmp::Ordering;
use std::iter::Sum;

use crate::{parse_eq, OPT_SKIP};
use proc_macro2::{Ident, Literal};
use syn::spanned::Spanned;
use syn::{Attribute, ExprPath};

#[derive(Default)]
pub struct FieldOpts {
    pub skip: bool,
    pub short: bool,
    pub variadic: Option<Ident>,
    pub position: Option<u16>,
    pub rename: Option<Literal>,
    pub formatter: Option<ExprPath>,
}

impl FieldOpts {
    pub fn is_default_field_name(&self) -> bool {
        self.position.is_none() && self.rename.is_none() && self.variadic.is_none()
    }

    pub fn name_prefix(&self) -> String {
        String::from(if self.short { "-" } else { "--" })
    }

    pub fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (&self.variadic, &other.variadic) {
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            _ => match (&self.position, &other.position) {
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
                _ => return None,
            },
        })
    }
}

impl TryFrom<Attribute> for FieldOpts {
    type Error = syn::Error;

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        let mut opts = Self::default();

        attr.parse_nested_meta(|meta| {
            let ident = match meta.path.get_ident() {
                Some(ident) => ident,
                None => {
                    return Err(syn::Error::new(meta.path.span(), "Expected `Ident`"));
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
                    let literal = parse_eq::<Literal>(meta.input)?;
                    if let Ok(pos) = literal.to_string().parse() {
                        opts.position = Some(pos);
                    } else {
                        return Err(syn::Error::new(literal.span(), "Position must be a u16"));
                    }
                }
                "formatter" => {
                    opts.formatter = Some(parse_eq(meta.input)?);
                }
                "rename" => {
                    opts.rename = Some(parse_eq(meta.input)?);
                }
                _ => return Err(syn::Error::new(ident.span(), "Unknown option")),
            };

            Ok(())
        })?;

        Ok(opts)
    }
}

impl Sum for FieldOpts {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(move |mut acc, opts| {
            if opts.skip {
                acc.skip = true;
            }
            if opts.short {
                acc.short = true;
            }
            if opts.variadic.is_some() {
                acc.variadic = opts.variadic;
            }
            if opts.position.is_some() {
                acc.position = opts.position;
            }
            if opts.rename.is_some() {
                acc.rename = opts.rename;
            }

            acc
        })
        .unwrap_or_default()
    }
}
