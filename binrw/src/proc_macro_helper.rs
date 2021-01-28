// code to impl trait if types are equal, taken from type_eq
// https://docs.rs/type_eq/0.1.2/type_eq/
pub struct Constrain;

/// A trait that determines that two type parameters are the same type.
/// There is only one implementation of this trait, `Constrain`, which ensures that all types
/// satisfy the condition of being equal to themselves.
pub unsafe trait TypeEq<A, B>: private::Sealed {}
unsafe impl<A> TypeEq<A, A> for Constrain {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Constrain {}
}

// code to generate good error message from const bool: thanks dtolnay
// https://github.com/dtolnay/case-studies/blob/master/bitfield-assertion/README.md
pub trait HasMarker {
    type Marker;
}

pub enum DifferentTypes {}
pub enum SameType {}

impl HasMarker for [(); 0] {
    type Marker = DifferentTypes;
}

impl HasMarker for [(); 1] {
    type Marker = SameType;
}

pub trait TypesNotEqual<A, B> {
    type Check;
}

impl<A, B> TypesNotEqual<A, B> for DifferentTypes {
    type Check = ();
}

pub type TypeNe<A, B, T> = <<T as HasMarker>::Marker as TypesNotEqual<A, B>>::Check;

#[macro_export]
#[doc(hidden)]
// Helper macro used by the proc macro for a certain kind of error reporting
macro_rules! require_types_not_equal {
    ($a:ty, $b:ty) => {
        let _: $crate::proc_macro_helper::TypeNe<$a, $b, [(); {
            // code to generate const bool depending on if trait is implementated or not, taken from impls
            // https://docs.rs/impls/1.0.3/impls/

            // Do not import types in order to prevent trait name collisions.

            /// Fallback trait with `False` for `IMPLS` if the type does not
            /// implement the given trait.
            trait DoesNotImpl {
                const IMPLS: usize = 0;
            }
            impl<T: ?Sized> DoesNotImpl for T {}

            /// Concrete type with `True` for `IMPLS` if the type implements the
            /// given trait. Otherwise, it falls back to `DoesNotImpl`.
            struct Wrapper<T: ?Sized>(::core::marker::PhantomData<T>);

            #[allow(dead_code)]
            impl<T: ?Sized + $crate::proc_macro_helper::TypeEq<$a, $b>> Wrapper<T> {
                const IMPLS: usize = 1;
            }

            <Wrapper<$crate::proc_macro_helper::Constrain>>::IMPLS
        }]>;
    }
}