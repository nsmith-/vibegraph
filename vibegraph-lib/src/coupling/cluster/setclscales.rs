//! From a merge sequence to `μR` and the two `μF`: `reweight.f`'s `setclscales`.
//!
//! The clustering says *which* vertices the event has; this walk says which of
//! them the scales are read off, and it asks only two questions of each line —
//! `isqcd` and `isjet` of the PDG code the merge graph put on it.
//!
//! Three indices per beam carry the answer. `jfirst` is the first vertex at
//! which that beam split; `jlast` is the last vertex at which its line was still
//! a *parton*; `jcentral` is the last at which it was still *coloured*. Each is
//! recorded before the predicate that turns its flag off, so all three name the
//! vertex at which the line stopped, inclusive. `μR` is then the geometric mean
//! of the participating vertices' scales — four factors when both beams
//! contribute, two when one does, one when neither — and `μF` on a beam is the
//! geometric mean of that beam's `jlast` and `jcentral` scales.
//!
//! Two rewrites run first. A final-state last merge between coloured lines
//! replaces the central vertex's scale with the geometric mean of the merged
//! pair's transverse masses; and where a colour line ends on an initial-state
//! vertex, that vertex takes the emitted leg's transverse mass instead of the
//! merge measure.
//!
//! The jet count is memoised per integration channel: the first event of a
//! channel is clustered restricted to that channel and its jet count stored, and
//! any later event whose unrestricted clustering yields a different count is
//! re-clustered restricted. The scale is therefore not a function of the event's
//! momenta alone — it also depends on which event reached the channel first.

use super::graph::ColorTable;
use super::kt::{cluster, Channel, ClusterFailure, ClusterSettings, Clustering};

/// `reweight.f`'s floor on the geometric mean of transverse masses, below which
/// the last-merge override does not fire.
pub const MT2LAST_FLOOR: f64 = 4.0;

/// The floor a beam carrying a parton density puts under its own factorisation
/// scale, in GeV². An event below it is dropped.
pub const MUF_FLOOR: f64 = 4.0;

/// Which `μR` formula the beam indices selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MurBranch {
    /// Both beams carry a parton line: four factors.
    BothPartonLines,
    Beam1PartonLine,
    Beam2PartonLine,
    BothColourLines,
    Beam1ColourLine,
    Beam2ColourLine,
    /// No colour line reaches either beam: the core vertex's own scale.
    Core,
    /// `μR` was fixed, so the formula never ran.
    NotEntered,
}

impl MurBranch {
    pub fn name(self) -> &'static str {
        match self {
            MurBranch::BothPartonLines => "L1153",
            MurBranch::Beam1PartonLine => "L1157",
            MurBranch::Beam2PartonLine => "L1160",
            MurBranch::BothColourLines => "L1163",
            MurBranch::Beam1ColourLine => "L1165",
            MurBranch::Beam2ColourLine => "L1167",
            MurBranch::Core => "L1169",
            MurBranch::NotEntered => "NOT_ENTERED",
        }
    }
}

/// Which `μF` branch last touched the factorisation scales.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MufBranch {
    None,
    /// A `2 → 1`, where both beams take the core scale.
    TwoToOne,
    /// The geometric mean of each beam's `jlast` and `jcentral` scales.
    Geometric,
    /// The same, with a colour line through the whole event collapsing the two
    /// beams onto one scale.
    GeometricCollapsed,
    /// No colour line reaches either beam and one factorisation scale is already
    /// set: only the vertex scales are back-filled.
    Backfill1,
    Backfill2,
    /// No colour line reaches either beam: both scales come from the core.
    CoreBoth,
    Beam1FromFirst,
    Beam2FromFirst,
    /// The matched-sample branch.
    MatchingWeight,
}

impl MufBranch {
    pub fn name(self) -> &'static str {
        match self {
            MufBranch::None => "NONE",
            MufBranch::TwoToOne => "NEXT3",
            MufBranch::Geometric => "GEOM",
            MufBranch::GeometricCollapsed => "GEOM_COLLAPSED",
            MufBranch::Backfill1 => "JC0_BACKFILL1",
            MufBranch::Backfill2 => "JC0_BACKFILL2",
            MufBranch::CoreBoth => "JC0_BOTH",
            MufBranch::Beam1FromFirst => "JC0_BEAM1",
            MufBranch::Beam2FromFirst => "JC0_BEAM2",
            MufBranch::MatchingWeight => "PDFWGT",
        }
    }
}

/// Why the event carries no scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleRefusal {
    Clustering(ClusterFailure),
    /// A merge with a jet daughter fell below `xqcut`.
    JetCut,
    /// The central vertex fell below `xmtc`.
    CentralCut,
    /// A beam carrying a parton density ended below the factorisation floor.
    FactorisationFloor,
    /// The jet count disagreed with the memo even after re-clustering restricted
    /// to the integration channel, which `reweight.f` treats as fatal.
    JetCountUnreachable,
}

/// The run-card constants the scale synthesis branches on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleSettings {
    pub scalefact: f64,
    pub fixed_ren: bool,
    pub fixed_fac: [bool; 2],
    /// Whether each beam carries a parton density, which decides whether the
    /// factorisation floor applies to it.
    pub beam_has_pdf: [bool; 2],
    pub ickkw: i64,
    pub xqcut: f64,
    pub xmtc: f64,
    pub pdfwgt: bool,
}

impl Default for ScaleSettings {
    fn default() -> Self {
        ScaleSettings {
            scalefact: 1.0,
            fixed_ren: false,
            fixed_fac: [false, false],
            beam_has_pdf: [true, true],
            ickkw: 0,
            xqcut: 0.0,
            xmtc: 0.0,
            pdfwgt: false,
        }
    }
}

/// One line as the walk left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineState {
    pub mask: u32,
    pub pdg: i64,
    pub ipart: [usize; 2],
    pub goodjet: bool,
}

/// What the jet memo did with one invocation's jet count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoStep {
    /// The channel's first event: the count is stored and the event re-clustered
    /// without the restriction.
    Stored,
    /// The count disagreed with the stored one, so the event is clustered again
    /// restricted to the integration channel.
    Reclustered,
    /// The count agreed and this clustering is the one the scales are read off.
    Accepted,
}

impl MemoStep {
    pub fn name(self) -> &'static str {
        match self {
            MemoStep::Stored => "STORE_AND_RECLUSTER",
            MemoStep::Reclustered => "RESTRICTED_RECLUSTER",
            MemoStep::Accepted => "ACCEPTED",
        }
    }
}

/// One invocation of the clustering inside a single event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attempt {
    /// Whether the merge graph was restricted to the integration channel.
    pub chcluster: bool,
    pub jets: usize,
    pub memo: MemoStep,
}

/// The scales, with everything the walk decided them from.
#[derive(Clone, Debug)]
pub struct ClusterScales {
    pub mu_r: f64,
    pub q2fact: [f64; 2],
    /// `jfirst` before the fixup that fills an unset one from `jlast`.
    pub jfirst_raw: [usize; 2],
    pub jfirst: [usize; 2],
    pub jlast: [usize; 2],
    pub jcentral: [usize; 2],
    pub mur_branch: MurBranch,
    pub muf_branch: MufBranch,
    /// The three override flags: the last-merge rewrite, and the central-vertex
    /// rewrite per beam.
    pub overrides: [bool; 3],
    pub jcode: i64,
    pub njets: usize,
    pub iqjets: Vec<i64>,
    /// `pt2ijcl` after every rewrite.
    pub pt2: Vec<f64>,
    pub mt2: Vec<f64>,
    pub lines: Vec<LineState>,
    pub attempts: Vec<Attempt>,
    /// The accepted clustering.
    pub clustering: Clustering,
    /// Every attempt's clustering, in order, when tracing; the last is the
    /// accepted one.
    pub traces: Vec<Clustering>,
}

/// The jet count remembered per integration channel, `-1` until the channel's
/// first event has filled it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JetMemo(pub Option<usize>);

/// Cluster the event and read the scales off the result.
///
/// `incoming` is what the run card already fixed: `scale` and `q2fact(1:2)`,
/// zero where the card leaves them dynamic.
pub fn setclscales(
    channel: &Channel<'_>,
    cluster_settings: &ClusterSettings,
    settings: &ScaleSettings,
    p: &[[f64; 4]],
    memo: &mut JetMemo,
    card_chcluster: bool,
    carried_on_shell: &[u32],
    incoming: (f64, [f64; 2]),
    trace: bool,
) -> Result<ClusterScales, ScaleRefusal> {
    let n = channel.set.n_external;
    let (mut scale, mut q2fact) = incoming;

    // The channel's first event is clustered restricted to it, so the jet count
    // the memo stores is the one that channel alone implies.
    let mut chcluster = card_chcluster || memo.0.is_none();
    let mut attempts: Vec<Attempt> = Vec::new();
    let mut traces: Vec<Clustering> = Vec::new();

    let (clustering, walk) = loop {
        let used = chcluster;
        let clustering = cluster(channel, cluster_settings, p, used, carried_on_shell, trace)
            .map_err(ScaleRefusal::Clustering)?;
        let walk = walk_tree(channel, &clustering, n, p);
        if trace {
            traces.push(clustering.clone());
        }
        chcluster = card_chcluster;
        match memo.0 {
            None => {
                attempts.push(Attempt {
                    chcluster: used,
                    jets: walk.jets_stored,
                    memo: MemoStep::Stored,
                });
                memo.0 = Some(walk.jets_stored);
            }
            Some(stored) => {
                attempts.push(Attempt {
                    chcluster: used,
                    jets: walk.jets_counted,
                    memo: if stored == walk.jets_counted {
                        MemoStep::Accepted
                    } else {
                        MemoStep::Reclustered
                    },
                });
                if stored == walk.jets_counted {
                    break (clustering, walk);
                }
                // A count the unrestricted merge graph cannot reach is fatal
                // once the clustering is already restricted to the channel.
                if clustering.graphs[0] == channel.this_config {
                    return Err(ScaleRefusal::JetCountUnreachable);
                }
                chcluster = true;
            }
        }
    };

    let core = n - 2;
    let mut pt2: Vec<f64> = clustering.merges.iter().map(|m| m.pt2).collect();
    let mut mt2: Vec<f64> = clustering.merges.iter().map(|m| m.mt2).collect();
    let jlast = walk.jlast;
    let jcentral = walk.jcentral;
    let jfirst = walk.jfirst;
    let mut overrides = [false; 3];

    // A final-state last merge between coloured lines hands the two innermost
    // vertices the geometric mean of the merged pair's transverse masses.
    if clustering.mt2last > MT2LAST_FLOOR && n > 3 {
        let last = &clustering.merges[core - 2];
        if jlast[0] == core
            && jlast[1] == core
            && channel.colors.is_qcd(walk.pdg[last.daughters[0] as usize])
            && channel.colors.is_qcd(walk.pdg[last.daughters[1] as usize])
            && channel.colors.is_qcd(walk.pdg[last.mother as usize])
        {
            mt2[core - 1] = clustering.mt2last;
            mt2[core - 2] = clustering.mt2last;
            overrides[0] = true;
        }
    }
    for beam in 0..2 {
        if jcentral[beam] > 0 && mt2[jcentral[beam] - 1] > 0.0 {
            pt2[jcentral[beam] - 1] = mt2[jcentral[beam] - 1];
            overrides[1 + beam] = true;
        }
    }

    if settings.xqcut > 0.0 {
        for step in 0..core - 1 {
            for daughter in clustering.merges[step].daughters {
                let leg = fs_leg(&walk.ipart, daughter);
                if leg > 0 && walk.iqjets[leg - 1] > 0 && pt2[step].sqrt() < settings.xqcut {
                    return Err(ScaleRefusal::JetCut);
                }
            }
        }
    }
    if settings.xmtc * settings.xmtc > 0.0 {
        for beam in 0..2 {
            if jcentral[beam] > 0 && pt2[jcentral[beam] - 1] < settings.xmtc * settings.xmtc {
                return Err(ScaleRefusal::CentralCut);
            }
        }
    }

    let mut mur_branch = MurBranch::NotEntered;
    let mut muf_branch = MufBranch::None;
    let already_set = settings.ickkw == 0
        && (settings.fixed_fac[0] || q2fact[0] > 0.0)
        && (settings.fixed_fac[1] || q2fact[1] > 0.0)
        && (settings.fixed_ren || scale > 0.0);
    if !already_set {
        for beam in 0..2 {
            if jlast[beam] > 0 {
                pt2[jlast[beam] - 1] = pt2[jlast[beam] - 1].max(at(&pt2, jfirst[beam]));
            }
        }

        if n == 3 && channel.set.n_incoming == 2 && (q2fact[0] == 0.0 || q2fact[1] == 0.0) {
            for beam in 0..2 {
                if !settings.fixed_fac[beam] {
                    q2fact[beam] = pt2[core - 1];
                }
            }
            muf_branch = MufBranch::TwoToOne;
        }
        if q2fact[0] == 0.0 || q2fact[1] == 0.0 {
            muf_branch = MufBranch::Geometric;
            for beam in 0..2 {
                if jlast[beam] > 0 && !settings.fixed_fac[beam] {
                    q2fact[beam] = (pt2[jlast[beam] - 1] * at(&pt2, jcentral[beam])).sqrt();
                }
            }
            if jcentral[0] > 0 && jcentral[0] == jcentral[1] {
                // A colour line through the whole event, so both beams take one
                // scale.
                if !settings.fixed_fac[0] && !settings.fixed_fac[1] {
                    q2fact[0] = q2fact[0].max(q2fact[1]);
                    q2fact[1] = q2fact[0];
                    muf_branch = MufBranch::GeometricCollapsed;
                }
            }
        }
        if !settings.fixed_fac[0] || settings.fixed_fac[1] {
            for beam in 0..2 {
                if !settings.fixed_fac[beam] {
                    q2fact[beam] *= settings.scalefact * settings.scalefact;
                }
            }
        }

        if scale == 0.0 {
            let (value, branch) = if jlast[0] > 0 && jlast[1] > 0 {
                (
                    (pt2[jlast[0] - 1]
                        * at(&pt2, jcentral[0])
                        * pt2[jlast[1] - 1]
                        * at(&pt2, jcentral[1]))
                        .powf(0.125),
                    MurBranch::BothPartonLines,
                )
            } else if jlast[0] > 0 {
                (
                    (pt2[jlast[0] - 1] * at(&pt2, jcentral[0])).powf(0.25),
                    MurBranch::Beam1PartonLine,
                )
            } else if jlast[1] > 0 {
                (
                    (pt2[jlast[1] - 1] * at(&pt2, jcentral[1])).powf(0.25),
                    MurBranch::Beam2PartonLine,
                )
            } else if jcentral[0] > 0 && jcentral[1] > 0 {
                (
                    (pt2[jcentral[0] - 1] * pt2[jcentral[1] - 1]).powf(0.25),
                    MurBranch::BothColourLines,
                )
            } else if jcentral[0] > 0 {
                (pt2[jcentral[0] - 1].sqrt(), MurBranch::Beam1ColourLine)
            } else if jcentral[1] > 0 {
                (pt2[jcentral[1] - 1].sqrt(), MurBranch::Beam2ColourLine)
            } else {
                (pt2[core - 1].sqrt(), MurBranch::Core)
            };
            mur_branch = branch;
            scale = settings.scalefact * value;
        }

        if jcentral[0] == 0 && jcentral[1] == 0 {
            if q2fact[0] > 0.0 && !settings.fixed_fac[0] {
                pt2[core - 1] = q2fact[0];
                if n > 3 {
                    pt2[core - 2] = q2fact[0];
                }
                muf_branch = MufBranch::Backfill1;
            } else if q2fact[1] > 0.0 && !settings.fixed_fac[1] {
                pt2[core - 1] = q2fact[1];
                if n > 3 {
                    pt2[core - 2] = q2fact[1];
                }
                muf_branch = MufBranch::Backfill2;
            } else {
                for beam in 0..2 {
                    if !settings.fixed_fac[beam] {
                        q2fact[beam] = settings.scalefact * settings.scalefact * pt2[core - 1];
                    }
                }
                muf_branch = MufBranch::CoreBoth;
            }
        } else if jcentral[0] == 0 {
            if !settings.fixed_fac[0] {
                q2fact[0] = settings.scalefact * settings.scalefact * at(&pt2, jfirst[0]);
            }
            muf_branch = MufBranch::Beam1FromFirst;
        } else if jcentral[1] == 0 {
            if !settings.fixed_fac[1] {
                q2fact[1] = settings.scalefact * settings.scalefact * at(&pt2, jfirst[1]);
            }
            muf_branch = MufBranch::Beam2FromFirst;
        } else if settings.ickkw == 2 || (settings.pdfwgt && settings.ickkw > 0) {
            muf_branch = MufBranch::MatchingWeight;
            for beam in 0..2 {
                if jlast[beam] > 0 && jfirst[beam] <= jlast[beam] && !settings.fixed_fac[beam] {
                    q2fact[beam] = settings.scalefact
                        * settings.scalefact
                        * at(&pt2, jfirst[beam]).min(q2fact[beam]);
                }
            }
        }

        for beam in 0..2 {
            if settings.beam_has_pdf[beam]
                && q2fact[beam] < MUF_FLOOR
                && !settings.fixed_fac[beam]
            {
                return Err(ScaleRefusal::FactorisationFloor);
            }
        }
    }

    Ok(ClusterScales {
        mu_r: scale,
        q2fact,
        jfirst_raw: walk.jfirst_raw,
        jfirst,
        jlast,
        jcentral,
        mur_branch,
        muf_branch,
        overrides,
        jcode: walk.jcode,
        njets: walk.jets_counted,
        iqjets: walk.iqjets,
        pt2,
        mt2,
        lines: walk.lines,
        attempts,
        clustering,
        traces,
    })
}

/// A vertex scale by its `1`-based index, with `0` — an index the beam walk
/// never set — reading as zero rather than off the end. Every branch that can
/// reach one guards on a companion index that is nonzero exactly when this one
/// is, so the fallback stands for a combination the walk cannot produce.
fn at(pt2: &[f64], index: usize) -> f64 {
    if index == 0 {
        0.0
    } else {
        pt2[index - 1]
    }
}

/// `ifsno`: the final-state leg a line is, or `0` when it is not a single one.
fn fs_leg(ipart: &[[usize; 2]], mask: u32) -> usize {
    let first = ipart[mask as usize][0];
    if first > 2 && mask == 1u32 << (first - 1) {
        first
    } else {
        0
    }
}

struct Walk {
    pdg: Vec<i64>,
    ipart: Vec<[usize; 2]>,
    jfirst_raw: [usize; 2],
    jfirst: [usize; 2],
    jlast: [usize; 2],
    jcentral: [usize; 2],
    jcode: i64,
    /// The count the memo stores on a channel's first event, over the final
    /// state alone.
    jets_stored: usize,
    /// The count every later event is checked against, over all external legs.
    /// The two differ only if a beam line is ever tagged as a jet.
    jets_counted: usize,
    iqjets: Vec<i64>,
    lines: Vec<LineState>,
}

/// Walk the merge sequence, tracking each beam's line and which legs count as
/// jets.
fn walk_tree(
    channel: &Channel<'_>,
    clustering: &Clustering,
    n: usize,
    p: &[[f64; 4]],
) -> Walk {
    let graph = clustering.graphs[0];
    let n_masks = 1usize << n;
    let mut pdg: Vec<i64> = (0..n_masks)
        .map(|mask| channel.pdg(mask as u32, graph))
        .collect();
    let mut ipart: Vec<[usize; 2]> = vec![[0; 2]; n_masks];
    let mut goodjet: Vec<bool> = vec![false; n_masks];
    for leg in 1..=n {
        ipart[1usize << (leg - 1)] = [leg, 0];
    }
    for step in 0..n.saturating_sub(3) {
        let merge = &clustering.merges[step];
        // The provenance walk compares the *lab* transverse momenta of the
        // external legs, not the possibly boosted lines the clustering left.
        ipartupdate(
            channel.colors,
            p,
            merge.mother,
            merge.daughters,
            &mut pdg,
            &mut ipart,
        );
    }

    let mut ibeam = [1u32, 2u32];
    let mut jfirst = [0usize; 2];
    let mut jlast = [0usize; 2];
    let mut jcentral = [0usize; 2];
    let mut qcdline = [false; 2];
    let mut partonline = [false; 2];
    for beam in 0..2 {
        qcdline[beam] = channel.colors.is_qcd(pdg[ibeam[beam] as usize]);
        partonline[beam] = qcdline[beam];
        goodjet[ibeam[beam] as usize] = partonline[beam];
    }
    for leg in 3..=n {
        let mask = 1usize << (leg - 1);
        goodjet[mask] = channel.colors.is_jet(pdg[mask]);
    }

    let mut iqjets: Vec<i64> = vec![0; n];
    let mut jcode = 1i64;
    let mut increase = false;
    if n > 3 {
        for step in 1..=(n - 2) {
            let merge = clustering.merges[step - 1];
            for i in 0..2 {
                for beam in 0..2 {
                    if merge.daughters[i] != ibeam[beam] {
                        continue;
                    }
                    ibeam[beam] = merge.mother;
                    // At the terminal vertex the line continuing past it is the
                    // other beam's, not the mother.
                    let (ida, imo) = if step < n - 2 {
                        ([merge.daughters[i], merge.daughters[1 - i]], merge.mother)
                    } else {
                        ([merge.daughters[i], merge.mother], merge.daughters[1 - i])
                    };
                    if partonline[beam] {
                        if jfirst[beam] == 0 {
                            jfirst[beam] = step;
                        }
                        jlast[beam] = step;
                        partonline[beam] =
                            goodjet[ida[1] as usize] && channel.colors.is_jet(pdg[imo as usize]);
                    } else if jfirst[beam] == 0 {
                        jfirst[beam] = step;
                        goodjet[imo as usize] = false;
                    } else {
                        goodjet[imo as usize] = false;
                    }
                    if !goodjet[ida[1] as usize]
                        || !channel.colors.is_jet(pdg[ida[0] as usize])
                        || !channel.colors.is_jet(pdg[imo as usize])
                    {
                        jcode += 1;
                        increase = true;
                    } else if increase {
                        jcode += 1;
                        increase = false;
                    }
                    if goodjet[ida[1] as usize] {
                        let leg = ipart[ida[1] as usize][0];
                        if leg >= 1 && leg <= n {
                            iqjets[leg - 1] = if partonline[beam] || pdg[ida[1] as usize] == 21 {
                                1
                            } else {
                                jcode
                            };
                        }
                    }
                    if qcdline[beam] {
                        jcentral[beam] = step;
                        qcdline[beam] = channel.colors.is_qcd(pdg[imo as usize]);
                    }
                }
            }
            if merge.mother != ibeam[0] && merge.mother != ibeam[1] {
                final_state_vertex(
                    channel,
                    clustering,
                    step,
                    n,
                    &mut pdg,
                    &mut ipart,
                    &mut goodjet,
                    &mut iqjets,
                );
            }
        }
        if !partonline[0] || !partonline[1] {
            if partonline[0] || partonline[1] {
                jcode -= 1;
            }
            for leg in 3..=n {
                if iqjets[leg - 1] > 1 && iqjets[leg - 1] <= jcode {
                    iqjets[leg - 1] = 0;
                }
            }
        }
    }

    let jfirst_raw = jfirst;
    for beam in 0..2 {
        if jfirst[beam] == 0 {
            jfirst[beam] = jlast[beam];
        }
    }
    let jets_stored = (3..=n).filter(|&leg| iqjets[leg - 1] > 0).count();
    let jets_counted = (1..=n).filter(|&leg| iqjets[leg - 1] > 0).count();

    let mut lines: Vec<LineState> = Vec::new();
    for leg in 1..=n {
        let mask = 1u32 << (leg - 1);
        lines.push(LineState {
            mask,
            pdg: pdg[mask as usize],
            ipart: ipart[mask as usize],
            goodjet: goodjet[mask as usize],
        });
    }
    for merge in &clustering.merges {
        lines.push(LineState {
            mask: merge.mother,
            pdg: pdg[merge.mother as usize],
            ipart: ipart[merge.mother as usize],
            goodjet: goodjet[merge.mother as usize],
        });
    }

    Walk {
        pdg,
        ipart,
        jfirst_raw,
        jfirst,
        jlast,
        jcentral,
        jcode,
        jets_stored,
        jets_counted,
        iqjets,
        lines,
    }
}

/// The final-state half of the walk: jet tagging, and the `goodjet` propagation
/// the initial-state half reads on later vertices. Nothing here moves `jfirst`,
/// `jlast` or `jcentral`.
#[allow(clippy::too_many_arguments)]
fn final_state_vertex(
    channel: &Channel<'_>,
    clustering: &Clustering,
    step: usize,
    n: usize,
    pdg: &mut [i64],
    ipart: &mut [[usize; 2]],
    goodjet: &mut [bool],
    iqjets: &mut [i64],
) {
    let merge = clustering.merges[step - 1];
    let colors = channel.colors;
    let mother = merge.mother as usize;
    let [d1, d2] = [merge.daughters[0] as usize, merge.daughters[1] as usize];
    let islast = step == n - 2;
    if !is_jet_vertex(colors, mother, d1, d2, pdg, ipart, islast) {
        let (pdgm, pdg1, pdg2) = (pdg[mother], pdg[d1], pdg[d2]);
        if colors.is_qcd(pdgm) && colors.is_qcd(pdg1) && colors.is_qcd(pdg2) {
            // A pure colour vertex leaves the jet tags alone.
        } else if ipart[mother][0] > 2 {
            let hardest = 1usize << (ipart[mother][0] - 1);
            if !colors.is_octet(pdg[hardest]) {
                if !colors.is_qcd(pdgm) && !colors.is_qcd(pdg1) && !colors.is_qcd(pdg2) {
                    // A vertex with no colour anywhere says nothing about jets.
                } else if ipart[mother][1] == 0 {
                    iqjets[ipart[mother][0] - 1] = 0;
                } else if iqjets[ipart[mother][0] - 1] > 0 && iqjets[ipart[mother][1] - 1] > 0 {
                    iqjets[ipart[mother][0] - 1] = 0;
                }
            } else if colors.is_octet(pdgm) {
                iqjets[ipart[mother][0] - 1] = 0;
            }
        }
        if ipart[mother][1] > 2 {
            let softest = 1usize << (ipart[mother][1] - 1);
            if !colors.is_octet(pdg[softest]) && !colors.is_octet(pdg[mother]) {
                iqjets[ipart[mother][1] - 1] = 0;
            }
        }
        goodjet[mother] = false;
        return;
    }

    for daughter in [d1, d2] {
        let leg = fs_leg(ipart, daughter as u32);
        if colors.is_jet(pdg[daughter]) && leg > 0 {
            iqjets[leg - 1] = 1;
        }
    }
    goodjet[mother] = colors.is_jet(pdg[mother]) && goodjet[d1] && goodjet[d2];

    // A gluon splitting to two gluons can name a hardest gluon that leads to a
    // non-jet vertex; the mother then follows the other one instead.
    if colors.is_octet(pdg[mother]) && colors.is_octet(pdg[d1]) && colors.is_octet(pdg[d2]) {
        let follow = if ipart[mother][0] == ipart[d1][0] {
            (!goodjet[d1] && goodjet[d2]).then_some(d2)
        } else {
            (!goodjet[d2] && goodjet[d1]).then_some(d1)
        };
        if let Some(source) = follow {
            let target = ipart[mother];
            let replacement = ipart[source];
            // The rewrite sweeps from the merge's own index, which is a vertex
            // number rather than a leg set, up to the last line there can be.
            for mask in step..ipart.len() {
                if ipart[mask] == target {
                    ipart[mask] = replacement;
                }
            }
        }
    }
}

/// `isjetvx`: whether a vertex radiates a jet.
fn is_jet_vertex(
    colors: &ColorTable,
    mother: usize,
    d1: usize,
    d2: usize,
    pdg: &[i64],
    ipart: &[[usize; 2]],
    islast: bool,
) -> bool {
    let (pdgm, pdg1, pdg2) = (pdg[mother], pdg[d1], pdg[d2]);
    if islast || !colors.is_qcd(pdgm) || !colors.is_qcd(pdg1) || !colors.is_qcd(pdg2) {
        return false;
    }
    let beam1 = (1..=2).contains(&ipart[d1][0]);
    let beam2 = (1..=2).contains(&ipart[d2][0]);
    if beam1 || beam2 {
        return (beam2 && colors.is_jet(pdg1)) || (beam1 && colors.is_jet(pdg2));
    }
    (colors.is_jet(pdg1) && (colors.is_jet(pdgm) || pdgm == pdg2))
        || (colors.is_jet(pdg2) && (colors.is_jet(pdgm) || pdgm == pdg1))
}

/// Fortran's `sign(1, k)`, which is `+1` at zero.
fn fortran_isign(k: i64) -> i64 {
    if k >= 0 {
        1
    } else {
        -1
    }
}

/// `ipartupdate`: which external leg each internal line stands for, and the jet
/// flavour a splitting hands its mother.
fn ipartupdate(
    colors: &ColorTable,
    momenta: &[[f64; 4]],
    imo: u32,
    daughters: [u32; 2],
    pdg: &mut [i64],
    ipart: &mut [[usize; 2]],
) {
    let (mo, d1, d2) = (imo as usize, daughters[0] as usize, daughters[1] as usize);
    let mut idmo = pdg[mo];
    let (id1, id2) = (pdg[d1], pdg[d2]);
    let beam1 = (1..=2).contains(&ipart[d1][0]);
    let beam2 = (1..=2).contains(&ipart[d2][0]);

    if beam1 || beam2 {
        ipart[mo][1] = 0;
        if beam1 && beam2 {
            // The terminal vertex, whose mother line keeps whatever it had.
        } else if beam2 {
            ipart[mo][0] = ipart[d2][0];
            if colors.is_jet(idmo) {
                if id1 < 21 && colors.is_jet(id1) && (id2 == 21 || id2 == 22) {
                    pdg[mo] = -id1;
                }
                if id2 < 21 && colors.is_jet(id2) && (id1 == 21 || id1 == 22) {
                    pdg[mo] = id2;
                }
            }
        } else {
            ipart[mo][0] = ipart[d1][0];
            if colors.is_jet(idmo) {
                if id2 < 21 && colors.is_jet(id2) && (id1 == 21 || id1 == 22) {
                    pdg[mo] = -id2;
                }
                if id1 < 21 && colors.is_jet(id1) && (id2 == 21 || id2 == 22) {
                    pdg[mo] = id1;
                }
            }
        }
        return;
    }

    if colors.is_jet(idmo) {
        if id1 < 21 && colors.is_jet(id1) && (id2 == 21 || id2 == 22) {
            pdg[mo] = id1;
        }
        if id2 < 21 && colors.is_jet(id2) && (id1 == 21 || id1 == 22) {
            pdg[mo] = id2;
        }
        idmo = pdg[mo];
    }

    let pt2 = |line: usize| {
        let leg = ipart[line][0];
        if leg == 0 {
            return 0.0;
        }
        let p = &momenta[leg - 1];
        p[1] * p[1] + p[2] * p[2]
    };
    let harder_is_first = pt2(d1) > pt2(d2);
    let (hard, soft) = if harder_is_first { (d1, d2) } else { (d2, d1) };
    let (cmo, c1, c2) = (
        colors.color(idmo),
        colors.color(id1),
        colors.color(id2),
    );

    if idmo == 21 && id1 == 21 && id2 == 21 {
        ipart[mo] = ipart[hard];
    } else if idmo == 21 && id1.abs() <= 6 && id2.abs() <= 6 {
        ipart[mo] = [ipart[hard][0], ipart[soft][0]];
    } else if cmo == 8 && c1.abs() == 3 && c2.abs() == 3 {
        ipart[mo] = [ipart[hard][0], ipart[soft][0]];
    } else if idmo == 21 && (id1 == 21 || id2 == 21) {
        let (other_pdg, gluon) = if id1 == 21 { (id2, d1) } else { (id1, d2) };
        if colors.is_qcd(other_pdg) {
            ipart[mo] = ipart[hard];
        } else {
            ipart[mo] = ipart[gluon];
        }
    } else if idmo == 21 {
        ipart[mo] = ipart[hard];
    } else if idmo == id1 || idmo == id1 + fortran_isign(id2) {
        ipart[mo] = [ipart[d1][0], 0];
    } else if idmo == id2 || idmo == id2 + fortran_isign(id1) {
        ipart[mo] = [ipart[d2][0], 0];
    } else if cmo.abs() == 3 && c1.abs() == 3 && c2 == 1 {
        ipart[mo] = [ipart[d1][0], 0];
    } else if cmo.abs() == 3 && c2.abs() == 3 && c1 == 1 {
        ipart[mo] = [ipart[d2][0], 0];
    } else if cmo.abs() == 3 && c2.abs() == 8 && c1.abs() == 3 {
        ipart[mo] = [ipart[d2][0], 0];
    } else if cmo.abs() == 3 && c2.abs() == 3 && c1 == 8 {
        ipart[mo] = [ipart[d1][0], 0];
    } else if cmo == 1 || cmo == 2 || c1 == 2 || c2 == 2 {
        ipart[mo] = [ipart[d1][0], ipart[d2][0]];
    } else if cmo.abs() == 3 && c1.abs() == 3 && c2.abs() == 3 {
        ipart[mo] = [ipart[d1][0], 0];
    } else if cmo.abs() == 6 && c1.abs() == 3 && c2.abs() == 3 {
        ipart[mo] = [ipart[hard][0], ipart[soft][0]];
    } else if cmo.abs() == 8 && c1.abs() == 1 && c2.abs() == 8 {
        ipart[mo] = ipart[d2];
    } else if cmo.abs() == 8 && c1.abs() == 8 && c2.abs() == 1 {
        ipart[mo] = ipart[d1];
    }
    // A colour structure none of the above names is where `reweight.f` stops the
    // run; the mother's provenance is simply left unset here, which shows up as
    // a mismatch rather than as a wrong scale.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::cluster::graph::{ChannelSet, ConfigForest, ForestLine};
    use crate::coupling::cluster::kt::{MergeKind, TIE_BREAK};

    fn line(index: i32, d1: i32, d2: i32, tprid: i64, sprop: i64) -> ForestLine {
        ForestLine {
            index,
            daughters: [d1, d2],
            tprid,
            sprop: vec![sprop],
            mass: 0.0,
            width: 0.0,
        }
    }

    /// `u ū → u ū` at `QCD = 2` on beams with no parton density: flavour admits
    /// the timelike gluon `{3,4}` and the spacelike gluon `{1,3}`, so exactly one
    /// beam–leg candidate exists per beam and both can be crossed at once.
    fn uux_to_uux() -> ChannelSet {
        ChannelSet {
            n_external: 4,
            n_incoming: 2,
            configs: vec![
                ConfigForest {
                    nqcd: 2,
                    lines: vec![line(-1, 4, 3, 0, 21)],
                },
                ConfigForest {
                    nqcd: 2,
                    lines: vec![line(-1, 1, 3, 21, 0), line(-2, -1, 4, 2, 0)],
                },
            ],
            external_pdg: vec![vec![2, -2, 2, -2]],
            contributes: vec![vec![true, true]],
        }
    }

    fn colors() -> ColorTable {
        ColorTable::new([(21, 8), (2, 3), (-2, -3)], 4)
    }

    fn partonic() -> ClusterSettings {
        ClusterSettings {
            hadronic: false,
            ..ClusterSettings::default()
        }
    }

    fn settings() -> ScaleSettings {
        ScaleSettings {
            beam_has_pdf: [false, false],
            ..ScaleSettings::default()
        }
    }

    /// One `u ū → u ū` event at `√ŝ = 500`, with the outgoing quark sent forward
    /// or backward.
    fn event(forward: bool) -> Vec<[f64; 4]> {
        let (pz, pt) = (241.0_f64, 66.0_f64);
        let sign = if forward { 1.0 } else { -1.0 };
        vec![
            [250.0, 0.0, 0.0, 250.0],
            [250.0, -0.0, -0.0, -250.0],
            [250.0, pt, 0.0, sign * pz],
            [250.0, -pt, 0.0, -sign * pz],
        ]
    }

    fn run(momenta: &[[f64; 4]]) -> ClusterScales {
        let set = uux_to_uux();
        let colors = colors();
        let tables = set.merge_tables(2);
        let channel = Channel {
            set: &set,
            table: &tables[0],
            colors: &colors,
            this_config: 2,
            iproc: 1,
        };
        let mut memo = JetMemo(Some(2));
        setclscales(
            &channel,
            &partonic(),
            &settings(),
            momenta,
            &mut memo,
            false,
            &[],
            (0.0, [0.0, 0.0]),
            true,
        )
        .expect("the clustering succeeds")
    }

    /// With no parton density the beam measure is `E²`, equal for both legs, so
    /// the two beam–leg candidates tie exactly. The comparison is strict, so the
    /// earlier-visited pair wins — and with the quark forward neither candidate
    /// is crossed, so nothing is inflated.
    #[test]
    fn an_exact_tie_goes_to_the_pair_visited_first() {
        let scales = run(&event(true));
        let first = scales.clustering.merges[0];
        assert_eq!(first.kind, MergeKind::Initial);
        assert_eq!(first.daughters, [0b0001, 0b0100]);
        assert_eq!(first.pt2, 62500.0);
        assert!(scales.clustering.candidates.iter().all(|c| !c.inflated));
        assert_eq!(scales.mu_r, 250.0);
    }

    /// Send the same quark backward and *both* admissible beam–leg candidates
    /// point against their beam, so the inflation no longer cancels: the minimum
    /// itself carries it. It survives into the scale only because a colour line
    /// reaches both beams, which is what makes `jfirst` differ from `jlast`.
    #[test]
    fn a_wholly_crossed_event_carries_the_tie_break_into_the_scale() {
        let scales = run(&event(false));
        let first = scales.clustering.merges[0];
        assert_eq!(first.daughters, [0b0001, 0b0100]);
        assert_eq!(first.pt2, 62500.0 * TIE_BREAK);
        assert_eq!(
            scales.clustering.candidates.iter().filter(|c| c.inflated).count(),
            2
        );
        assert_eq!(scales.jfirst, [1, 2]);
        assert_eq!(scales.jlast, [2, 2]);
        assert_eq!(scales.jcentral, [2, 2]);
        // The inflated first vertex reaches the second through the `jfirst`
        // floor, and both scales come off that vertex.
        assert_eq!(scales.mur_branch, MurBranch::BothPartonLines);
        assert_eq!(scales.muf_branch, MufBranch::GeometricCollapsed);
        let expected = (62500.0 * TIE_BREAK).sqrt();
        assert!((scales.mu_r - expected).abs() < 1e-9 * expected);
        assert!((scales.mu_r - 250.000125).abs() < 1e-6);
        assert_eq!(scales.q2fact, [62500.0 * TIE_BREAK; 2]);
    }

    /// The same event with colourless beams: the clustering still inflates the
    /// crossed candidate, but no colour line reaches a beam, so `jlast` and
    /// `jcentral` stay zero and the scale is read off the core instead — the
    /// inflation never arrives.
    #[test]
    fn colourless_beams_keep_the_tie_break_out_of_the_scale() {
        let mut set = uux_to_uux();
        set.external_pdg = vec![vec![-11, 11, -11, 11]];
        let colors = ColorTable::new([(21, 8), (11, 1), (-11, 1), (22, 1)], 4);
        let tables = set.merge_tables(2);
        let channel = Channel {
            set: &set,
            table: &tables[0],
            colors: &colors,
            this_config: 2,
            iproc: 1,
        };
        let mut memo = JetMemo(Some(0));
        let momenta = event(false);
        let scales = setclscales(
            &channel,
            &partonic(),
            &settings(),
            &momenta,
            &mut memo,
            false,
            &[],
            (0.0, [0.0, 0.0]),
            true,
        )
        .expect("the clustering succeeds");
        assert_eq!(
            scales.clustering.candidates.iter().filter(|c| c.inflated).count(),
            2
        );
        assert_eq!(scales.jlast, [0, 0]);
        assert_eq!(scales.jcentral, [0, 0]);
        assert_eq!(scales.mur_branch, MurBranch::Core);
        assert_eq!(scales.muf_branch, MufBranch::CoreBoth);
        assert_eq!(scales.mu_r, 250.0);
    }
}
