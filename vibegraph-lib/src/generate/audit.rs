//! The check every reweighted event's denominator is put through before its
//! file is kept.

use tracing::info;

use super::{err, GenerateError};
use crate::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_OUTGOING};
use crate::proton::BeamOrdering;
use crate::reweight::Launch;

/// How far the reweighter's card-point `|M|²` may sit from the integrand's, relative.
///
/// The two are independent compilations of the same subprocess — the
/// reweighter's from the member's own diagrams, pruned at a generic parameter
/// point; the integrand's from the flavour group's representative, pruned at the
/// card — so they agree to rounding, not bit for bit. An indexing error (another
/// member, part or ordering) or a coupling taken at another scale moves `|M|²` at
/// order one.
const CARD_POINT_TOLERANCE: f64 = 1e-9;

/// Every reweighted event's denominator checked against the integrand that drew
/// it: the `|M|²` the reweighter evaluates at the card's own parameters against
/// the one the integrand evaluated for the drawn subprocess, ordering and coupling,
/// and the reweighter's subprocess flavours against the event record's.
///
/// The ratios themselves are not checked here: a wrong subprocess that still
/// reproduces the card-point `|M|²` (a member of the same flavour group, which
/// shares it) is what the flavour comparison is for. Both checks share with the
/// reweighter the event's momenta, its beam mirror and the model's parameters, and
/// are blind to an error in any of them.
#[derive(Default)]
pub(super) struct ReweightAudit {
    /// Events checked per part and beam ordering.
    terms: Vec<((usize, BeamOrdering), usize)>,
    largest: f64,
    m2_mismatches: usize,
    flavour_mismatches: usize,
    /// Events with a hypothesis ratio that is not finite.
    non_finite: usize,
    first: Option<String>,
}

impl ReweightAudit {
    pub(super) fn check(
        &mut self,
        term: (usize, BeamOrdering),
        m0: f64,
        m2: f64,
        pdgs: &[i32],
        record: &LheEvent,
        (ratios, launches): (&[f64], &[Launch]),
    ) {
        if let Some(k) = ratios.iter().position(|r| !r.is_finite()) {
            self.non_finite += 1;
            self.first.get_or_insert_with(|| {
                format!(
                    "part {} {:?}: hypothesis `{}` weighs the event {}",
                    term.0, term.1, launches[k].id, ratios[k]
                )
            });
        }
        match self.terms.iter_mut().find(|(t, _)| *t == term) {
            Some((_, n)) => *n += 1,
            None => self.terms.push((term, 1)),
        }
        let deviation = (m0 - m2).abs() / m2.abs().max(f64::MIN_POSITIVE);
        self.largest = self.largest.max(deviation);
        if !(deviation <= CARD_POINT_TOLERANCE) {
            self.m2_mismatches += 1;
            self.first.get_or_insert_with(|| {
                format!(
                    "part {} {:?}: |M|^2 {m0:e} against the integrand's {m2:e}",
                    term.0, term.1
                )
            });
        }
        let incoming: Vec<i32> = record
            .particles
            .iter()
            .filter(|p| p.status == STATUS_INCOMING)
            .map(|p| p.pdg)
            .collect();
        let mut outgoing: Vec<i32> = record
            .particles
            .iter()
            .filter(|p| p.status == STATUS_OUTGOING)
            .map(|p| p.pdg)
            .collect();
        outgoing.sort_unstable();
        let n_in = incoming.len();
        let mut own_in = pdgs[..n_in.min(pdgs.len())].to_vec();
        if term.1 == BeamOrdering::Exchanged {
            own_in.reverse();
        }
        let mut own_out = pdgs[n_in.min(pdgs.len())..].to_vec();
        own_out.sort_unstable();
        if own_in != incoming || own_out != outgoing {
            self.flavour_mismatches += 1;
            self.first.get_or_insert_with(|| {
                format!(
                    "part {} {:?}: reweighted {pdgs:?} for an event recorded as \
                     {incoming:?} > {outgoing:?}",
                    term.0, term.1
                )
            });
        }
    }

    /// Report what was checked, and refuse a file whose weights were taken against
    /// another subprocess than the one each event was drawn in, or are not finite.
    /// A refused file is removed rather than left beside the error.
    pub(super) fn finish(&self, out: &std::path::Path) -> Result<(), GenerateError> {
        let events: usize = self.terms.iter().map(|(_, n)| n).sum();
        let mut terms = self.terms.clone();
        terms.sort_by_key(|((part, ordering), _)| (*part, *ordering == BeamOrdering::Exchanged));
        let breakdown = terms
            .iter()
            .map(|((part, ordering), n)| {
                let ordering = match ordering {
                    BeamOrdering::Direct => "direct",
                    BeamOrdering::Exchanged => "exchanged",
                };
                format!("part {part} {ordering} {n}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        info!(
            "reweight: card-point |M|^2 checked against the integrand on {events} events \
             ({breakdown}); largest relative deviation {:.1e}; beyond \
             {CARD_POINT_TOLERANCE:.0e}: {}; flavour mismatches: {}; non-finite weights: {}",
            self.largest, self.m2_mismatches, self.flavour_mismatches, self.non_finite
        );
        let why = if self.m2_mismatches + self.flavour_mismatches > 0 {
            format!(
                "reweighting evaluated {} of {events} events in another subprocess than the \
                 integrand drew them in",
                self.m2_mismatches.max(self.flavour_mismatches)
            )
        } else if self.non_finite > 0 {
            format!(
                "{} of {events} events carry a non-finite reweighting weight",
                self.non_finite
            )
        } else {
            return Ok(());
        };
        let removed = match std::fs::remove_file(out) {
            Ok(()) => "removed it".to_string(),
            Err(e) => format!("could not remove it ({e})"),
        };
        Err(err(format!(
            "{why}, so the weights in {} are wrong and {removed}; first: {}",
            out.display(),
            self.first.as_deref().unwrap_or("")
        )))
    }
}
