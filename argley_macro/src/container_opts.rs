use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Lit, Token};

use crate::{TryCollectStable, ATTR};

#[derive(Default)]
pub struct ContainerOpts {
    pub drop_name: bool,
    pub as_repr: Option<Ident>,
    pub to_string: bool,
    pub static_args: Option<syn::ExprArray>,
}

impl ContainerOpts {
    pub fn should_collect_enum_fields(&self) -> bool {
        !self.to_string && self.as_repr.is_none()
    }
}

impl TryFrom<Vec<Attribute>> for ContainerOpts {
    type Error = syn::Error;

    fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut repr = None;

        let mut attrs = attrs
            .into_iter()
            .filter_map(|attr| {
                let path = attr.path().get_ident()?;

                match path.to_string().as_str() {
                    "repr" => {
                        repr = Some(attr.parse_args::<Ident>());
                        None
                    }
                    v if v == ATTR => Some(ContainerOpts::try_from(attr)),
                    _ => None,
                }
            })
            .try_collect()?
            .into_iter()
            .collect::<Self>();

        if let Some(as_repr) = &attrs.as_repr {
            match repr {
                None => return Err(syn::Error::new(as_repr.span(), "missing repr")),
                Some(Err(e)) => return Err(syn::Error::new(as_repr.span(), e)),
                Some(Ok(repr)) => {
                    attrs.as_repr = Some(repr);
                }
            }
        }

        Ok(attrs)
    }
}

impl TryFrom<Attribute> for ContainerOpts {
    type Error = syn::Error;

    fn try_from(attr: Attribute) -> syn::Result<Self> {
        let mut opts = Self::default();

        attr.parse_nested_meta(|meta| {
            let error_span = if let Some(path) = meta.path.get_ident() {
                match path.to_string().as_str() {
                    "to_string" => {
                        opts.to_string = true;
                        return Ok(());
                    }
                    "drop_name" => {
                        opts.drop_name = true;
                        return Ok(());
                    }
                    "as_repr" => {
                        opts.as_repr = Some(path.clone());
                        return Ok(());
                    }
                    "static_args" => {
                        meta.input.parse::<Token![=]>()?;
                        let static_args = meta.input.parse::<syn::ExprArray>()?;
                        for el in &static_args.elems {
                            let span = match el {
                                Expr::Lit(ref lit) => match lit.lit {
                                    Lit::Str(_) => continue,
                                    ref other => other.span(),
                                },
                                other => other.span(),
                            };

                            return Err(syn::Error::new(span, "expected string literal"));
                        }

                        opts.static_args = Some(static_args);

                        return Ok(());
                    }
                    _ => path.span(),
                }
            } else {
                meta.path.span()
            };

            Err(syn::Error::new(error_span, "unknown option"))
        })?;

        Ok(opts)
    }
}

impl FromIterator<ContainerOpts> for ContainerOpts {
    fn from_iter<T: IntoIterator<Item = ContainerOpts>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::default(), move |mut acc, opts| {
                if opts.drop_name {
                    acc.drop_name = true;
                }
                if opts.to_string {
                    acc.to_string = true;
                }
                if opts.as_repr.is_some() {
                    acc.as_repr = opts.as_repr;
                }

                if let Some(add_args) = opts.static_args {
                    if let Some(ref mut static_args) = acc.static_args {
                        static_args.elems.extend(add_args.elems);
                    } else {
                        acc.static_args = Some(add_args);
                    }
                }

                acc
            })
    }
}
