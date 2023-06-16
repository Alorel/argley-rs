//! Turn a struct into arguments for a [`Command`]. See the [derive macro](argley_macro::Arg) for
//! options you can pass in.
//!
//! ```
//!# use argley::prelude::*;
//!# use std::path::{Path, PathBuf};
//!# use std::ptr;
//!# use std::process::Command;
//!
//! #[derive(Arg)]
//! struct BasicArgs<'a> {
//!     #[arg(position = 1)]
//!     str_ref: &'a str,
//!
//!     #[arg(variadic)]
//!     number: u8,
//!
//!     #[arg(rename = "p", short)]
//!     path: PathBuf,
//!     opt_skipped: Option<String>,
//!
//!     #[arg(position = 0)]
//!     opt_present: Option<&'static str>,
//!
//!     false_arg: bool,
//!     true_arg: bool,
//!
//!     #[arg(skip)]
//!     _skipped_arg: *const u8,
//!     empty_collection: Vec<&'a Path>,
//!     full_collection: Vec<String>,
//! }
//!
//! let args = BasicArgs {
//!   str_ref: "hello",
//!   number: 42,
//!   path: Path::new("world").to_owned(),
//!   opt_skipped: None,
//!   opt_present: Some("present".into()),
//!   false_arg: false,
//!   true_arg: true,
//!   _skipped_arg: ptr::null(),
//!   empty_collection: Vec::new(),
//!   full_collection: vec!["a".into(), "b".into()],
//! };
//!
//! let mut command = Command::new("foo");
//! command.add_arg_set(&args);
//!
//! let resulting_args = command.get_args().collect::<Vec<_>>();
//! assert_eq!(&resulting_args[..], &[
//!     "-p",
//!     "world",
//!     "--true_arg",
//!     "--full_collection",
//!     "a",
//!     "b",
//!     "present",
//!     "hello",
//!     "42",
//! ]);
//! ```

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

use std::ffi::{OsStr, OsString};
use std::process::Command;

#[cfg(feature = "derive")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "derive")))]
pub use argley_macro::Arg;

mod arg_impls;

/// An argument that can be passed to an [`ArgConsumer`] such as a [`Command`].
pub trait Arg {
    /// Add a named argument to the given [`ArgConsumer`]. Similar to
    /// [`add_unnamed_to`](Arg::add_unnamed_to), but includes the name of the property if
    /// applicable.
    ///
    /// # Returns
    /// True if it's been added successfully, false otherwise. False is always returned if:
    ///
    /// - This is called on `false`
    /// - This is called on an empty collection
    /// - This is called on [`Option::None`](None)
    ///
    /// # Example
    ///
    /// ```
    /// # use argley::prelude::*;
    /// let mut command = std::process::Command::new("echo");
    ///
    /// assert!("foo".add_to("--arg1", &mut command));
    /// assert!(Some("bar").add_to("--arg2", &mut command));
    /// assert!(!None::<&'static str>.add_to("--arg3", &mut command));
    /// assert!(!false.add_to("--arg4", &mut command));
    /// assert!(true.add_to("--arg5", &mut command));
    ///
    /// let args = command.get_args().collect::<Vec<_>>();
    /// assert_eq!(&args[..], &["--arg1", "foo", "--arg2", "bar", "--arg5"]);
    /// ```
    fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
        consumer.add_arg(name);
        self.add_unnamed_to(consumer)
    }

    /// Add the value of this argument to the given [`ArgConsumer`].
    ///
    /// # Returns
    /// True if it's been added successfully, false otherwise. False is always returned if:
    ///
    /// - This is called on a boolean - those must be handled by [`add_to`](Arg::add_to)
    /// - This is called on an empty collection
    /// - This is called on [`Option::None`](None)
    ///
    /// # Example
    ///
    /// ```
    /// # use argley::prelude::*;
    /// let mut command = std::process::Command::new("echo");
    ///
    /// assert!("foo".add_unnamed_to(&mut command));
    /// assert!(Some("bar").add_unnamed_to(&mut command));
    /// assert!(!None::<&'static str>.add_unnamed_to(&mut command));
    /// assert!(!false.add_unnamed_to(&mut command));
    /// assert!(!true.add_unnamed_to(&mut command));
    ///
    /// let args = command.get_args().collect::<Vec<_>>();
    /// assert_eq!(&args[..], &["foo", "bar"]);
    /// ```
    fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool;
}

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

impl ArgConsumer for Vec<OsString> {
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

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{Arg, ArgConsumer};
}
