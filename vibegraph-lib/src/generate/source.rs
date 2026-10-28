//! The accept/reject pass as a replayable source of Les Houches events, and the
//! record-layer tables an accepted point is labelled from.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tracing::{info, warn};

use super::audit::ReweightAudit;
use super::MAX_TRIALS_PER_EVENT;
use crate::coupling::cluster::configs::qcd_order;
use crate::coupling::scales::MatchedRecord;
use crate::hadronic::FixedBeamIntegrand;
use crate::helas::repr::lorentz::LorentzVector;
use crate::lhef::build::{pt_clust_scales, scalup, EventHeader, Intermediate, SubprocessRecord};
use crate::lhef::emit::{EventSource, WeightedEvent};
use crate::lhef::record::LheEvent;
use crate::lhef::resonance::SubprocessResonances;
use crate::lhef::LhefError;
use crate::multiplicity::MultiplicitySum;
use crate::pdf::PdfMember;
use crate::phasespace::GEV2_TO_PB;
use crate::proton::{BeamOrdering, FlavorGroups, ProtonEvent, ProtonIntegrand, ProtonSelection};
use crate::reweight::engine::Reweighter;
use crate::runcard::RunCard;
use crate::systematics::record::{LoRecord, PdfChain};
use crate::ufo::UFOModel;
use crate::unweight::{UnweightStats, Unweighter};

type V = LorentzVector<f64>;

/// The accept/reject pass as a replayable source of events.
///
/// Each accepted point is turned straight into a record: the momenta come back
/// from the same channel map at the same `u` the trial was accepted on, and the
/// discrete labels are selected off the diagonals at that point.
pub(super) struct SampleSource<'s, 'a> {
    integrand: &'s FixedBeamIntegrand<'a>,
    records: &'s [SubprocessRecord],
    /// The unweighter as the scan left it, kept so a restart returns the pass to
    /// its initial state rather than continuing with accumulated statistics.
    pristine: Unweighter,
    unweighter: Unweighter,
    rng: ChaCha8Rng,
    seed: u64,
    /// The card the run was set up from, which supplies the record's scale where
    /// nothing in the matrix element moves with `αs` and so no per-event scale
    /// prescription was installed.
    card: &'s RunCard,
    alpha_qed: f64,
    momenta: Vec<V>,
    /// The factor from the integrand's natural units to the file's.
    unit: f64,
    /// Per subprocess, its card line's process number (`IDPRUP`).
    process_of: &'s [i32],
    /// Per subprocess, the resonances its records list; empty on a card without
    /// decay chains.
    resonances: &'s [SubprocessResonances],
    pub(super) tally: ResonanceTally,
    /// The reweighting hypotheses, indexed by subprocess.
    reweighter: Option<Reweighter<'s>>,
    evaluation: Vec<V>,
    pub(super) audit: ReweightAudit,
}

impl<'s, 'a> SampleSource<'s, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        integrand: &'s FixedBeamIntegrand<'a>,
        records: &'s [SubprocessRecord],
        unweighter: Unweighter,
        seed: u64,
        card: &'s RunCard,
        alpha_qed: f64,
        process_of: &'s [i32],
        resonances: &'s [SubprocessResonances],
        reweighter: Option<Reweighter<'s>>,
    ) -> Self {
        SampleSource {
            integrand,
            records,
            pristine: unweighter.clone(),
            unweighter,
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            card,
            alpha_qed,
            momenta: Vec::new(),
            unit: integrand.observable().per_natural_unit(),
            process_of,
            resonances,
            tally: ResonanceTally::default(),
            reweighter,
            evaluation: Vec::new(),
            audit: ReweightAudit::default(),
        }
    }

    pub(super) fn stats(&self) -> &UnweightStats {
        self.unweighter.stats()
    }
}

impl EventSource for SampleSource<'_, '_> {
    fn next_event(&mut self) -> Option<WeightedEvent> {
        let point =
            self.unweighter
                .next_event(self.integrand, &mut self.rng, MAX_TRIALS_PER_EVENT)?;
        self.integrand
            .event_in_channel(point.channel, &point.u, &mut self.momenta);
        let selection = self.integrand.select_event(
            &self.momenta,
            point.channel,
            [
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
            ],
        )?;
        let externals: Vec<[f64; 4]> = self
            .integrand
            .incoming()
            .iter()
            .chain(self.momenta.iter())
            .map(|p| [p.e(), p.px(), p.py(), p.pz()])
            .collect();
        // A point the scale prescription rejects has no scale to report, so the
        // source stops rather than inventing one, and the strategy reports a
        // sample it could not fill.
        let (scale, alpha_qcd) = self
            .integrand
            .record_scales(&self.momenta, point.channel, &point.u, self.card)
            .ok()?;
        let header = EventHeader {
            process_id: self.process_of[selection.subprocess],
            // The strategy imposes the file's weight convention; this slot is
            // overwritten before the record is written.
            weight: 0.0,
            scale,
            alpha_qed: self.alpha_qed,
            alpha_qcd,
        };
        let record = event_record(
            &self.records[selection.subprocess],
            &externals,
            &selection.helicity,
            selection.flow,
            header,
            self.resonances
                .get(selection.subprocess)
                .map(|table| (table, None)),
            (selection.config, selection.leading),
            &mut self.tally,
        )?;
        let mut reweights = Vec::new();
        if let Some(rw) = self.reweighter.as_mut() {
            self.evaluation.clear();
            self.evaluation.extend_from_slice(self.integrand.incoming());
            self.evaluation.extend_from_slice(&self.momenta);
            let m0 = rw.ratios(
                selection.subprocess,
                &self.evaluation,
                Some(alpha_qcd),
                &mut reweights,
            );
            self.audit.check(
                (0, BeamOrdering::Direct),
                m0,
                selection.m2,
                rw.pdgs(selection.subprocess),
                &record,
                (&reweights, rw.launches()),
            );
        }
        Some(WeightedEvent {
            record,
            weight: point.weight,
            part: 0,
            reweights,
            lo_record: None,
        })
    }

    fn restart(&mut self) {
        self.unweighter = self.pristine.clone();
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.tally = ResonanceTally::default();
        self.audit = ReweightAudit::default();
    }

    fn sigma_pb(&self) -> f64 {
        self.unweighter.sigma_from_events() * self.unit
    }
}

/// Events written without the intermediate records their configuration names.

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ResonanceTally {
    /// Events whose colour flow their configuration does not reach at leading
    /// colour, for which MadEvent's `addmothers` writes no intermediate either.
    subleading: u64,
    /// Events whose intermediates could not be given a colour their daughters
    /// fit, written without them rather than with a wrong one.
    refused: u64,
}

/// One event's record, with the resonances of its configuration as status-2
/// records where `resonances` names a table: the lines MadEvent's `cut_bw` flags
/// at the event's momenta ([`SubprocessResonances::on_shell`]), each carrying the
/// PDG code `pdgs[config][line]` where the record's flavours differ from the
/// table's, its own otherwise.
#[allow(clippy::too_many_arguments)]
pub(super) fn event_record(
    record: &SubprocessRecord,
    externals: &[[f64; 4]],
    helicity: &[i32],
    flow: usize,
    header: EventHeader,
    resonances: Option<(&SubprocessResonances, Option<&[Vec<i32>]>)>,
    (config, leading): (Option<usize>, bool),
    tally: &mut ResonanceTally,
) -> Option<LheEvent> {
    let Some((table, pdgs)) = resonances else {
        return record.event(externals, helicity, flow, header).ok();
    };
    let Some(config) = config.filter(|_| leading) else {
        tally.subleading += 1;
        return record.event(externals, helicity, flow, header).ok();
    };
    let lines = table.lines(config);
    let intermediates: Vec<Intermediate> = table
        .on_shell(config, &externals[record.n_in()..])
        .into_iter()
        .map(|i| Intermediate {
            pdg: pdgs.map_or(lines[i].pdg, |p| p[config][i]),
            color: lines[i].color,
            slots: lines[i].slots,
        })
        .collect();
    match record.event_with_intermediates(externals, helicity, flow, header, &intermediates) {
        Ok(event) => Some(event),
        Err(LhefError::IntermediateColor { .. } | LhefError::IntermediateNesting { .. }) => {
            tally.refused += 1;
            record.event(externals, helicity, flow, header).ok()
        }
        Err(_) => None,
    }
}

/// A matched (`ickkw > 0`) event's record: the resonances `addmothers` writes
/// under matching and the `<scales>` line after the particles.
///
/// The status-2 lines are the clustered configuration's timelike propagators
/// whose leg sets the clustering found on their Breit–Wigner
/// ([`SubprocessResonances::clustered_on_shell`]), none where the drawn flow is
/// below leading colour in that configuration (`addmothers.f:129-131`, 195).
/// `pdgs[config][line]` carries each line's code in the record's flavours.
/// `ptclus` is indexed like the record's legs: the record reorders only the
/// incoming ones, which have none.
#[allow(clippy::too_many_arguments)]
pub(super) fn matched_event_record(
    record: &SubprocessRecord,
    externals: &[[f64; 4]],
    helicity: &[i32],
    flow: usize,
    header: EventHeader,
    resonances: Option<(&SubprocessResonances, &[Vec<i32>])>,
    (config, leading): (Option<usize>, bool),
    matched: &MatchedRecord,
    tally: &mut ResonanceTally,
) -> Option<LheEvent> {
    let n_in = record.n_in();
    let intermediates: Vec<Intermediate> = match (resonances, config) {
        (Some((table, pdgs)), Some(config)) if leading => {
            let lines = table.lines(config);
            table
                .clustered_on_shell(config, &matched.on_shell, n_in)
                .into_iter()
                .map(|i| Intermediate {
                    pdg: pdgs[config][i],
                    color: lines[i].color,
                    slots: lines[i].slots,
                })
                .collect()
        }
        (Some(_), Some(_)) => {
            tally.subleading += 1;
            Vec::new()
        }
        _ => Vec::new(),
    };
    let mut event =
        match record.event_with_intermediates(externals, helicity, flow, header, &intermediates) {
            Ok(event) => event,
            Err(LhefError::IntermediateColor { .. } | LhefError::IntermediateNesting { .. }) => {
                tally.refused += 1;
                record.event(externals, helicity, flow, header).ok()?
            }
            Err(_) => return None,
        };
    let line = pt_clust_scales(&event, matched.ptclus.get(n_in..)?);
    event.trailer.push(line);
    Some(event)
}

/// Say how many events went out without the intermediates their configuration
/// names.
pub(super) fn report_resonances(tally: ResonanceTally, written: usize) {
    if tally.subleading > 0 {
        info!(
            "resonances: {} of {written} events carry a colour flow their configuration reaches \
             only below leading colour and list no intermediate record, as MadEvent's do",
            tally.subleading
        );
    }
    if tally.refused > 0 {
        warn!(
            "resonances: {} of {written} events list no intermediate record because a \
             resonance's daughters leave colour lines that do not fit it",
            tally.refused
        );
    }
}

/// Per part, per flavour group, the index of its first member in the run's
/// flattened list of concrete subprocesses: every member of every part, in order.
pub(super) fn member_offsets<'g>(
    parts: impl IntoIterator<Item = &'g FlavorGroups>,
) -> Vec<Vec<usize>> {
    let mut next = 0;
    parts
        .into_iter()
        .map(|groups| {
            groups
                .groups()
                .iter()
                .map(|g| {
                    let first = next;
                    next += g.members().len();
                    first
                })
                .collect()
        })
        .collect()
}

/// Every concrete flavour assignment of every group, under both beam orderings, as
/// the record layer sees it: `[group][member][ordering]`, with the exchanged slot
/// absent where the two beams carry the same parton and there is only one ordering.
pub(super) type FlavorRecords = Vec<Vec<[Option<SubprocessRecord>; 2]>>;

/// The record slot an ordering occupies.
pub(super) fn ordering_slot(ordering: BeamOrdering) -> usize {
    usize::from(ordering == BeamOrdering::Exchanged)
}

/// One multiplicity's record-layer tables: its flavour records, and where the card
/// has decay chains, each group's resonances and their per-member PDG codes.
pub(super) struct PartRecords {
    pub(super) records: FlavorRecords,
    pub(super) resonances: Vec<SubprocessResonances>,
    /// `[group][member][config][line]`: the PDG code a resonance line of the
    /// group's representative carries in a member.
    pub(super) member_pdgs: Vec<Vec<Vec<Vec<i32>>>>,
}

/// What an unmatched proton event's `<mgrwt>` record is filled from
/// (`reweight.f:1450-1458`): the QCD power of each integration configuration's
/// diagram, and the central member the densities were read from.
pub(super) struct RecordFill<'s> {
    /// `[part][group][config]`: the QCD coupling power of the configuration's
    /// diagram, MadEvent's `nqcd`.
    nqcd: Vec<Vec<Vec<u32>>>,
    central: &'s PdfMember,
}

impl<'s> RecordFill<'s> {
    pub(super) fn new(parts: &[FlavorGroups], model: &UFOModel, central: &'s PdfMember) -> Self {
        let nqcd = parts
            .iter()
            .map(|groups| {
                groups
                    .groups()
                    .iter()
                    .map(|group| {
                        let evaluator = group.evaluator();
                        (0..evaluator.n_configs())
                            .map(|c| {
                                evaluator
                                    .config_diagram(c)
                                    .and_then(|d| group.diagrams().get(d))
                                    .map_or(0, |diagram| {
                                        u32::try_from(qcd_order(diagram, model)).unwrap_or(0)
                                    })
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        RecordFill { nqcd, central }
    }

    /// The `<mgrwt>` record of an unmatched event of part `k`: the QCD power
    /// of the configuration the event is labelled in (MadEvent's
    /// `nqcd(igraphs(1))`; the group's highest where no configuration carried
    /// weight), its `μR`, one chain entry per beam holding the incoming
    /// parton's code, `x` and `μF`, and `f₁ f₂` as the total factor, the
    /// densities at those entries (`asref = 0` without matching).
    fn record(
        &self,
        k: usize,
        selection: &ProtonSelection,
        event: &ProtonEvent,
        record: &LheEvent,
    ) -> Option<LoRecord> {
        let configs = &self.nqcd[k][selection.group];
        let n_qcd = match selection.config {
            Some(c) => *configs.get(c)?,
            None => configs.iter().copied().max().unwrap_or(0),
        };
        let mut beams = [PdfChain::default(), PdfChain::default()];
        let mut tot_fact = 1.0;
        for (b, chain) in beams.iter_mut().enumerate() {
            let parton = record.particles.get(b)?;
            let (x, q) = (event.x[b], selection.scales.mu_f[b]);
            let pdg = parton.pdg;
            tot_fact *= self.central.try_xfx_q2(pdg, x, q * q).ok()? / x;
            chain.entries.push((i64::from(pdg), x, q));
        }
        Some(LoRecord {
            n_qcd,
            mu_r: selection.scales.mu_r,
            emission_scales: Vec::new(),
            beams,
            beam_order: match selection.ordering {
                BeamOrdering::Direct => [1, 2],
                BeamOrdering::Exchanged => [2, 1],
            },
            tot_fact,
        })
    }
}

/// The accept/reject pass over a hadronic integrand as a replayable source.
///
/// The difference from a fixed-beam run is entirely in what an accepted point
/// carries: its own beam momentum fractions, its own scales, and a concrete flavour
/// assignment drawn from the parton luminosities at those fractions. On a card of
/// several parts (final-state multiplicities, or sets of outgoing masses) a
/// channel of any of them may be drawn, and the event is labelled from its own
/// part's tables.
pub(super) struct ProtonSampleSource<'s, 'a> {
    integrand: &'s MultiplicitySum<'a>,
    /// Per part, in the integrand's part order.
    parts: &'s [PartRecords],
    pristine: Unweighter,
    pub(super) unweighter: Unweighter,
    rng: ChaCha8Rng,
    seed: u64,
    alpha_qed: f64,
    pub(super) tally: ResonanceTally,
    /// The reweighting hypotheses, indexed by concrete subprocess across every
    /// part: `member_offset[part][group] + member`.
    reweighter: Option<Reweighter<'s>>,
    member_offset: Vec<Vec<usize>>,
    mirrored: Vec<V>,
    pub(super) audit: ReweightAudit,
    /// Where the run writes `<mgrwt>` records, what they are filled from.
    records: Option<RecordFill<'s>>,
}

impl<'s, 'a> ProtonSampleSource<'s, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        integrand: &'s MultiplicitySum<'a>,
        parts: &'s [PartRecords],
        unweighter: Unweighter,
        seed: u64,
        alpha_qed: f64,
        reweighter: Option<Reweighter<'s>>,
        records: Option<RecordFill<'s>>,
    ) -> Self {
        ProtonSampleSource {
            integrand,
            parts,
            pristine: unweighter.clone(),
            unweighter,
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            alpha_qed,
            tally: ResonanceTally::default(),
            reweighter,
            member_offset: member_offsets(integrand.parts().iter().map(ProtonIntegrand::groups)),
            mirrored: Vec::new(),
            audit: ReweightAudit::default(),
            records,
        }
    }
}

impl EventSource for ProtonSampleSource<'_, '_> {
    fn next_event(&mut self) -> Option<WeightedEvent> {
        let point =
            self.unweighter
                .next_event(self.integrand, &mut self.rng, MAX_TRIALS_PER_EVENT)?;
        let (k, event) = self.integrand.event_in_channel(point.channel, &point.u)?;
        let integrand: &ProtonIntegrand<'_> = &self.integrand.parts()[k];
        let part = &self.parts[k];
        let selection = integrand.select_event(
            &event,
            [
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
            ],
        )?;
        let externals: Vec<[f64; 4]> = event
            .lab
            .iter()
            .map(|p| [p.e(), p.px(), p.py(), p.pz()])
            .collect();
        // The coupling the matrix element itself ran at, off the PDF set's own
        // tabulation — the one the densities were fitted with.
        let alpha_qcd = integrand
            .alpha_s_source()
            .map(|source| source.eval(selection.scales.mu_r))
            .unwrap_or(0.0);
        let member = &integrand.groups().groups()[selection.group].members()[selection.member];
        let header = EventHeader {
            // `IDPRUP` is the member's process number, `@N`: MadEvent writes the
            // number of the event's `P<n>` directory, which is the `@N` of the
            // processes grouped into it.
            process_id: member.process as i32,
            // The strategy imposes the file's weight convention; this slot is
            // overwritten before the record is written.
            weight: 0.0,
            scale: scalup(&selection.scales),
            alpha_qed: self.alpha_qed,
            alpha_qcd,
        };
        let resonances = part.resonances.get(selection.group).map(|table| {
            (
                table,
                part.member_pdgs[selection.group][selection.member].as_slice(),
            )
        });
        let subprocess = part.records[selection.group][selection.member]
            [ordering_slot(selection.ordering)]
        .as_ref()?;
        let record = match &selection.record {
            Some(matched) => matched_event_record(
                subprocess,
                &externals,
                &selection.helicity,
                selection.flow,
                header,
                resonances,
                (selection.scales.clustered_config, selection.leading),
                matched,
                &mut self.tally,
            )?,
            None => event_record(
                subprocess,
                &externals,
                &selection.helicity,
                selection.flow,
                header,
                resonances.map(|(table, pdgs)| (table, Some(pdgs))),
                (selection.config, selection.leading),
                &mut self.tally,
            )?,
        };
        let mut reweights = Vec::new();
        if let Some(rw) = self.reweighter.as_mut() {
            // The member's own amplitude, in the frame and leg order the group's
            // was evaluated in for this ordering.
            let group = &integrand.groups().groups()[selection.group];
            let momenta = match selection.ordering {
                BeamOrdering::Direct => &event.cm,
                BeamOrdering::Exchanged => {
                    group.mirror_into(&event.cm, &mut self.mirrored);
                    &self.mirrored
                }
            };
            let sub = self.member_offset[k][selection.group] + selection.member;
            let m0 = rw.ratios(sub, momenta, Some(alpha_qcd), &mut reweights);
            self.audit.check(
                (k, selection.ordering),
                m0,
                selection.m2,
                rw.pdgs(sub),
                &record,
                (&reweights, rw.launches()),
            );
        }
        let lo_record = match &self.records {
            Some(fill) if selection.record.is_none() => {
                Some(fill.record(k, &selection, &event, &record)?)
            }
            _ => None,
        };
        Some(WeightedEvent {
            record,
            weight: point.weight,
            part: k,
            reweights,
            lo_record,
        })
    }

    fn restart(&mut self) {
        self.unweighter = self.pristine.clone();
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.tally = ResonanceTally::default();
        self.audit = ReweightAudit::default();
    }

    fn sigma_pb(&self) -> f64 {
        self.unweighter.sigma_from_events() * GEV2_TO_PB
    }
}
