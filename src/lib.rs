//! Turn a struct into arguments for a [`Command`](::std::process::Command). See the
//! [derive macro](argley_macro::Arg) for options you can pass in.
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
//!
//! Support for [`async-std`](async_std) and [`tokio`] can be enabled via their respective features.

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub use arg::Arg;
pub use arg_consumer::{ArgConsumer, CollectedArgs};

mod arg;
mod arg_consumer;
mod arg_impls;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::Arg;
    pub use crate::ArgConsumer;
}
