use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use crate::{uint::Uint, utils::BitIterator, Vec};

#[macro_use]
pub mod macros;
pub mod utils;

use utils::k_adicity;

#[macro_use]
pub mod arithmetic;

pub mod models;
pub use self::models::*;

#[macro_export]
macro_rules! field_new {
    ($name:ident, $c0:expr) => {
        $name { 0: $c0 }
    };
    ($name:ident, $c0:expr, $c1:expr $(,)?) => {
        $name { c0: $c0, c1: $c1 }
    };
    ($name:ident, $c0:expr, $c1:expr, $c2:expr $(,)?) => {
        $name {
            c0: $c0,
            c1: $c1,
            c2: $c2,
        }
    };
}

/// The interface for a generic field.
pub trait Field:
    'static
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Eq
    + Ord
    + Neg<Output = Self>
    + Sized
    + Hash
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
    + for<'a> DivAssign<&'a Self>
    + core::iter::Sum<Self>
    + for<'a> core::iter::Sum<&'a Self>
    + core::iter::Product<Self>
    + for<'a> core::iter::Product<&'a Self>
{
    /// Returns an element chosen uniformly at random using a user-provided RNG.
    fn random<R: rand_core::RngCore + ?Sized>(rng: &mut R) -> Self;

    /// Returns a field element if the set of bytes forms a valid field element,
    /// otherwise returns None. This function is primarily intended for sampling
    /// random field elements from a hash-function or RNG output.
    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::from_random_bytes_with_flags(bytes).map(|f| f.0)
    }

    /// Returns a field element with an extra sign bit used for group parsing if
    /// the set of bytes forms a valid field element, otherwise returns
    /// None. This function is primarily intended for sampling
    /// random field elements from a hash-function or RNG output.
    fn from_random_bytes_with_flags(bytes: &[u8]) -> Option<(Self, u8)>;

    /// Returns the zero element of the field, the additive identity.
    fn zero() -> Self;

    /// Returns the one element of the field, the multiplicative identity.
    fn one() -> Self;

    /// Returns true iff this element is zero.
    fn is_zero(&self) -> bool;

    /// Returns true iff this element is one.
    fn is_one(&self) -> bool;

    /// Returns the characteristic of the field.
    fn characteristic<'a>() -> &'a [u64];

    /// Returns `self + self`.
    fn double(&self) -> Self;

    /// Doubles `self` in place.
    fn double_assign(&mut self) -> &mut Self;

    /// Returns `self * self`.
    fn square(&self) -> Self;

    /// Squares `self` in place.
    fn square_assign(&mut self) -> &mut Self;

    /// Computes the multiplicative inverse of `self` if `self` is nonzero.
    fn inverse(&self) -> Option<Self>;

    // Sets `self` to `self`'s inverse if it exists. Otherwise it is a no-op.
    fn inverse_assign(&mut self) -> Option<&mut Self>;

    /// Exponentiates this element by a power of the base prime modulus via
    /// the Frobenius automorphism.
    fn frobenius_map(&mut self, power: usize);

    /// Exponentiates this element by a number represented with `u64` limbs,
    /// least significant limb first.
    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::one();

        let mut found_one = false;

        for i in BitIterator::new(exp) {
            if found_one {
                res.square_assign();
            } else {
                found_one = i;
            }

            if i {
                res *= self;
            }
        }
        res
    }
}

/// A trait that defines parameters for a field that can be used for FFTs.
pub trait FftParameters: 'static + Send + Sync + Sized {
    type BigInt: Uint;

    /// Let `N` be the size of the multiplicative group defined by the field.
    /// Then `TWO_ADICITY` is the two-adicity of `N`, i.e. the integer `s`
    /// such that `N = 2^s * t` for some odd integer `t`.
    const TWO_ADICITY: u32;

    /// 2^s root of unity computed by GENERATOR^t
    const TWO_ADIC_ROOT_OF_UNITY: Self::BigInt;

    /// An integer `b` such that there exists a multiplicative subgroup
    /// of size `b^k` for some integer `k`.
    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    /// The integer `k` such that there exists a multiplicative subgroup
    /// of size `Self::SMALL_SUBGROUP_BASE^k`.
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    /// GENERATOR^((MODULUS-1) / (2^s *
    /// SMALL_SUBGROUP_BASE^SMALL_SUBGROUP_BASE_ADICITY)) Used for mixed-radix FFT.
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self::BigInt> = None;
}

/// A trait that defines parameters for a prime field.
pub trait FpParameters: FftParameters {
    /// The modulus of the field.
    const MODULUS: Self::BigInt;

    /// The number of bits needed to represent the `Self::MODULUS`.
    const MODULUS_BITS: u32;

    /// The number of bits that must be shaved from the beginning of
    /// the representation when randomly sampling.
    const REPR_SHAVE_BITS: u32;

    /// Let `M` be the power of 2^64 nearest to `Self::MODULUS_BITS`. Then
    /// `R = M % Self::MODULUS`.
    const R: Self::BigInt;

    /// R2 = R^2 % Self::MODULUS
    const R2: Self::BigInt;

    /// INV = -MODULUS^{-1} mod 2^64
    const INV: u64;

    /// A multiplicative generator of the field.
    /// `Self::GENERATOR` is an element having multiplicative order
    /// `Self::MODULUS - 1`.
    const GENERATOR: Self::BigInt;

    /// The number of bits that can be reliably stored.
    /// (Should equal `SELF::MODULUS_BITS - 1`)
    const CAPACITY: u32;

    /// t for 2^s * t = MODULUS - 1
    const T: Self::BigInt;

    /// (t - 1) / 2
    const T_MINUS_ONE_DIV_TWO: Self::BigInt;

    /// (Self::MODULUS - 1) / 2
    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt;
}

/// The interface for fields that are able to be used in FFTs.
pub trait FftField: Field + From<u128> + From<u64> + From<u32> + From<u16> + From<u8> {
    type FftParams: FftParameters;

    /// Returns the 2^s root of unity.
    fn two_adic_root_of_unity() -> Self;

    /// Returns the 2^s * small_subgroup_base^small_subgroup_base_adicity root of unity
    /// if a small subgroup is defined.
    fn large_subgroup_root_of_unity() -> Option<Self>;

    /// Returns the multiplicative generator of `char()` - 1 order.
    fn multiplicative_generator() -> Self;

    /// Returns the root of unity of order n, if one exists.
    /// If no small multiplicative subgroup is defined, this is the 2-adic root of unity of order n
    /// (for n a power of 2).
    /// If a small multiplicative subgroup is defined, this is the root of unity of order n for
    /// the larger subgroup generated by `FftParams::LARGE_SUBGROUP_ROOT_OF_UNITY`
    /// (for n = 2^i * FftParams::SMALL_SUBGROUP_BASE^j for some i, j).
    fn get_root_of_unity(n: usize) -> Option<Self> {
        let mut omega: Self;
        if let Some(large_subgroup_root_of_unity) = Self::large_subgroup_root_of_unity() {
            let q = Self::FftParams::SMALL_SUBGROUP_BASE.expect(
                "LARGE_SUBGROUP_ROOT_OF_UNITY should only be set in conjunction with SMALL_SUBGROUP_BASE",
            ) as usize;
            let small_subgroup_base_adicity = Self::FftParams::SMALL_SUBGROUP_BASE_ADICITY.expect(
                "LARGE_SUBGROUP_ROOT_OF_UNITY should only be set in conjunction with SMALL_SUBGROUP_BASE_ADICITY",
            );

            let q_adicity = k_adicity(q, n);
            let q_part = q.pow(q_adicity);

            let two_adicity = k_adicity(2, n);
            let two_part = 1 << two_adicity;

            if n != two_part * q_part
                || (two_adicity > Self::FftParams::TWO_ADICITY)
                || (q_adicity > small_subgroup_base_adicity)
            {
                return None;
            }

            omega = large_subgroup_root_of_unity;
            for _ in q_adicity..small_subgroup_base_adicity {
                omega = omega.pow(&[q as u64]);
            }

            for _ in two_adicity..Self::FftParams::TWO_ADICITY {
                omega.square_in_place();
            }
        } else {
            // Compute the next power of 2.
            let size = n.next_power_of_two() as u64;
            let log_size_of_group = size.trailing_zeros();

            if n != size as usize || log_size_of_group > Self::FftParams::TWO_ADICITY {
                return None;
            }

            // Compute the generator for the multiplicative subgroup.
            // It should be 2^(log_size_of_group) root of unity.
            omega = Self::two_adic_root_of_unity();
            for _ in log_size_of_group..Self::FftParams::TWO_ADICITY {
                omega.square_in_place();
            }
        }
        Some(omega)
    }
}

/// The interface for a prime field.
pub trait PrimeField:
    FftField<FftParams = <Self as PrimeField>::Params>
    + FromStr
    + From<<Self as PrimeField>::BigInt>
    + Into<<Self as PrimeField>::BigInt>
{
    type Params: FpParameters<BigInt = Self::BigInt>;
    type BigInt: Uint;

    /// Returns a prime field element from its underlying representation.
    fn from_repr(repr: Self::BigInt) -> Option<Self>;

    /// Returns the underlying representation of the prime field element.
    fn into_repr(&self) -> Self::BigInt;

    /// Return the a QNR^T
    fn qnr_to_t() -> Self {
        Self::two_adic_root_of_unity()
    }

    /// Returns the field size in bits.
    fn size_in_bits() -> usize {
        Self::Params::MODULUS_BITS as usize
    }

    /// Returns the trace.
    fn trace() -> Self::BigInt {
        Self::Params::T
    }

    /// Returns the trace minus one divided by two.
    fn trace_minus_one_div_two() -> Self::BigInt {
        Self::Params::T_MINUS_ONE_DIV_TWO
    }

    /// Returns the modulus minus one divided by two.
    fn modulus_minus_one_div_two() -> Self::BigInt {
        Self::Params::MODULUS_MINUS_ONE_DIV_TWO
    }
}

/// The interface for a field that supports an efficient square-root operation.
pub trait SquareRootField: Field {
    /// Returns the Legendre symbol.
    fn legendre(&self) -> LegendreSymbol;

    /// Returns the square root of self, if it exists.
    #[must_use]
    fn sqrt(&self) -> Option<Self>;

    /// Sets `self` to be the square root of `self`, if it exists.
    fn sqrt_in_place(&mut self) -> Option<&mut Self>;
}

#[derive(Debug, PartialEq)]
pub enum LegendreSymbol {
    Zero = 0,
    QuadraticResidue = 1,
    QuadraticNonResidue = -1,
}

impl LegendreSymbol {
    pub fn is_zero(&self) -> bool {
        *self == LegendreSymbol::Zero
    }

    pub fn is_qnr(&self) -> bool {
        *self == LegendreSymbol::QuadraticNonResidue
    }

    pub fn is_qr(&self) -> bool {
        *self == LegendreSymbol::QuadraticResidue
    }
}

use crate::uint::U256;

impl_field_bigint_conv!(Fp256, U256, Fp256Parameters);
//impl_field_bigint_conv!(Fp320, U320, Fp320Parameters);
//impl_field_bigint_conv!(Fp384, U384, Fp384Parameters);
//impl_field_bigint_conv!(Fp768, U768, Fp768Parameters);
//impl_field_bigint_conv!(Fp832, U832, Fp832Parameters);

pub fn batch_inversion<F: Field>(v: &mut [F]) {
    // Montgomery’s Trick and Fast Implementation of Masked AES
    // Genelle, Prouff and Quisquater
    // Section 3.2

    // First pass: compute [a, ab, abc, ...]
    let mut prod = Vec::with_capacity(v.len());
    let mut tmp = F::one();
    for f in v.iter().filter(|f| !f.is_zero()) {
        tmp.mul_assign(f);
        prod.push(tmp);
    }

    // Invert `tmp`.
    tmp = tmp.inverse().unwrap(); // Guaranteed to be nonzero.

    // Second pass: iterate backwards to compute inverses
    for (f, s) in v
        .iter_mut()
        // Backwards
        .rev()
        // Ignore normalized elements
        .filter(|f| !f.is_zero())
        // Backwards, skip last element, fill in one for last term.
        .zip(prod.into_iter().rev().skip(1).chain(Some(F::one())))
    {
        // tmp := tmp * f; f := tmp * s = 1/f
        let new_tmp = tmp * *f;
        *f = tmp * &s;
        tmp = new_tmp;
    }
}
