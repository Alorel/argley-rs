#[cfg(test)]
mod test {
    use argley::prelude::*;

    #[test]
    fn unit_struct() {
        #[derive(Arg)]
        struct Unit;

        let mut result = Vec::new();
        assert!(!Unit.add_unnamed_to(&mut result));
        assert!(result.is_empty());
    }

    #[test]
    fn no_props() {
        #[derive(Arg)]
        struct NoProps {}

        let mut result = Vec::new();
        assert!(!NoProps {}.add_unnamed_to(&mut result));
        assert!(result.is_empty());
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

        let mut result = Vec::new();

        assert!(WithFormatter(Newtype("formatter-test")).add_unnamed_to(&mut result));
        assert_eq!(&result[..], &["formatter-test"]);
    }

    mod r#enum {
        use std::ffi::OsString;

        use argley::prelude::*;

        #[derive(Arg)]
        struct Nested(u8);

        #[derive(Arg)]
        enum Foo {
            Unit,
            Tuple(&'static str, #[arg(rename = "some-num")] u8, u8),
            Named {
                #[arg(position = 0)]
                a: Nested,

                #[arg(skip)]
                _b: u8,

                #[arg(short)]
                c: u8,
            },
            NamedNoSkip {
                #[arg(variadic)]
                a: u8,
            },
        }

        #[test]
        fn named() {
            let mut result = Vec::new();
            assert!(Foo::Named {
                a: Nested(10),
                _b: 20,
                c: 30,
            }
            .add_unnamed_to(&mut result));

            assert_eq!(&result[..], &["-c", "30", "10"]);
        }

        #[test]
        fn tuple() {
            let mut result = Vec::new();
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
            let mut result = Vec::new();
            assert!(Foo::NamedNoSkip { a: 0 }.add_unnamed_to(&mut result));
            assert_eq!(&result[..], &["0"]);
        }

        #[test]
        fn unit() {
            let mut result = Vec::<OsString>::new();
            assert!(!Foo::Unit.add_unnamed_to(&mut result));
            assert!(result.is_empty());
        }
    }
}
