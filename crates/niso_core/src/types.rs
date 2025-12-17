//! Core types for NISO
//!
//! Gantree: L0_Foundation → CoreTypes
//!
//! Provides fundamental type aliases and validated wrapper types
//! used throughout the NISO system.

use crate::error::{NisoError, NisoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Type Aliases
// ============================================================================

/// Qubit identifier (0-indexed)
/// Gantree: QubitId // pub type QubitId = usize
pub type QubitId = usize;

/// Rotation angle in radians
/// Gantree: Angle // pub type Angle = f64
pub type Angle = f64;

/// Measurement counts: bitstring -> count
/// Gantree: Counts // pub type Counts = HashMap<String, u64>
pub type Counts = HashMap<String, u64>;

/// Parameter vector for variational circuits
/// Gantree: ParamVec // pub type ParamVec = Vec<f64>
pub type ParamVec = Vec<f64>;

// ============================================================================
// Probability (Validated Wrapper)
// ============================================================================

/// Probability value in range [0, 1]
/// Gantree: Probability // 범위 검증 구조체
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Probability(f64);

impl Probability {
    /// Create a new Probability with validation
    /// Gantree: new(f64) -> Result<Self> // 생성+검증
    pub fn new(value: f64) -> NisoResult<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(NisoError::InvalidProbability(value));
        }
        Ok(Self(value))
    }

    /// Create without validation (for internal use only)
    /// # Safety
    /// Caller must ensure value is in [0, 1]
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn new_unchecked(value: f64) -> Self {
        debug_assert!((0.0..=1.0).contains(&value));
        Self(value)
    }

    /// Get the probability value
    /// Gantree: value() -> f64 // 값 반환
    #[inline]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Get the complement (1 - p)
    /// Gantree: complement() -> f64 // 1-p
    #[inline]
    pub fn complement(&self) -> f64 {
        1.0 - self.0
    }

    /// Zero probability
    pub const ZERO: Self = Self(0.0);

    /// Certainty (p = 1)
    pub const ONE: Self = Self(1.0);

    /// Half probability
    pub const HALF: Self = Self(0.5);
}

impl Default for Probability {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Probability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.0)
    }
}

impl TryFrom<f64> for Probability {
    type Error = NisoError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// ============================================================================
// Bitstring
// ============================================================================

/// Bitstring for measurement results
/// Gantree: Bitstring // 비트열 타입
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bitstring {
    bits: Vec<bool>,
}

impl Bitstring {
    /// Create from a vector of bools
    pub fn new(bits: Vec<bool>) -> Self {
        Self { bits }
    }

    /// Create from string (e.g., "0110")
    /// Gantree: parse(s) -> Self // 파싱
    pub fn parse(s: &str) -> NisoResult<Self> {
        let bits: Result<Vec<bool>, _> = s
            .chars()
            .map(|c| match c {
                '0' => Ok(false),
                '1' => Ok(true),
                _ => Err(NisoError::InvalidBitstring(s.to_string())),
            })
            .collect();
        Ok(Self { bits: bits? })
    }

    /// Create zero bitstring of given length
    pub fn zeros(n: usize) -> Self {
        Self {
            bits: vec![false; n],
        }
    }

    /// Get the number of bits
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Count number of 1s (Hamming weight)
    /// Gantree: popcount() -> usize // 1 카운트
    pub fn popcount(&self) -> usize {
        self.bits.iter().filter(|&&b| b).count()
    }

    /// Get parity (true if odd number of 1s)
    /// Gantree: parity() -> bool // 홀짝
    pub fn parity(&self) -> bool {
        self.popcount() % 2 == 1
    }

    /// Get parity sign (+1 for even, -1 for odd)
    pub fn parity_sign(&self) -> i32 {
        if self.parity() {
            -1
        } else {
            1
        }
    }
    /// Get bit at index (LSB = index 0)
    pub fn get(&self, index: usize) -> Option<bool> {
        self.bits.get(index).copied()
    }

    /// Convert to usize (for small bitstrings)
    pub fn to_usize(&self) -> usize {
        self.bits
            .iter()
            .rev()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| 1 << i)
            .sum()
    }
}

impl fmt::Display for Bitstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in &self.bits {
            write!(f, "{}", if b { '1' } else { '0' })?;
        }
        Ok(())
    }
}

impl From<&str> for Bitstring {
    fn from(s: &str) -> Self {
        Self::parse(s).expect("Invalid bitstring")
    }
}

// ============================================================================
// MeasurementBasis
// ============================================================================

/// Measurement basis for a single qubit
/// Gantree: Basis // X/Y/Z
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Basis {
    /// X (Hadamard) basis
    X,
    /// Y basis
    Y,
    /// Z (computational) basis
    Z,
}

impl Basis {
    /// Parse from character
    pub fn from_char(c: char) -> NisoResult<Self> {
        match c.to_ascii_uppercase() {
            'X' => Ok(Basis::X),
            'Y' => Ok(Basis::Y),
            'Z' => Ok(Basis::Z),
            _ => Err(NisoError::InvalidBasis(c.to_string())),
        }
    }

    /// Convert to character
    pub fn to_char(&self) -> char {
        match self {
            Basis::X => 'X',
            Basis::Y => 'Y',
            Basis::Z => 'Z',
        }
    }
}

impl fmt::Display for Basis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// Measurement basis string (e.g., "XXXXXXX" for 7 qubits)
/// Gantree: BasisString // 기저 문자열
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasisString {
    bases: Vec<Basis>,
}

impl BasisString {
    /// Create from string
    pub fn from_str(s: &str) -> NisoResult<Self> {
        let bases: Result<Vec<Basis>, _> = s.chars().map(Basis::from_char).collect();
        Ok(Self { bases: bases? })
    }

    /// Create uniform basis for n qubits
    pub fn uniform(basis: Basis, n: usize) -> Self {
        Self {
            bases: vec![basis; n],
        }
    }

    /// All X basis
    pub fn all_x(n: usize) -> Self {
        Self::uniform(Basis::X, n)
    }

    /// All Y basis
    pub fn all_y(n: usize) -> Self {
        Self::uniform(Basis::Y, n)
    }

    /// All Z basis
    pub fn all_z(n: usize) -> Self {
        Self::uniform(Basis::Z, n)
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.bases.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }

    /// Get basis at index
    pub fn get(&self, index: usize) -> Option<Basis> {
        self.bases.get(index).copied()
    }

    /// Iterate over bases
    pub fn iter(&self) -> impl Iterator<Item = &Basis> {
        self.bases.iter()
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.bases.iter().map(|b| b.to_char()).collect()
    }
}

impl fmt::Display for BasisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_valid() {
        assert!(Probability::new(0.0).is_ok());
        assert!(Probability::new(0.5).is_ok());
        assert!(Probability::new(1.0).is_ok());
    }

    #[test]
    fn test_probability_invalid() {
        assert!(Probability::new(-0.1).is_err());
        assert!(Probability::new(1.1).is_err());
    }

    #[test]
    fn test_probability_complement() {
        let p = Probability::new(0.3).unwrap();
        assert!((p.complement() - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_bitstring_popcount() {
        let bs = Bitstring::parse("01101").unwrap();
        assert_eq!(bs.popcount(), 3);
    }

    #[test]
    fn test_bitstring_parity() {
        let even = Bitstring::parse("0110").unwrap();
        let odd = Bitstring::parse("0111").unwrap();
        assert!(!even.parity()); // even parity
        assert!(odd.parity()); // odd parity
    }

    #[test]
    fn test_bitstring_parity_sign() {
        let even = Bitstring::parse("0110").unwrap();
        let odd = Bitstring::parse("0111").unwrap();
        assert_eq!(even.parity_sign(), 1);
        assert_eq!(odd.parity_sign(), -1);
    }

    #[test]
    fn test_basis_string() {
        let bs = BasisString::from_str("XYZXYZ").unwrap();
        assert_eq!(bs.len(), 6);
        assert_eq!(bs.get(0), Some(Basis::X));
        assert_eq!(bs.get(1), Some(Basis::Y));
        assert_eq!(bs.get(2), Some(Basis::Z));
    }

    #[test]
    fn test_basis_string_uniform() {
        let bs = BasisString::all_x(7);
        assert_eq!(bs.to_string(), "XXXXXXX");
    }
}
