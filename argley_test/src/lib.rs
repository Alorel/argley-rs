#[cfg(test)]
mod test {
    use std::ffi::{OsStr, OsString};
    use std::fmt::{Display, Formatter};

    use derive_more::Display;
    use static_assertions::{assert_impl_one, assert_not_impl_all};

    use argley::prelude::*;
    use argley::CollectedArgs;

    #[test]
    fn unit_struct() {
        #[derive(Arg)]
        struct Unit;

        let mut result = CollectedArgs::new();
        assert!(!Unit.add_unnamed_to(&mut result), "add_unnamed_to");
        assert!(result.is_empty(), "is_empty");

        assert_eq!(result, Unit.collect_to::<CollectedArgs>(), "collect_to");
    }

    #[test]
    fn no_props() {
        #[derive(Arg)]
        struct NoProps {}

        let mut result = CollectedArgs::new();
        assert!(!NoProps {}.add_unnamed_to(&mut result), "add_unnamed_to");
        assert!(result.is_empty(), "is_empty");

        assert_eq!(
            result,
            NoProps {}.collect_to::<CollectedArgs>(),
            "collect_to"
        );
    }

    #[test]
    fn drop_name() {
        #[derive(Arg)]
        #[arg(drop_name)]
        struct Dropped(&'static str);

        #[derive(Arg)]
        struct Kept(&'static str);

        #[derive(Arg)]
        struct TopLevel {
            dropped: Dropped,
            kept: Kept,
        }

        let mut result = CollectedArgs::new();
        let toplevel = TopLevel {
            dropped: Dropped("d"),
            kept: Kept("k"),
        };

        assert!(toplevel.add_unnamed_to(&mut result), "add_unnamed_to");
        assert_eq!(&result[..], &["d", "--kept", "k"], "result");
        assert_eq!(result, toplevel.collect_to::<CollectedArgs>(), "collect_to");
    }

    #[test]
    fn formatter() {
        struct Newtype(&'static str);
        impl Newtype {
            fn args(&self) -> &'static str {
                self.0
            }
        }

        #[derive(Arg)]
        struct WithFormatter(#[arg(formatter = Newtype::args)] Newtype);

        let mut result = CollectedArgs::new();
        let source = WithFormatter(Newtype("formatter-test"));

        assert!(source.add_unnamed_to(&mut result), "add_unnamed_to");
        assert_eq!(&result[..], &["formatter-test"], "result");
        assert_eq!(result, source.collect_to::<CollectedArgs>(), "collect_to");
    }

    #[test]
    fn to_string() {
        #[derive(Arg, Display)]
        #[arg(to_string)]
        #[display(fmt = "foo: {_0}")]
        struct Newtype(u8);

        #[derive(Arg)]
        #[arg(to_string)]
        struct Wrapper<T>(T);
        impl<T: Display> Display for Wrapper<T> {
            fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
                Ok(())
            }
        }

        #[allow(dead_code)]
        struct NonDisplay;

        let mut result = CollectedArgs::new();
        Newtype(42).add_unnamed_to(&mut result);
        assert_eq!(&result[..], &["foo: 42"]);

        assert_not_impl_all!(Wrapper<NonDisplay>: Arg);
        assert_impl_one!(Wrapper<u8>: Arg);
    }

    #[test]
    fn collect_to_string() {
        let args = ["--foo", "bar", "--qux", "baz"];
        let as_str = args.collect_to::<OsString>();

        assert_eq!(as_str, OsStr::new("--foo bar --qux baz"));
    }

    mod enums {
        use std::borrow::Cow;
        use std::ffi::OsString;

        use argley::prelude::*;
        use argley::CollectedArgs;

        #[derive(Arg)]
        struct Nested(u8);

        /// CI test: we error out on warnings and we're checking that we don't produce an unused
        /// fn argument here.
        #[derive(Arg)]
        enum CompileTestEnum1 {
            #[allow(dead_code)]
            Variant,
        }

        #[derive(Arg)]
        enum Foo {
            Unit,

            #[arg(value = &Nested(100))]
            ValuedUnit,
            Tuple(&'static str, #[arg(rename = "some-num")] u8, u8),
            Named {
                #[arg(position = 0)]
                a: Nested,

                #[arg(skip)]
                _b: u8,

                #[arg(short)]
                c: Cow<'static, str>,
            },
            NamedNoSkip {
                #[arg(variadic)]
                a: u8,
            },
        }

        #[test]
        fn named() {
            let mut result = CollectedArgs::new();
            assert!(Foo::Named {
                a: Nested(10),
                _b: 20,
                c: "sea".into(),
            }
            .add_unnamed_to(&mut result));

            assert_eq!(&result[..], &["-c", "sea", "10"]);
        }

        #[test]
        fn tuple() {
            let mut result = CollectedArgs::new();
            assert!(Foo::Tuple("foo", 0, 1).add_unnamed_to(&mut result));
            assert_eq!(&result[..], &["--some-num", "0", "foo", "1"]);
        }

        #[test]
        fn empty() {
            #[derive(Arg)]
            enum Foo {}
        }

        #[test]
        fn named_no_skip() {
            let mut result = CollectedArgs::new();
            assert!(Foo::NamedNoSkip { a: 0 }.add_unnamed_to(&mut result));
            assert_eq!(&result[..], &["0"]);
        }

        #[test]
        fn unit() {
            let mut result = Vec::<OsString>::new();
            assert!(!Foo::Unit.add_unnamed_to(&mut result));
            assert!(result.is_empty());
        }

        #[test]
        fn unit_valued() {
            let mut result = CollectedArgs::new();
            assert!(Foo::ValuedUnit.add_unnamed_to(&mut result));
            assert_eq!(&result[..], &["100"]);
        }

        #[test]
        fn as_repr() {
            #[derive(Arg)]
            #[arg(as_repr)]
            #[repr(u8)]
            #[derive(Copy, Clone)]
            enum AsReprEnum {
                _A = 10,
                B = 20,
            }

            let result = AsReprEnum::B.collect_to::<CollectedArgs>();
            assert_eq!(&result[..], &["20"]);
        }
    }
}
