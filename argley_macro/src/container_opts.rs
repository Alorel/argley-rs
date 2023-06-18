use syn::Attribute;

use crate::{TryCollectStable, ATTR};

#[derive(Default)]
pub struct ContainerOpts {
    pub drop_name: bool,
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
            if meta.path.is_ident("drop_name") {
                opts.drop_name = true;
                Ok(())
            } else {
                Err(syn::Error::new_spanned(meta.path, "unknown option"))
            }
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
                acc
            })
    }
}
