//! # Galois Field GF(2^32) Implementation
//!
//! This module implements arithmetic operations in the Galois Field GF(2^32),
//! which is the mathematical foundation for Phase 1 (AHGF).
//!
//! ## Mathematical Background
//!
//! GF(2^32) is a finite field with 2^32 elements, where:
//! - Addition is XOR (⊕)
//! - Multiplication is polynomial multiplication modulo an irreducible polynomial
//! - The field has characteristic 2
//!
//! ## Irreducible Polynomial
//!
//! We use the polynomial: P(x) = x^32 + x^7 + x^3 + x^2 + 1
//! Binary representation: 0x10000008D (bit 32, 7, 3, 2, 0 set)

use std::ops::{Add, Mul, Sub};

/// Galois Field GF(2^32) element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GF2_32(pub u32);

/// Irreducible polynomial for GF(2^32): x^32 + x^7 + x^3 + x^2 + 1
const IRREDUCIBLE_POLY: u64 = 0x10000008D;

impl GF2_32 {
    /// Create new GF(2^32) element
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Zero element
    pub fn zero() -> Self {
        Self(0)
    }

    /// One element (multiplicative identity)
    pub fn one() -> Self {
        Self(1)
    }

    /// Additive inverse (in GF(2^n), x + x = 0, so -x = x)
    pub fn neg(self) -> Self {
        self
    }

    /// Multiplicative inverse using Extended Euclidean Algorithm
    pub fn inv(self) -> Option<Self> {
        if self.0 == 0 {
            return None;
        }

        // Extended Euclidean Algorithm in GF(2^32)
        let mut t = 0u32;
        let mut new_t = 1u32;
        let mut r = IRREDUCIBLE_POLY as u32;
        let mut new_r = self.0;

        while new_r != 0 {
            let (quotient, remainder) = gf_div_mod(r, new_r);
            
            let temp_t = t ^ gf_mul_simple(quotient, new_t);
            t = new_t;
            new_t = temp_t;

            r = new_r;
            new_r = remainder;
        }

        if r > 1 {
            None // Not invertible
        } else {
            Some(Self(t))
        }
    }

    /// Power operation: self^exp
    pub fn pow(self, mut exp: u32) -> Self {
        if exp == 0 {
            return Self::one();
        }

        let mut result = Self::one();
        let mut base = self;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exp >>= 1;
        }

        result
    }

    /// Frobenius automorphism: φ(x) = x^(2^k)
    pub fn frobenius(self, k: u32) -> Self {
        self.pow(1u32 << k)
    }

    /// Get raw value
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Addition in GF(2^32) is XOR
impl Add for GF2_32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

/// Subtraction in GF(2^32) is also XOR (since -x = x in characteristic 2)
impl Sub for GF2_32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

/// Multiplication in GF(2^32)
impl Mul for GF2_32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(gf_mul(self.0, other.0))
    }
}

/// Multiply two GF(2^32) elements
fn gf_mul(a: u32, b: u32) -> u32 {
    let mut result = 0u64;
    let mut temp_a = a as u64;
    let mut temp_b = b as u64;

    for _ in 0..32 {
        if temp_b & 1 == 1 {
            result ^= temp_a;
        }
        temp_b >>= 1;
        temp_a <<= 1;
    }

    // Reduce modulo irreducible polynomial
    for i in (32..64).rev() {
        if result & (1u64 << i) != 0 {
            result ^= IRREDUCIBLE_POLY << (i - 32);
        }
    }

    result as u32
}

/// Simple multiplication without reduction (for internal use)
fn gf_mul_simple(a: u32, b: u32) -> u32 {
    let mut result = 0u32;
    let mut temp_a = a;
    let mut temp_b = b;

    for _ in 0..32 {
        if temp_b & 1 == 1 {
            result ^= temp_a;
        }
        temp_b >>= 1;
        temp_a = temp_a.wrapping_shl(1);
    }

    result
}

/// Division with remainder in GF(2^32)
fn gf_div_mod(dividend: u32, divisor: u32) -> (u32, u32) {
    if divisor == 0 {
        return (0, dividend);
    }

    let mut quotient = 0u32;
    let mut remainder = dividend;

    let divisor_degree = 31 - divisor.leading_zeros();

    for i in (0u32..=31).rev() {
        if remainder & (1 << i) != 0 {
            let shift = i.saturating_sub(divisor_degree);
            quotient |= 1 << shift;
            remainder ^= divisor << shift;
        }
    }

    (quotient, remainder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let a = GF2_32::new(0x12345678);
        let b = GF2_32::new(0x87654321);
        let c = a + b;
        assert_eq!(c.0, 0x12345678 ^ 0x87654321);
    }

    #[test]
    fn test_subtraction() {
        let a = GF2_32::new(0x12345678);
        let b = GF2_32::new(0x87654321);
        let c = a - b;
        // In GF(2^n), subtraction is same as addition
        assert_eq!(c.0, 0x12345678 ^ 0x87654321);
    }

    #[test]
    fn test_multiplication_identity() {
        let a = GF2_32::new(0x12345678);
        let one = GF2_32::one();
        let result = a * one;
        assert_eq!(result.0, a.0);
    }

    #[test]
    fn test_multiplication_zero() {
        let a = GF2_32::new(0x12345678);
        let zero = GF2_32::zero();
        let result = a * zero;
        assert_eq!(result.0, 0);
    }

    #[test]
    fn test_power() {
        let a = GF2_32::new(2);
        let result = a.pow(0);
        assert_eq!(result, GF2_32::one());

        let result = a.pow(1);
        assert_eq!(result, a);
    }

    #[test]
    fn test_frobenius() {
        let a = GF2_32::new(0x12345678);
        let result = a.frobenius(1);
        // φ(x) = x^2 in GF(2^32)
        assert_eq!(result, a.pow(2));
    }

    #[test]
    fn test_additive_inverse() {
        let a = GF2_32::new(0x12345678);
        let neg_a = a.neg();
        let sum = a + neg_a;
        assert_eq!(sum, GF2_32::zero());
    }
}

