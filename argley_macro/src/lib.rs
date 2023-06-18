//! Macro for [`argley`](https://docs.rs/argley).

#![warn(missing_docs)]
#![allow(clippy::manual_let_else)]

use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, DeriveInput, Generics, Token};

use crate::container_opts::ContainerOpts;
use crate::parsed_fields::{FunctionSignature, ParsedFields};

mod container_opts;
mod field_ident;
mod field_opts;
mod parsed_fields;
mod parsed_variant;
mod struct_field;

const ATTR: &str = "arg";
const OPT_SKIP: &str = "skip";
const ARG_CONSUMER: &str = "consumer";
const PROP_ANY_ADDED: &str = "any_added";

struct Runtime {
    struct_name: Ident,
    generics: Generics,
    fields: ParsedFields,
    opts: ContainerOpts,
}

/// Derive the `Arg` trait.
///
/// # Field attributes
///
/// | Attribute | Description |
/// |---|---|
/// | `arg(skip)` | Exclude this property. Unavailable for fields in tuple enum variants. |
/// | `arg(short)` | Prefix with `-` instead of `--`. Ignored on variadic/positional arguments |
/// | `arg(position = INTEGER)` | Positional argument. |
/// | `arg(variadic)` | Shorthand for putting an argument in the final position |
/// | `arg(rename = "new_name")` | Rename the argument |
/// | `arg(formatter = path::to::formatter)` | Format the field with the given function. Has a signature of `fn(&T) -> impl Arg` |
///
/// # Container attributes
///
/// | Attribute | Description |
/// |---|---|
/// | `arg(drop_name)` | Derive an `Arg::add_to` that ignores its `name` parameter |
/// | `arg(to_string)` | Derive an `Arg::add_unnamed_to` that uses `self.to_string()` as the argument |
/// | `arg(as_repr)` | For use on enums - use `(*self as REPR)` as the argument on enums with `#[repr(INT)]` |
///
/// # Variant attributes
///
/// | Attribute | Description |
/// |---|---|
/// | `arg(value = EXPRESSION)` | Make the given variant push the given expression as its arguments (e.g. `&[&str]` or `PathBuf`) |
#[proc_macro_derive(Arg, attributes(arg))]
pub fn derive_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Runtime {
        struct_name,
        mut generics,
        fields,
        opts,
    } = parse_macro_input!(input as Runtime);

    let consumer = new_ident(ARG_CONSUMER);

    let named_impl = if opts.drop_name {
        Some(quote! {
            #[inline]
            fn add_to(&self, _: &str, #consumer: &mut impl ::argley::ArgConsumer) -> bool {
                ::argley::Arg::add_unnamed_to(self, #consumer)
            }
        })
    } else {
        None
    };

    let fields = if opts.to_string {
        if has_generics(&generics) {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote! { Self: ::std::string::ToString });
        }

        let sig = FunctionSignature {
            inline: false,
            consumer_arg: ARG_CONSUMER,
        };

        quote! {
            #sig {
                ::argley::ArgConsumer::add_arg(#consumer, ::std::string::ToString::to_string(self));
                true
            }
        }
    } else if let Some(as_repr) = opts.as_repr {
        if has_generics(&generics) {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote! { Self: ::std::borrow::ToOwned });
        }

        let sig = FunctionSignature {
            inline: true,
            consumer_arg: ARG_CONSUMER,
        };

        quote! {
            #sig {
                ::argley::Arg::add_unnamed_to(&(::std::borrow::ToOwned::to_owned(self) as #as_repr), #consumer)
            }
        }
    } else {
        fields.into_token_stream()
    };

    let (g1, g2, g3) = generics.split_for_impl();
    (quote! {
        #[automatically_derived]
        impl #g1 ::argley::Arg for #struct_name #g2 #g3 {
            #named_impl

            #fields
        }
    })
    .into()
}

impl Parse for Runtime {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput {
            ident: struct_name,
            generics,
            data,
            attrs,
            ..
        } = input.parse::<DeriveInput>()?;

        let opts = ContainerOpts::try_from(attrs)?;

        Ok(Self {
            fields: if opts.should_collect_enum_fields() {
                ParsedFields::try_from(data)
            } else {
                ParsedFields::new_empty_from(&data)
            }?,
            opts,
            struct_name,
            generics,
        })
    }
}

fn new_ident(label: &str) -> Ident {
    Ident::new(label, Span::call_site())
}

/// Tmp stable version
trait TryCollectStable<T, E> {
    fn try_collect(self) -> Result<Vec<T>, E>;

    fn try_collect_to(self, vec: &mut Vec<T>) -> Result<(), E>;
}

impl<T, E, I: Iterator<Item = Result<T, E>>> TryCollectStable<T, E> for I {
    fn try_collect(self) -> Result<Vec<T>, E> {
        let mut vec = Vec::new();
        self.try_collect_to(&mut vec)?;
        Ok(vec)
    }

    fn try_collect_to(self, vec: &mut Vec<T>) -> Result<(), E> {
        for item in self {
            vec.push(item?);
        }

        Ok(())
    }
}

fn parse_eq<T: Parse>(stream: ParseStream) -> syn::Result<T> {
    stream.parse::<Token![=]>()?;
    stream.parse()
}

fn has_generics(generics: &Generics) -> bool {
    !(generics.params.is_empty() && generics.where_clause.is_none())
}
