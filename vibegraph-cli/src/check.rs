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

use crate::integrate::IntegrateError;

/// Relative tolerance for the momentum and mass-shell identities.
///
/// The writer emits ten decimal places in exponential form, so a record carries
/// about eleven significant digits; the identities below are sums of that many
/// digits over a handful of legs. Three orders of magnitude of headroom over the
/// representation error keeps a slow accumulation from being flagged while still
/// catching a leg that is simply wrong.
const DEFAULT_TOLERANCE: f64 = 1e-6;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Les Houches file to read back.
    pub events: PathBuf,

    /// Relative tolerance for momentum conservation and mass shells.
    #[arg(long, default_value_t = DEFAULT_TOLERANCE)]
    pub tolerance: f64,

    /// Fail unless the file holds at least this many events.
    #[arg(long)]
    pub min_events: Option<usize>,
}

fn err(msg: impl Into<String>) -> IntegrateError {
    IntegrateError::Message(msg.into())
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

pub fn run(args: &CheckArgs) -> Result<(), IntegrateError> {
    let text = std::fs::read_to_string(&args.events)
        .map_err(|e| err(format!("cannot read {}: {e}", args.events.display())))?;
    let file = LheFile::parse(&text)
        .map_err(|e| err(format!("cannot parse {}: {e}", args.events.display())))?;

    let mut complaints = Vec::new();
    check_init(&file.init, &mut complaints);
    for (index, event) in file.events.iter().enumerate() {
        check_event(
            index + 1,
            event,
            &file.init,
            args.tolerance,
            &mut complaints,
        );
    }
    if let Some(minimum) = args.min_events {
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

fn check_init(init: &LheInit, complaints: &mut Vec<Complaint>) {
    let mut say = |what: String| complaints.push(Complaint { event: None, what });

    if init.processes.is_empty() {
        say("<init> declares no processes".into());
    }
    for (beam, energy) in init.beam_energy.iter().enumerate() {
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
    if incoming.len() != 2 {
        say(format!("has {} incoming legs, not 2", incoming.len()));
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
