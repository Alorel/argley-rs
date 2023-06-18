use syn::spanned::Spanned;
use syn::Attribute;

use crate::{TryCollectStable, ATTR};

#[derive(Default)]
pub struct ContainerOpts {
    pub drop_name: bool,
    pub to_string: bool,
}

impl TryFrom<Vec<Attribute>> for ContainerOpts {
    type Error = syn::Error;

    fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
        let attrs = attrs
            .into_iter()
            .filter_map(move |attr| {
                if attr.path().is_ident(ATTR) {
                    Some(ContainerOpts::try_from(attr))
                } else {
                    None
                }
            })
            .try_collect()?;

        Ok(attrs.into_iter().collect())
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
                acc
            })
    }
}
