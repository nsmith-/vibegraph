//! `vibegraph check-events` — read a Les Houches file back and check that what
//! it says is internally consistent.
//!
//! This is the acceptance end of the pipeline: a released binary, given nothing
//! but an `.lhe` file, has to be able to say whether that file is well formed
//! without a Rust toolchain, a Python installation or a shower to feed it to.
//!
//! # What it can and cannot see
//!
//! Reader and writer are the same crate and share their assumptions, so a
//! **self-consistently wrong format** passes here: a file both agree on but a
//! real shower rejects is invisible. The format evidence lives elsewhere — the
//! byte-for-byte round trip of MadGraph's own banked `.lhe.gz` files — and a run
//! through a shower is still owed.
//!
//! It is equally blind to the physics. Momentum conservation and mass shells are
//! properties of the *record*, and a wrong matrix element, a wrong cut or a
//! mis-adapted sampler produces events that satisfy every one of them. What this
//! catches is a file truncated, corrupted, or written by a code path that lost
//! track of its own bookkeeping.

use std::path::PathBuf;

use clap::Args;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{
    LheEvent, LheInit, WeightStrategy, STATUS_INCOMING, STATUS_INTERMEDIATE, STATUS_OUTGOING,
};

use crate::error::{err, CliError};

/// Relative tolerance for the momentum and mass-shell identities.
///
/// The writer emits ten decimal places in exponential form, so a record carries
/// about eleven significant digits; the identities below are sums of that many
/// digits over a handful of legs. Three orders of magnitude of headroom over the
/// representation error keeps a slow accumulation from being flagged while still
/// catching a leg that is simply wrong.
const DEFAULT_TOLERANCE: f64 = 1e-6;

#[derive(Args, Debug)]
pub(crate) struct CheckArgs {
    /// Les Houches file to read back.
    pub(crate) events: PathBuf,

    /// Relative tolerance for momentum conservation and mass shells.
    #[arg(long, default_value_t = DEFAULT_TOLERANCE)]
    pub(crate) tolerance: f64,

    /// Fail unless the file holds at least this many events.
    #[arg(long)]
    pub(crate) min_events: Option<usize>,
}

/// One thing wrong with one event, or with the file as a whole.
struct Complaint {
    event: Option<usize>,
    what: String,
}

impl std::fmt::Display for Complaint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.event {
            Some(i) => write!(f, "event {i}: {}", self.what),
            None => write!(f, "{}", self.what),
        }
    }
}

pub(crate) fn run(args: &CheckArgs) -> Result<(), CliError> {
    let text = std::fs::read_to_string(&args.events)
        .map_err(|e| err(format!("cannot read {}: {e}", args.events.display())))?;
    let file = LheFile::parse(&text)
        .map_err(|e| err(format!("cannot parse {}: {e}", args.events.display())))?;

    let complaints = complaints(&file, args.tolerance, args.min_events);
    if !complaints.is_empty() {
        let shown: Vec<String> = complaints.iter().take(20).map(|c| c.to_string()).collect();
        let more = complaints.len().saturating_sub(shown.len());
        let tail = if more > 0 {
            format!("\n  … and {more} more")
        } else {
            String::new()
        };
        return Err(err(format!(
            "{} failed {} check(s):\n  {}{tail}",
            args.events.display(),
            complaints.len(),
            shown.join("\n  ")
        )));
    }

    report(&file);
    Ok(())
}

/// Everything wrong with `file`, in file order: `<init>` first, then each event,
/// then the file as a whole.
fn complaints(file: &LheFile, tolerance: f64, min_events: Option<usize>) -> Vec<Complaint> {
    let mut complaints = Vec::new();
    check_init(&file.init, &mut complaints);
    for (index, event) in file.events.iter().enumerate() {
        check_event(index + 1, event, &file.init, tolerance, &mut complaints);
    }
    if let Some(minimum) = min_events {
        if file.events.len() < minimum {
            complaints.push(Complaint {
                event: None,
                what: format!(
                    "file holds {} events, fewer than the {minimum} required",
                    file.events.len()
                ),
            });
        }
    }
    complaints
}

/// Whether `<init>` describes a decay run: MadEvent's convention names the
/// decaying particle as beam 1, at its mass, and leaves beam 2 empty — no
/// particle and no energy.
fn is_decay_run(init: &LheInit) -> bool {
    init.beam_pdg[1] == 0 && init.beam_energy[1] == 0.0
}

fn check_init(init: &LheInit, complaints: &mut Vec<Complaint>) {
    let mut say = |what: String| complaints.push(Complaint { event: None, what });

    if init.processes.is_empty() {
        say("<init> declares no processes".into());
    }
    let beams = if is_decay_run(init) { 1 } else { 2 };
    for (beam, energy) in init.beam_energy.iter().enumerate().take(beams) {
        if !energy.is_finite() || *energy <= 0.0 {
            say(format!("<init> beam {} energy is {energy}", beam + 1));
        }
    }
    for process in &init.processes {
        if !process.xsec_pb.is_finite() || process.xsec_pb <= 0.0 {
            say(format!(
                "process {} has cross section {}",
                process.id, process.xsec_pb
            ));
        }
        if !process.xmax.is_finite() || process.xmax <= 0.0 {
            say(format!(
                "process {} has XMAXUP {}",
                process.id, process.xmax
            ));
        }
    }
}

fn check_event(
    index: usize,
    event: &LheEvent,
    init: &LheInit,
    tolerance: f64,
    complaints: &mut Vec<Complaint>,
) {
    let mut say = |what: String| {
        complaints.push(Complaint {
            event: Some(index),
            what,
        })
    };

    let Some(process) = init.processes.iter().find(|p| p.id == event.process_id) else {
        say(format!(
            "refers to process {}, which <init> does not declare",
            event.process_id
        ));
        return;
    };

    if !event.weight.is_finite() {
        say(format!("weight is {}", event.weight));
    }
    // XMAXUP is what a consumer re-unweighting the sample divides by, so a
    // weight above it silently truncates that consumer's tail.
    if event.weight.abs() > process.xmax * (1.0 + tolerance) {
        say(format!(
            "weight {} exceeds the process's XMAXUP {}",
            event.weight, process.xmax
        ));
    }
    if !event.scale.is_finite() || event.scale <= 0.0 {
        say(format!("SCALUP is {}", event.scale));
    }
    if init.weight_strategy == WeightStrategy::UnitWeight
        && (event.weight.abs() - 1.0).abs() > tolerance
    {
        say(format!(
            "IDWTUP = +3 promises unit weights, but this one is {}",
            event.weight
        ));
    }

    let incoming: Vec<_> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_INCOMING)
        .collect();
    let outgoing: Vec<_> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING)
        .collect();
    let expected_incoming = if is_decay_run(init) { 1 } else { 2 };
    if incoming.len() != expected_incoming {
        say(format!(
            "has {} incoming legs, not {expected_incoming}",
            incoming.len()
        ));
    }
    if outgoing.is_empty() {
        say("has no outgoing legs".into());
    }
    for particle in &event.particles {
        if !matches!(
            particle.status,
            STATUS_INCOMING | STATUS_OUTGOING | STATUS_INTERMEDIATE
        ) {
            say(format!("leg has unknown ISTUP {}", particle.status));
        }
        for mother in particle.mothers {
            if mother < 0 || mother as usize > event.particles.len() {
                say(format!(
                    "leg names mother {mother}, outside 0..={}",
                    event.particles.len()
                ));
            }
        }
    }

    // Only the initial and final states enter the balance: an intermediate is a
    // listed resonance whose decay products are already counted.
    let mut balance = [0.0f64; 4];
    let mut scale = 0.0f64;
    for particle in incoming.iter().chain(outgoing.iter()) {
        let sign = if particle.status == STATUS_INCOMING {
            1.0
        } else {
            -1.0
        };
        for (component, value) in balance.iter_mut().zip(particle.momentum) {
            *component += sign * value;
        }
        scale += particle.momentum[0].abs();
    }
    for (component, residual) in balance.iter().enumerate() {
        if residual.abs() > tolerance * scale.max(1.0) {
            say(format!(
                "momentum component {component} does not balance: {residual:e} against a scale of \
                 {scale:e}"
            ));
        }
    }

    for particle in &event.particles {
        if particle.status == STATUS_INTERMEDIATE {
            continue;
        }
        let [e, px, py, pz] = particle.momentum;
        let virtuality = e * e - px * px - py * py - pz * pz;
        let residual = virtuality - particle.mass * particle.mass;
        // Referenced to E², the one scale in the identity that is never a
        // difference of comparable numbers.
        if residual.abs() > tolerance * (e * e).max(1.0) {
            say(format!(
                "leg {} is off its mass shell: p² = {virtuality:e} against m² = {:e}",
                particle.pdg,
                particle.mass * particle.mass
            ));
        }
    }
}

fn report(file: &LheFile) {
    let strategy = file.init.weight_strategy;
    let weights: Vec<f64> = file.events.iter().map(|e| e.weight).collect();
    let n = weights.len() as f64;
    let mean = if weights.is_empty() {
        0.0
    } else {
        weights.iter().sum::<f64>() / n
    };
    let declared: f64 = file.init.processes.iter().map(|p| p.xsec_pb).sum();

    println!("events        {}", file.events.len());
    println!(
        "beams         {} {} at {:.4} + {:.4} GeV",
        file.init.beam_pdg[0],
        file.init.beam_pdg[1],
        file.init.beam_energy[0],
        file.init.beam_energy[1]
    );
    println!("IDWTUP        {}", strategy.as_i32());
    println!("XSECUP        {declared:.6} pb");
    // For IDWTUP = -4 the sample's own estimate of σ is the mean weight, which
    // agrees with XSECUP only up to the sample's statistics — printed rather
    // than enforced for exactly that reason.
    match strategy {
        WeightStrategy::MeanCrossSectionPb => {
            println!("mean XWGTUP   {mean:.6} pb");
        }
        _ => {
            println!("mean XWGTUP   {mean:.6}");
        }
    }
}

#[cfg(test)]
mod tests {
    use vibegraph::lhef::record::{LheParticle, LheProcess};

    use super::*;

    const TOLERANCE: f64 = DEFAULT_TOLERANCE;

    fn leg(pdg: i32, status: i32, momentum: [f64; 4]) -> LheParticle {
        LheParticle {
            pdg,
            status,
            mothers: if status == STATUS_INCOMING {
                [0, 0]
            } else {
                [1, 2]
            },
            color: [0, 0],
            momentum,
            mass: 0.0,
            lifetime: 0.0,
            spin: 9.0,
        }
    }

    /// A well-formed one-event file: `e+ e- > mu+ mu-` at 91.2 GeV under
    /// `IDWTUP = -4`, with every identity the checks read holding exactly.
    fn good() -> LheFile {
        let e = 45.6;
        let init = LheInit {
            beam_pdg: [-11, 11],
            beam_energy: [e, e],
            pdf_group: [0, 0],
            pdf_set: [0, 0],
            weight_strategy: WeightStrategy::MeanCrossSectionPb,
            processes: vec![LheProcess {
                xsec_pb: 2000.0,
                xerr_pb: 1.0,
                xmax: 2000.0,
                id: 1,
            }],
            trailer: Vec::new(),
            source: None,
        };
        let event = LheEvent {
            process_id: 1,
            weight: 2000.0,
            scale: 2.0 * e,
            alpha_qed: 1.0 / 132.5,
            alpha_qcd: 0.118,
            particles: vec![
                leg(-11, STATUS_INCOMING, [e, 0.0, 0.0, e]),
                leg(11, STATUS_INCOMING, [e, 0.0, 0.0, -e]),
                leg(-13, STATUS_OUTGOING, [e, 0.0, e, 0.0]),
                leg(13, STATUS_OUTGOING, [e, 0.0, -e, 0.0]),
            ],
            trailer: Vec::new(),
            source: None,
        };
        LheFile {
            init,
            events: vec![event],
        }
    }

    fn said(file: &LheFile, min_events: Option<usize>) -> Vec<String> {
        complaints(file, TOLERANCE, min_events)
            .iter()
            .map(Complaint::to_string)
            .collect()
    }

    /// `file` draws exactly one complaint, and it contains `expected`.
    #[track_caller]
    fn complains_once(file: &LheFile, min_events: Option<usize>, expected: &str) {
        let said = said(file, min_events);
        assert!(
            said.len() == 1 && said[0].contains(expected),
            "expected one complaint containing {expected:?}, got {said:?}"
        );
    }

    /// The fixture every other test damages passes as it stands, so a complaint
    /// below is the damage's and not the fixture's.
    #[test]
    fn a_well_formed_file_draws_no_complaint() {
        assert_eq!(said(&good(), Some(1)), Vec::<String>::new());
    }

    #[test]
    fn init_complaints() {
        let mut f = good();
        f.init.processes.clear();
        f.events.clear();
        complains_once(&f, None, "<init> declares no processes");

        let mut f = good();
        f.init.beam_energy[1] = -1.0;
        complains_once(&f, None, "<init> beam 2 energy is -1");

        let mut f = good();
        f.init.processes[0].xsec_pb = f64::NAN;
        complains_once(&f, None, "process 1 has cross section NaN");

        let mut f = good();
        f.init.processes[0].xmax = 0.0;
        f.events[0].weight = 0.0;
        complains_once(&f, None, "process 1 has XMAXUP 0");
    }

    /// A decay run leaves beam 2 empty, which is not a bad beam energy.
    #[test]
    fn a_decay_run_checks_one_beam_and_one_incoming_leg() {
        let mut f = good();
        f.init.beam_pdg = [23, 0];
        f.init.beam_energy = [91.2, 0.0];
        let z = leg(23, STATUS_INCOMING, [91.2, 0.0, 0.0, 0.0]);
        f.events[0].particles = vec![
            LheParticle { mass: 91.2, ..z },
            leg(-13, STATUS_OUTGOING, [45.6, 0.0, 45.6, 0.0]),
            leg(13, STATUS_OUTGOING, [45.6, 0.0, -45.6, 0.0]),
        ];
        assert_eq!(said(&f, None), Vec::<String>::new());
        let mother = f.events[0].particles[0];
        f.events[0].particles.insert(0, mother);
        assert!(
            said(&f, None)
                .iter()
                .any(|c| c.contains("has 2 incoming legs, not 1")),
            "{:?}",
            said(&f, None)
        );
    }

    #[test]
    fn an_event_of_an_undeclared_process_is_named() {
        let mut f = good();
        f.events[0].process_id = 7;
        complains_once(
            &f,
            None,
            "event 1: refers to process 7, which <init> does not declare",
        );
    }

    #[test]
    fn weight_and_scale_complaints() {
        let mut f = good();
        f.events[0].weight = f64::INFINITY;
        let said_inf = said(&f, None);
        assert!(
            said_inf.iter().any(|c| c.contains("weight is inf")),
            "{said_inf:?}"
        );

        let mut f = good();
        f.events[0].weight = 2000.0 * (1.0 + 10.0 * TOLERANCE);
        complains_once(&f, None, "exceeds the process's XMAXUP 2000");

        let mut f = good();
        f.events[0].scale = 0.0;
        complains_once(&f, None, "SCALUP is 0");

        let mut f = good();
        f.init.weight_strategy = WeightStrategy::UnitWeight;
        f.init.processes[0].xmax = 1.0;
        f.events[0].weight = 0.5;
        complains_once(
            &f,
            None,
            "IDWTUP = +3 promises unit weights, but this one is 0.5",
        );
    }

    #[test]
    fn leg_bookkeeping_complaints() {
        let mut f = good();
        f.events[0].particles[1].status = STATUS_OUTGOING;
        f.events[0].particles[1].momentum[3] = 45.6;
        let all = said(&f, None);
        assert!(
            all.iter().any(|c| c.contains("has 1 incoming legs, not 2")),
            "{all:?}"
        );

        let mut f = good();
        f.events[0].particles.truncate(2);
        f.events[0].particles[1].status = STATUS_INCOMING;
        let all = said(&f, None);
        assert!(
            all.iter().any(|c| c.contains("has no outgoing legs")),
            "{all:?}"
        );

        let mut f = good();
        f.events[0].particles[2].status = 3;
        let all = said(&f, None);
        assert!(
            all.iter().any(|c| c.contains("leg has unknown ISTUP 3")),
            "{all:?}"
        );

        let mut f = good();
        f.events[0].particles[3].mothers = [1, 5];
        complains_once(&f, None, "leg names mother 5, outside 0..=4");
    }

    #[test]
    fn kinematic_complaints() {
        let mut f = good();
        f.events[0].particles[2].momentum[1] = 1e-3;
        f.events[0].particles[3].momentum[1] = 1e-3;
        let all = said(&f, None);
        assert!(
            all.iter()
                .any(|c| c.contains("momentum component 1 does not balance")),
            "{all:?}"
        );

        let mut f = good();
        f.events[0].particles[2].mass = 0.105_658;
        complains_once(&f, None, "leg -13 is off its mass shell");

        let mut f = good();
        f.events.push(f.events[0].clone());
        complains_once(
            &f,
            Some(3),
            "file holds 2 events, fewer than the 3 required",
        );
    }

    /// An intermediate is a listed resonance whose products are already counted,
    /// so it enters neither the balance nor the mass-shell check.
    #[test]
    fn an_intermediate_leg_is_left_out_of_the_kinematics() {
        let mut f = good();
        let mut z = leg(23, STATUS_INTERMEDIATE, [91.2, 0.0, 0.0, 1.0]);
        z.mass = 50.0;
        f.events[0].particles.push(z);
        assert_eq!(said(&f, None), Vec::<String>::new());
    }
}
