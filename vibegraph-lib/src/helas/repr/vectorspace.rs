//! Vector space trait and macros for algebraic structures.

use num_traits::Zero;
use std::ops::{Add, Div, Mul, Neg, Sub};

// TODO: in the future we could add a trait to track the raise/lower index structure
// Then covectors and vectors would be type-distinguished vector spaces.

/// Vector space over a real scalar field `F`.
///
/// Implementations must satisfy:
/// - Associativity: `(a + b) + c = a + (b + c)`
/// - Commutativity: `a + b = b + a`
/// - Identity: `a + zero() = a`
/// - Inverse: `a + (-a) = zero()`
/// - Distributivity: `α(a + b) = αa + αb` and `(α + β)a = αa + βa`
/// - Compatibility: `α(βa) = (αβ)a`
/// - Scalar identity: `1·a = a`
pub trait VectorSpace<F>:
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

/// Implement abelian group structure under + for a newtype wrapper around a fixed-size array.
///
/// Any type that has .0 as a fixed-size array of a field that has +, -, and
/// unary - can use this to get the corresponding implementations for free.
///
/// # Example
/// ```ignore
/// struct Blah<F: Real>([F; 4]);
/// impl_addition_for_array! { Blah<F>, 4 }
/// ```
macro_rules! impl_add_for_array {
    ($newtype:ident<$field:ident>, $len:expr) => {
        impl<$field: Real> std::ops::Add for $newtype<$field> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                $newtype(std::array::from_fn(|i| self.0[i] + rhs.0[i]))
            }
        }

        impl<$field: Real> std::ops::Sub for $newtype<$field> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                $newtype(std::array::from_fn(|i| self.0[i] - rhs.0[i]))
            }
        }

        impl<$field: Real> std::ops::Neg for $newtype<$field> {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self {
                $newtype(std::array::from_fn(|i| -self.0[i]))
            }
        }
    };
}

pub(super) use impl_add_for_array;

/// Implement multiplcation for a newtype wrapper around a fixed-size array.
///
/// This implements Mul<F> and Div<F> for any type that has .0 as a fixed-size
/// array of a scalar type (F or possibly over F) that itself has * and / available
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
    ($newtype:ident<$field:ident>, $scalar:ty, $len:expr) => {
        impl<$field: Real> std::ops::Mul<$scalar> for $newtype<$field> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: $scalar) -> Self {
                $newtype(std::array::from_fn(|i| self.0[i] * rhs))
            }
        }

        impl<$field: Real> std::ops::Div<$scalar> for $newtype<$field> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: $scalar) -> Self {
                $newtype(std::array::from_fn(|i| self.0[i] / rhs))
            }
        }
    };
}

pub(super) use impl_mul_for_array;
