//! Macro for [`argley`](https://docs.rs/argley).

#![warn(missing_docs)]
#![allow(clippy::manual_let_else)]

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Generics, Token};

use crate::parsed_fields::ParsedFields;

mod field_ident;
mod field_opts;
mod parsed_fields;
mod parsed_variant;
mod struct_field;

const ATTR: &str = "arg";
const OPT_SKIP: &str = "skip";
const ARG_CONSUMER: &str = "consumer";
const PROP_ANY_ADDED: &str = "any_added";

/// Derive the `Arg` trait.
///
/// Field attributes:
///
/// | Attribute | Description |
/// |---|---|
/// | `arg(skip)` | Exclude this property |
/// | `arg(short)` | Prefix with `-` instead of `--`. Ignored on variadic/positional arguments |
/// | `arg(position = INTEGER)` | Positional argument. |
/// | `arg(variadic)` | Shorthand for putting an argument in the final position |
/// | `arg(rename = "new_name")` | Rename the argument |
/// | `arg(formatter = path::to::formatter)` | Format the field with the given function. Has a signature of `fn(&T) -> impl Arg` |
#[proc_macro_derive(Arg, attributes(arg))]
pub fn derive_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Runtime {
        struct_name,
        generics,
        fields,
    } = parse_macro_input!(input as Runtime);

    let (g1, g2, g3) = generics.split_for_impl();

    (quote! {
        #[automatically_derived]
        impl #g1 ::argley::Arg for #struct_name #g2 #g3 {
            #fields
        }
    })
    .into()
}

struct Runtime {
    struct_name: Ident,
    generics: Generics,
    fields: ParsedFields,
}

impl Parse for Runtime {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput {
            ident: struct_name,
            generics,
            data,
            ..
        } = input.parse::<DeriveInput>()?;

        Ok(Self {
            fields: ParsedFields::try_from(data)?,
            struct_name,
            generics,
        })
    }
}

fn new_ident(label: &str) -> Ident {
    Ident::new(label, Span::call_site())
}

fn try_collect<T, E>(iter: impl Iterator<Item = Result<T, E>>) -> Result<Vec<T>, E> {
    let mut vec = Vec::new();
    try_collect_to(iter, &mut vec)?;
    Ok(vec)
}
fn try_collect_to<T, E>(
    iter: impl Iterator<Item = Result<T, E>>,
    vec: &mut Vec<T>,
) -> Result<(), E> {
    for item in iter {
        vec.push(item?);
    }

    Ok(())
}

fn parse_eq<T: Parse>(stream: ParseStream) -> syn::Result<T> {
    stream.parse::<Token![=]>()?;
    stream.parse()
}
