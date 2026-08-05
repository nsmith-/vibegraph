//! The kT clustering itself: `cluster.f`'s walk from an event's external
//! momenta down to a `2 → 2` core.
//!
//! Each step measures every pair of surviving lines the merge graph allows,
//! merges the smallest, and repeats. Four properties of that loop decide the
//! answer as much as the measure does.
//!
//! * **A beam–leg pair is measured by the leg alone.** `djb(leg)` — the beam
//!   enters only through the tie-break below. A final-state pair is measured by
//!   `dj`, or by its own invariant mass when it is tagged as an on-shell
//!   resonance.
//! * **A crossed beam–leg candidate is inflated by `1 + 1e-6`**, which prefers
//!   clustering an outgoing leg onto the beam it followed. On a genuine tie it
//!   decides the winner; when every admissible candidate is crossed it does not
//!   cancel, and the factor survives into the scale.
//! * **The winner is the first minimiser in visit order**, since the comparison
//!   is strict: pairs are visited by outer line then inner line, and an exact
//!   tie goes to the earlier pair.
//! * **An initial-state merge can boost and rotate the whole event.** The
//!   measures are not invariant, so every later measure — and the core's own —
//!   is evaluated in the rotated frame. With four external legs the guard is
//!   never satisfied, which is why a `2 → 2` cluster scale is frame-free and a
//!   longer one is not.

use super::graph::{ChannelSet, ColorTable, MergeTable};

/// `cluster.f`'s inflation of a beam–leg candidate whose legs point in opposite
/// directions.
pub const TIE_BREAK: f64 = 1.0 + 1e-6;

/// The sentinel `cluster.f` leaves on a pair no channel allows. A candidate at or
/// above it can never win, so an infinite measure is inert rather than fatal.
pub const NO_MEASURE: f64 = 1.0e37;

/// The squared boost invariant below which `cluster.f:736` declines to change
/// frame, in GeV².
const BOOST_FLOOR: f64 = 100.0;

/// Which arm of the measure a candidate took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measure {
    /// No channel allows the pair, so it was never measured.
    None,
    /// `djb` of the final-state line of a beam–leg pair.
    BeamLeg,
    /// `dj`'s Durham arm, for beams with no parton density.
    Durham,
    /// `dj`'s hadronic arm.
    Hadronic,
    /// `dj`'s massless–massive arm, with the light leg first or second.
    MasslessMassive(u8),
    /// `dj`'s guard against a pair with no three-momentum.
    Degenerate,
    /// The pair's invariant mass, for a mother tagged on-shell.
    ResonanceMass,
}

impl Measure {
    /// The name the instrumented reference prints for this arm.
    pub fn name(self) -> &'static str {
        match self {
            Measure::None => "NONE",
            Measure::BeamLeg => "IS_DJB",
            Measure::Durham => "FS_DJ_DURHAM",
            Measure::Hadronic => "FS_DJ_HAD",
            Measure::MasslessMassive(1) => "FS_DJ_MLESS_MASSIVE_1",
            Measure::MasslessMassive(_) => "FS_DJ_MLESS_MASSIVE_2",
            Measure::Degenerate => "FS_DJ_DEGENERATE",
            Measure::ResonanceMass => "FS_SUMDOT_BW",
        }
    }
}

/// Whether a merge took a beam line with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeKind {
    Initial,
    Final,
    /// The terminal vertex: the two beam lines and the leftover blob, written
    /// after the last real merge rather than chosen by a measure.
    Core,
}

/// One merge of the sequence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Merge {
    /// `idacl(n, 1:2)`: the lower-position line first.
    pub daughters: [u32; 2],
    /// `imocl(n)`.
    pub mother: u32,
    pub kind: MergeKind,
    /// `pt2ijcl(n)`: the winning measure, or the core's `djb`.
    pub pt2: f64,
    /// `mt2ij(n)`: the emitted leg's beam measure, nonzero only for an
    /// initial-state merge.
    pub mt2: f64,
    pub z: f64,
    /// `icluster(1:4, n)`: the original leg numbers, and the resonance tag.
    pub icluster: [i32; 4],
}

/// One candidate pair, whether or not the merge graph allowed it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub pass: usize,
    /// The two lines' positions in the surviving list, outer first.
    pub position: [usize; 2],
    /// Their original leg numbers.
    pub leg: [usize; 2],
    pub daughters: [u32; 2],
    pub mother: u32,
    pub admissible: bool,
    pub measure: Measure,
    /// The measure before the crossing inflation.
    pub raw: f64,
    pub inflated: bool,
    pub pt2: f64,
    pub z: f64,
    pub n_graphs: usize,
}

/// A frame change at an initial-state merge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boost {
    pub merge: usize,
    pub fired: bool,
    pub lines_left: usize,
    pub frame: [f64; 4],
    pub invariant: f64,
}

/// What one clustering produced.
#[derive(Clone, Debug)]
pub struct Clustering {
    /// `nexternal - 2` entries; the last is the terminal vertex.
    pub merges: Vec<Merge>,
    /// `mt2last`: the geometric mean of the last real merge's daughters' beam
    /// measures, set only when that merge was final-state.
    pub mt2last: f64,
    /// The channels surviving every intersection, after the integration channel
    /// is allowed to claim the list.
    pub graphs: Vec<usize>,
    /// The same list before that claim.
    pub graphs_before_claim: Vec<usize>,
    /// `ibwlist`: (leg set, forest line) of each resonance tagged on-shell.
    pub tagged: Vec<(u32, i32)>,
    /// `pcl(0:4, mask)` of every line the clustering built, in whatever frame it
    /// left them.
    pub lines: Vec<[f64; 5]>,
    pub candidates: Vec<Candidate>,
    pub boosts: Vec<Boost>,
}

/// The clustering refused the event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterFailure {
    /// No pair of lines is allowed by any channel, so there is nothing to merge.
    NoAdmissiblePair,
    /// A mother the clustering built is not a line of any surviving channel,
    /// which `cluster.f:703` treats as a hard error.
    InvalidCombination,
}

/// The run-card constants the clustering branches on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusterSettings {
    /// Either beam carries a parton density, which is what both `dj` and `djb`
    /// switch their measure on.
    pub hadronic: bool,
    /// The hidden run-card `d`, squared in the denominator of the hadronic
    /// final-state measure.
    pub d_parameter: f64,
    pub bwcutoff: f64,
    pub small_width_treatment: f64,
}

impl Default for ClusterSettings {
    fn default() -> Self {
        ClusterSettings {
            hadronic: true,
            d_parameter: 1.0,
            bwcutoff: 15.0,
            small_width_treatment: 1e-6,
        }
    }
}

/// One process directory's static data, as one subprocess of it sees them.
pub struct Channel<'a> {
    pub set: &'a ChannelSet,
    pub table: &'a MergeTable,
    pub colors: &'a ColorTable,
    /// The channel being integrated, which selects the merge table, tags the
    /// resonances and claims the surviving graph list at the core.
    pub this_config: usize,
    /// The subprocess of the group, from `1`.
    pub iproc: usize,
}

impl Channel<'_> {
    fn forest(&self) -> &super::graph::ConfigForest {
        &self.set.configs[self.this_config - 1]
    }

    pub fn pdg(&self, mask: u32, graph: usize) -> i64 {
        self.table.ipdgcl.get(&(mask, graph)).copied().unwrap_or(0)
    }
}

/// `dot` (`kin_functions.f:593`), with the clamp that returns an exactly massless
/// leg as massless rather than as the residue eleven printed digits leave.
pub fn mg_dot(p1: &[f64; 4], p2: &[f64; 4]) -> f64 {
    let dot = p1[0] * p2[0] - p1[1] * p2[1] - p1[2] * p2[2] - p1[3] * p2[3];
    if dot.abs() < 1e-6 {
        // The Fortran literal is single precision and underflows to zero, so the
        // clamp is against the Euclidean product itself.
        let euclidean = (p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2] + p1[3] * p2[3])
            .max(f64::from(1e-99f32));
        if dot / euclidean < 1e-6 {
            return 0.0;
        }
    }
    dot
}

fn spatial(p: &[f64; 5]) -> [f64; 4] {
    [p[0], p[1], p[2], p[3]]
}

/// `SumDot(p1, p2, +1)`: the pair's invariant mass squared.
fn sum_dot(p1: &[f64; 5], p2: &[f64; 5]) -> f64 {
    let total = [p1[0] + p2[0], p1[1] + p2[1], p1[2] + p2[2], p1[3] + p2[3]];
    mg_dot(&total, &total)
}

/// `djb`: one line's measure against the beams.
pub fn djb(settings: &ClusterSettings, p: &[f64; 5]) -> f64 {
    if settings.hadronic {
        (p[0] - p[3]) * (p[0] + p[3])
    } else {
        let e = p[0].max(0.0);
        e * e
    }
}

/// `dj`: two final-state lines against each other.
pub fn dj(
    settings: &ClusterSettings,
    colors: &ColorTable,
    p1: &[f64; 5],
    p2: &[f64; 5],
) -> (f64, Measure) {
    if !settings.hadronic {
        let a1 = (p1[1] * p1[1] + p1[2] * p1[2] + p1[3] * p1[3]).sqrt();
        let a2 = (p2[1] * p2[1] + p2[2] * p2[2] + p2[3] * p2[3]).sqrt();
        if a1 * a2 != 0.0 {
            let cos = (p1[1] * p2[1] + p1[2] * p2[2] + p1[3] * p2[3]) / (a1 * a2);
            return (
                2.0 * (p1[0] * p1[0]).min(p2[0] * p2[0]) * (1.0 - cos),
                Measure::Durham,
            );
        }
        return (0.0, Measure::Degenerate);
    }
    let pt1 = p1[1] * p1[1] + p1[2] * p1[2];
    let pt2 = p2[1] * p2[1] + p2[2] * p2[2];
    let a1 = (pt1 + p1[3] * p1[3]).sqrt();
    let a2 = (pt2 + p2[3] * p2[3]).sqrt();
    let maxjetflavor = colors.maxjetflavor();
    let massive = |m2: f64| (m2 >= 3.0 && maxjetflavor > 4) || (m2 >= 1.0 && maxjetflavor > 3);
    if p1[4] < 1.0 && massive(p2[4]) {
        return (djb(settings, p1) * TIE_BREAK, Measure::MasslessMassive(1));
    }
    if p2[4] < 1.0 && massive(p1[4]) {
        return (djb(settings, p2) * TIE_BREAK, Measure::MasslessMassive(2));
    }
    let eta1 = 0.5 * ((a1 + p1[3]) / (a1 - p1[3])).ln();
    let eta2 = 0.5 * ((a2 + p2[3]) / (a2 - p2[3])).ln();
    let angle = (eta1 - eta2).cosh() - (p1[1] * p2[1] + p1[2] * p2[2]) / (pt1 * pt2).sqrt();
    let d = settings.d_parameter;
    (
        p1[4].max(p2[4]) + pt1.min(pt2) * 2.0 * angle / (d * d),
        Measure::Hadronic,
    )
}

/// `zclus`: the Pythia initial-state evolution variable a beam–leg merge records.
/// Nothing downstream of the scale reads it; it is carried so a merge record can
/// be compared field for field.
fn zclus(p2: &[f64; 5], p1: &[f64; 5], part: &[f64; 5]) -> f64 {
    let star = [p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2], p1[3] - p2[3]];
    let previous = sum_dot(p1, part);
    let reduced = {
        let star5 = [star[0], star[1], star[2], star[3], 0.0];
        sum_dot(&star5, part)
    };
    if reduced < 1.0 {
        return 0.0;
    }
    reduced / previous
}

/// Fortran's `sign(1d0, x)`, which is `+1` at `+0.0` and `−1` at `−0.0`.
fn fortran_sign(x: f64) -> f64 {
    if x >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// `boostx`: `p`, given in the rest frame of `q`, expressed in `q`'s frame.
fn boost(p: &[f64; 4], q: &[f64; 4]) -> [f64; 4] {
    let qq = q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if qq == 0.0 {
        return *p;
    }
    let pq = p[1] * q[1] + p[2] * q[2] + p[3] * q[3];
    let m = (q[0] * q[0] - qq).max(1e-99).sqrt();
    let lf = ((q[0] - m) * pq / qq + p[0]) / m;
    [
        (p[0] * q[0] + pq) / m,
        p[1] + q[1] * lf,
        p[2] + q[2] * lf,
        p[3] + q[3] * lf,
    ]
}

/// The rotation `constr` builds to carry `p1` onto `p2`.
#[derive(Clone, Copy, Debug, Default)]
struct Rotation {
    axis: [f64; 3],
    norm2: f64,
    cos: f64,
    sin: f64,
}

fn cross(p1: &[f64; 4], p2: &[f64; 4]) -> [f64; 3] {
    [
        p1[2] * p2[3] - p1[3] * p2[2],
        p1[3] * p2[1] - p1[1] * p2[3],
        p1[1] * p2[2] - p1[2] * p2[1],
    ]
}

fn constr(p1: &[f64; 4], p2: &[f64; 4]) -> Rotation {
    let mut cos = p1[1] * p2[1] + p1[2] * p2[2] + p1[3] * p2[3];
    cos /= (p1[1] * p1[1] + p1[2] * p1[2] + p1[3] * p1[3]).sqrt();
    cos /= (p2[1] * p2[1] + p2[2] * p2[2] + p2[3] * p2[3]).sqrt();
    let clamped = if cos - 1.0 > 0.0 { 0.0 } else { cos };
    let sin = (1.0 - clamped * clamped).sqrt();
    let axis = cross(p1, p2);
    let norm2 = axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2];
    if norm2 <= 1e-34 {
        return Rotation {
            axis,
            norm2: 0.0,
            cos,
            sin,
        };
    }
    Rotation {
        axis,
        norm2,
        cos,
        sin,
    }
}

fn rotate(p: &[f64; 4], r: &Rotation, forward: bool) -> [f64; 4] {
    if r.norm2 == 0.0 {
        return *p;
    }
    let nn = r.norm2.sqrt();
    let na = (r.axis[0] * p[1] + r.axis[1] * p[2] + r.axis[2] * p[3]) / r.norm2;
    let mut along = [0.0; 3];
    let mut perp = [0.0; 3];
    for i in 0..3 {
        along[i] = r.axis[i] * na;
        perp[i] = p[i + 1] - along[i];
    }
    let perp4 = [0.0, perp[0], perp[1], perp[2]];
    let axis4 = [0.0, r.axis[0], r.axis[1], r.axis[2]];
    let cr = cross(&axis4, &perp4);
    let mut out = [p[0], 0.0, 0.0, 0.0];
    for i in 0..3 {
        out[i + 1] = if forward {
            along[i] + r.cos * perp[i] + r.sin / nn * cr[i]
        } else {
            along[i] + r.cos * perp[i] - r.sin / nn * cr[i]
        };
    }
    out
}

/// `checkbw` and `cut_bw`: which of the integration channel's timelike
/// propagators this event puts on shell.
///
/// The tagging reads the integration channel and only it, so it is a property of
/// the channel and not of the process: two channels of the same subprocess tag
/// different lines on the same momenta.
fn checkbw(channel: &Channel<'_>, settings: &ClusterSettings, p: &[[f64; 4]]) -> Vec<(u32, i32)> {
    let forest = channel.forest();
    let n = channel.set.n_external;
    let on_bw = cut_bw(channel, settings, p);
    let mut tagged = Vec::new();
    for k in 1..=n.saturating_sub(3) {
        let index = -(k as i32);
        let Some(mask) = forest.mask(index) else {
            continue;
        };
        if on_bw.contains(&index) {
            tagged.push((mask, index));
        }
    }
    tagged
}

/// The lines `cut_bw` leaves flagged: a timelike propagator whose invariant mass
/// sits inside `bwcutoff` widths of its pole, with the closer of two identical
/// nested resonances winning.
fn cut_bw(channel: &Channel<'_>, settings: &ClusterSettings, p: &[[f64; 4]]) -> Vec<i32> {
    let forest = channel.forest();
    let n = channel.set.n_external;
    let n_in = channel.set.n_incoming;
    // The identical-daughter test reads the first subprocess with a timelike
    // propagator on the outermost line, not the subprocess being integrated.
    let iproc = (1..=channel.set.n_proc())
        .find(|&q| {
            forest
                .lines
                .iter()
                .find(|l| l.index == -1)
                .and_then(|l| l.sprop.get(q - 1).copied())
                .unwrap_or(0)
                != 0
        })
        .unwrap_or(1);
    let mut momenta: Vec<[f64; 4]> = vec![[0.0; 4]; n + 1];
    momenta[1..=n].copy_from_slice(&p[..n]);
    let mut internal: Vec<[f64; 4]> = vec![[0.0; 4]; n + 1];
    let momentum = |i: i32, ext: &Vec<[f64; 4]>, int: &Vec<[f64; 4]>| -> [f64; 4] {
        if i > 0 {
            ext[i as usize]
        } else {
            int[(-i) as usize]
        }
    };
    let mut on_bw: Vec<i32> = Vec::new();
    let mut mass_of: Vec<(i32, f64)> = Vec::new();
    let mut tsgn = 1.0f64;
    for k in 1..=n.saturating_sub(3) {
        let index = -(k as i32);
        let Some(line) = forest.lines.iter().find(|l| l.index == index) else {
            continue;
        };
        if line.daughters[0] == 1 || (n_in == 2 && line.daughters[0] == 2) {
            tsgn = -1.0;
        }
        let d1 = momentum(line.daughters[0], &momenta, &internal);
        let d2 = momentum(line.daughters[1], &momenta, &internal);
        let mut q = [0.0; 4];
        for j in 0..4 {
            q[j] = d1[j] + tsgn * d2[j];
        }
        internal[k] = q;
        if tsgn < 0.0 {
            continue;
        }
        if !(line.width > 0.0) {
            continue;
        }
        let xmass = mg_dot(&q, &q).sqrt();
        mass_of.push((index, xmass));
        let width = line.width.max(line.mass * settings.small_width_treatment);
        let onshell =
            (xmass - line.mass).abs() < settings.bwcutoff * width && width / line.mass < 0.1;
        if !onshell {
            continue;
        }
        on_bw.push(index);
        // Only one of two nested lines carrying the same propagator may be
        // tagged; the one further from its pole loses.
        let sprop = line.sprop.get(iproc - 1).copied().unwrap_or(0);
        let mut identical = 0i32;
        for &daughter in &line.daughters {
            let same = if daughter < 0 {
                forest
                    .lines
                    .iter()
                    .find(|l| l.index == daughter)
                    .and_then(|l| l.sprop.get(iproc - 1).copied())
                    .unwrap_or(0)
                    == sprop
            } else {
                channel.set.external_pdg[iproc - 1][daughter as usize - 1] == sprop
            };
            if same {
                identical = daughter;
            }
        }
        if identical > 0 {
            on_bw.retain(|&i| i != index);
        } else if identical < 0 {
            let inner = momentum(identical, &momenta, &internal);
            let inner_mass = mg_dot(&inner, &inner).sqrt();
            if (xmass - line.mass).abs() > (inner_mass - line.mass).abs() {
                on_bw.retain(|&i| i != index);
            } else {
                on_bw.retain(|&i| i != identical);
            }
        }
    }
    on_bw
}

/// Cluster one event down to a `2 → 2` core.
///
/// `p` carries the external momenta in the order the channel's leg numbers use,
/// beams first. `chcluster` restricts the merge graph to the integration channel
/// alone, which the caller sets while the jet memo is being filled.
pub fn cluster(
    channel: &Channel<'_>,
    settings: &ClusterSettings,
    p: &[[f64; 4]],
    chcluster: bool,
    carried_on_shell: &[u32],
    trace: bool,
) -> Result<Clustering, ClusterFailure> {
    let n = channel.set.n_external;
    let n_masks = 1usize << n;
    let mut pcl: Vec<[f64; 5]> = vec![[0.0; 5]; n_masks];
    let mut pt2ij: Vec<f64> = vec![NO_MEASURE; n_masks];
    let mut zij: Vec<f64> = vec![0.0; n_masks];
    let mut is_bw: Vec<bool> = vec![false; n_masks];

    let tagged = checkbw(channel, settings, p);
    for &(mask, _) in &tagged {
        is_bw[mask as usize] = true;
    }
    // `cluster.f`'s on-shell flags live in a common block that `checkbw` clears
    // only for the leg sets the *integration channel's* own forest names. A leg
    // set some other channel flagged keeps its flag into the next event, so a
    // replay of one particular run has to be told which — the flags are not a
    // function of the event. They change the measure of a final-state pair and
    // the mass the merged line carries; they do not enter the resonance list the
    // merge graph is filtered by, which is rebuilt per event.
    for &mask in carried_on_shell {
        if (mask as usize) < n_masks {
            is_bw[mask as usize] = true;
        }
    }
    let tag_masks: Vec<u32> = tagged.iter().map(|&(mask, _)| mask).collect();

    let mut lines: Vec<(usize, u32)> = (1..=n).map(|i| (i, 1u32 << (i - 1))).collect();
    for i in 1..=n {
        let mask = (1usize << (i - 1)) as usize;
        pcl[mask][..4].copy_from_slice(&p[i - 1]);
        pcl[mask][4] = mg_dot(&p[i - 1], &p[i - 1]);
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut boosts: Vec<Boost> = Vec::new();
    let mut merges: Vec<Merge> = Vec::new();
    let mut mt2last = 0.0f64;
    let mut graphs: Vec<usize> = Vec::new();

    let mut winner: Option<(usize, usize)> = None;
    let mut min_pt2 = NO_MEASURE;
    // The boost frame and the rotation it built outlive the merge that set them:
    // `cluster.f` keeps `pcmsp` and `constr`'s output from the last initial-state
    // merge, and the un-boost at the core reads whatever is left in them.
    let mut frame = [0.0f64; 4];
    let mut rotation = Rotation::default();

    // First pass: every pair of external legs, outer index then inner.
    for i in 3..=n {
        for j in 1..i {
            let idi = 1u32 << (i - 1);
            let idj = 1u32 << (j - 1);
            let idij = idi + idj;
            let partner = lines[if j == 1 { 1 } else { 0 }].1;
            let n_graphs =
                channel
                    .table
                    .seed_count(idij, chcluster, channel.this_config, &tag_masks);
            pt2ij[idij as usize] = NO_MEASURE;
            let mut record = Candidate {
                pass: 0,
                position: [i, j],
                leg: [i, j],
                daughters: [idi, idj],
                mother: idij,
                admissible: n_graphs > 0,
                measure: Measure::None,
                raw: NO_MEASURE,
                inflated: false,
                pt2: NO_MEASURE,
                z: 0.0,
                n_graphs,
            };
            if n_graphs > 0 {
                measure_pair(
                    channel,
                    settings,
                    &pcl,
                    &is_bw,
                    idi,
                    idj,
                    idij,
                    j,
                    partner,
                    &mut pt2ij,
                    &mut zij,
                    &mut record,
                );
                if pt2ij[idij as usize] < min_pt2 {
                    min_pt2 = pt2ij[idij as usize];
                    winner = Some((j, i));
                }
                record.z = zij[idij as usize];
            }
            record.pt2 = pt2ij[idij as usize];
            if trace {
                candidates.push(record);
            }
        }
    }

    // A `2 → 1` has nothing to merge: the single outgoing line is the core, and
    // its own mass is the scale.
    if n == 3 && channel.set.n_incoming == 2 {
        let blob = lines[2].1;
        merges.push(Merge {
            daughters: [lines[0].1, lines[1].1],
            mother: blob,
            kind: MergeKind::Core,
            pt2: pcl[blob as usize][4],
            mt2: 0.0,
            z: 0.0,
            icluster: [1, 3, 2, 0],
        });
        return Ok(Clustering {
            merges,
            mt2last,
            graphs: vec![channel.this_config],
            graphs_before_claim: vec![channel.this_config],
            tagged,
            lines: pcl,
            candidates,
            boosts,
        });
    }

    let Some((mut iwin, mut jwin)) = winner else {
        return Err(ClusterFailure::NoAdmissiblePair);
    };

    let mut lines_left = n;
    for step in 1..=(n - 2) {
        let mother = lines[iwin - 1].1 + lines[jwin - 1].1;
        let d1 = lines[iwin - 1].1;
        let d2 = lines[jwin - 1].1;
        let mut icluster = [lines[iwin - 1].0 as i32, lines[jwin - 1].0 as i32, 0, 0];
        if is_bw[mother as usize] {
            if let Some(&(_, line)) = tagged.iter().find(|&&(mask, _)| mask == mother) {
                icluster[3] = line;
            }
        }
        graphs = if graphs.is_empty() {
            channel
                .table
                .seed(mother, chcluster, channel.this_config, &tag_masks)
                .ok_or(ClusterFailure::InvalidCombination)?
        } else {
            channel
                .table
                .narrow(mother, &graphs)
                .ok_or(ClusterFailure::InvalidCombination)?
        };

        let mut mt2 = 0.0;
        if iwin < 3 {
            mt2 = djb(settings, &pcl[d2 as usize]);
            let spectator = lines[2 - iwin].1;
            icluster[2] = lines[2 - iwin].0 as i32;
            for k in 0..4 {
                pcl[mother as usize][k] = pcl[d1 as usize][k] - pcl[d2 as usize][k];
                frame[k] = -pcl[mother as usize][k] - pcl[spectator as usize][k];
            }
            frame[0] = -frame[0];
            pcl[mother as usize][4] = 0.0;
            if pcl[d1 as usize][4] > 0.0 || pcl[d2 as usize][4] > 0.0 {
                pcl[mother as usize][4] = pcl[d1 as usize][4].max(pcl[d2 as usize][4]);
            }
            let invariant = frame[0] * frame[0]
                - frame[1] * frame[1]
                - frame[2] * frame[2]
                - frame[3] * frame[3];
            let boosted = invariant > BOOST_FLOOR && lines_left > 4;
            if boosted {
                let along_z = [1.0, 0.0, 0.0, 1.0];
                let carried = boost(&spatial(&pcl[mother as usize]), &frame);
                rotation = constr(&carried, &along_z);
                for &(_, mask) in lines.iter().take(lines_left) {
                    let moved = rotate(
                        &boost(&spatial(&pcl[mask as usize]), &frame),
                        &rotation,
                        true,
                    );
                    pcl[mask as usize][..4].copy_from_slice(&moved);
                }
                let moved = rotate(
                    &boost(&spatial(&pcl[mother as usize]), &frame),
                    &rotation,
                    true,
                );
                pcl[mother as usize][..4].copy_from_slice(&moved);
            }
            if trace {
                boosts.push(Boost {
                    merge: step,
                    fired: boosted,
                    lines_left,
                    frame,
                    invariant,
                });
            }
        } else {
            for k in 0..4 {
                pcl[mother as usize][k] = pcl[d1 as usize][k] + pcl[d2 as usize][k];
            }
            pcl[mother as usize][4] = 0.0;
            if pcl[d1 as usize][4] > 0.0 || pcl[d2 as usize][4] > 0.0 {
                pcl[mother as usize][4] = pcl[d1 as usize][4].max(pcl[d2 as usize][4]);
            }
            if is_bw[mother as usize] {
                pcl[mother as usize][4] = min_pt2;
            }
        }

        merges.push(Merge {
            daughters: [d1, d2],
            mother,
            kind: if iwin < 3 {
                MergeKind::Initial
            } else {
                MergeKind::Final
            },
            pt2: min_pt2,
            mt2,
            z: zij[mother as usize],
            icluster,
        });

        lines_left -= 1;
        lines[iwin - 1].1 = mother;
        for position in jwin..=lines_left {
            lines[position - 1] = lines[position];
        }

        if lines_left <= 3 {
            if iwin > 2 {
                mt2last =
                    (djb(settings, &pcl[d1 as usize]) * djb(settings, &pcl[d2 as usize])).sqrt();
                let invariant = frame[0] * frame[0]
                    - frame[1] * frame[1]
                    - frame[2] * frame[2]
                    - frame[3] * frame[3];
                if invariant > BOOST_FLOOR {
                    let blob = lines[2].1;
                    let back = rotate(&spatial(&pcl[blob as usize]), &rotation, false);
                    let mut inverse = frame;
                    for k in 1..4 {
                        inverse[k] = -inverse[k];
                    }
                    let moved = boost(&back, &inverse);
                    pcl[blob as usize][..4].copy_from_slice(&moved);
                }
            }
            let blob = lines[2].1;
            let graphs_before_claim = graphs.clone();
            if graphs.contains(&channel.this_config) {
                graphs = vec![channel.this_config];
            }
            merges.push(Merge {
                daughters: [lines[0].1, lines[1].1],
                mother: blob,
                kind: MergeKind::Core,
                pt2: djb(settings, &pcl[blob as usize]),
                mt2: 0.0,
                z: 1.0,
                icluster: [1, 3, 2, 0],
            });
            return Ok(Clustering {
                merges,
                mt2last,
                graphs,
                graphs_before_claim,
                tagged,
                lines: pcl,
                candidates,
                boosts,
            });
        }

        // Every measure is re-taken: an initial-state merge may have moved the
        // frame, and the surviving graph list has narrowed.
        min_pt2 = NO_MEASURE;
        let mut next: Option<(usize, usize)> = None;
        for i in 3..=lines_left {
            for j in 1..i {
                let idi = lines[i - 1].1;
                let idj = lines[j - 1].1;
                let idij = idi + idj;
                let partner = lines[if j == 1 { 1 } else { 0 }].1;
                let narrowed = channel.table.narrow(idij, &graphs);
                let n_graphs = narrowed.as_ref().map_or(0, Vec::len);
                pt2ij[idij as usize] = NO_MEASURE;
                let mut record = Candidate {
                    pass: step,
                    position: [i, j],
                    leg: [lines[i - 1].0, lines[j - 1].0],
                    daughters: [idi, idj],
                    mother: idij,
                    admissible: narrowed.is_some(),
                    measure: Measure::None,
                    raw: NO_MEASURE,
                    inflated: false,
                    pt2: NO_MEASURE,
                    z: 0.0,
                    n_graphs,
                };
                if narrowed.is_some() {
                    measure_pair(
                        channel,
                        settings,
                        &pcl,
                        &is_bw,
                        idi,
                        idj,
                        idij,
                        j,
                        partner,
                        &mut pt2ij,
                        &mut zij,
                        &mut record,
                    );
                    if pt2ij[idij as usize] < min_pt2 {
                        min_pt2 = pt2ij[idij as usize];
                        next = Some((j, i));
                    }
                    record.z = zij[idij as usize];
                }
                record.pt2 = pt2ij[idij as usize];
                if trace {
                    candidates.push(record);
                }
            }
        }
        let Some((next_i, next_j)) = next else {
            return Err(ClusterFailure::NoAdmissiblePair);
        };
        iwin = next_i;
        jwin = next_j;
    }

    Err(ClusterFailure::NoAdmissiblePair)
}

/// Give one admissible pair its measure, and inflate it if a beam–leg pair's two
/// lines point in opposite directions.
#[allow(clippy::too_many_arguments)]
fn measure_pair(
    channel: &Channel<'_>,
    settings: &ClusterSettings,
    pcl: &[[f64; 5]],
    is_bw: &[bool],
    idi: u32,
    idj: u32,
    idij: u32,
    j: usize,
    partner: u32,
    pt2ij: &mut [f64],
    zij: &mut [f64],
    record: &mut Candidate,
) {
    if j >= 3 {
        if is_bw[idij as usize] {
            pt2ij[idij as usize] = sum_dot(&pcl[idi as usize], &pcl[idj as usize]);
            record.measure = Measure::ResonanceMass;
        } else {
            let (value, measure) = dj(
                settings,
                channel.colors,
                &pcl[idi as usize],
                &pcl[idj as usize],
            );
            pt2ij[idij as usize] = value;
            record.measure = measure;
        }
        zij[idij as usize] = 0.0;
        record.raw = pt2ij[idij as usize];
        return;
    }
    pt2ij[idij as usize] = djb(settings, &pcl[idi as usize]);
    zij[idij as usize] = zclus(
        &pcl[idi as usize],
        &pcl[idj as usize],
        &pcl[partner as usize],
    );
    record.measure = Measure::BeamLeg;
    record.raw = pt2ij[idij as usize];
    if fortran_sign(pcl[idi as usize][3]) != fortran_sign(pcl[idj as usize][3]) {
        pt2ij[idij as usize] *= TIE_BREAK;
        record.inflated = true;
    }
}
