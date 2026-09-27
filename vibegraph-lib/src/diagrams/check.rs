//! The one check between a parsed proc card and everything downstream of it.
//!
//! [`super::parse`] reads every construct MadGraph's process language has;
//! [`check_supported`] decides, in one pass over the whole card, which of them
//! this generator can honour, and reports *every* one it cannot rather than the
//! first. What it hands on is a [`SupportedCard`], a narrower type with no field
//! for a refused feature: code downstream cannot read a `$` restriction or a
//! decay chain, because the type it is given has nowhere to hold one.
//! Supporting a feature means giving it a field here, and removing its
//! [`Unsupported`] variant, in the same change as the code that honours it.
//!
//! # The backlog against MadGraph
//!
//! [`Unsupported`] is the list of MadGraph leading-order process features this
//! generator does not implement yet, each with the work that would lift it.
//!
//! | Variant | MadGraph feature | Lifted by |
//! |---|---|---|
//! | [`ChainOrders`](Unsupported::ChainOrders) | `A > B C QED>0, B > D E @1 QED=2` | a meaning for an overall order a part also constrains other than from above (not planned) |
//! | [`DecayedPolarization`](Unsupported::DecayedPolarization) | `p p > w+{0} w-, w+ > e+ ve` | a helicity-projected propagator at the resonance |
//! | [`DecayOnShellVeto`](Unsupported::DecayOnShellVeto) | `e+ e- > z h, h > e+ e- mu+ mu- $ z` | marking a decay's own propagators, per decay |
//! | [`PropagatorPolarization`](Unsupported::PropagatorPolarization) | `{A}`, `{G}`, `{H}`, `{Q}`, `{W}`, `{S}` | helicity-projected propagators (not planned) |
//! | [`SquaredOrder`](Unsupported::SquaredOrder) | `QCD^2<=4`, `aEW`, `aS` | amplitudes split by coupling order (not planned) |
//! | [`WeightedOrder`](Unsupported::WeightedOrder) | `WEIGHTED==4`, `WEIGHTED>4` | amplitudes split by coupling order (not planned) |
//! | [`LoopSpec`](Unsupported::LoopSpec) | `[QCD]`, `[real=QCD]` | NLO |
//! | [`PhotonTag`](Unsupported::PhotonTag) | `!a!` | NLO |
//! | [`MixedMultiplicity`](Unsupported::MixedMultiplicity) | `add process` with another final-state count | MLM merging |
//! | [`ProcessOption`](Unsupported::ProcessOption) | `--diagram_filter`, `--optimize`, `--standalone` | not planned |
//! | [`SetOption`](Unsupported::SetOption) | a physics-bearing `set` off its default | per option |
//! | [`LaunchDialogue`](Unsupported::LaunchDialogue) | run-card edits after `launch` | not planned: the run card is its own file |
//! | [`ModelOption`](Unsupported::ModelOption) | `import model X -modelname`, `add model` | not planned |
//! | [`Command`](Unsupported::Command) | any other command that could change the card | not planned |
//!
//! Two variants are not backlog but MadGraph errors this check is the natural
//! place to report: [`MixedInitialStates`](Unsupported::MixedInitialStates) and
//! [`InitialState`](Unsupported::InitialState).
//!
//! What the check cannot decide without a model — names that resolve to no
//! particle, order names the model does not define, and the same subprocess
//! reached from two process lines — is refused where the model is, during
//! enumeration.

use std::fmt::Display;

use thiserror::Error;

use super::alias::AliasTable;
use super::parse::{
    Command, CouplingOp, LegParticle, LegState, ModelImport, ProcCardAst, ProcessDefinition,
    ProcessLine,
};

// ── The narrow type ───────────────────────────────────────────────────────────

/// A proc card every feature of which this generator honours.
#[derive(Debug, Clone)]
pub struct SupportedCard {
    /// The last `import model`, if any.
    pub model: Option<ModelImport>,
    /// The card's processes after `generate` resets, in card order.
    pub processes: Vec<SupportedProcess>,
}

/// One process line of a [`SupportedCard`].
#[derive(Debug, Clone)]
pub struct SupportedProcess {
    /// MadGraph's process number: the `@N` if the line gives one, otherwise the
    /// line's position among the process lines since the last `generate`.
    pub id: u32,
    pub initial: Vec<SupportedLeg>,
    pub final_state: Vec<SupportedLeg>,
    /// `/ A B`, as written.
    pub forbidden_particles: Vec<String>,
    /// `> A B | C >`, as written: alternatives, each a list of names that must
    /// all be s-channel propagators of a kept diagram.
    pub required_s_channels: Vec<Vec<String>>,
    /// `$$ A B`, as written: names no s-channel propagator of a kept diagram
    /// may carry.
    pub forbidden_s_channels: Vec<String>,
    /// `$ A B`, as written: names whose s-channel propagators every diagram
    /// keeps, but whose Breit–Wigner window is vetoed in the integration
    /// configurations that carry them.
    pub forbidden_onshell_s_channels: Vec<String>,
    /// Amplitude-level coupling-order constraints, left to right. A
    /// `WEIGHTED` entry is always `<=` or `=`.
    pub orders: Vec<AmplitudeOrder>,
    /// The labels as defined when the line was read.
    pub aliases: AliasTable,
    /// The overall orders of a decay-chain line (`@1 QED=2`), as upper bounds on
    /// every stitched diagram's orders; each part's [`orders`](Self::orders)
    /// already carries them folded in. Empty on a decay and on a line without
    /// overall orders.
    pub chain_orders: Vec<AmplitudeOrder>,
    /// The decays of a decay-chain line, each with its own decays, in the order written
    /// (`p p > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~` has `t > w+ b`, holding
    /// `w+ > e+ ve`, then `t~ > w- b~`). Empty on a line without decays.
    pub decays: Vec<SupportedProcess>,
}

/// One external leg of a [`SupportedProcess`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedLeg {
    pub particle: LegParticle,
    /// The token as written (`2e+` for a leg from a repeat count).
    pub token: String,
    /// The text between `{` and `}` of a polarized leg (`0`, `T`, `L`, `+1`),
    /// as written. Its helicity codes need the particle's spin, so they are
    /// read where the model is.
    pub polarization: Option<String>,
}

impl Display for SupportedLeg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.particle)?;
        if let Some(pol) = &self.polarization {
            write!(f, "{{{pol}}}")?;
        }
        Ok(())
    }
}

/// An amplitude-level coupling-order constraint: a bound on the order of each
/// diagram, not of a product of two.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AmplitudeOrder {
    pub name: String,
    pub op: CouplingOp,
    pub value: i64,
}

impl Display for AmplitudeOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.name, self.op, self.value)
    }
}

impl Display for SupportedProcess {
    /// The legs, the s-channel restrictions, the coupling-order constraints and
    /// the decays, spelled the way MadGraph's own generate line spells them.
    /// Carrying everything that selects diagrams is what keeps two processes
    /// that differ only in a constraint from printing identically. The
    /// forbidden particles are not printed, nor the process number except before
    /// a decay chain's overall orders, so this is not a round trip.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let side = |legs: &[SupportedLeg]| {
            legs.iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };
        write!(f, "{} > ", side(&self.initial))?;
        if !self.required_s_channels.is_empty() {
            let alternatives: Vec<String> = self
                .required_s_channels
                .iter()
                .map(|a| a.join(" "))
                .collect();
            write!(f, "{} > ", alternatives.join(" | "))?;
        }
        write!(f, "{}", side(&self.final_state))?;
        if !self.forbidden_onshell_s_channels.is_empty() {
            write!(f, " $ {}", self.forbidden_onshell_s_channels.join(" "))?;
        }
        if !self.forbidden_s_channels.is_empty() {
            write!(f, " $$ {}", self.forbidden_s_channels.join(" "))?;
        }
        for order in &self.orders {
            write!(f, " {order}")?;
        }
        for decay in &self.decays {
            if decay.decays.is_empty() {
                write!(f, ", {decay}")?;
            } else {
                write!(f, ", ({decay})")?;
            }
        }
        if !self.chain_orders.is_empty() {
            write!(f, " @{}", self.id)?;
            for order in &self.chain_orders {
                write!(f, " {}={}", order.name, order.value)?;
            }
        }
        Ok(())
    }
}

// ── The backlog ───────────────────────────────────────────────────────────────

/// A feature of a card that this generator does not honour. See the module
/// documentation for the table of variants and what lifts each.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Unsupported {
    /// An overall coupling order on a decay-chain line (`@1 QED=2` after the process
    /// number) that one of the chain's parts also constrains with `==` or `>`.
    /// MadGraph folds an overall order into each part's own orders as the lesser of
    /// the two upper bounds (`diagram_generation.py:570`); a part's lower or exact
    /// bound on the same order has no counterpart there.
    #[error(
        "'{process}': the overall order '{order}' is also constrained by '{constraint}' in \
         a part of the chain; an overall order caps each part's upper bound, and a lower \
         or exact bound on the same order has no meaning beside it"
    )]
    ChainOrders {
        process: String,
        order: String,
        constraint: String,
    },
    /// More than two initial particles: no phase space or flux here or in
    /// MadEvent describes one.
    #[error("'{process}': {n} initial-state particles; a process has one or two")]
    InitialState { process: String, n: usize },
    /// A polarization on a leg a decay chain decays, `p p > w+{0} w-, w+ >
    /// e+ ve`: the resonance is then a propagator, and MadGraph replaces its
    /// numerator by the projection on the named helicities. Also a polarized
    /// decaying particle of a `1 → n` process, `t{L} > w+ b`: at rest its
    /// helicity is a spin projection on an axis the wavefunction routine
    /// chooses, which nothing here pins against MadGraph.
    #[error(
        "'{process}': polarization '{leg}' is on a particle the line decays, which is not \
         supported: a decay chain's resonance would need a helicity-projected propagator, and \
         a decaying particle at rest a spin axis pinned against MadGraph"
    )]
    DecayedPolarization { process: String, leg: String },
    /// A forbidden on-shell s-channel (`$ A`) on a decay of a decay chain,
    /// `e+ e- > z h, h > e+ e- mu+ mu- $ z`. MadGraph marks the decay
    /// amplitude's own propagators; the veto here is installed from one list per
    /// card and marks the core's propagators only, so a decay's list would go
    /// unread. A `$` on the core of a chain is supported.
    #[error(
        "'{process}': the forbidden on-shell s-channel '$ {names}' is on the decay \
         '{decay}', which is not supported: the veto marks the core process's propagators \
         only. A '$' on the core of the chain is supported"
    )]
    DecayOnShellVeto {
        process: String,
        decay: String,
        names: String,
    },
    /// The propagator-only polarization codes (`{A}` auxiliary, `{G}` metric,
    /// `{H}`, `{Q}`, `{W}`, `{S}`), which replace a resonance's propagator
    /// numerator by a projection.
    #[error(
        "'{process}': polarization '{leg}' names a propagator projection, which is not \
         supported"
    )]
    PropagatorPolarization { process: String, leg: String },
    /// A squared-order constraint, `QCD^2<=4`, and the `aEW` / `aS` spellings
    /// MadGraph turns into one. It bounds the order of an interference term in
    /// |M|², a statement about pairs of diagrams; this generator selects
    /// diagrams by their own orders and squares the whole amplitude.
    #[error(
        "'{process}': squared-order constraint '{constraint}' is not supported: it bounds the \
         order of interference terms in |M|^2, and this generator selects diagrams by their \
         coupling orders and squares the whole amplitude. Ask for the amplitude-level order \
         instead (e.g. 'QED<=n'), which keeps every term a diagram of that order contributes to"
    )]
    SquaredOrder { process: String, constraint: String },
    /// `WEIGHTED==n` or `WEIGHTED>n`. MadGraph turns an `==` or `>` amplitude
    /// constraint into a squared-order one as well, which bounds interference
    /// terms rather than diagrams. `WEIGHTED<=n` and `WEIGHTED=n` are
    /// supported: they bound each diagram's weighted order, the filter the
    /// automatic order search uses.
    #[error(
        "'{process}': the WEIGHTED constraint '{constraint}' is not supported: MadGraph reads \
         '==' and '>' as squared-order constraints on interference terms. 'WEIGHTED<=n' is \
         supported"
    )]
    WeightedOrder { process: String, constraint: String },
    /// A loop / perturbation specification, `[QCD]`, `[real=QCD]`.
    #[error(
        "'{process}': '[...]' asks for NLO (or split-order) output, which this LO generator \
         does not produce"
    )]
    LoopSpec { process: String },
    /// A tagged photon, `!a!`, which exists for NLO electroweak corrections.
    #[error("'{process}': tagged photons ('!a!') are an NLO feature, not supported")]
    PhotonTag { process: String },
    /// A `--` option on a process line other than `--no_warning=duplicate`.
    #[error("'{process}': process option '{flag}' is not supported")]
    ProcessOption { process: String, flag: String },
    /// Processes with different final-state multiplicities in one card, which
    /// only make sense merged (MLM).
    #[error(
        "'{first}' and '{other}' have different final-state multiplicities; combining them \
         needs jet merging, which is not supported"
    )]
    MixedMultiplicity { first: String, other: String },
    /// Processes with different numbers of initial particles in one card. A
    /// MadGraph error too.
    #[error("'{first}' and '{other}' have different numbers of initial-state particles")]
    MixedInitialStates { first: String, other: String },
    /// A `set` of an interface option this generator does not honour, moved
    /// off MadGraph's default.
    #[error("'set {option} {value}': {why}")]
    SetOption {
        option: String,
        value: String,
        why: &'static str,
    },
    /// Lines a script feeds to `launch`'s questions: run-card and param-card
    /// edits, which this generator reads from its own card files.
    #[error(
        "'launch' is followed by card edits ({lines}); put them in the run card or param \
         card this generator reads instead"
    )]
    LaunchDialogue { lines: String },
    /// Options on `import model`, or a second model merged in with `add model`.
    #[error("'{command}': model options and model merging are not supported")]
    ModelOption { command: String },
    /// Any other command that could change what the card means.
    #[error("'{command}' is not a command this generator reads")]
    Command { command: String },
}

/// Every [`Unsupported`] feature of a card, in the order they appear.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct UnsupportedCard(pub Vec<Unsupported>);

impl Display for UnsupportedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the proc card uses {} unsupported feature{}:",
            self.0.len(),
            if self.0.len() == 1 { "" } else { "s" }
        )?;
        for u in &self.0 {
            write!(f, "\n  - {u}")?;
        }
        Ok(())
    }
}

// ── `set` options ─────────────────────────────────────────────────────────────

/// Where a MadGraph interface option goes.
enum SetClass {
    /// Cannot reach the processes, the diagrams or the cross section.
    Benign,
    /// Changes what MadGraph generates. Accepted only at MadGraph's default.
    Physics {
        default: &'static str,
        why: &'static str,
    },
}

const S_TOOLING: &[&str] = &[
    "stdout_level",
    "fortran_compiler",
    "cpp_compiler",
    "f2py_compiler",
    "f2py_compiler_py2",
    "f2py_compiler_py3",
    "timeout",
    "web_browser",
    "eps_viewer",
    "text_editor",
    "use_pigz",
    "automatic_html_opening",
    "notification_center",
    "run_mode",
    "nb_core",
    "cluster_type",
    "cluster_queue",
    "cluster_status_update",
    "cluster_walltime",
    "cluster_requirement",
    "cluster_vacatetime",
    "cluster_temp_path",
    "cluster_local_path",
    "cluster_size",
    "cluster_retry_wait",
    "cluster_nb_retry",
    "checkpointing",
    "enforce_shared_disk",
    "auto_update",
    "auto_convert_model",
    "crash_on_error",
    "output_dependencies",
    "acknowledged_v3.1_syntax",
    "lhapdf",
    "lhapdf_py2",
    "lhapdf_py3",
    "pineappl",
    "fastjet",
    "eMELA",
    "golem",
    "samurai",
    "ninja",
    "collier",
    "dmtcp",
    "pythia8_path",
    "hwpp_path",
    "thepeg_path",
    "hepmc_path",
    "madanalysis_path",
    "madanalysis5_path",
    "pythia-pgs_path",
    "rivet_path",
    "yoda_path",
    "contur_path",
    "td_path",
    "delphes_path",
    "exrootanalysis_path",
    "syscalc_path",
    "mg5amc_py8_interface_path",
    "low_mem_multicore_nlo_generation",
];

/// MadGraph's own grouping of subprocesses and choice of integration
/// channels: both change how MadEvent organises the work and none changes a
/// cross section, and this generator makes those choices itself.
const S_ORGANISATION: &[&str] = &[
    "group_subprocesses",
    "max_npoint_for_channel",
    "max_t_for_channel",
];

/// Read only by loop / NLO generation, which the check refuses on its own.
const S_NLO: &[&str] = &[
    "loop_optimized_output",
    "loop_color_flows",
    "nlo_mixed_expansion",
];

fn set_class(option: &str) -> Option<SetClass> {
    if S_TOOLING.contains(&option) || S_ORGANISATION.contains(&option) || S_NLO.contains(&option) {
        return Some(SetClass::Benign);
    }
    let physics = |default, why| Some(SetClass::Physics { default, why });
    match option {
        "complex_mass_scheme" => physics(
            "false",
            "the complex-mass scheme changes every massive propagator and coupling",
        ),
        "gauge" => physics(
            "unitary",
            "this generator works in unitary gauge; another gauge changes the diagram set",
        ),
        "zerowidth_tchannel" => physics(
            "true",
            "MadGraph's default removes widths from t-channel propagators; the other setting \
             changes the amplitude",
        ),
        "default_unset_couplings" => physics(
            "99",
            "it bounds every coupling order a process line leaves unconstrained",
        ),
        "ignore_six_quark_processes" => {
            physics("false", "it removes subprocesses from the enumeration")
        }
        "include_lepton_initiated_processes" => physics(
            "false",
            "it adds lepton-initiated subprocesses to proton beams",
        ),
        "EWscheme" => physics(
            "",
            "it changes the electroweak input scheme the couplings are derived from",
        ),
        _ => None,
    }
}

/// Whether a `set` value spells MadGraph's default.
fn is_default(value: &str, default: &str) -> bool {
    let v = value.to_lowercase();
    match default {
        "true" => matches!(v.as_str(), "true" | "t" | "1" | "on" | ".true."),
        "false" => matches!(v.as_str(), "false" | "f" | "0" | "off" | ".false." | "none"),
        "" => false,
        d => v == d,
    }
}

/// Commands that cannot change which processes a card describes or how they
/// are computed.
const BENIGN_COMMANDS: &[&str] = &[
    "output", "display", "help", "history", "exit", "quit", "save", "open", "check", "done",
    "tutorial", "info",
];

// ── The check ─────────────────────────────────────────────────────────────────

/// Check every feature of a card at once, and narrow it to what is supported.
pub fn check_supported(ast: &ProcCardAst) -> Result<SupportedCard, UnsupportedCard> {
    let mut refused = Vec::new();

    for command in &ast.commands {
        match command {
            Command::ImportModel { import, options } if !options.is_empty() => {
                refused.push(Unsupported::ModelOption {
                    command: format!(
                        "import model {}{} {}",
                        import.name,
                        import
                            .restrict_variant
                            .as_deref()
                            .map(|r| format!("-{r}"))
                            .unwrap_or_default(),
                        options.join(" ")
                    ),
                })
            }
            Command::AddModel(args) => refused.push(Unsupported::ModelOption {
                command: format!("add model {}", args.join(" ")),
            }),
            Command::Import { kind, args } => refused.push(Unsupported::Command {
                command: format!("import {kind} {}", args.join(" ")),
            }),
            Command::Set { option, args } => {
                let value = args.join(" ");
                match set_class(option) {
                    Some(SetClass::Benign) => {}
                    Some(SetClass::Physics { default, why }) => {
                        if !is_default(&value, default) {
                            refused.push(Unsupported::SetOption {
                                option: option.clone(),
                                value,
                                why,
                            })
                        }
                    }
                    None => refused.push(Unsupported::SetOption {
                        option: option.clone(),
                        value,
                        why: "not a MadGraph interface option this generator knows",
                    }),
                }
            }
            Command::Launch { dialogue, .. } if !dialogue.is_empty() => {
                refused.push(Unsupported::LaunchDialogue {
                    lines: dialogue.join("; "),
                })
            }
            Command::Other { verb, args } if !BENIGN_COMMANDS.contains(&verb.as_str()) => refused
                .push(Unsupported::Command {
                    command: format!("{verb} {args}").trim().to_owned(),
                }),
            _ => {}
        }
    }

    let mut processes = Vec::new();
    let mut first: Option<(usize, usize, String)> = None;
    for card_process in ast.processes() {
        let line = card_process.line;
        let before = refused.len();
        check_line(line, &mut refused);
        let def = &line.definition;
        let n_in = def.initial().count();
        let n_out = def.final_state().count();
        match &first {
            None => first = Some((n_in, n_out, line.text.clone())),
            Some((i, _, first)) if *i != n_in => refused.push(Unsupported::MixedInitialStates {
                first: first.clone(),
                other: line.text.clone(),
            }),
            Some((_, o, first)) if *o != n_out && def.decay_chains.is_empty() => {
                refused.push(Unsupported::MixedMultiplicity {
                    first: first.clone(),
                    other: line.text.clone(),
                })
            }
            Some(_) => {}
        }
        if refused.len() == before {
            let mut process = narrow(
                def,
                card_process.id,
                card_process.aliases,
                &def.overall_orders,
            );
            process.chain_orders = overall_orders(&def.overall_orders);
            processes.push(process);
        }
    }

    if refused.is_empty() {
        Ok(SupportedCard {
            model: ast.model().cloned(),
            processes,
        })
    } else {
        Err(UnsupportedCard(refused))
    }
}

/// The unsupported features of one process line.
fn check_line(line: &ProcessLine, refused: &mut Vec<Unsupported>) {
    let process = || line.text.clone();
    for flag in &line.flags {
        if flag != "--no_warning=duplicate" {
            refused.push(Unsupported::ProcessOption {
                process: process(),
                flag: flag.clone(),
            });
        }
    }
    let def = &line.definition;
    for (name, value) in &def.overall_orders {
        if let Some(constraint) = part_bound_other_than_above(def, name) {
            refused.push(Unsupported::ChainOrders {
                process: process(),
                order: format!("{name}={value}"),
                constraint,
            });
        }
    }
    match def.initial().count() {
        1 | 2 => {}
        n => refused.push(Unsupported::InitialState {
            process: process(),
            n,
        }),
    }
    // The initial leg of a 1 -> n line is the decaying particle, as it is in a
    // decay chain's own decay.
    let decay = def.initial().count() == 1;
    check_definition(def, &process(), decay, refused);
}

/// A constraint on the order `name`, in `def` or any of its decays, that is not an
/// upper bound, as written.
fn part_bound_other_than_above(def: &ProcessDefinition, name: &str) -> Option<String> {
    def.orders
        .iter()
        .find(|o| o.name == name && !matches!(o.op, CouplingOp::Le | CouplingOp::Eq))
        .map(|o| o.to_string())
        .or_else(|| {
            def.decay_chains
                .iter()
                .find_map(|d| part_bound_other_than_above(d, name))
        })
}

/// The features of a definition and of its decays. The number of initial
/// particles is checked on the line only: a decay has one by construction.
/// `decay` marks a decay's own definition, whose initial leg is the resonance.
fn check_definition(
    def: &ProcessDefinition,
    text: &str,
    decay: bool,
    refused: &mut Vec<Unsupported>,
) {
    let process = || text.to_owned();
    if def.loop_spec.is_some() {
        refused.push(Unsupported::LoopSpec { process: process() });
    }
    if def.legs.iter().any(|l| l.tagged) {
        refused.push(Unsupported::PhotonTag { process: process() });
    }
    let decayed: Vec<&LegParticle> = def
        .decay_chains
        .iter()
        .filter_map(|d| d.initial().next())
        .map(|l| &l.particle)
        .collect();
    for leg in &def.legs {
        if let Some(pol) = &leg.polarization {
            let leg_text = format!("{}{{{pol}}}", leg.token);
            let projection = pol
                .chars()
                .any(|c| matches!(c.to_ascii_uppercase(), 'A' | 'G' | 'H' | 'Q' | 'W' | 'S'));
            if projection {
                refused.push(Unsupported::PropagatorPolarization {
                    process: process(),
                    leg: leg_text,
                });
            } else if (leg.state == LegState::Final && decayed.contains(&&leg.particle))
                || (decay && leg.state == LegState::Initial)
            {
                refused.push(Unsupported::DecayedPolarization {
                    process: process(),
                    leg: leg_text,
                });
            }
        }
    }
    for order in &def.orders {
        if order.squared || order.name == "aEW" || order.name == "aS" {
            refused.push(Unsupported::SquaredOrder {
                process: process(),
                constraint: order.to_string(),
            });
        } else if order.name == "WEIGHTED" && !matches!(order.op, CouplingOp::Le | CouplingOp::Eq) {
            refused.push(Unsupported::WeightedOrder {
                process: process(),
                constraint: order.to_string(),
            });
        }
    }
    for decay in &def.decay_chains {
        if !decay.forbidden_onsh_s_channels.is_empty() {
            refused.push(Unsupported::DecayOnShellVeto {
                process: process(),
                decay: decay_text(decay),
                names: decay.forbidden_onsh_s_channels.join(" "),
            });
        }
        check_definition(decay, text, true, refused);
    }
}

/// A decay as its legs spell it, `h > e+ e- mu+ mu-`.
fn decay_text(def: &ProcessDefinition) -> String {
    let side = |state: LegState| {
        def.legs
            .iter()
            .filter(|l| l.state == state)
            .map(|l| l.token.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };
    format!("{} > {}", side(LegState::Initial), side(LegState::Final))
}

/// A checked definition as the narrow type.
/// Overall orders as upper bounds.
fn overall_orders(overall: &[(String, i64)]) -> Vec<AmplitudeOrder> {
    overall
        .iter()
        .map(|(name, value)| AmplitudeOrder {
            name: name.clone(),
            op: CouplingOp::Le,
            value: *value,
        })
        .collect()
}

/// A part's own orders with each overall order folded in as MadGraph folds it
/// (`diagram_generation.py:570`): the lesser of the part's upper bound and the
/// overall one, or the overall one where the part has none. A part carrying any
/// order then skips the lowest-order search, as MadGraph's does
/// (`diagram_generation.py:1972`).
fn capped_by_overall(orders: &mut Vec<AmplitudeOrder>, overall: &[(String, i64)]) {
    for (name, value) in overall {
        match orders.iter_mut().find(|o| &o.name == name) {
            Some(o) => o.value = o.value.min(*value),
            None => orders.push(AmplitudeOrder {
                name: name.clone(),
                op: CouplingOp::Le,
                value: *value,
            }),
        }
    }
}

fn narrow(
    def: &ProcessDefinition,
    id: u32,
    aliases: AliasTable,
    overall: &[(String, i64)],
) -> SupportedProcess {
    let legs = |legs: &mut dyn Iterator<Item = &super::parse::Leg>| {
        legs.map(|l| SupportedLeg {
            particle: l.particle.clone(),
            token: l.token.clone(),
            polarization: l.polarization.clone(),
        })
        .collect()
    };
    let mut orders: Vec<AmplitudeOrder> = def
        .orders
        .iter()
        .map(|o| AmplitudeOrder {
            name: o.name.clone(),
            op: o.op,
            value: o.value,
        })
        .collect();
    capped_by_overall(&mut orders, overall);
    SupportedProcess {
        id,
        initial: legs(&mut def.initial()),
        final_state: legs(&mut def.final_state()),
        forbidden_particles: def.forbidden_particles.clone(),
        required_s_channels: def.required_s_channels.clone(),
        forbidden_s_channels: def.forbidden_s_channels.clone(),
        forbidden_onshell_s_channels: def.forbidden_onsh_s_channels.clone(),
        orders,
        chain_orders: Vec::new(),
        decays: def
            .decay_chains
            .iter()
            .map(|decay| narrow(decay, id, aliases.clone(), overall))
            .collect(),
        aliases,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::parse::parse_proc_card_ast;

    fn check(card: &str) -> Result<SupportedCard, UnsupportedCard> {
        check_supported(&parse_proc_card_ast(card).expect("the card parses"))
    }

    fn refused(card: &str) -> Vec<Unsupported> {
        check(card).expect_err("the card is refused").0
    }

    /// A `$` on a decay of a chain is refused, at any depth, while one on the
    /// chain's core, and one on a `1 → n` process line, are supported.
    #[test]
    fn a_forbidden_onshell_line_on_a_decay_is_refused() {
        for card in [
            "generate e+ e- > z h, h > e+ e- mu+ mu- $ z",
            "generate p p > t t~, (t > w+ b, w+ > e+ ve $ a), t~ > w- b~",
        ] {
            let all = refused(card);
            assert!(
                matches!(all.as_slice(), [Unsupported::DecayOnShellVeto { .. }]),
                "{card}: {all:?}"
            );
        }
        for card in [
            "generate e+ e- > mu+ mu- z $ z, z > e+ e-",
            "generate t > b e+ ve $ w+",
        ] {
            assert!(check(card).is_ok(), "{card}");
        }
    }

    #[test]
    fn a_plain_card_is_supported_and_keeps_its_numbers() {
        let card =
            check("import model sm\ngenerate p p > e+ e- @3\nadd process p p > mu+ mu- QCD=0\n")
                .unwrap();
        assert_eq!(card.model.unwrap().name, "sm");
        let ids: Vec<u32> = card.processes.iter().map(|p| p.id).collect();
        assert_eq!(ids, [3, 2]);
        assert_eq!(card.processes[1].to_string(), "p p > mu+ mu- QCD=0");
    }

    /// Every refused feature of a card is reported, not just the first.
    #[test]
    fn every_unsupported_feature_is_reported_at_once() {
        let all = refused(
            "set complex_mass_scheme True\n\
             generate p p > z > e+ e- $ a / h\n\
             add process p p > w+{0} w- [QCD] QED^2<=4\n\
             add process t > w+ b\n\
             add process p p > t t~, t > w+ b\n\
             add process p p > e+ e- j\n\
             output x\nlaunch\nset ebeam1 45\ndone\n",
        );
        let kinds: Vec<&str> = all
            .iter()
            .map(|u| match u {
                Unsupported::SetOption { .. } => "set",
                Unsupported::LoopSpec { .. } => "[]",
                Unsupported::SquaredOrder { .. } => "^2",
                Unsupported::MixedMultiplicity { .. } => "mlm",
                Unsupported::MixedInitialStates { .. } => "mixed",
                Unsupported::LaunchDialogue { .. } => "launch",
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(kinds, ["set", "launch", "[]", "^2", "mixed", "mlm"]);
    }

    /// A decay chain's overall orders fold into every part as the lesser upper
    /// bound, and ride on the line as the bound on the stitched diagrams.
    #[test]
    fn overall_orders_cap_every_part_and_bound_the_chain() {
        let card =
            check("generate p p > t t~ QED=1, (t > w+ b, w+ > e+ ve) @3 QED=2 QCD=2").unwrap();
        let p = &card.processes[0];
        let bounds = |orders: &[AmplitudeOrder]| {
            orders
                .iter()
                .map(|o| (o.name.clone(), o.op, o.value))
                .collect::<Vec<_>>()
        };
        use CouplingOp::{Eq, Le};
        // The core's own `QED=1` is the lesser bound and keeps its spelling.
        assert_eq!(
            bounds(&p.orders),
            [("QED".into(), Eq, 1), ("QCD".into(), Le, 2)]
        );
        assert_eq!(
            bounds(&p.decays[0].orders),
            [("QED".into(), Le, 2), ("QCD".into(), Le, 2)]
        );
        assert_eq!(
            bounds(&p.decays[0].decays[0].orders),
            [("QED".into(), Le, 2), ("QCD".into(), Le, 2)]
        );
        assert_eq!(
            bounds(&p.chain_orders),
            [("QED".into(), Le, 2), ("QCD".into(), Le, 2)]
        );
        assert!(p.decays[0].chain_orders.is_empty());
        assert!(p.to_string().ends_with(" @3 QED=2 QCD=2"), "{p}");
        // A part's lower bound on an overall order has no meaning beside it.
        let refusal = refused("generate p p > t t~ QED>0, t > w+ b @1 QED=2");
        assert!(matches!(
            &refusal[..],
            [Unsupported::ChainOrders { constraint, .. }] if constraint == "QED>0"
        ));
    }

    /// `>`, `$` and `$$` reach the narrow type as written, and print in
    /// MadGraph's spelling and order (`Process.nice_string` writes `$` before
    /// `$$`).
    #[test]
    fn s_channel_restrictions_are_carried() {
        let card = check("define v = z | a\ngenerate e+ e- > v h | z > mu+ mu- h $$ t t~").unwrap();
        let p = &card.processes[0];
        assert_eq!(p.required_s_channels, [vec!["v", "h"], vec!["z"]]);
        assert_eq!(p.forbidden_s_channels, ["t", "t~"]);
        assert_eq!(p.to_string(), "e+ e- > v h | z > mu+ mu- h $$ t t~");

        let card = check("generate u u~ > w+ b w- b~ $ t t~ $$ h").unwrap();
        let p = &card.processes[0];
        assert_eq!(p.forbidden_onshell_s_channels, ["t", "t~"]);
        assert_eq!(p.to_string(), "u u~ > w+ b w- b~ $ t t~ $$ h");
    }

    #[test]
    fn a_second_generate_discards_the_first_processes() {
        let card = check("generate t > w+ b\ngenerate e+ e- > mu+ mu-\n").unwrap();
        assert_eq!(card.processes.len(), 1);
        assert_eq!(card.processes[0].to_string(), "e+ e- > mu+ mu-");
    }

    #[test]
    fn mixed_initial_states_are_refused() {
        let all = refused("generate e+ e- > mu+ mu-\nadd process e+ e- e+ > mu+ mu-\n");
        assert!(all
            .iter()
            .any(|u| matches!(u, Unsupported::InitialState { n: 3, .. })));
        assert!(all
            .iter()
            .any(|u| matches!(u, Unsupported::MixedInitialStates { .. })));
    }

    #[test]
    fn set_options_are_classified_against_their_defaults() {
        assert!(check("set group_subprocesses False\nset complex_mass_scheme F\n").is_ok());
        assert!(check("set gauge unitary\nset zerowidth_tchannel True\n").is_ok());
        assert!(matches!(
            refused("set gauge Feynman")[..],
            [Unsupported::SetOption { .. }]
        ));
        assert!(matches!(
            refused("set ebeam1 45")[..],
            [Unsupported::SetOption { .. }]
        ));
    }

    #[test]
    fn benign_and_unknown_commands() {
        assert!(
            check("generate e+ e- > mu+ mu-\noutput x -nojpeg\ndisplay diagrams\nlaunch\n").is_ok()
        );
        assert!(matches!(
            refused("Generate e+ e- > mu+ mu-")[..],
            [Unsupported::Command { .. }]
        ));
        assert!(matches!(
            refused("generate e+ e- > mu+ mu- --optimize")[..],
            [Unsupported::ProcessOption { .. }]
        ));
        assert!(check(
            "generate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu- --no_warning=duplicate"
        )
        .is_ok());
    }

    /// An external leg's polarization reaches the narrow type as written; a
    /// propagator projection, and a polarization on a resonance a decay chain
    /// decays, do not.
    #[test]
    fn polarizations_on_external_legs_are_carried_and_on_propagators_refused() {
        let card = check("generate e+ e-{L} > w+{0} w-{T}").unwrap();
        let p = &card.processes[0];
        assert_eq!(p.initial[1].polarization.as_deref(), Some("L"));
        assert_eq!(p.final_state[0].polarization.as_deref(), Some("0"));
        assert_eq!(p.final_state[1].polarization.as_deref(), Some("T"));
        assert_eq!(p.initial[0].polarization, None);
        assert_eq!(p.to_string(), "e+ e-{L} > w+{0} w-{T}");
        assert!(matches!(
            refused("generate e+ e- > z{A} a")[..],
            [Unsupported::PropagatorPolarization { .. }]
        ));
        let decayed = refused("generate p p > w+{0} w-, w+ > e+ ve");
        assert!(decayed
            .iter()
            .any(|u| matches!(u, Unsupported::DecayedPolarization { leg, .. } if leg == "w+{0}")));
        assert!(matches!(
            refused("generate t{L} > w+ b")[..],
            [Unsupported::DecayedPolarization { .. }]
        ));
        assert!(check("generate t > w+{0} b").is_ok());
        let resonance = refused("generate p p > w+ w-, w+{0} > e+ ve");
        assert!(resonance
            .iter()
            .any(|u| matches!(u, Unsupported::DecayedPolarization { leg, .. } if leg == "w+{0}")));
    }

    #[test]
    fn weighted_and_alias_orders() {
        assert!(check("generate e+ e- > mu+ mu- WEIGHTED<=4").is_ok());
        assert!(check("generate e+ e- > mu+ mu- WEIGHTED=4").is_ok());
        for op in ["==", ">"] {
            assert!(matches!(
                refused(&format!("generate e+ e- > mu+ mu- WEIGHTED{op}4"))[..],
                [Unsupported::WeightedOrder { .. }]
            ));
        }
        assert!(matches!(
            refused("generate e+ e- > mu+ mu- aEW=1")[..],
            [Unsupported::SquaredOrder { .. }]
        ));
    }
}
