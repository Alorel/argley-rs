use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::{Arg, ArgConsumer};

// ========== MACROS ==========

macro_rules! impl_to_string {
    ($($ty: ty) +) => {
        $(
            impl Arg for $ty {
                fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
                    consumer.add_arg(self.to_string());
                    true
                }
            }
        )+
    };
}

macro_rules! impl_direct {
    ($([$owned: ty, $borrowed: ty]) +) => {
        $(
           impl Arg for $borrowed {
               #[inline]
               fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
                   consumer.add_arg(self);
                   true
               }
           }
           impl Arg for &$borrowed {
               #[inline]
               fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
                   <$borrowed>::add_unnamed_to(*self, consumer)
               }
           }
           impl Arg for $owned {
               #[inline]
               fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
                   <$borrowed>::add_unnamed_to(self, consumer)
               }
           }
        )+
    };
}

macro_rules! impl_slice_body {
    () => {
        #[inline]
        fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
            self.as_slice().add_to(name, consumer)
        }

        #[inline]
        fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
            self.as_slice().add_unnamed_to(consumer)
        }
    };
}

macro_rules! impl_deref {
    (body) => {
        #[inline]
        fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
            Arg::add_to(&**self, name, consumer)
        }

        #[inline]
        fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
            Arg::add_unnamed_to(&**self, consumer)
        }
    };
    ($($ty: ty) +) => {
        $(
            impl<T: Arg> Arg for $ty {
                impl_deref!(body);
            }
        )+
    };
}

macro_rules! impl_iter {
    (body => $check: ident) => {
        fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
            if self.$check() {
               false
            } else {
                consumer.add_arg(name);
                process_iter(self, consumer)
            }
        }

        fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
            if self.$check() {
               false
            } else {
                process_iter(self, consumer)
            }
        }
    };
    ([$check: ident] => $($ty: ty) +) => {
        $(
          impl<T: Arg> Arg for $ty {
              impl_iter!(body => $check);
          }
        )+
    };
}

// ========== MACRO_CALLS ==========

impl_to_string!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize f32 f64);
impl_direct!([String, str] [PathBuf, Path] [OsString, OsStr]);
impl_deref!(Box<T> Rc<T> Arc<T>);
impl_iter!([is_empty] => [T] BTreeSet<T> LinkedList<T> VecDeque<T>);

// ========== CUSTOM_IMPLS ==========

impl<T: Arg> Arg for Vec<T> {
    impl_slice_body!();
}

impl<T: Arg, const N: usize> Arg for [T; N] {
    impl_slice_body!();
}

impl<T: Arg, S> Arg for HashSet<T, S> {
    impl_iter!(body => is_empty);
}

impl<'a, T: ?Sized + Arg + ToOwned + 'a> Arg for Cow<'a, T> {
    impl_deref!(body);
}

impl<T: Arg> Arg for Option<T> {
    fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
        if let Some(value) = self {
            Arg::add_to(value, name, consumer)
        } else {
            false
        }
    }

    fn add_unnamed_to(&self, consumer: &mut impl ArgConsumer) -> bool {
        if let Some(value) = self {
            Arg::add_unnamed_to(value, consumer)
        } else {
            false
        }
    }
}

impl Arg for bool {
    fn add_to(&self, name: &str, consumer: &mut impl ArgConsumer) -> bool {
        if *self {
            consumer.add_arg(name);
            true
        } else {
            false
        }
    }

    #[inline]
    fn add_unnamed_to(&self, _: &mut impl ArgConsumer) -> bool {
        false
    }
}

// ========== HELPERS ==========

fn process_iter<'a, E, I>(iter: I, consumer: &mut impl ArgConsumer) -> bool
where
    E: Arg + 'a + ?Sized,
    I: IntoIterator<Item = &'a E> + Copy,
{
    for element in iter {
        element.add_unnamed_to(consumer);
    }
    true
}
