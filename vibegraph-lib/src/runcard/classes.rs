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
use Stage::{Generate, Integrate};

/// Where a recognized run-card parameter goes.
pub(crate) enum FieldClass {
    /// Read by this crate. The string names the consumer.
    Consumed(#[cfg_attr(not(test), allow(dead_code))] &'static str),
    /// Not read, and unable to reach the cross section, the event record or the
    /// cuts. The string argues that, rather than reporting an absent consumer.
    IgnoredBenign(#[cfg_attr(not(test), allow(dead_code))] &'static str),
    /// Not implemented, and able to change what this generator produces. Refused
    /// when a card moves it off the MadGraph default.
    ///
    /// "Not implemented" rather than "not read": a field may be read precisely in
    /// order to decline the branch it selects, which is what the refusal then
    /// covers. `tmin_for_channel` is that case.
    IgnoredPhysics { why: &'static str },
}

const R_NEVENTS: &str = "GenerateSettings::nevents, the event budget a generation draws against";
const R_ISEED: &str = "RunCard::integration_seed, the integration's seed when no seed is given \
                       on the command line and the card's is not 0";
const R_LPP: &str = "RunCard::beam_mode, the beam-configuration check, and the per-beam \
                     parton-density flags the scale prescription branches on";
const R_EBEAM: &str = "RunCard::ebeam1/ebeam2, the beam energies of both the proton and the \
                       fixed-energy integrands";
const R_PDLABEL: &str = "RunCard::pdf_choice, which on proton beams admits only 'lhapdf' and \
                         refuses every built-in parameterisation";
const R_LHAID: &str = "pdf::select::load_for_run_card, which reads the LHAPDF set and member \
                       this id names";
const R_SCALES: &str = "coupling::scales::ScaleChoice::from_run_card ('d' arrives as the \
                        clustering's d parameter)";
const R_MAXJETFLAVOR: &str = "cuts::Cuts::compile, through the leg classification, and the \
                              colour table the clustering asks which flavours count as jets";
const R_CUT_DECAYS: &str = "cuts::Cuts::compile_with, which at F switches the cuts off on every \
                            leg a decay chain's forced lines produce (setcuts.f:192)";
const R_BWCUTOFF: &str = "cuts::Cuts::compile_with, the half-width in widths of each decay-chain \
                          window (myamp.f:179) and of the channel maps' forced draws; and \
                          coupling::scales::ScaleChoice::from_run_card, whose clustering \
                          reads it for the on-shell test";
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
const R_SDE_STRATEGY: &str = "hadronic::configurations_weighted_by_amp2, which \
                              compile_configuration_weights and the scale source both call, \
                              and which reads it together with tmin_for_channel: matrix1.f weights \
                              integration configuration c by AMP2_c * CC_c, and genps.f's \
                              get_channel_cut collapses CC_c to 1 exactly at 1 with \
                              tmin_for_channel = -1. At 2 the squared amplitude is discarded \
                              and the weight is that product of inverse propagator \
                              denominators alone (coupling.cluster.graph.ChannelSet.\
                              channel_cuts). The field therefore reaches every per-event \
                              quantity that follows a configuration: the channel a point's \
                              cluster scale is taken in, and — through SELECT_COLOR's ICONFIG \
                              — the colour flow its event record is written with";
const R_FRAME: &str = "RunCard::frame_id, read by hadronic::refuse_polarized_frame: a polarized \
                       massive leg makes the squared amplitude frame dependent, and every \
                       evaluator here takes partonic centre-of-mass momenta, MadGraph's default \
                       frame (frame_id 6), so a card naming another frame is refused on such a \
                       process. An amplitude summed over its helicities, or polarized only on \
                       massless legs, is Lorentz invariant and the frame cannot move it";
const R_XQCUT: &str = "runcard::matching::resolve, which rewrites the jet cuts from it \
                       (setcuts.f:156-189); and ScaleChoice::from_run_card, whose clustering \
                       rejects a point with a jet vertex below it";
const R_ICKKW: &str = "runcard::matching::resolve, which admits MadGraph's [0, 1]; and \
                       ScaleChoice::from_run_card, which clusters every event and makes \
                       reweight.f's second setclscales call under 1";
const R_ALPSFACT: &str = "runcard::matching::resolve, which forces it to 1 under use_syst \
                          (setrun.f:151-159); and ScaleChoice::alpsfact, the factor on the \
                          clustering scale of the matched alpha_s reweighting";
const R_ASRWGTFLAVOR: &str = "ScaleChoice::asrwgtflavor, the heaviest quark flavour the \
                              matched alpha_s reweighting counts as a parton";
const R_USE_SYST: &str = "runcard::matching::resolve, where it forces alpsfact to 1 \
                          (setrun.f:151-159); and GenerateSettings::systematics, which writes \
                          every proton event's <mgrwt> record, matched or not, and the file's \
                          <MGRunCard> under it, and resolves it off on fixed-energy beams";
const R_SYST_REQUEST: &str = "RunCard::split, which reads MadGraph's systematics request from \
                              it into GenerateSettings::systematics: the program, then the \
                              systematics_arguments list when the card writes that line and the \
                              sys_* fields otherwise (systematics.py:1371). The variation \
                              weights are written to the <rwgt> block and never enter a cross \
                              section";
const R_AUTO_PTJ_MJJ: &str = "runcard::matching::resolve, which sets ptj and mmjj to xqcut \
                              under it (setcuts.f:156-181)";

const B_JOB: &str = "MadEvent job and code-generation bookkeeping: it names output files, \
                     seeds MadGraph-side random number generators, or passes compiler flags, \
                     so it reaches no momentum, no weight and no written record";
const B_INTEGRATOR: &str = "a directive for MadEvent's own integrator or helicity code \
                            generation. This crate integrates with its own sampler and sums \
                            helicities explicitly, so a reference cross section is invariant \
                            under it up to Monte-Carlo error";
const B_SYSCALC: &str = "read only by SysCalc, MadGraph's retired systematics program, which \
                         RunCard::split refuses; MadGraph's systematics module ignores it";
const B_EVA: &str = "an EVA lepton-PDF parameter, reachable only at |lpp| of 3 or 4; the \
                     parser admits only the (0,0) and (1,1) beam configurations";
const B_EXTRA_SCALE: &str = "the Ellis-Sexton 'extra scale' family: it appears in the LO \
                             template only as a common-block declaration \
                             (Template/LO/Source/run.inc) and no LO Fortran reads it, because \
                             it is an NLO quantity";
const B_MLM: &str = "an MLM input MadEvent reads only on branches this crate never reaches. \
                     highestmult sets hmult, which rewgt reads only in 'hmult .or. ickkw == 1' \
                     (reweight.f:1424), so it is inert at the only matched ickkw MadGraph \
                     admits; clusinfo adds the <clustering> block to a matched event \
                     (unwgt.f:838), CKKW-L bookkeeping no weight, momentum or other record \
                     field depends on; pdgs_for_merging_cut qualifies only the ktdurham and \
                     ptlund merging cuts, which are hard errors as unimplemented cuts";
const B_ISOLATION: &str = "Frixione photon isolation, read by cuts.f only inside its ptgmin \
                           block; an active ptgmin is already a hard error";
const B_MXX: &str = "qualifies the mxx_min_pdg cut alone, and an active mxx_min_pdg is already \
                     a hard error";
const B_BIAS_PARAMETERS: &str = "the bias module's payload; a bias module is itself refused, \
                                 so nothing ever reads it";
const B_FRAME_ID: &str = "a system parameter: MadGraph recomputes it from me_frame as the sum of \
                          2^n over the listed legs whatever a card writes \
                          (banner.py:4705), so me_frame is what is read instead";

const P_POLBEAM: &str = "beam polarisation: polarised matrix-element sums, and the SPINUP \
                         entries that follow from them, are not implemented";
const R_PDLABEL_BEAM: &str = "RunCard::from_values, which resolves the per-beam PDF labels as \
                              banner.py's PDLabelBlock does at proton beams: two equal labels \
                              set pdlabel, and two different ones are refused, as MadGraph \
                              refuses an asymmetric proton-proton PDF. At fixed-energy beams \
                              no density is read and the labels are inert";
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
                                  channel: off its default it multiplies get_channel_cut by an \
                                  exponential suppression of every spacelike line below \
                                  t/s_tot = tmin, and at SDE_strategy = 1 it is that \
                                  suppression alone that replaces the constant 1. Neither is \
                                  formed here -- the channel weight this crate builds is \
                                  get_channel_cut's SDE_strategy = 2 branch, the \
                                  denominator product without the tmin factor, and at \
                                  SDE_strategy = 1 genps.f reads an uninitialised t in that \
                                  factor anyway. The field is read -- \
                                  hadronic::configurations_weighted_by_amp2 refuses it beside \
                                  SDE_strategy -- but reading is not implementing it, so \
                                  it is refused here, at the parse, and again where the \
                                  weight is chosen, rather than taking a configuration weight \
                                  under a rule that does not describe it";
const P_NHEL: &str = "Monte-Carlo over helicities in place of the explicit sum, which changes \
                      both the estimator and the per-event weight";
const P_LIMHEL: &str = "the threshold below which MadGraph drops a helicity configuration; \
                        raising it drops contributions this crate keeps";
const R_EVENT_NORM: &str = "lhef::emit::EmitPlan::event_norm, the normalisation of the \
                            written XWGTUP and the IDWTUP, through GenerateSettings::event_norm; \
                            RunCard::from_values refuses a value outside MadGraph's list at the \
                            parse";
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

/// Which step of a run a recognized parameter belongs to.
///
/// An integration records the card it was trained on, and a later step reads
/// every [`Stage::Integrate`] field back from that record and never again from a
/// card. The [`Stage::Generate`] fields reach only the event budget or the
/// written record, so a generation may set them anew without re-integrating
/// ([`super::RunCard::split`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Read while the grids are trained, or riding with the card that trained
    /// them.
    Integrate,
    /// Read only when events are drawn and written.
    Generate,
}

/// One row per name in [`super::PARAM_DEFAULTS`], in that table's order: the
/// name, the step it belongs to, and where it goes.
#[rustfmt::skip]
pub(crate) static FIELD_CLASSES: &[(&str, Stage, FieldClass)] = &[
    ("run_tag",                 Integrate, IgnoredBenign(B_JOB)),
    ("gridpack",                Integrate, IgnoredBenign(B_JOB)),
    ("time_of_flight",          Generate,  IgnoredPhysics { why: P_TIME_OF_FLIGHT }),
    ("nevents",                 Generate,  Consumed(R_NEVENTS)),
    ("iseed",                   Integrate, Consumed(R_ISEED)),
    ("bypass_check",            Integrate, IgnoredBenign(B_JOB)),
    ("python_seed",             Integrate, IgnoredBenign(B_JOB)),
    ("lpp1",                    Integrate, Consumed(R_LPP)),
    ("lpp2",                    Integrate, Consumed(R_LPP)),
    ("ebeam1",                  Integrate, Consumed(R_EBEAM)),
    ("ebeam2",                  Integrate, Consumed(R_EBEAM)),
    ("polbeam1",                Integrate, IgnoredPhysics { why: P_POLBEAM }),
    ("polbeam2",                Integrate, IgnoredPhysics { why: P_POLBEAM }),
    ("nb_proton1",              Integrate, IgnoredPhysics { why: P_ION_COMPOSITION }),
    ("nb_proton2",              Integrate, IgnoredPhysics { why: P_ION_COMPOSITION }),
    ("nb_neutron1",             Integrate, IgnoredPhysics { why: P_ION_COMPOSITION }),
    ("nb_neutron2",             Integrate, IgnoredPhysics { why: P_ION_COMPOSITION }),
    ("mass_ion1",               Integrate, IgnoredPhysics { why: P_ION_MASS }),
    ("mass_ion2",               Integrate, IgnoredPhysics { why: P_ION_MASS }),
    ("pdlabel",                 Integrate, Consumed(R_PDLABEL)),
    ("pdlabel1",                Integrate, Consumed(R_PDLABEL_BEAM)),
    ("pdlabel2",                Integrate, Consumed(R_PDLABEL_BEAM)),
    ("lhaid",                   Integrate, Consumed(R_LHAID)),
    ("fixed_ren_scale",         Integrate, Consumed(R_SCALES)),
    ("fixed_fac_scale",         Integrate, Consumed(R_SCALES)),
    ("fixed_fac_scale1",        Integrate, Consumed(R_SCALES)),
    ("fixed_fac_scale2",        Integrate, Consumed(R_SCALES)),
    ("fixed_extra_scale",       Integrate, IgnoredBenign(B_EXTRA_SCALE)),
    ("scale",                   Integrate, Consumed(R_SCALES)),
    ("dsqrt_q2fact1",           Integrate, Consumed(R_SCALES)),
    ("dsqrt_q2fact2",           Integrate, Consumed(R_SCALES)),
    ("mue_ref_fixed",           Integrate, IgnoredBenign(B_EXTRA_SCALE)),
    ("dynamical_scale_choice",  Integrate, Consumed(R_SCALES)),
    ("mue_over_ref",            Integrate, IgnoredBenign(B_EXTRA_SCALE)),
    ("ievo_eva",                Integrate, IgnoredBenign(B_EVA)),
    ("evaorder",                Integrate, IgnoredBenign(B_EVA)),
    ("eva_xcut",                Integrate, IgnoredBenign(B_EVA)),
    ("bias_module",             Integrate, IgnoredPhysics { why: P_BIAS_MODULE }),
    ("bias_parameters",         Integrate, IgnoredBenign(B_BIAS_PARAMETERS)),
    ("scalefact",               Integrate, Consumed(R_SCALES)),
    ("ickkw",                   Integrate, Consumed(R_ICKKW)),
    ("highestmult",             Integrate, IgnoredBenign(B_MLM)),
    ("ktscheme",                Integrate, IgnoredPhysics { why: P_KTSCHEME }),
    ("alpsfact",                Integrate, Consumed(R_ALPSFACT)),
    ("chcluster",               Integrate, IgnoredPhysics { why: P_CHCLUSTER }),
    ("pdfwgt",                  Integrate, Consumed(R_SCALES)),
    ("asrwgtflavor",            Integrate, Consumed(R_ASRWGTFLAVOR)),
    ("clusinfo",                Generate,  IgnoredBenign(B_MLM)),
    ("custom_fcts",             Integrate, IgnoredPhysics { why: P_CUSTOM_FCTS }),
    ("lhe_version",             Generate,  IgnoredPhysics { why: P_LHE_VERSION }),
    ("boost_event",             Generate,  IgnoredPhysics { why: P_BOOST_EVENT }),
    ("me_frame",                Integrate, Consumed(R_FRAME)),
    ("frame_id",                Integrate, IgnoredBenign(B_FRAME_ID)),
    ("event_norm",              Generate,  Consumed(R_EVENT_NORM)),
    ("keep_log",                Integrate, IgnoredBenign(B_JOB)),
    ("auto_ptj_mjj",            Integrate, Consumed(R_AUTO_PTJ_MJJ)),
    ("bwcutoff",                Integrate, Consumed(R_BWCUTOFF)),
    ("cut_decays",              Integrate, Consumed(R_CUT_DECAYS)),
    ("dsqrt_shat",              Integrate, Consumed(R_CUT_LITERAL)),
    ("dsqrt_shatmax",           Integrate, Consumed(R_CUT_LITERAL)),
    ("nhel",                    Integrate, IgnoredPhysics { why: P_NHEL }),
    ("limhel",                  Integrate, IgnoredPhysics { why: P_LIMHEL }),
    ("ptj",                     Integrate, Consumed(R_CUT_SINGLE)),
    ("ptb",                     Integrate, Consumed(R_CUT_SINGLE)),
    ("pta",                     Integrate, Consumed(R_CUT_LITERAL)),
    ("ptl",                     Integrate, Consumed(R_CUT_SINGLE)),
    ("misset",                  Integrate, Consumed(R_UNIMPL)),
    ("ptheavy",                 Integrate, Consumed(R_UNIMPL)),
    ("ptonium",                 Integrate, Consumed(R_UNIMPL)),
    ("ptjmax",                  Integrate, Consumed(R_CUT_SINGLE)),
    ("ptbmax",                  Integrate, Consumed(R_CUT_SINGLE)),
    ("ptamax",                  Integrate, Consumed(R_CUT_SINGLE)),
    ("ptlmax",                  Integrate, Consumed(R_CUT_SINGLE)),
    ("missetmax",               Integrate, Consumed(R_UNIMPL)),
    ("ej",                      Integrate, Consumed(R_CUT_SINGLE)),
    ("eb",                      Integrate, Consumed(R_CUT_SINGLE)),
    ("ea",                      Integrate, Consumed(R_CUT_SINGLE)),
    ("el",                      Integrate, Consumed(R_CUT_SINGLE)),
    ("ejmax",                   Integrate, Consumed(R_CUT_SINGLE)),
    ("ebmax",                   Integrate, Consumed(R_CUT_SINGLE)),
    ("eamax",                   Integrate, Consumed(R_CUT_SINGLE)),
    ("elmax",                   Integrate, Consumed(R_CUT_SINGLE)),
    ("etaj",                    Integrate, Consumed(R_CUT_SINGLE)),
    ("etab",                    Integrate, Consumed(R_CUT_SINGLE)),
    ("etaa",                    Integrate, Consumed(R_CUT_SINGLE)),
    ("etal",                    Integrate, Consumed(R_CUT_SINGLE)),
    ("etaonium",                Integrate, Consumed(R_UNIMPL)),
    ("etajmin",                 Integrate, Consumed(R_CUT_SINGLE)),
    ("etabmin",                 Integrate, Consumed(R_CUT_SINGLE)),
    ("etaamin",                 Integrate, Consumed(R_CUT_SINGLE)),
    ("etalmin",                 Integrate, Consumed(R_CUT_SINGLE)),
    ("drjj",                    Integrate, Consumed(R_CUT_DR)),
    ("drbb",                    Integrate, Consumed(R_CUT_DR)),
    ("drll",                    Integrate, Consumed(R_CUT_DR)),
    ("draa",                    Integrate, Consumed(R_CUT_DR)),
    ("drbj",                    Integrate, Consumed(R_CUT_DR)),
    ("draj",                    Integrate, Consumed(R_CUT_DR)),
    ("drjl",                    Integrate, Consumed(R_CUT_DR)),
    ("drab",                    Integrate, Consumed(R_CUT_DR)),
    ("drbl",                    Integrate, Consumed(R_CUT_DR)),
    ("dral",                    Integrate, Consumed(R_CUT_DR)),
    ("drjjmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drbbmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drllmax",                 Integrate, Consumed(R_CUT_DR)),
    ("draamax",                 Integrate, Consumed(R_CUT_DR)),
    ("drbjmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drajmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drjlmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drabmax",                 Integrate, Consumed(R_CUT_DR)),
    ("drblmax",                 Integrate, Consumed(R_CUT_DR)),
    ("dralmax",                 Integrate, Consumed(R_CUT_DR)),
    ("mmjj",                    Integrate, Consumed(R_CUT_MM)),
    ("mmbb",                    Integrate, Consumed(R_CUT_MM)),
    ("mmaa",                    Integrate, Consumed(R_CUT_MM)),
    ("mmll",                    Integrate, Consumed(R_CUT_LITERAL)),
    ("mmjjmax",                 Integrate, Consumed(R_CUT_MM)),
    ("mmbbmax",                 Integrate, Consumed(R_CUT_MM)),
    ("mmaamax",                 Integrate, Consumed(R_CUT_MM)),
    ("mmllmax",                 Integrate, Consumed(R_CUT_MM)),
    ("mmnl",                    Integrate, Consumed(R_CUT_LITERAL)),
    ("mmnlmax",                 Integrate, Consumed(R_CUT_LITERAL)),
    ("ptllmin",                 Integrate, Consumed(R_CUT_LITERAL)),
    ("ptllmax",                 Integrate, Consumed(R_CUT_LITERAL)),
    ("xptj",                    Integrate, Consumed(R_UNIMPL)),
    ("xptb",                    Integrate, Consumed(R_UNIMPL)),
    ("xpta",                    Integrate, Consumed(R_UNIMPL)),
    ("xptl",                    Integrate, Consumed(R_UNIMPL)),
    ("ptj1min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj1max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj2min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj2max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj3min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj3max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj4min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptj4max",                 Integrate, Consumed(R_UNIMPL)),
    ("cutuse",                  Integrate, Consumed(R_UNIMPL)),
    ("ptl1min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl1max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl2min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl2max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl3min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl3max",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl4min",                 Integrate, Consumed(R_UNIMPL)),
    ("ptl4max",                 Integrate, Consumed(R_UNIMPL)),
    ("htjmin",                  Integrate, Consumed(R_CUT_LITERAL)),
    ("htjmax",                  Integrate, Consumed(R_CUT_LITERAL)),
    ("ihtmin",                  Integrate, Consumed(R_CUT_LITERAL)),
    ("ihtmax",                  Integrate, Consumed(R_CUT_LITERAL)),
    ("ht2min",                  Integrate, Consumed(R_UNIMPL)),
    ("ht3min",                  Integrate, Consumed(R_UNIMPL)),
    ("ht4min",                  Integrate, Consumed(R_UNIMPL)),
    ("ht2max",                  Integrate, Consumed(R_UNIMPL)),
    ("ht3max",                  Integrate, Consumed(R_UNIMPL)),
    ("ht4max",                  Integrate, Consumed(R_UNIMPL)),
    ("ptgmin",                  Integrate, Consumed(R_PTGMIN)),
    ("r0gamma",                 Integrate, IgnoredBenign(B_ISOLATION)),
    ("xn",                      Integrate, IgnoredBenign(B_ISOLATION)),
    ("epsgamma",                Integrate, IgnoredBenign(B_ISOLATION)),
    ("isoem",                   Integrate, IgnoredBenign(B_ISOLATION)),
    ("xetamin",                 Integrate, Consumed(R_UNIMPL)),
    ("deltaeta",                Integrate, Consumed(R_UNIMPL)),
    ("ktdurham",                Integrate, Consumed(R_UNIMPL)),
    ("dparameter",              Integrate, Consumed(R_UNIMPL)),
    ("ptlund",                  Integrate, Consumed(R_UNIMPL)),
    ("pdgs_for_merging_cut",    Integrate, IgnoredBenign(B_MLM)),
    ("maxjetflavor",            Integrate, Consumed(R_MAXJETFLAVOR)),
    ("xqcut",                   Integrate, Consumed(R_XQCUT)),
    ("use_syst",                Generate,  Consumed(R_USE_SYST)),
    ("systematics_program",     Generate,  Consumed(R_SYST_REQUEST)),
    ("systematics_arguments",   Generate,  Consumed(R_SYST_REQUEST)),
    ("sys_scalefact",           Generate,  Consumed(R_SYST_REQUEST)),
    ("sys_alpsfact",            Generate,  Consumed(R_SYST_REQUEST)),
    ("sys_matchscale",          Generate,  IgnoredBenign(B_SYSCALC)),
    ("sys_pdf",                 Generate,  Consumed(R_SYST_REQUEST)),
    ("sys_scalecorrelation",    Generate,  IgnoredBenign(B_SYSCALC)),
    ("gridrun",                 Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("fixed_couplings",         Integrate, IgnoredPhysics { why: P_FIXED_COUPLINGS }),
    ("mc_grouped_subproc",      Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("xmtcentral",              Integrate, Consumed(R_SCALES)),
    ("d",                       Integrate, Consumed(R_SCALES)),
    ("gseed",                   Integrate, IgnoredBenign(B_JOB)),
    ("issgridfile",             Integrate, IgnoredBenign(B_JOB)),
    ("job_strategy",            Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("hard_survey",             Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("tmin_for_channel",        Integrate, IgnoredPhysics { why: P_TMIN_FOR_CHANNEL }),
    ("second_refine_treshold",  Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("survey_splitting",        Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("survey_nchannel_per_job", Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("refine_evt_by_job",       Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("small_width_treatment",   Integrate, IgnoredPhysics { why: P_SMALL_WIDTH }),
    ("hel_recycling",           Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("hel_filtering",           Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("hel_splitamp",            Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("hel_zeroamp",             Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("SDE_strategy",            Integrate, Consumed(R_SDE_STRATEGY)),
    ("global_flag",             Integrate, IgnoredBenign(B_JOB)),
    ("aloha_flag",              Integrate, IgnoredBenign(B_JOB)),
    ("matrix_flag",             Integrate, IgnoredBenign(B_JOB)),
    ("vector_size",             Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("nb_warp",                 Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("vecsize_memmax",          Integrate, IgnoredBenign(B_INTEGRATOR)),
    ("pt_min_pdg",              Integrate, Consumed(R_UNIMPL)),
    ("pt_max_pdg",              Integrate, Consumed(R_UNIMPL)),
    ("E_min_pdg",               Integrate, Consumed(R_UNIMPL)),
    ("E_max_pdg",               Integrate, Consumed(R_UNIMPL)),
    ("eta_min_pdg",             Integrate, Consumed(R_UNIMPL)),
    ("eta_max_pdg",             Integrate, Consumed(R_UNIMPL)),
    ("mxx_min_pdg",             Integrate, Consumed(R_UNIMPL)),
    ("mxx_only_part_antipart",  Integrate, IgnoredBenign(B_MXX)),
];

/// The parameters a generation reads, in [`super::PARAM_DEFAULTS`]'s order.
pub(super) fn generate_fields() -> impl Iterator<Item = &'static str> {
    FIELD_CLASSES
        .iter()
        .filter(|(_, stage, _)| *stage == Generate)
        .map(|(name, _, _)| *name)
}

/// Refuse a card that moves an [`FieldClass::IgnoredPhysics`] field off the
/// MadGraph default it was resolved against.
///
/// Only a refusal is possible here. Nothing is derived and nothing is rewritten.
pub(super) fn refuse_ignored_physics(
    values: &BTreeMap<String, ParamValue>,
) -> Result<(), RunCardError> {
    for (name, _, class) in FIELD_CLASSES {
        let IgnoredPhysics { why } = class else {
            continue;
        };
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
    use crate::runcard::{card_spelling, parse_value, Kind, RunCard, PARAM_DEFAULTS};

    /// The systematics fields a card asks for weights through are consumed,
    /// and the two only SysCalc reads stay benign.
    #[test]
    fn the_systematics_request_fields_are_consumed() {
        let class = |name: &str| {
            FIELD_CLASSES
                .iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, _, c)| c)
                .expect("classified")
        };
        for name in [
            "use_syst",
            "systematics_program",
            "systematics_arguments",
            "sys_scalefact",
            "sys_alpsfact",
            "sys_pdf",
        ] {
            assert!(matches!(class(name), Consumed(_)), "{name}");
        }
        for name in ["sys_matchscale", "sys_scalecorrelation"] {
            assert!(matches!(class(name), IgnoredBenign(_)), "{name}");
        }
    }

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
        for (name, _, class) in FIELD_CLASSES {
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

    /// Every recognized name carries exactly one stage, and the generate stage is
    /// exactly the fourteen fields that reach only the event budget or the
    /// written record: a row moved between stages fails here by name.
    ///
    /// What this cannot see is a field whose stage is wrong in both the table
    /// and this list; the list is written out by hand, not derived from the
    /// table.
    #[test]
    fn every_run_card_field_has_one_stage_and_fourteen_are_generate() {
        for (name, _) in PARAM_DEFAULTS {
            let stages = FIELD_CLASSES.iter().filter(|(n, _, _)| n == name).count();
            assert_eq!(stages, 1, "'{name}' carries {stages} stages");
        }
        let generate: Vec<&str> = generate_fields().collect();
        assert_eq!(
            generate,
            [
                "time_of_flight",
                "nevents",
                "clusinfo",
                "lhe_version",
                "boost_event",
                "event_norm",
                "use_syst",
                "systematics_program",
                "systematics_arguments",
                "sys_scalefact",
                "sys_alpsfact",
                "sys_matchscale",
                "sys_pdf",
                "sys_scalecorrelation",
            ]
        );
        assert_eq!(generate.len(), 14);
        let iseed = FIELD_CLASSES.iter().find(|(n, _, _)| *n == "iseed");
        assert!(
            matches!(iseed, Some((_, Integrate, Consumed(_)))),
            "iseed is the integration's seed"
        );
    }

    /// A one-line card moving each [`FieldClass::IgnoredPhysics`] field off its
    /// default is refused, and the refusal names that field.
    ///
    /// One field is perturbed at a time, so an interaction that is unsafe only
    /// jointly is outside what this can see.
    #[test]
    fn ignored_physics_fields_are_refused() {
        let mut checked = 0;
        for (name, _, class) in FIELD_CLASSES {
            let IgnoredPhysics { .. } = class else {
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
                other => {
                    panic!("'{name} = {perturbed}' on proton beams was not refused: {other:?}")
                }
            }

            checked += 1;
        }
        assert_eq!(checked, 20, "the refused inventory changed size");
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

    /// Every `Opaque` payload default against `banner.py`'s, all of them at
    /// once: the value MadGraph writes into a card for its own default must parse
    /// to the stored default, so a card that spells out MadGraph's default for a
    /// list or dict field reads as no override. `defaults_match_banner_py_dump`
    /// checks the same per parameter and stops at the first; this names every
    /// one that disagrees.
    #[test]
    fn opaque_defaults_match_banner_py() {
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

        let mut opaque = 0;
        let mut mismatched: Vec<String> = Vec::new();
        for (name, _) in PARAM_DEFAULTS {
            let stored = param_default(name).expect("recognized");
            if !matches!(stored, ParamValue::Opaque(_)) {
                continue;
            }
            opaque += 1;
            let actual = obj
                .get(*name)
                .unwrap_or_else(|| panic!("'{name}' absent from the banner.py dump"));
            let spelled = card_spelling(actual);
            if parse_value(&spelled, Kind::Opaque).as_ref() != Some(&stored) {
                mismatched.push(format!("{name}: stored {stored:?}, written {spelled:?}"));
            }
        }
        assert_eq!(opaque, 14, "the opaque inventory changed size");
        assert!(
            mismatched.is_empty(),
            "opaque defaults that disagree with banner.py:\n{}",
            mismatched.join("\n")
        );
    }

    /// A card line spelling MadGraph's own non-empty list or dict default reads
    /// as that default, not as an override: `{'default': False}` is the
    /// `mxx_only_part_antipart` line every banked card writes.
    #[test]
    fn madgraphs_own_list_and_dict_defaults_read_as_defaults() {
        let card = RunCard::parse(concat!(
            "  {'default': False}\t= mxx_only_part_antipart ! if True the invariant mass\n",
            "  21, 1, 2, 3, 4, 5, 6 = pdgs_for_merging_cut\n",
        ))
        .expect("MadGraph's own defaults parse");
        for name in ["mxx_only_part_antipart", "pdgs_for_merging_cut"] {
            assert_eq!(card.get(name), param_default(name).as_ref(), "{name}");
        }
    }

    /// Every field the refusal covers is one no banked card moves — the property
    /// that makes the enforcement invisible to every reference cross section.
    #[test]
    fn the_default_card_is_accepted() {
        RunCard::parse("").expect("an empty card is MadGraph's own configuration");
        RunCard::parse("  0 = lpp1\n  0 = lpp2\n  none = pdlabel1\n  none = pdlabel2\n")
            .expect("a fixed-energy card may name no per-beam PDF set");
    }
}
