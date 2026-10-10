//! The SU(3) colour representation a particle carries, as a runtime label.
//!
//! The colour algebra itself is symbolic and exact (`helas::color`); this
//! module only names the representations a UFO colour charge can select.

/// An SU(3) color representation, tagged by its UFO color charge.
///
/// Colorize and the `Identity` resolution key off it.
///
/// The ordering is the declaration order and carries no group-theoretic meaning:
/// it exists so that types holding a rep can derive one, and nothing reads it as
/// a ranking of representations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorRep {
    /// The trivial **1** (leptons, photon, …).
    Singlet,
    /// The fundamental **3** (quarks).
    Triplet,
    /// The antifundamental **3̄** (antiquarks).
    AntiTriplet,
    /// The adjoint **8** (gluons).
    Octet,
    /// The symmetric two-index **6** (sextet diquarks).
    Sextet,
    /// The conjugate **6̄**.
    AntiSextet,
}

impl ColorRep {
    /// Map a UFO `color` charge to a representation: `1 → Singlet`,
    /// `3 → Triplet`, `-3 → AntiTriplet`, `6 → Sextet`, `-6 → AntiSextet`,
    /// `8 → Octet`. The self-conjugate reps also accept their negated charge,
    /// which the antiparticle constructor produces (`color: -self.color`):
    /// `-1 → Singlet`, `-8 → Octet`. Any other value returns `None`.
    pub(crate) fn from_ufo(color: i32) -> Option<Self> {
        match color {
            1 | -1 => Some(ColorRep::Singlet),
            3 => Some(ColorRep::Triplet),
            -3 => Some(ColorRep::AntiTriplet),
            6 => Some(ColorRep::Sextet),
            -6 => Some(ColorRep::AntiSextet),
            8 | -8 => Some(ColorRep::Octet),
            _ => None,
        }
    }

    /// The conjugate representation (`3 ↔ 3̄`, `6 ↔ 6̄`; self-conjugate
    /// otherwise).
    pub(crate) fn anti(self) -> Self {
        match self {
            ColorRep::Singlet => ColorRep::Singlet,
            ColorRep::Triplet => ColorRep::AntiTriplet,
            ColorRep::AntiTriplet => ColorRep::Triplet,
            ColorRep::Sextet => ColorRep::AntiSextet,
            ColorRep::AntiSextet => ColorRep::Sextet,
            ColorRep::Octet => ColorRep::Octet,
        }
    }
}

#[cfg(test)]
mod color_rep_tests {
    use super::ColorRep;

    /// The antiparticle constructor negates the UFO `color` charge, so the
    /// self-conjugate reps arrive as `-1` (singlet) and `-8` (octet) on internal
    /// lines; both must resolve to the same rep as their positive charge.
    #[test]
    fn from_ufo_self_conjugate_negated() {
        assert_eq!(ColorRep::from_ufo(-1), Some(ColorRep::Singlet));
        assert_eq!(ColorRep::from_ufo(1), Some(ColorRep::Singlet));
        assert_eq!(ColorRep::from_ufo(-8), Some(ColorRep::Octet));
        assert_eq!(ColorRep::from_ufo(8), Some(ColorRep::Octet));
    }

    /// The triplet stays chiral: `3` and `-3` are distinct conjugate reps.
    #[test]
    fn from_ufo_triplet_is_chiral() {
        assert_eq!(ColorRep::from_ufo(3), Some(ColorRep::Triplet));
        assert_eq!(ColorRep::from_ufo(-3), Some(ColorRep::AntiTriplet));
        assert_eq!(
            ColorRep::from_ufo(3).map(ColorRep::anti),
            ColorRep::from_ufo(-3)
        );
    }

    /// The sextet is chiral too, and unlike the triplet its charge sign is the
    /// only thing that distinguishes the two: a UFO writes the **6** as `6` and
    /// the **6̄** as `-6`.
    #[test]
    fn from_ufo_sextet_is_chiral() {
        assert_eq!(ColorRep::from_ufo(6), Some(ColorRep::Sextet));
        assert_eq!(ColorRep::from_ufo(-6), Some(ColorRep::AntiSextet));
        assert_eq!(
            ColorRep::from_ufo(6).map(ColorRep::anti),
            ColorRep::from_ufo(-6)
        );
    }

    /// Any charge outside `1`, `3`, `6`, `8` and their negations is refused
    /// rather than folded into a nearby rep.
    #[test]
    fn from_ufo_unknown_charge_is_refused() {
        for charge in [0, 2, 4, 5, 7, 9, 10, -2, -4, -10] {
            assert_eq!(ColorRep::from_ufo(charge), None, "colour charge {charge}");
        }
    }
}
