//! The Les Houches `<init>` and `<event>` blocks as data.
//!
//! Field names follow the accord's Fortran common blocks (`IDBMUP`, `XWGTUP`,
//! `ICOLUP`, …) so that a reader can check this against the accord directly; the
//! Rust names say what the field is.

/// The accord's `IDWTUP`: how a consumer must combine the event weights.
///
/// Only the two strategies this project can produce are named. Reading a file
/// written by something else keeps whatever integer it carried, since the writer
/// has to round-trip it and a consumer may still know what it means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightStrategy {
    /// `+3`: every event carries unit weight and the cross section is `XSECUP`.
    UnitWeight,
    /// `-4`: `XWGTUP` is a cross section in picobarns and the total is the
    /// **mean** of the event weights, not their sum. This is what MadGraph writes
    /// for an unweighted sample, and it is what keeps the overweight events an
    /// accept/reject pass hands over at a weight above one visible *as* weights.
    ///
    /// It is not the only way to carry them: stochastic rounding
    /// ([`emit::StochasticRounding`](super::emit::StochasticRounding)) represents
    /// the same tail unbiasedly with unit weights, by writing an event
    /// `floor(w) + Bernoulli(frac(w))` times. The choice is between a visible
    /// weight and a multiplicity, not between representable and not.
    MeanCrossSectionPb,
    /// Any other value, kept verbatim.
    Other(i32),
}

impl WeightStrategy {
    pub fn as_i32(self) -> i32 {
        match self {
            WeightStrategy::UnitWeight => 3,
            WeightStrategy::MeanCrossSectionPb => -4,
            WeightStrategy::Other(v) => v,
        }
    }

    /// The strategy an `IDWTUP` names, normalising the two values that have one.
    pub fn from_i32(value: i32) -> Self {
        match value {
            3 => WeightStrategy::UnitWeight,
            -4 => WeightStrategy::MeanCrossSectionPb,
            other => WeightStrategy::Other(other),
        }
    }
}

/// One process's entry in the `<init>` block: `XSECUP`, `XERRUP`, `XMAXUP`,
/// `LPRUP`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LheProcess {
    /// `XSECUP` — the process's cross section in picobarns.
    pub xsec_pb: f64,
    /// `XERRUP` — its statistical uncertainty, in picobarns.
    pub xerr_pb: f64,
    /// `XMAXUP` — the largest `XWGTUP` this process will emit. A consumer that
    /// re-unweights reads it, so it must bound the weights actually written.
    pub xmax: f64,
    /// `LPRUP` — the process id the `<event>` lines refer back to.
    pub id: i32,
}

/// The `<init>` block.
#[derive(Clone, Debug, PartialEq)]
pub struct LheInit {
    /// `IDBMUP` — the beam particles' PDG codes.
    pub beam_pdg: [i32; 2],
    /// `EBMUP` — the beam energies in GeV.
    pub beam_energy: [f64; 2],
    /// `PDFGUP` — the PDF author group per beam, `0` for a beam with no parton
    /// densities.
    pub pdf_group: [i32; 2],
    /// `PDFSUP` — the PDF set id per beam (the LHAPDF id, when the group is `0`
    /// and the set is non-zero).
    pub pdf_set: [i32; 2],
    /// `IDWTUP`.
    pub weight_strategy: WeightStrategy,
    /// One entry per process; `NPRUP` is its length.
    pub processes: Vec<LheProcess>,
    /// Lines between the last process entry and `</init>`, kept verbatim —
    /// MadGraph puts its `<generator>` tag there.
    pub trailer: Vec<String>,
}

/// `ISTUP` for an incoming leg.
pub const STATUS_INCOMING: i32 = -1;
/// `ISTUP` for a leg leaving the hard process.
pub const STATUS_OUTGOING: i32 = 1;
/// `ISTUP` for an intermediate resonance the record lists explicitly.
pub const STATUS_INTERMEDIATE: i32 = 2;
/// `ICOLUP` for "this leg carries no line in this slot".
pub const NO_COLOR_LINE: i32 = 0;
/// `SPINUP` for a leg whose helicity was summed over rather than selected.
pub const SPIN_UNKNOWN: f64 = 9.0;

/// One line of an `<event>` block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LheParticle {
    /// `IDUP` — the PDG code.
    pub pdg: i32,
    /// `ISTUP` — [`STATUS_INCOMING`], [`STATUS_OUTGOING`] or
    /// [`STATUS_INTERMEDIATE`].
    pub status: i32,
    /// `MOTHUP` — 1-based positions of this leg's mothers within the same event,
    /// `0` for none.
    pub mothers: [i32; 2],
    /// `ICOLUP` — `[colour, anticolour]` line labels, [`NO_COLOR_LINE`] for an
    /// empty slot. Slot 1 is the *physical* colour whichever way the leg runs.
    pub color: [i32; 2],
    /// The four-momentum as `[E, px, py, pz]`, in the crate's layout. The file
    /// writes `px py pz E`; the permutation lives in the (de)serialiser, not
    /// here.
    ///
    /// These are physical momenta and not the all-outgoing crossing an amplitude
    /// works in: an incoming leg along the negative beam axis carries `pz < 0`.
    pub momentum: [f64; 4],
    /// The mass in GeV — the pole mass for a stable leg, the virtuality for an
    /// intermediate one.
    pub mass: f64,
    /// `VTIMUP` — the proper lifetime `c·τ` in mm.
    pub lifetime: f64,
    /// `SPINUP` — the selected helicity, or [`SPIN_UNKNOWN`].
    pub spin: f64,
}

/// One `<event>` block.
#[derive(Clone, Debug, PartialEq)]
pub struct LheEvent {
    /// `IDPRUP` — which `<init>` process entry this event belongs to.
    pub process_id: i32,
    /// `XWGTUP` — the event weight, in the units [`WeightStrategy`] implies.
    pub weight: f64,
    /// `SCALUP` — the factorisation scale in GeV (see
    /// [`build::scalup`](super::build::scalup)).
    pub scale: f64,
    /// `AQEDUP` — the electromagnetic coupling the event was evaluated at.
    pub alpha_qed: f64,
    /// `AQCDUP` — the strong coupling at `μR`.
    pub alpha_qcd: f64,
    /// The legs, incoming first. `NUP` is its length.
    pub particles: Vec<LheParticle>,
    /// Lines between the last particle and `</event>`, kept verbatim — MadGraph
    /// puts `<mgrwt>` and `<rwgt>` blocks there.
    pub trailer: Vec<String>,
}

impl LheEvent {
    /// `NUP` — the number of legs the record lists.
    pub fn nup(&self) -> usize {
        self.particles.len()
    }

    /// The colour lines of this event as endpoint sets: for each label, the
    /// `(leg, slot)` pairs carrying it, with legs 0-based and slot `0` colour.
    ///
    /// This is the representation two records must be compared in. The labels
    /// themselves are arbitrary — any consistent relabelling of a flow describes
    /// the same event — so only the partition they induce is physical. The
    /// endpoint lists are returned in a canonical order, so that neither the
    /// labels' values nor the order they first appear in survives into the
    /// comparison.
    pub fn color_connectivity(&self) -> Vec<Vec<(usize, usize)>> {
        let mut labels: Vec<i32> = self
            .particles
            .iter()
            .flat_map(|p| p.color)
            .filter(|&c| c != NO_COLOR_LINE)
            .collect();
        labels.sort_unstable();
        labels.dedup();
        let mut lines: Vec<Vec<(usize, usize)>> = labels
            .into_iter()
            .map(|label| {
                let mut ends = Vec::new();
                for (leg, p) in self.particles.iter().enumerate() {
                    for (slot, &c) in p.color.iter().enumerate() {
                        if c == label {
                            ends.push((leg, slot));
                        }
                    }
                }
                ends
            })
            .collect();
        lines.sort_unstable();
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_strategy_round_trips_through_its_integer() {
        for s in [
            WeightStrategy::UnitWeight,
            WeightStrategy::MeanCrossSectionPb,
            WeightStrategy::Other(1),
            WeightStrategy::Other(-1),
        ] {
            assert_eq!(WeightStrategy::from_i32(s.as_i32()), s);
        }
        // The two named values are normalised, so an `Other` spelling of them is
        // never produced by parsing.
        assert_eq!(WeightStrategy::from_i32(-4).as_i32(), -4);
        assert_eq!(WeightStrategy::from_i32(3), WeightStrategy::UnitWeight);
    }

    fn particle(color: [i32; 2]) -> LheParticle {
        LheParticle {
            pdg: 21,
            status: STATUS_OUTGOING,
            mothers: [1, 2],
            color,
            momentum: [0.0; 4],
            mass: 0.0,
            lifetime: 0.0,
            spin: 1.0,
        }
    }

    fn event(colors: &[[i32; 2]]) -> LheEvent {
        LheEvent {
            process_id: 1,
            weight: 1.0,
            scale: 100.0,
            alpha_qed: 0.0075,
            alpha_qcd: 0.1,
            particles: colors.iter().copied().map(particle).collect(),
            trailer: Vec::new(),
        }
    }

    /// Two records differing only by a relabelling of the colour lines describe
    /// the same event, and must compare equal in the representation the oracles
    /// use.
    #[test]
    fn connectivity_is_blind_to_a_relabelling_but_not_to_a_reconnection() {
        let a = event(&[[504, 501], [501, 502], [503, 502], [504, 503]]);
        let relabelled = event(&[[601, 602], [602, 603], [604, 603], [601, 604]]);
        assert_eq!(a.color_connectivity(), relabelled.color_connectivity());

        // Swapping one leg's two slots is a different flow, not a relabelling.
        let swapped = event(&[[501, 504], [501, 502], [503, 502], [504, 503]]);
        assert_ne!(a.color_connectivity(), swapped.color_connectivity());
    }
}
