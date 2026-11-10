//! The configuration shares an integration banks
//! ([`IntegrateArtifact::configurations`]) against MadEvent's own per-channel
//! cross sections (`validation/madgraph/config_share_reference.json`, written by
//! `gen_config_share.sh`).
//!
//! MadEvent integrates each configuration of a `P` directory in its own `G`
//! channel under the factor `AMP2_c / Σ AMP2`, so `G<n>/results.dat` holds that
//! configuration's share of the directory's cross section, summed over the
//! directory's subprocesses and beam orderings and over the configurations it
//! folds into `G<n>` by symmetry (`symfact.dat`). Each banked share is mapped
//! onto a MadEvent channel by propagator content, never by index: a channel's
//! propagators (`configs.inc`) against those of the configuration's diagrams,
//! each as the external legs on its far side from leg 1, its particle class and
//! whether it is spacelike. The map must be onto, and every share carrying
//! weight must land on exactly one channel.
//!
//! # What the rows can and cannot see
//!
//! * A share's partition, at the precision of the seed means (0.1–1% per
//!   channel): a configuration weighted by the wrong amplitudes, a dropped or
//!   double-counted configuration, a share attributed to the wrong set or
//!   configuration, or a coupling at which `AMP2` is read that moves the ratio.
//!   `e+ e- > mu+ mu-` puts 87% and 13% of σ in its γ and Z configurations, so
//!   swapping them reads hundreds of standard errors off.
//! * Not a partition difference below the gate's resolution: a `GET_CHANNEL_CUT`
//!   factor moving shares by a few per mille, say, would pass. Nor anything
//!   within one MadEvent channel: `e+ e- > u u~ g`'s γ configurations with the
//!   gluon off the quark and off the antiquark are folded into one `G` by
//!   MadEvent's symmetry, so only their sum is compared.
//! * The sum rule (Σ shares = σ) is checked per seed; it is blind to a share
//!   moved between two configurations, which is what the per-channel comparison
//!   is for.
//!
//! The known-wrong comparison runs beside it: each channel's density share
//! (`ChannelGrid::sigma_pb`, the sampler's partition) mapped the same way. On
//! `e+ e- > mu+ mu-` the γ and Z channels' maps coincide, the two terms split σ
//! evenly, and the density share reads far from MadEvent's channels; the row
//! asserts it does, so a gate that compared the wrong field would fail.
//!
//! The proton row runs under `extended-validation`: it reads the fetched PDF set.

#[path = "../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use vibegraph::artifact::IntegrateArtifact;
use vibegraph::diagrams::diagram::Diagram;
use vibegraph::ufo::UFOModel;

/// Integration seeds per row.
const SEEDS: [u64; 5] = [11, 22, 33, 44, 55];

/// A channel's fraction of σ, this side's seed mean against MadEvent's, in
/// combined standard errors.
const PULL_LIMIT: f64 = 3.5;

/// A channel's absolute share against MadEvent's, in combined standard errors:
/// loose, since it carries the two sides' σ difference as well.
const ABSOLUTE_PULL_LIMIT: f64 = 5.0;

/// The channels' fraction pulls of one row, `Σ pull² / n`, at most this. With at most
/// eight channels the 99.9% point of χ²/n is 3.3 (eight) and 10.8 (one); the
/// limit is the eight-channel one, the one-channel case being bounded by
/// [`PULL_LIMIT`] already.
const ROW_CHI2_LIMIT: f64 = 3.3;

#[derive(Deserialize)]
struct Reference {
    madgraph_version: String,
    rows: BTreeMap<String, Row>,
}

#[derive(Deserialize)]
struct Row {
    /// The proc card's lines after `import model sm`, `;`-separated.
    process: String,
    run_card: String,
    sde_strategy: i64,
    directories: BTreeMap<String, Directory>,
    runs: Vec<MgRun>,
}

#[derive(Deserialize)]
struct Directory {
    subprocesses: Vec<Subprocess>,
    channels: BTreeMap<String, Vec<(Vec<usize>, String, String)>>,
    symfact: BTreeMap<String, i64>,
}

#[derive(Deserialize)]
struct Subprocess {
    ordering: String,
    process: String,
}

#[derive(Deserialize)]
struct MgRun {
    sigma_pb: f64,
    channels: BTreeMap<String, BTreeMap<String, (f64, f64)>>,
}

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn reference() -> Reference {
    let path = madgraph_dir().join("config_share_reference.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("the reference parses")
}

/// One propagator as both sides describe it: the external legs (1-based) on
/// its far side from leg 1, its particle class, `s` or `t`.
type Line = (Vec<usize>, String, String);

fn particle_class(pdg: i64) -> &'static str {
    match pdg.abs() {
        22 => "a",
        23 => "z",
        24 => "w",
        25 => "h",
        21 => "g",
        1..=6 => "q",
        11..=16 => "l",
        other => panic!("no class for PDG {other}"),
    }
}

/// A diagram's propagators as [`Line`]s, with the two incoming legs exchanged
/// where `swap_beams`, sorted.
fn signature(diagram: &Diagram, model: &UFOModel, swap_beams: bool) -> Vec<Line> {
    let n = diagram.legs.len();
    let mut lines: Vec<Line> = diagram
        .props
        .iter()
        .map(|prop| {
            let mut side: BTreeSet<usize> = prop
                .momentum
                .iter()
                .enumerate()
                .filter(|(_, &c)| c != 0)
                .map(|(i, _)| match (swap_beams, i) {
                    (true, 0) => 2,
                    (true, 1) => 1,
                    _ => i + 1,
                })
                .collect();
            if side.contains(&1) {
                side = (1..=n).filter(|l| !side.contains(l)).collect();
            }
            let pdg = model.particle(prop.particle).pdg_code;
            let kind = if prop.is_spacelike(diagram.n_in) {
                "t"
            } else {
                "s"
            };
            (
                side.into_iter().collect(),
                particle_class(pdg).to_string(),
                kind.to_string(),
            )
        })
        .collect();
    lines.sort();
    lines
}

/// A MadEvent channel: its directory and `G` name, and the signatures of every
/// configuration it integrates (its own and those `symfact.dat` folds into it).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MgChannel {
    dir: String,
    name: String,
}

/// Each share of `artifact` that carries a configuration, mapped onto the
/// MadEvent channel whose propagators its diagrams carry. Panics where a share
/// with weight maps onto no channel or onto several, or a channel receives no
/// share.
fn map_shares(artifact: &IntegrateArtifact, row: &Row) -> BTreeMap<usize, MgChannel> {
    let model = artifact
        .loaded_model()
        .expect("the embedded model rebuilds");
    let model = &model.model;
    let mut map = BTreeMap::new();
    let mut reached = BTreeSet::new();
    for (k, share) in artifact.configurations.iter().enumerate() {
        let Some(c) = share.configuration else {
            assert_eq!(
                share.sigma, 0.0,
                "set {} has weight no configuration carries",
                share.set
            );
            continue;
        };
        let set = &artifact.sets[share.set];
        let label = set.label();
        let mut found = Vec::new();
        for (dir_name, dir) in &row.directories {
            let Some(sub) = dir.subprocesses.iter().find(|s| s.process == label) else {
                continue;
            };
            let swap = sub.ordering == "mirror";
            let ours: BTreeSet<Vec<Line>> = share
                .diagrams
                .iter()
                .map(|&d| signature(&set.diagrams[d], model, swap))
                .collect();
            for (g, sig) in &dir.channels {
                if dir.symfact[g] < 1 {
                    continue;
                }
                let class: Vec<&Vec<Line>> = dir
                    .channels
                    .iter()
                    .filter(|(k, _)| *k == g || dir.symfact[*k] == -g.parse::<i64>().unwrap())
                    .map(|(_, s)| s)
                    .collect();
                assert!(class.contains(&sig));
                if class.iter().any(|s| ours.contains(*s)) {
                    found.push(MgChannel {
                        dir: dir_name.clone(),
                        name: format!("G{g}"),
                    });
                }
            }
        }
        assert_eq!(
            found.len(),
            1,
            "{label} configuration {c} (diagrams {:?}) maps onto {found:?}",
            share.diagrams
        );
        reached.insert(found[0].clone());
        map.insert(k, found.remove(0));
    }
    for (dir_name, dir) in &row.directories {
        for (g, &f) in &dir.symfact {
            if f >= 1 {
                let channel = MgChannel {
                    dir: dir_name.clone(),
                    name: format!("G{g}"),
                };
                assert!(reached.contains(&channel), "no share maps onto {channel:?}");
            }
        }
    }
    map
}

/// One of this side's seeds: the run's σ and, per MadEvent channel, the
/// shares mapped onto it and the density shares of the channels whose
/// configurations do.
struct OurSeed {
    sigma: f64,
    shares: BTreeMap<MgChannel, (f64, f64)>,
    density: BTreeMap<MgChannel, (f64, f64)>,
}

fn add(into: &mut BTreeMap<MgChannel, (f64, f64)>, at: &MgChannel, value: f64, err: f64) {
    let e = into.entry(at.clone()).or_insert((0.0, 0.0));
    e.0 += value;
    e.1 = (e.1 * e.1 + err * err).sqrt();
}

fn integrate(name: &str, row: &Row, seed: u64, budget: [&str; 2]) -> OurSeed {
    let work = std::env::temp_dir().join(format!(
        "vibegraph-config-share-{name}-{seed}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&work).unwrap();
    let card = work.join("proc_card.dat");
    let mut text = String::from("import model sm\n");
    for line in row.process.split(';') {
        text.push_str(line);
        text.push('\n');
    }
    std::fs::write(&card, text).unwrap();
    let out = common::vibegraph(&work, &work)
        .pdf_data_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf"))
        .arg("integrate")
        .arg(&card)
        .arg("--run-card")
        .arg(madgraph_dir().join(&row.run_card))
        .args(["--out", "out", "--fixed-budget", "--no-network", "-q"])
        .args(["--neval", budget[0], "--niter", budget[1]])
        .args(["--seed", &seed.to_string()])
        .output()
        .expect("spawn vibegraph");
    assert!(
        out.status.success(),
        "{name} seed {seed}: {}",
        common::stderr_of(&out)
    );
    let artifact = IntegrateArtifact::read_from_path(&work.join("out/grid.bin.zst")).unwrap();
    std::fs::remove_dir_all(&work).ok();

    let summed: f64 = artifact.configurations.iter().map(|c| c.sigma).sum();
    assert!(
        (summed - artifact.sigma_pb).abs() <= 1e-12 * artifact.sigma_pb,
        "{name} seed {seed}: the shares sum to {summed:e}, σ is {:e}",
        artifact.sigma_pb
    );
    for (j, channel) in artifact.channels.iter().enumerate() {
        assert!(
            !channel.members.is_empty(),
            "{name} channel {j} serves nothing"
        );
        for member in &channel.members {
            assert!(
                !member.diagrams.is_empty() && member.share.is_some(),
                "{name} channel {j}: {member:?}"
            );
            let set = &artifact.sets[member.set.set];
            assert!(member.diagrams.iter().all(|&d| d < set.diagrams.len()));
        }
        if let vibegraph::artifact::ChannelKey::MergedChannel { pairs, .. } = channel.key {
            assert!(channel.members.len() >= pairs, "{name} channel {j}");
        }
    }
    let map = map_shares(&artifact, row);
    let mut shares = BTreeMap::new();
    for (&k, at) in &map {
        let c = &artifact.configurations[k];
        add(&mut shares, at, c.sigma, c.sigma_err);
    }
    // The density share: each channel's term, on the MadEvent channel its
    // configuration maps onto. Only defined where every channel serves one
    // configuration, as at fixed energy.
    let mut density = BTreeMap::new();
    for channel in &artifact.channels {
        let targets: BTreeSet<&MgChannel> = channel
            .members
            .iter()
            .filter_map(|m| m.share.and_then(|k| map.get(&k)))
            .collect();
        if let [at] = targets.into_iter().collect::<Vec<_>>()[..] {
            add(&mut density, at, channel.sigma_pb, channel.sigma_err_pb);
        }
    }
    OurSeed {
        sigma: artifact.sigma_pb,
        shares,
        density,
    }
}

fn mg_channel(run: &MgRun, at: &MgChannel) -> (f64, f64) {
    run.channels[&at.dir][&at.name]
}

/// Integrate `name` over [`SEEDS`] and compare it channel by channel; returns
/// the density share's largest pull, the known-wrong comparison.
///
/// The gate reads each channel's fraction of its own side's σ: the partition is
/// what is compared, and the two sides' totals differ at MadEvent's own
/// campaign-to-campaign scatter (on `e+ e- > mu+ mu-`, 0.08% between this file's
/// fresh-directory seeds and `grammar_sigma_reference.json`'s, five times the
/// spread of either), which moves every channel alike and is the σ gates'
/// business. A fraction's error is the channel's own carried over σ, the
/// correlation with σ left out, so it is the larger. The absolute shares are
/// compared too, at [`ABSOLUTE_PULL_LIMIT`], which a wrong unit or a dropped
/// factor would fail.
fn check_row(name: &str, budget: [&str; 2]) -> f64 {
    let reference = reference();
    assert_eq!(reference.madgraph_version, "3.7.1");
    let row = &reference.rows[name];
    assert_eq!(
        row.sde_strategy, 1,
        "the oracle's partition is sde_strategy 1's"
    );
    assert!(
        row.runs.len() >= 5,
        "{name}: {} MadEvent seeds",
        row.runs.len()
    );
    let ours: Vec<OurSeed> = SEEDS
        .iter()
        .map(|&s| integrate(name, row, s, budget))
        .collect();
    let channels: BTreeSet<MgChannel> = ours[0].shares.keys().cloned().collect();
    for o in &ours {
        assert_eq!(o.shares.keys().cloned().collect::<BTreeSet<_>>(), channels);
    }

    let total = seeds::combine(&ours.iter().map(|o| (o.sigma, 0.0)).collect::<Vec<_>>());
    let mg_total = seeds::combine(
        &row.runs
            .iter()
            .map(|r| (r.sigma_pb, 0.0))
            .collect::<Vec<_>>(),
    );
    eprintln!(
        "{name}: σ = {:.6} ± {:.6} pb here, {:.6} ± {:.6} pb MadEvent (seed spreads)",
        total.mean, total.spread, mg_total.mean, mg_total.spread
    );
    eprintln!(
        "  {:<16} {:>22} {:>22} {:>6} {:>17} {:>17} {:>6} {:>9} {:>8}",
        "channel",
        "here (pb)",
        "MadEvent (pb)",
        "pull",
        "share here",
        "share MadEvent",
        "pull",
        "χ²/dof",
        "density"
    );
    let mut chi2 = 0.0;
    let mut worst_density: f64 = 0.0;
    let mut failures = Vec::new();
    for at in &channels {
        let here = seeds::combine(&ours.iter().map(|o| o.shares[at]).collect::<Vec<_>>());
        let mg = seeds::combine(
            &row.runs
                .iter()
                .map(|r| mg_channel(r, at))
                .collect::<Vec<_>>(),
        );
        let fraction = |(v, e): (f64, f64), sigma: f64| (v / sigma, e / sigma);
        let here_f = seeds::combine(
            &ours
                .iter()
                .map(|o| fraction(o.shares[at], o.sigma))
                .collect::<Vec<_>>(),
        );
        let mg_f = seeds::combine(
            &row.runs
                .iter()
                .map(|r| fraction(mg_channel(r, at), r.sigma_pb))
                .collect::<Vec<_>>(),
        );
        let pull = here.pull(mg.mean, mg.err);
        let pull_f = here_f.pull(mg_f.mean, mg_f.err);
        chi2 += pull_f * pull_f;
        let density_pull = ours.iter().all(|o| o.density.contains_key(at)).then(|| {
            seeds::combine(&ours.iter().map(|o| o.density[at]).collect::<Vec<_>>())
                .pull(mg.mean, mg.err)
        });
        if let Some(p) = density_pull {
            worst_density = worst_density.max(p.abs());
        }
        eprintln!(
            "  {:<16} {:>10.6} ± {:<9.6} {:>10.6} ± {:<9.6} {:>6.2} {:>9.3e} ± {:<8.1e} {:>9.3e} ± {:<8.1e} \
             {:>6.2} {:>4.2}/{:<4.2} {:>8}",
            format!("{} {}", at.dir, at.name),
            here.mean,
            here.err,
            mg.mean,
            mg.err,
            pull,
            here_f.mean,
            here_f.err,
            mg_f.mean,
            mg_f.err,
            pull_f,
            here.chi2_per_dof,
            mg.chi2_per_dof,
            density_pull.map_or("-".to_string(), |p| format!("{p:.1}"))
        );
        if pull_f.abs() > PULL_LIMIT {
            failures.push(format!("{} {}: share pull {pull_f:.2}", at.dir, at.name));
        }
        if pull.abs() > ABSOLUTE_PULL_LIMIT {
            failures.push(format!("{} {}: absolute pull {pull:.2}", at.dir, at.name));
        }
    }
    let chi2_per_channel = chi2 / channels.len() as f64;
    eprintln!(
        "  share χ²/n over {} channels: {chi2_per_channel:.2}",
        channels.len()
    );
    assert!(failures.is_empty(), "{name}: {failures:?}");
    assert!(
        chi2_per_channel <= ROW_CHI2_LIMIT,
        "{name}: share χ²/n {chi2_per_channel:.2} over {} channels",
        channels.len()
    );
    worst_density
}

use common::PdfDataPath;

#[test]
fn ee_to_mumu_shares_match_madevents_channels() {
    let density = check_row("ee_mumu", ["100000", "6"]);
    assert!(
        density > 50.0,
        "the density share reads MadEvent's channels within {density:.1} standard errors: the \
         known-wrong comparison has no power here"
    );
}

#[test]
fn bhabha_shares_match_madevents_channels() {
    check_row("ee_ee", ["100000", "6"]);
}

#[test]
fn ee_to_uuxg_shares_match_madevents_channels() {
    check_row("ee_uuxg", ["100000", "6"]);
}

#[cfg(feature = "extended-validation")]
#[test]
fn pp_to_eej_shares_match_madevents_channels() {
    check_row("pp_eej", ["200000", "6"]);
}
