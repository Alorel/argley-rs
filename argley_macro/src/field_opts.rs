use std::cmp::Ordering;
use std::iter::Sum;

use proc_macro2::{Ident, Literal};
use syn::ExprPath;

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
