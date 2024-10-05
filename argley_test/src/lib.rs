#[cfg(test)]
mod test {
    #![allow(dead_code)]

    use std::ffi::{OsStr, OsString};
    use std::fmt::{Display, Formatter};

    use derive_more::Display;
    use static_assertions::{assert_impl_one, assert_not_impl_all};

    use argley::prelude::*;
    use argley::CollectedArgs;

    type Str = &'static str;

    #[derive(Arg)]
    struct Nested(u8);

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
        struct Dropped(Str);

        #[derive(Arg)]
        struct Kept(Str);

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
        struct Newtype(Str);
        impl Newtype {
            fn args(&self) -> Str {
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

    mod static_args {
        use std::ffi::{OsStr, OsString};

        use argley::prelude::*;

        use super::Str;

        #[test]
        fn unit() {
            #[derive(Arg)]
            #[arg(static_args = ["foo", "bar"])]
            struct StaticUnit;

            let as_str = StaticUnit.collect_to::<OsString>();
            assert_eq!(as_str, OsStr::new("foo bar"));
        }

        #[test]
        fn newtype() {
            #[derive(Arg)]
            #[arg(static_args = ["--foo", "-bar"])]
            struct StaticNewtype(Str, Str);

            let as_str = StaticNewtype("baz", "qux").collect_to::<OsString>();
            assert_eq!(as_str, OsStr::new("--foo -bar baz qux"));
        }

        #[test]
        fn standard() {
            #[derive(Arg)]
            #[arg(static_args = ["x"])]
            struct Std {
                foo: u8,
                bar: Str,
            }

            let as_str = Std {
                foo: 42,
                bar: "baz",
            }
            .collect_to::<OsString>();
            assert_eq!(as_str, OsStr::new("x --foo 42 --bar baz"));
        }

        mod enums {
            use std::ffi::{OsStr, OsString};

            use argley::prelude::*;

            use super::super::{Nested, Str};

            #[derive(Arg)]
            #[arg(static_args = ["z"])]
            enum StaticArgsEnum {
                Unit,

                #[arg(value = "valun")]
                ValuedUnit,

                Tuple(Str, Nested),

                Named {
                    foo: Str,
                    bar: Str,
                },
            }

            #[test]
            fn unit() {
                let as_str = StaticArgsEnum::Unit.collect_to::<OsString>();
                assert_eq!(as_str, OsStr::new("z"));
            }

            #[test]
            fn valued_unit() {
                let as_str = StaticArgsEnum::ValuedUnit.collect_to::<OsString>();
                assert_eq!(as_str, OsStr::new("z valun"));
            }

            #[test]
            fn tuple() {
                let as_str = StaticArgsEnum::Tuple("666", Nested(42)).collect_to::<OsString>();
                assert_eq!(as_str, OsStr::new("z 666 42"));
            }

            #[test]
            fn named() {
                let as_str = StaticArgsEnum::Named {
                    foo: "you",
                    bar: "things",
                }
                .collect_to::<OsString>();
                assert_eq!(as_str, OsStr::new("z --foo you --bar things"));
            }
        }
    }

    mod enums {
        use std::borrow::Cow;
        use std::ffi::OsString;

        use argley::prelude::*;
        use argley::CollectedArgs;

        use super::{Nested, Str};

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
            Tuple(Str, #[arg(rename = "some-num")] u8, u8),
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

        #[derive(Arg)]
        #[allow(unused)]
        enum NoVariants {}

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
