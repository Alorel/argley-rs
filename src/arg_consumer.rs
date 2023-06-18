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

impl ArgConsumer for Command {
    #[inline]
    fn add_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.arg(arg)
    }

    #[inline]
    fn add_args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.args(args)
    }
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
