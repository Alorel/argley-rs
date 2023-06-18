use crate::ArgConsumer;

#[cfg(feature = "derive")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "derive")))]
pub use argley_macro::Arg;

/// An argument that can be passed to an [`ArgConsumer`] such as a [`Command`](std::process::Command).
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

    /// Shorthand for creating an [`ArgConsumer`], passing it to
    /// [`add_unnamed_to`](Arg::add_unnamed_to) and returning it.
    ///
    /// # Example
    ///
    /// ```
    /// # use {argley::prelude::*, std::ffi::OsString};
    ///
    /// #[derive(Arg)]
    /// struct Args {
    ///   foo: &'static str,
    /// }
    ///
    /// let args = Args { foo: "bar" };
    ///
    /// let collect = args.collect_to::<Vec<OsString>>();
    /// let is_eq_to: Vec<OsString> = vec!["--foo".into(), "bar".into()];
    ///
    /// assert_eq!(collect, is_eq_to);
    /// ```
    fn collect_to<C: Default + ArgConsumer>(&self) -> C {
        let mut consumer = C::default();
        self.add_unnamed_to(&mut consumer);
        consumer
    }
}
