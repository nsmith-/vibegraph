//! Vector space trait and macros for algebraic structures.

use num_traits::Zero;
use std::ops::{Add, Div, Mul, Neg, Sub};

// TODO: in the future we could add a trait to track the raise/lower index structure
// Then covectors and vectors would be type-distinguished vector spaces.

/// Vector space over a scalar field `F`.
///
/// Implementations must satisfy:
/// - Associativity: `(a + b) + c = a + (b + c)`
/// - Commutativity: `a + b = b + a`
/// - Identity: `a + zero() = a`
/// - Inverse: `a + (-a) = zero()`
/// - Distributivity: `α(a + b) = αa + αb` and `(α + β)a = αa + βa`
/// - Compatibility: `α(βa) = (αβ)a`
/// - Scalar identity: `1·a = a`
#[allow(dead_code)] // Future work may use this to implement e.g. inner product spaces
pub(super) trait VectorSpace<F: Copy>:
    Add<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
    + Mul<F, Output = Self>
    + Div<F, Output = Self>
    + Zero
    + Copy
    + Clone
    + Sized
    + 'static
{
}

/// Helper for the VectorSpace impl macros to get the underlying array from a newtype wrapper.
pub(super) trait ArrayBacked<F: Copy, const N: usize> {
    fn as_array(&self) -> &[F; N];
    fn from_array(arr: [F; N]) -> Self;
}

/// Implement abelian group structure under + for a newtype wrapper around a fixed-size array.
///
/// Any type that has ArrayBacked<F, N> and implements Add, Sub, and Neg for the
/// underlying array type can use this to get the corresponding implementations
///
/// The capture in brackets allows to specify the generic bounds for the newtype
///
/// # Example:
/// ```ignore
/// impl_add_for_array!(impl[F: Real] MyVector<F>, scalar = F, len = 4);
/// ```
macro_rules! impl_add_for_array {
    (impl[$($gen:tt)*] $newtype:ty) => {
        impl<$($gen)*> std::ops::Add for $newtype {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                let lhs = self.as_array();
                let rhs = rhs.as_array();
                <$newtype>::from_array(std::array::from_fn(|i| lhs[i] + rhs[i]))
            }
        }

        impl<$($gen)*> std::ops::Sub for $newtype {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                let lhs = self.as_array();
                let rhs = rhs.as_array();
                <$newtype>::from_array(std::array::from_fn(|i| lhs[i] - rhs[i]))
            }
        }

        impl<$($gen)*> std::ops::Neg for $newtype {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self {
                let arr = self.as_array();
                <$newtype>::from_array(std::array::from_fn(|i| -arr[i]))
            }
        }
    };
}

pub(super) use impl_add_for_array;

/// Implement multiplcation for a newtype wrapper around a fixed-size array.
///
/// This implements Mul<F> and Div<F> for any type that implements
/// ArrayBacked<F, N> and has * and / defined for the underlying array type and
/// the scalar type F.
///
/// We purposefully only implement right multiplication by a scalar to encourage
/// putting lighter-weight operations at the end.
/// Note Rust is left-associative (i.e. a * b * c = (a * b) * c if types allow)
/// so we might prefer to only have left multiplication, but division must be on
/// right, so better to group things that way, i.e. the types force
/// v * s1 / s2 = v * ( s1 / s2 )
///
/// TODO: revisit this decision
macro_rules! impl_mul_for_array {
    (impl[$($gen:tt)*] $newtype:ty, scalar = $scalar:ty) => {
        impl<$($gen)*> std::ops::Mul<$scalar> for $newtype {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: $scalar) -> Self {
                let arr = self.as_array();
                <$newtype>::from_array(std::array::from_fn(|i| arr[i] * rhs))
            }
        }

        impl<$($gen)*> std::ops::Div<$scalar> for $newtype {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: $scalar) -> Self {
                let arr = self.as_array();
                <$newtype>::from_array(std::array::from_fn(|i| arr[i] / rhs))
            }
        }
    };
}

pub(super) use impl_mul_for_array;

/// Blanket macro to implement the full VectorSpace trait for a newtype wrapper around a fixed-size array.
///
/// The newtype must implement ArrayBacked<F, N> to allow the macro to get the
/// underlying array for the impls, and to construct new instances from arrays.
macro_rules! impl_vectorspace {
    (impl[$($gen:tt)*] $newtype:ty, scalar = $scalar:ty) => {
        crate::helas::repr::vectorspace::impl_add_for_array!(impl[$($gen)*] $newtype);
        crate::helas::repr::vectorspace::impl_mul_for_array!(impl[$($gen)*] $newtype, scalar = $scalar);

        impl<$($gen)*> Zero for $newtype {
            #[inline(always)]
            fn zero() -> Self {
                <$newtype>::from_array(std::array::from_fn(|_| <$scalar>::zero()))
            }

            #[inline(always)]
            fn is_zero(&self) -> bool {
                let arr = self.as_array();
                arr.iter().all(|&x| x.is_zero())
            }
        }

        impl<$($gen)*> crate::helas::repr::vectorspace::VectorSpace<$scalar> for $newtype {}
    };
}

pub(super) use impl_vectorspace;
