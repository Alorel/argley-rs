use crate::Arg;
use std::ffi::{OsStr, OsString};
use std::process::Command;

/// [`Arg`]s collected in a [`Vec`]
pub type CollectedArgs = Vec<OsString>;

/// An [`Arg`] consumer
pub trait ArgConsumer {
    /// Add one argument
    fn add_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self;

    /// Add multiple arguments
    fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self;

    /// Add a set of arguments from an [`Arg`] implementation
    #[inline]
    fn add_arg_set(&mut self, args: &impl Arg) -> &mut Self
    where
        Self: Sized,
    {
        args.add_unnamed_to(self);
        self
    }
}

macro_rules! command_arg_consumer {
    ($ty: ty) => {
        impl ArgConsumer for $ty {
            #[inline]
            fn add_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
                self.arg(arg)
            }

            #[inline]
            fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
                self.args(args)
            }
        }
    };
}

command_arg_consumer!(Command);

#[cfg(feature = "tokio")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "tokio")))]
command_arg_consumer!(tokio::process::Command);

#[cfg(feature = "async-std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async-std")))]
command_arg_consumer!(async_std::process::Command);

impl ArgConsumer for OsString {
    fn add_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        if !self.is_empty() {
            pre_push_one(self, arg.as_ref());
        }
        self.push(arg.as_ref());
        self
    }

    fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        let mut args = args.into_iter();

        if let Some(first) = args.next() {
            self.add_arg(first.as_ref());
        } else {
            return self;
        }

        // Avoid the !self.is_empty() check per iteration
        for arg in args {
            pre_push_one(self, arg.as_ref());
            self.push(arg.as_ref());
        }

        self
    }
}

fn pre_push_one(this: &mut OsString, arg: &OsStr) {
    this.reserve(arg.len() + 1);
    this.push(" ");
}

impl ArgConsumer for CollectedArgs {
    #[inline]
    fn add_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.push(arg.as_ref().to_owned());
        self
    }

    fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.extend(args.into_iter().map(move |v| v.as_ref().to_owned()));
        self
    }
}
