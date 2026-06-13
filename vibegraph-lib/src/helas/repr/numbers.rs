//! Quantum numbers and related utilities.
//!
//! This module will hold types and utilities related to quantum numbers, such
//! as helicity, color, and charge.

/// Spinor helicity label: the sign of the projection of spin onto momentum.
///
/// Corresponds to the HELAS `nhel` parameter (±1). The name `Up`/`Down`
/// matches the convention that `Up` (positive helicity, right-handed) has
/// `λ = +½` and `Down` (negative helicity, left-handed) has `λ = −½`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpinorHelicity {
    /// Positive helicity: `nhel = +1`.
    Up,
    /// Negative helicity: `nhel = −1`.
    Down,
}

impl SpinorHelicity {
    /// Return `+1` or `−1` as an `i32`.
    #[inline(always)]
    pub fn sign(self) -> i32 {
        match self {
            SpinorHelicity::Up => 1,
            SpinorHelicity::Down => -1,
        }
    }

    /// Return the opposite helicity (Up ↔ Down).
    pub fn flip(self) -> Self {
        match self {
            SpinorHelicity::Up => SpinorHelicity::Down,
            SpinorHelicity::Down => SpinorHelicity::Up,
        }
    }
}

impl std::fmt::Display for SpinorHelicity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpinorHelicity::Up => write!(f, "↑"),
            SpinorHelicity::Down => write!(f, "↓"),
        }
    }
}

/// Chirality label for chiral projections and bilinears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chirality {
    /// Left-handed: `P_L = (1 - γ^5)/2` — projects onto left-chiral (undotted Weyl) components.
    Left,
    /// Right-handed: `P_R = (1 + γ^5)/2` — projects onto right-chiral (dotted Weyl) components.
    Right,
    /// Both: identity projector — includes both chiralities.
    Both,
}

impl Chirality {
    /// Return the opposite chirality (Left ↔ Right, Both ↔ Both).
    pub fn flip(self) -> Self {
        match self {
            Chirality::Left => Chirality::Right,
            Chirality::Right => Chirality::Left,
            Chirality::Both => Chirality::Both,
        }
    }
}

impl std::fmt::Display for Chirality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chirality::Left => write!(f, "L"),
            Chirality::Right => write!(f, "R"),
            Chirality::Both => write!(f, "Both"),
        }
    }
}

/// Particle-vs-antiparticle label.
///
/// Corresponds to the HELAS `nsf` parameter (+1 for particle, −1 for
/// antiparticle).  The signed momentum stored in
/// [`DiracWf`](crate::helas::wavefn::DiracWf) is `p * nsf.sign()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Charge {
    /// Particle (e.g. e⁻, q): `nsf = +1`.
    Particle,
    /// Antiparticle (e.g. e⁺, q̄): `nsf = −1`.
    Antiparticle,
}

impl Charge {
    /// Return `+1` or `−1` as an `i32`.
    #[inline(always)]
    pub fn sign(self) -> i32 {
        match self {
            Charge::Particle => 1,
            Charge::Antiparticle => -1,
        }
    }

    /// Return the opposite charge (particle ↔ antiparticle).
    pub fn anti(self) -> Self {
        match self {
            Charge::Particle => Charge::Antiparticle,
            Charge::Antiparticle => Charge::Particle,
        }
    }
}

impl std::fmt::Display for Charge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Charge::Particle => write!(f, "particle"),
            Charge::Antiparticle => write!(f, "antiparticle"),
        }
    }
}
