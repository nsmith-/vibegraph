//! Where every recognized run-card parameter goes.
//!
//! [`super::PARAM_DEFAULTS`] says which names a card may carry; this table says
//! what happens to each of them. A name is either read by this crate, or it is
//! not — and where it is not, the entry carries a positive argument for why the
//! run is unaffected, never a bare "nothing reads it". The distinction is the
//! whole point: a parameter MadGraph acts on and this crate silently drops is a
//! wrong answer, so those are refused at parse time rather than ignored.
//!
//! Three sources of evidence stand behind the table, and the third is why it is
//! written out rather than derived:
//!
//! 1. the parameter name as a literal in this workspace's sources;
//! 2. the fourteen [`super::RunCard`] struct fields, which are read as fields
//!    and never by name;
//! 3. names *built* at run time — [`crate::cuts`] forms `pt{c}`, `dr{tag}` and
//!    `mm{tag}` over its letter classes and class-pair tags, so most of the cut
//!    block has a real consumer and no literal occurrence anywhere.

use std::collections::BTreeMap;

use super::{param_default, ParamValue, RunCardError};
use FieldClass::{Consumed, IgnoredBenign, IgnoredPhysics};

/// Where a recognized run-card parameter goes.
pub enum FieldClass {
    /// Read by this crate. The string names the consumer.
    Consumed(&'static str),
    /// Not read, and unable to reach the cross section, the event record or the
    /// cuts. The string argues that, rather than reporting an absent consumer.
    IgnoredBenign(&'static str),
    /// Not read, and able to change what this generator produces. Refused when a
    /// card moves it off the MadGraph default.
    IgnoredPhysics {
        why: &'static str,
        when: Applicability,
    },
}

/// When an [`FieldClass::IgnoredPhysics`] field is capable of biting at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Applicability {
    Always,
    /// Only when both beams carry a parton density (`lpp1 == lpp2 == 1`). Real
    /// fixed-energy cards do move such a field off its default, so a flat
    /// "must equal the default" rule would reject runs MadGraph accepted.
    ProtonBeams,
}

impl Applicability {
    /// Whether the field can bite on a run with this beam configuration.
    pub fn applies(self, lpp1: i64, lpp2: i64) -> bool {
        match self {
            Applicability::Always => true,
            Applicability::ProtonBeams => lpp1 != 0 && lpp2 != 0,
        }
    }
}

const R_NEVENTS: &str = "RunCard::nevents, the event budget the CLI generates against";
const R_LPP: &str = "RunCard::beam_mode, the beam-configuration check, and the per-beam \
                     parton-density flags the scale prescription branches on";
const R_EBEAM: &str = "RunCard::ebeam1/ebeam2, the beam energies of both the proton and the \
                       fixed-energy integrands";
const R_PDLABEL: &str = "coupling::alphas::pdf_label_alpha_s, reached through \
                         RunningAlphaS::from_run_card";
const R_LHAID: &str = "RunningAlphaS::from_run_card; the PDF set id";
const R_SCALES: &str = "coupling::scales::ScaleChoice::from_run_card ('d' arrives as the \
                        clustering's d parameter)";
const R_MAXJETFLAVOR: &str = "cuts::Cuts::compile, through the leg classification, and the \
                              colour table the clustering asks which flavours count as jets";
const R_CUT_LITERAL: &str = "cuts::Cuts::compile, by literal name";
const R_CUT_SINGLE: &str = "cuts::Cuts::compile's single-leg block; the name is built as \
                            pt{c} / e{c}max / eta{c}min and so on over the jet, b, photon and \
                            lepton letters";
const R_CUT_DR: &str = "cuts::pair_dr; the name is built as dr{tag} over the class-pair tags";
const R_CUT_MM: &str = "cuts::pair_mass; the name is built as mm{tag} over the same-class \
                        pair tags";
const R_UNIMPL: &str = "cuts::detect_unimplemented — parsed and detected rather than applied, \
                        so a value off the default is already a hard error";
const R_PTGMIN: &str = "cuts::detect_unimplemented, and cuts::Cuts::compile, which raises the \
                        photon pT threshold to it";
const R_SDE_STRATEGY: &str = "hadronic::EventScaleSource::draws_configuration, which reads it \
                              together with tmin_for_channel: the per-point integration \
                              configuration the cluster scale is taken in is drawn from the \
                              squared amplitude only at 1, the value at which matrix1.f's \
                              enhancement weight AMP2_c * CC_c collapses to AMP2_c. At 2 the \
                              squared amplitude is discarded there and the weight is a product \
                              of propagator denominators this crate does not form, so the \
                              scale keeps the channel the point was sampled in — which is a \
                              partition choice of ours rather than MadEvent's, and is why no \
                              banked row that clusters carries that value";
const R_XQCUT: &str = "cuts::detect_unimplemented, and ScaleChoice::from_run_card, which \
                       refuses a card that switches matching on";

const B_JOB: &str = "MadEvent job and code-generation bookkeeping: it names output files, \
                     seeds MadGraph-side random number generators, or passes compiler flags, \
                     so it reaches no momentum, no weight and no written record. 'iseed' is \
                     MadEvent's own generator seed; this crate's seed comes from the CLI";
const B_INTEGRATOR: &str = "a directive for MadEvent's own integrator or helicity code \
                            generation. This crate integrates with its own sampler and sums \
                            helicities explicitly, so a reference cross section is invariant \
                            under it up to Monte-Carlo error";
const B_SYST: &str = "post-hoc systematics reweighting, which writes only the <mgrwt>/<rwgt> \
                      block of the event file and never enters a cross section";
const B_EVA: &str = "an EVA lepton-PDF parameter, reachable only at |lpp| of 3 or 4; the \
                     parser admits only the (0,0) and (1,1) beam configurations";
const B_EXTRA_SCALE: &str = "the Ellis-Sexton 'extra scale' family: it appears in the LO \
                             template only as a common-block declaration \
                             (Template/LO/Source/run.inc) and no LO Fortran reads it, because \
                             it is an NLO quantity";
const B_MLM: &str = "MLM matching. A card with ickkw nonzero or xqcut positive is refused when \
                     the scale prescription compiles, and ktdurham, ptlund and dparameter are \
                     hard errors as unimplemented cuts, so no matching path is reachable";
const B_ISOLATION: &str = "Frixione photon isolation, read by cuts.f only inside its ptgmin \
                           block; an active ptgmin is already a hard error";
const B_MXX: &str = "qualifies the mxx_min_pdg cut alone, and an active mxx_min_pdg is already \
                     a hard error. Its stored default is an empty payload where MadGraph's is \
                     {'default': False}, which is a second reason not to compare it";
const B_BIAS_PARAMETERS: &str = "the bias module's payload; a bias module is itself refused, \
                                 so nothing ever reads it";
const B_CUT_DECAYS: &str = "selects whether legs produced by a decay chain receive cuts, and \
                            decay-chain process syntax is refused, so no such leg exists";
const B_FRAME: &str = "selects the frame for a matrix element that is not Lorentz invariant, \
                       or for a polarised sum. Every amplitude here is Lorentz invariant and \
                       beam polarisation is refused, so the frame cannot change a value. \
                       me_frame's stored default is also an empty payload where MadGraph's is \
                       [1, 2], so comparing it would misfire";

const P_POLBEAM: &str = "beam polarisation: polarised matrix-element sums, and the SPINUP \
                         entries that follow from them, are not implemented";
const P_PDLABEL_BEAM: &str = "the per-beam PDF set. It selects the parton densities and, \
                              through pdfwrap.f, alpha_s(M_Z), while only the single pdlabel \
                              is read here. MadGraph itself rejects a card whose two beams \
                              both carry densities and name different sets";
const P_ION_COMPOSITION: &str = "ion beam composition: setrun.f builds IDBMUP from it, so it \
                                 changes the beam particle written into the event record";
const P_ION_MASS: &str = "ion beam mass: genps.f uses it as the beam mass, so it changes the \
                          phase-space kinematics";
const P_SMALL_WIDTH: &str = "floors every width at this fraction of the mass. The clustering \
                             applies the floor at a hardcoded 1e-6, and MadGraph applies the \
                             same floor to the propagators at generation time, so honouring \
                             the card here would move the clustering without moving the \
                             matrix element";
const P_KTSCHEME: &str = "selects the clustering distance measure. At 2, cluster.f takes \
                          Pythia's pydj between two final-state legs and pyjb against a beam, \
                          in place of the dj, djb and zclus this crate implements; the \
                          initial-state branch reads it as 'ickkw == 2 .or. ktscheme == 2', so \
                          it fires with matching switched off. A different measure is a \
                          different merge sequence, and so a different renormalisation and \
                          factorisation scale on every event";
const P_CHCLUSTER: &str = "restricts the clustering to the integration channel's own diagram: \
                           when cluster.f seeds the admissible graphs for a merge it keeps only \
                           those equal to iconfig, which changes which pairs may combine and \
                           with them the scales read off the resulting tree. The test sits \
                           outside any matching switch, so it applies to an ordinary run";
const P_TMIN_FOR_CHANNEL: &str = "limits the non-singular reach of a t-channel integration \
                                  channel; nothing shows a cross section invariant under \
                                  truncating one channel's reach";
const P_NHEL: &str = "Monte-Carlo over helicities in place of the explicit sum, which changes \
                      both the estimator and the per-event weight";
const P_LIMHEL: &str = "the threshold below which MadGraph drops a helicity configuration; \
                        raising it drops contributions this crate keeps";
const P_EVENT_NORM: &str = "the normalisation of XWGTUP (average, sum or unity) — a factor of \
                            the event count in the written record";
const P_TIME_OF_FLIGHT: &str = "writes a nonzero VTIMUP for long-lived particles, where this \
                                crate always writes zero";
const P_BOOST_EVENT: &str = "boosts the whole event before it is written";
const P_LHE_VERSION: &str = "the Les Houches format version; the writer emits 3.0 \
                             unconditionally";
const P_BIAS_MODULE: &str = "a bias module multiplies every event weight";
const P_CUSTOM_FCTS: &str = "user hook files that overwrite dummy functions, the cuts among \
                             them";
const P_FIXED_COUPLINGS: &str = "MadGraph itself aborts on False ('form factor with \
                                 fixed_couplings not supported anymore'), so refusing it is \
                                 the honest behaviour";

/// One row per name in [`super::PARAM_DEFAULTS`], in that table's order.
#[rustfmt::skip]
pub static FIELD_CLASSES: &[(&str, FieldClass)] = &[
    ("run_tag",                 IgnoredBenign(B_JOB)),
    ("gridpack",                IgnoredBenign(B_JOB)),
    ("time_of_flight",          IgnoredPhysics { why: P_TIME_OF_FLIGHT, when: Applicability::Always }),
    ("nevents",                 Consumed(R_NEVENTS)),
    ("iseed",                   IgnoredBenign(B_JOB)),
    ("bypass_check",            IgnoredBenign(B_JOB)),
    ("python_seed",             IgnoredBenign(B_JOB)),
    ("lpp1",                    Consumed(R_LPP)),
    ("lpp2",                    Consumed(R_LPP)),
    ("ebeam1",                  Consumed(R_EBEAM)),
    ("ebeam2",                  Consumed(R_EBEAM)),
    ("polbeam1",                IgnoredPhysics { why: P_POLBEAM, when: Applicability::Always }),
    ("polbeam2",                IgnoredPhysics { why: P_POLBEAM, when: Applicability::Always }),
    ("nb_proton1",              IgnoredPhysics { why: P_ION_COMPOSITION, when: Applicability::Always }),
    ("nb_proton2",              IgnoredPhysics { why: P_ION_COMPOSITION, when: Applicability::Always }),
    ("nb_neutron1",             IgnoredPhysics { why: P_ION_COMPOSITION, when: Applicability::Always }),
    ("nb_neutron2",             IgnoredPhysics { why: P_ION_COMPOSITION, when: Applicability::Always }),
    ("mass_ion1",               IgnoredPhysics { why: P_ION_MASS, when: Applicability::Always }),
    ("mass_ion2",               IgnoredPhysics { why: P_ION_MASS, when: Applicability::Always }),
    ("pdlabel",                 Consumed(R_PDLABEL)),
    ("pdlabel1",                IgnoredPhysics { why: P_PDLABEL_BEAM, when: Applicability::ProtonBeams }),
    ("pdlabel2",                IgnoredPhysics { why: P_PDLABEL_BEAM, when: Applicability::ProtonBeams }),
    ("lhaid",                   Consumed(R_LHAID)),
    ("fixed_ren_scale",         Consumed(R_SCALES)),
    ("fixed_fac_scale",         Consumed(R_SCALES)),
    ("fixed_fac_scale1",        Consumed(R_SCALES)),
    ("fixed_fac_scale2",        Consumed(R_SCALES)),
    ("fixed_extra_scale",       IgnoredBenign(B_EXTRA_SCALE)),
    ("scale",                   Consumed(R_SCALES)),
    ("dsqrt_q2fact1",           Consumed(R_SCALES)),
    ("dsqrt_q2fact2",           Consumed(R_SCALES)),
    ("mue_ref_fixed",           IgnoredBenign(B_EXTRA_SCALE)),
    ("dynamical_scale_choice",  Consumed(R_SCALES)),
    ("mue_over_ref",            IgnoredBenign(B_EXTRA_SCALE)),
    ("ievo_eva",                IgnoredBenign(B_EVA)),
    ("evaorder",                IgnoredBenign(B_EVA)),
    ("eva_xcut",                IgnoredBenign(B_EVA)),
    ("bias_module",             IgnoredPhysics { why: P_BIAS_MODULE, when: Applicability::Always }),
    ("bias_parameters",         IgnoredBenign(B_BIAS_PARAMETERS)),
    ("scalefact",               Consumed(R_SCALES)),
    ("ickkw",                   Consumed(R_SCALES)),
    ("highestmult",             IgnoredBenign(B_MLM)),
    ("ktscheme",                IgnoredPhysics { why: P_KTSCHEME, when: Applicability::Always }),
    ("alpsfact",                IgnoredBenign(B_MLM)),
    ("chcluster",               IgnoredPhysics { why: P_CHCLUSTER, when: Applicability::Always }),
    ("pdfwgt",                  Consumed(R_SCALES)),
    ("asrwgtflavor",            IgnoredBenign(B_MLM)),
    ("clusinfo",                IgnoredBenign(B_MLM)),
    ("custom_fcts",             IgnoredPhysics { why: P_CUSTOM_FCTS, when: Applicability::Always }),
    ("lhe_version",             IgnoredPhysics { why: P_LHE_VERSION, when: Applicability::Always }),
    ("boost_event",             IgnoredPhysics { why: P_BOOST_EVENT, when: Applicability::Always }),
    ("me_frame",                IgnoredBenign(B_FRAME)),
    ("frame_id",                IgnoredBenign(B_FRAME)),
    ("event_norm",              IgnoredPhysics { why: P_EVENT_NORM, when: Applicability::Always }),
    ("keep_log",                IgnoredBenign(B_JOB)),
    ("auto_ptj_mjj",            IgnoredBenign(B_MLM)),
    ("bwcutoff",                Consumed(R_SCALES)),
    ("cut_decays",              IgnoredBenign(B_CUT_DECAYS)),
    ("dsqrt_shat",              Consumed(R_CUT_LITERAL)),
    ("dsqrt_shatmax",           Consumed(R_CUT_LITERAL)),
    ("nhel",                    IgnoredPhysics { why: P_NHEL, when: Applicability::Always }),
    ("limhel",                  IgnoredPhysics { why: P_LIMHEL, when: Applicability::Always }),
    ("ptj",                     Consumed(R_CUT_SINGLE)),
    ("ptb",                     Consumed(R_CUT_SINGLE)),
    ("pta",                     Consumed(R_CUT_LITERAL)),
    ("ptl",                     Consumed(R_CUT_SINGLE)),
    ("misset",                  Consumed(R_UNIMPL)),
    ("ptheavy",                 Consumed(R_UNIMPL)),
    ("ptonium",                 Consumed(R_UNIMPL)),
    ("ptjmax",                  Consumed(R_CUT_SINGLE)),
    ("ptbmax",                  Consumed(R_CUT_SINGLE)),
    ("ptamax",                  Consumed(R_CUT_SINGLE)),
    ("ptlmax",                  Consumed(R_CUT_SINGLE)),
    ("missetmax",               Consumed(R_UNIMPL)),
    ("ej",                      Consumed(R_CUT_SINGLE)),
    ("eb",                      Consumed(R_CUT_SINGLE)),
    ("ea",                      Consumed(R_CUT_SINGLE)),
    ("el",                      Consumed(R_CUT_SINGLE)),
    ("ejmax",                   Consumed(R_CUT_SINGLE)),
    ("ebmax",                   Consumed(R_CUT_SINGLE)),
    ("eamax",                   Consumed(R_CUT_SINGLE)),
    ("elmax",                   Consumed(R_CUT_SINGLE)),
    ("etaj",                    Consumed(R_CUT_SINGLE)),
    ("etab",                    Consumed(R_CUT_SINGLE)),
    ("etaa",                    Consumed(R_CUT_SINGLE)),
    ("etal",                    Consumed(R_CUT_SINGLE)),
    ("etaonium",                Consumed(R_UNIMPL)),
    ("etajmin",                 Consumed(R_CUT_SINGLE)),
    ("etabmin",                 Consumed(R_CUT_SINGLE)),
    ("etaamin",                 Consumed(R_CUT_SINGLE)),
    ("etalmin",                 Consumed(R_CUT_SINGLE)),
    ("drjj",                    Consumed(R_CUT_DR)),
    ("drbb",                    Consumed(R_CUT_DR)),
    ("drll",                    Consumed(R_CUT_DR)),
    ("draa",                    Consumed(R_CUT_DR)),
    ("drbj",                    Consumed(R_CUT_DR)),
    ("draj",                    Consumed(R_CUT_DR)),
    ("drjl",                    Consumed(R_CUT_DR)),
    ("drab",                    Consumed(R_CUT_DR)),
    ("drbl",                    Consumed(R_CUT_DR)),
    ("dral",                    Consumed(R_CUT_DR)),
    ("drjjmax",                 Consumed(R_CUT_DR)),
    ("drbbmax",                 Consumed(R_CUT_DR)),
    ("drllmax",                 Consumed(R_CUT_DR)),
    ("draamax",                 Consumed(R_CUT_DR)),
    ("drbjmax",                 Consumed(R_CUT_DR)),
    ("drajmax",                 Consumed(R_CUT_DR)),
    ("drjlmax",                 Consumed(R_CUT_DR)),
    ("drabmax",                 Consumed(R_CUT_DR)),
    ("drblmax",                 Consumed(R_CUT_DR)),
    ("dralmax",                 Consumed(R_CUT_DR)),
    ("mmjj",                    Consumed(R_CUT_MM)),
    ("mmbb",                    Consumed(R_CUT_MM)),
    ("mmaa",                    Consumed(R_CUT_MM)),
    ("mmll",                    Consumed(R_CUT_LITERAL)),
    ("mmjjmax",                 Consumed(R_CUT_MM)),
    ("mmbbmax",                 Consumed(R_CUT_MM)),
    ("mmaamax",                 Consumed(R_CUT_MM)),
    ("mmllmax",                 Consumed(R_CUT_MM)),
    ("mmnl",                    Consumed(R_CUT_LITERAL)),
    ("mmnlmax",                 Consumed(R_CUT_LITERAL)),
    ("ptllmin",                 Consumed(R_CUT_LITERAL)),
    ("ptllmax",                 Consumed(R_CUT_LITERAL)),
    ("xptj",                    Consumed(R_UNIMPL)),
    ("xptb",                    Consumed(R_UNIMPL)),
    ("xpta",                    Consumed(R_UNIMPL)),
    ("xptl",                    Consumed(R_UNIMPL)),
    ("ptj1min",                 Consumed(R_UNIMPL)),
    ("ptj1max",                 Consumed(R_UNIMPL)),
    ("ptj2min",                 Consumed(R_UNIMPL)),
    ("ptj2max",                 Consumed(R_UNIMPL)),
    ("ptj3min",                 Consumed(R_UNIMPL)),
    ("ptj3max",                 Consumed(R_UNIMPL)),
    ("ptj4min",                 Consumed(R_UNIMPL)),
    ("ptj4max",                 Consumed(R_UNIMPL)),
    ("cutuse",                  Consumed(R_UNIMPL)),
    ("ptl1min",                 Consumed(R_UNIMPL)),
    ("ptl1max",                 Consumed(R_UNIMPL)),
    ("ptl2min",                 Consumed(R_UNIMPL)),
    ("ptl2max",                 Consumed(R_UNIMPL)),
    ("ptl3min",                 Consumed(R_UNIMPL)),
    ("ptl3max",                 Consumed(R_UNIMPL)),
    ("ptl4min",                 Consumed(R_UNIMPL)),
    ("ptl4max",                 Consumed(R_UNIMPL)),
    ("htjmin",                  Consumed(R_UNIMPL)),
    ("htjmax",                  Consumed(R_UNIMPL)),
    ("ihtmin",                  Consumed(R_UNIMPL)),
    ("ihtmax",                  Consumed(R_UNIMPL)),
    ("ht2min",                  Consumed(R_UNIMPL)),
    ("ht3min",                  Consumed(R_UNIMPL)),
    ("ht4min",                  Consumed(R_UNIMPL)),
    ("ht2max",                  Consumed(R_UNIMPL)),
    ("ht3max",                  Consumed(R_UNIMPL)),
    ("ht4max",                  Consumed(R_UNIMPL)),
    ("ptgmin",                  Consumed(R_PTGMIN)),
    ("r0gamma",                 IgnoredBenign(B_ISOLATION)),
    ("xn",                      IgnoredBenign(B_ISOLATION)),
    ("epsgamma",                IgnoredBenign(B_ISOLATION)),
    ("isoem",                   IgnoredBenign(B_ISOLATION)),
    ("xetamin",                 Consumed(R_UNIMPL)),
    ("deltaeta",                Consumed(R_UNIMPL)),
    ("ktdurham",                Consumed(R_UNIMPL)),
    ("dparameter",              Consumed(R_UNIMPL)),
    ("ptlund",                  Consumed(R_UNIMPL)),
    ("pdgs_for_merging_cut",    IgnoredBenign(B_MLM)),
    ("maxjetflavor",            Consumed(R_MAXJETFLAVOR)),
    ("xqcut",                   Consumed(R_XQCUT)),
    ("use_syst",                IgnoredBenign(B_SYST)),
    ("systematics_program",     IgnoredBenign(B_SYST)),
    ("systematics_arguments",   IgnoredBenign(B_SYST)),
    ("sys_scalefact",           IgnoredBenign(B_SYST)),
    ("sys_alpsfact",            IgnoredBenign(B_SYST)),
    ("sys_matchscale",          IgnoredBenign(B_SYST)),
    ("sys_pdf",                 IgnoredBenign(B_SYST)),
    ("sys_scalecorrelation",    IgnoredBenign(B_SYST)),
    ("gridrun",                 IgnoredBenign(B_INTEGRATOR)),
    ("fixed_couplings",         IgnoredPhysics { why: P_FIXED_COUPLINGS, when: Applicability::Always }),
    ("mc_grouped_subproc",      IgnoredBenign(B_INTEGRATOR)),
    ("xmtcentral",              Consumed(R_SCALES)),
    ("d",                       Consumed(R_SCALES)),
    ("gseed",                   IgnoredBenign(B_JOB)),
    ("issgridfile",             IgnoredBenign(B_JOB)),
    ("job_strategy",            IgnoredBenign(B_INTEGRATOR)),
    ("hard_survey",             IgnoredBenign(B_INTEGRATOR)),
    ("tmin_for_channel",        IgnoredPhysics { why: P_TMIN_FOR_CHANNEL, when: Applicability::Always }),
    ("second_refine_treshold",  IgnoredBenign(B_INTEGRATOR)),
    ("survey_splitting",        IgnoredBenign(B_INTEGRATOR)),
    ("survey_nchannel_per_job", IgnoredBenign(B_INTEGRATOR)),
    ("refine_evt_by_job",       IgnoredBenign(B_INTEGRATOR)),
    ("small_width_treatment",   IgnoredPhysics { why: P_SMALL_WIDTH, when: Applicability::Always }),
    ("hel_recycling",           IgnoredBenign(B_INTEGRATOR)),
    ("hel_filtering",           IgnoredBenign(B_INTEGRATOR)),
    ("hel_splitamp",            IgnoredBenign(B_INTEGRATOR)),
    ("hel_zeroamp",             IgnoredBenign(B_INTEGRATOR)),
    ("SDE_strategy",            Consumed(R_SDE_STRATEGY)),
    ("global_flag",             IgnoredBenign(B_JOB)),
    ("aloha_flag",              IgnoredBenign(B_JOB)),
    ("matrix_flag",             IgnoredBenign(B_JOB)),
    ("vector_size",             IgnoredBenign(B_INTEGRATOR)),
    ("nb_warp",                 IgnoredBenign(B_INTEGRATOR)),
    ("vecsize_memmax",          IgnoredBenign(B_INTEGRATOR)),
    ("pt_min_pdg",              Consumed(R_UNIMPL)),
    ("pt_max_pdg",              Consumed(R_UNIMPL)),
    ("E_min_pdg",               Consumed(R_UNIMPL)),
    ("E_max_pdg",               Consumed(R_UNIMPL)),
    ("eta_min_pdg",             Consumed(R_UNIMPL)),
    ("eta_max_pdg",             Consumed(R_UNIMPL)),
    ("mxx_min_pdg",             Consumed(R_UNIMPL)),
    ("mxx_only_part_antipart",  IgnoredBenign(B_MXX)),
];

/// Refuse a card that moves an [`FieldClass::IgnoredPhysics`] field off the
/// MadGraph default it was resolved against.
///
/// The beam configuration is passed in because [`Applicability::ProtonBeams`]
/// fields are only capable of biting on a run whose beams carry parton
/// densities, and fixed-energy cards do set them.
///
/// Only a refusal is possible here. Nothing is derived and nothing is rewritten:
/// MadGraph resolves `pdlabel` from `pdlabel1`/`pdlabel2`, and mirroring that
/// would quietly change a parsed value on every fixed-energy card.
pub(super) fn refuse_ignored_physics(
    values: &BTreeMap<String, ParamValue>,
    lpp1: i64,
    lpp2: i64,
) -> Result<(), RunCardError> {
    for (name, class) in FIELD_CLASSES {
        let IgnoredPhysics { why, when } = class else {
            continue;
        };
        if !when.applies(lpp1, lpp2) {
            continue;
        }
        let (Some(current), Some(default)) = (values.get(*name), param_default(name)) else {
            continue;
        };
        if *current != default {
            return Err(RunCardError::UnsupportedField {
                name: (*name).to_string(),
                value: describe(current),
                default: describe(&default),
                why,
            });
        }
    }
    Ok(())
}

/// A parameter value as the refusal message prints it. The cut detector renders
/// its own refusals the same way, so the two hard errors a card can hit read
/// alike.
fn describe(v: &ParamValue) -> String {
    match v {
        ParamValue::Float(x) => x.to_string(),
        ParamValue::Int(i) => i.to_string(),
        ParamValue::Bool(b) => b.to_string(),
        ParamValue::Str(s) | ParamValue::Opaque(s) => format!("'{s}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runcard::{RunCard, PARAM_DEFAULTS};

    /// Every recognized name is classified exactly once, and every
    /// classification names a recognized parameter.
    ///
    /// What this cannot see is a *wrong* classification: a field that does reach
    /// the cross section but is parked in [`FieldClass::IgnoredBenign`] passes
    /// here and passes everything else in this file. The reason strings are the
    /// oracle for that, and they are written so a reader can check them one at a
    /// time against an existing guard or a line of MadGraph source.
    #[test]
    fn every_run_card_field_is_classified() {
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for (name, class) in FIELD_CLASSES {
            *seen.entry(name).or_default() += 1;
            let why = match class {
                Consumed(s) | IgnoredBenign(s) => s,
                IgnoredPhysics { why, .. } => why,
            };
            assert!(!why.is_empty(), "'{name}' carries an empty reason");
            assert!(
                param_default(name).is_some(),
                "'{name}' is classified but is not a recognized parameter"
            );
        }
        for (name, count) in &seen {
            assert_eq!(*count, 1, "'{name}' is classified {count} times");
        }
        for (name, _) in PARAM_DEFAULTS {
            assert!(
                seen.contains_key(name),
                "'{name}' has a default but no classification"
            );
        }
        assert_eq!(seen.len(), PARAM_DEFAULTS.len());
    }

    /// A one-line card moving each [`FieldClass::IgnoredPhysics`] field off its
    /// default is refused, and the refusal names that field.
    ///
    /// One field is perturbed at a time, so an interaction that is unsafe only
    /// jointly is outside what this can see.
    #[test]
    fn ignored_physics_fields_are_refused() {
        let mut checked = 0;
        let mut proton_only = 0;
        for (name, class) in FIELD_CLASSES {
            let IgnoredPhysics { when, .. } = class else {
                continue;
            };
            let perturbed = perturb(name);
            let proton = format!("  1 = lpp1\n  1 = lpp2\n  {perturbed} = {name}\n");
            match RunCard::parse(&proton) {
                Err(e @ RunCardError::UnsupportedField { .. }) => {
                    let RunCardError::UnsupportedField { name: got, .. } = &e else {
                        unreachable!()
                    };
                    assert_eq!(got, name, "the refusal named the wrong field");
                    println!("  {e}");
                }
                other => panic!("'{name} = {perturbed}' on proton beams was not refused: {other:?}"),
            }

            // A `ProtonBeams` field is inert on fixed-energy beams, and real
            // cards set one there, so it must still parse.
            if *when == Applicability::ProtonBeams {
                proton_only += 1;
                let fixed = format!("  0 = lpp1\n  0 = lpp2\n  {perturbed} = {name}\n");
                RunCard::parse(&fixed).unwrap_or_else(|e| {
                    panic!("'{name} = {perturbed}' on fixed-energy beams was refused: {e}")
                });
            }
            checked += 1;
        }
        assert_eq!(checked, 23, "the refused inventory changed size");
        assert_eq!(proton_only, 2, "the beam-dependent inventory changed size");
    }

    /// A card line that moves `name` off its default, in the syntax the parser
    /// reads. Numeric defaults are shifted by one, which stays clear of every
    /// sentinel in the table; booleans are negated; strings and opaque payloads
    /// get a marker.
    fn perturb(name: &str) -> String {
        match param_default(name).expect("recognized parameter") {
            ParamValue::Float(x) => format!("{}", x + 1.0),
            ParamValue::Int(i) => format!("{}", i + 1),
            ParamValue::Bool(b) => if b { "False" } else { "True" }.to_string(),
            ParamValue::Str(_) | ParamValue::Opaque(_) => "unsupported_marker".to_string(),
        }
    }

    /// The `Opaque` payload defaults this crate stores that are *not* MadGraph's.
    ///
    /// `defaults_match_banner_py_dump` compares every scalar default against the
    /// `banner.py` dump but skips opaque payloads, so these four have never been
    /// checked. They are pinned rather than fixed: each is classified
    /// [`FieldClass::IgnoredBenign`] for a reason independent of its default, so
    /// the mismatch changes nothing today — but a card writing MadGraph's own
    /// default for one of them reads here as an override, which is exactly the
    /// trap an enforcement over these names would spring. If a MadGraph bump
    /// moves this set, that has to be seen.
    #[test]
    fn opaque_defaults_known_to_differ_from_banner_py() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/runcard_defaults.json"
        );
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "missing defaults oracle {path}: {e}\n\
                 run `pixi run -e madgraph dump-runcard-defaults` to (re)generate it"
            )
        });
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap();
        let obj = dump.as_object().expect("oracle is a JSON object");

        let mut mismatched: Vec<&str> = Vec::new();
        for (name, _) in PARAM_DEFAULTS {
            let ParamValue::Opaque(stored) = param_default(name).expect("recognized") else {
                continue;
            };
            let actual = obj
                .get(*name)
                .unwrap_or_else(|| panic!("'{name}' absent from the banner.py dump"));
            // The dump is Python's repr; `{}` and `[]` are what the parser
            // normalizes to the stored empty payload.
            let empty = matches!(actual.as_object(), Some(m) if m.is_empty())
                || matches!(actual.as_array(), Some(a) if a.is_empty());
            if stored.is_empty() && !empty {
                mismatched.push(name);
            }
        }
        mismatched.sort_unstable();
        assert_eq!(
            mismatched,
            [
                "me_frame",
                "mxx_only_part_antipart",
                "pdgs_for_merging_cut",
                "systematics_arguments",
            ],
            "the set of opaque defaults that disagree with banner.py moved"
        );
    }

    /// Every field the refusal covers is one no banked card moves — the property
    /// that makes the enforcement invisible to every reference cross section.
    /// The fixed-energy cards that do set `pdlabel1`/`pdlabel2` are why
    /// [`Applicability::ProtonBeams`] exists.
    #[test]
    fn the_default_card_is_accepted() {
        RunCard::parse("").expect("an empty card is MadGraph's own configuration");
        RunCard::parse("  0 = lpp1\n  0 = lpp2\n  none = pdlabel1\n  none = pdlabel2\n")
            .expect("a fixed-energy card may name no per-beam PDF set");
    }
}
