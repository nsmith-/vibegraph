//! Write one row's MadEvent seed record from the files its seeded runs kept.
//!
//! `validation/madgraph/bank_seeded_sigma.sh` runs a row once per seed, each in a
//! fresh process directory, and keeps per seed `s<iseed>/`:
//!
//! * `SubProcesses/results.dat`, MadEvent's combined result;
//! * `SubProcesses/P*/symfact.dat` and every `SubProcesses/P*/G*/results.dat` it
//!   names (a channel with a positive multiplicity);
//! * `banner.txt`, the run's banner, whose run card says which `iseed` and
//!   `nevents` the run made;
//! * `init.txt`, the event file's `<init>` block, where the event file had one;
//! * `vg_seed_result.txt`, `madevent_seeds.sh`'s line `value err wall_s card_sha`.
//!
//! A seed of a partitioned row keeps one such run per window, `s<iseed>/w<k>/`,
//! with the seed's own `vg_seed_result.txt` beside them.
//!
//! This tool reads those files, computes per seed the reported value and the
//! equal-weight recombination of each channel's iterations, and writes
//! `<kept>/seed_record.json` in the format `vibegraph-lib/tests/common/seed_record.rs`
//! defines and the σ gates read. It prints one line per seed, the seed-policy
//! readings, and the screen of the four-times-budget seeds against the budget's
//! mean.
//!
//! Usage:
//!
//! ```text
//! seed-record --row ROW --kept DIR --budget NEVENTS --generator TEXT \
//!             --tree TEXT --mg-version VERSION [--validated-ew]
//! ```

#[path = "../../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

#[path = "../../../vibegraph-lib/tests/common/seed_record.rs"]
mod seed_record;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use seed_record::{PartValues, SeedRecord, SeedRun, RECORD_FILE, SCHEMA};

/// How far the sum of the channels' printed values may sit from the printed
/// total, relative to the total. Both are printed to five significant digits, so
/// each carries a relative rounding of at most `5e-5`; twice that per side with
/// headroom.
const CHANNEL_SUM_TOLERANCE: f64 = 2e-4;

/// A Fortran-printed real: `0.10767E+02`, `1.2D-03`, or the exponent-without-letter
/// form `0.123-100` Fortran writes when the exponent needs three digits.
fn fortran_float(field: &str) -> Option<f64> {
    let s = field.trim().replace(['D', 'd'], "E");
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }
    let bytes = s.as_bytes();
    let split = (1..bytes.len())
        .rev()
        .find(|&i| (bytes[i] == b'+' || bytes[i] == b'-') && bytes[i - 1] != b'E')?;
    let mantissa: f64 = s[..split].parse().ok()?;
    let exponent: i32 = s[split..].parse().ok()?;
    Some(mantissa * 10f64.powi(exponent))
}

/// One integration channel's `results.dat`.
#[derive(Clone, Debug, PartialEq)]
struct Channel {
    /// The channel's combined value, the first field of its first line.
    value: f64,
    /// Its refine iterations, `(value, error)`: the second and third fields of
    /// every six-field line after the first.
    iterations: Vec<(f64, f64)>,
}

impl Channel {
    fn parse(text: &str) -> Result<Channel, String> {
        let mut lines = text.lines();
        let first = lines.next().ok_or("empty results.dat")?;
        let value = first
            .split_whitespace()
            .next()
            .and_then(fortran_float)
            .ok_or_else(|| format!("no value on the first line {first:?}"))?;
        let mut iterations = Vec::new();
        for line in lines {
            if line.contains('<') {
                break;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() != 6 {
                break;
            }
            let v = fortran_float(fields[1]).ok_or_else(|| format!("iteration line {line:?}"))?;
            let e = fortran_float(fields[2]).ok_or_else(|| format!("iteration line {line:?}"))?;
            iterations.push((v, e));
        }
        Ok(Channel { value, iterations })
    }

    /// The unweighted mean of the iterations and its error, `√Σeᵢ² / n`. A
    /// channel that recorded no iteration contributes its combined value with no
    /// error of its own.
    fn equal_weight(&self) -> (f64, f64) {
        if self.iterations.is_empty() {
            return (self.value, 0.0);
        }
        let n = self.iterations.len() as f64;
        let mean = self.iterations.iter().map(|(v, _)| v).sum::<f64>() / n;
        let err = self
            .iterations
            .iter()
            .map(|(_, e)| e * e)
            .sum::<f64>()
            .sqrt()
            / n;
        (mean, err)
    }
}

/// The value and error on the first line of the combined `results.dat`.
fn parse_total(text: &str) -> Result<(f64, f64), String> {
    let first = text.lines().next().ok_or("empty results.dat")?;
    let mut fields = first.split_whitespace().map(fortran_float);
    match (fields.next().flatten(), fields.next().flatten()) {
        (Some(v), Some(e)) => Ok((v, e)),
        _ => Err(format!("no value and error on {first:?}")),
    }
}

/// The channel tags `symfact.dat` gives a positive multiplicity: the `G<tag>`
/// directories MadEvent sums into its total (`get_Gdir`).
fn parse_symfact(text: &str) -> Result<Vec<String>, String> {
    let mut tags = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [tag, mfactor] = fields[..] else {
            return Err(format!("symfact.dat line {line:?}"));
        };
        let mfactor: i64 = mfactor
            .parse()
            .map_err(|_| format!("symfact.dat multiplicity {mfactor:?}"))?;
        if mfactor > 0 {
            tags.push(tag.to_string());
        }
    }
    Ok(tags)
}

/// A key of the banner's `<MGRunCard>` block, as written: `  value = key ! comment`.
fn banner_value(banner: &str, key: &str) -> Option<String> {
    let start = banner.find("<MGRunCard>")?;
    let end = banner[start..].find("</MGRunCard>")? + start;
    banner[start..end].lines().find_map(|line| {
        let line = line.split('!').next()?;
        let (value, name) = line.split_once('=')?;
        (name.trim() == key).then(|| value.trim().to_string())
    })
}

/// The `<init>` block's per-process cross sections, `LPRUP → (XSECUP, XERRUP)`.
fn parse_init(text: &str) -> Result<BTreeMap<String, (f64, f64)>, String> {
    let body = text
        .split("<init>")
        .nth(1)
        .and_then(|rest| rest.split("</init>").next())
        .ok_or("no <init> block")?;
    let mut lines = body.lines().filter(|l| !l.trim().is_empty());
    let beams = lines.next().ok_or("empty <init> block")?;
    let nprup: usize = beams
        .split_whitespace()
        .last()
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| format!("no NPRUP on {beams:?}"))?;
    let mut out = BTreeMap::new();
    for line in lines.take(nprup) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(format!("<init> process line {line:?}"));
        }
        let xsec = fortran_float(fields[0]).ok_or_else(|| format!("XSECUP on {line:?}"))?;
        let xerr = fortran_float(fields[1]).ok_or_else(|| format!("XERRUP on {line:?}"))?;
        out.insert(fields[3].to_string(), (xsec, xerr));
    }
    if out.len() != nprup {
        return Err(format!(
            "<init> declares {nprup} processes, lists {}",
            out.len()
        ));
    }
    Ok(out)
}

/// The process number of a `P<N>_...` subprocess directory: the proc card's `@N`.
fn subprocess_id(name: &str) -> Option<String> {
    let digits: String = name
        .strip_prefix('P')?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    (!digits.is_empty()).then_some(digits)
}

/// Everything one run (one process directory, one `generate_events`) kept.
#[derive(Debug)]
struct RunValues {
    iseed: u64,
    nevents: u64,
    reported: (f64, f64),
    equal_weight: (f64, f64),
    channels: usize,
    iterations: (usize, usize),
    /// Per process: the `<init>` value where the event file listed several, and
    /// the equal-weight sum over that process's channels.
    by_lprup: BTreeMap<String, PartValues>,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn sorted_dirs(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    Ok(out)
}

fn read_run(dir: &Path) -> Result<RunValues, String> {
    let reported = parse_total(&read(&dir.join("SubProcesses/results.dat"))?)?;
    let banner = read(&dir.join("banner.txt"))?;
    let card_int = |key: &str| -> Result<u64, String> {
        let v = banner_value(&banner, key)
            .ok_or_else(|| format!("{}: no {key} in the banner's run card", dir.display()))?;
        v.parse::<f64>()
            .map(|f| f as u64)
            .map_err(|_| format!("{}: {key} = {v:?}", dir.display()))
    };
    let iseed = card_int("iseed")?;
    let nevents = card_int("nevents")?;

    let mut sum = 0.0;
    let mut ew = 0.0;
    let mut ew_var = 0.0;
    let mut channels = 0usize;
    let mut iterations = (usize::MAX, 0usize);
    let mut parts: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for pdir in sorted_dirs(&dir.join("SubProcesses"))? {
        let symfact = pdir.join("symfact.dat");
        if !symfact.is_file() {
            continue;
        }
        let name = pdir.file_name().unwrap().to_string_lossy().into_owned();
        let id = subprocess_id(&name)
            .ok_or_else(|| format!("{}: not a P<N>_ directory", pdir.display()))?;
        for tag in parse_symfact(&read(&symfact)?)? {
            let path = pdir.join(format!("G{tag}")).join("results.dat");
            let channel =
                Channel::parse(&read(&path)?).map_err(|e| format!("{}: {e}", path.display()))?;
            let (m, e) = channel.equal_weight();
            sum += channel.value;
            ew += m;
            ew_var += e * e;
            let part = parts.entry(id.clone()).or_default();
            part.0 += m;
            part.1 += e * e;
            channels += 1;
            iterations.0 = iterations.0.min(channel.iterations.len());
            iterations.1 = iterations.1.max(channel.iterations.len());
        }
    }
    if channels == 0 {
        return Err(format!(
            "{}: no channel listed by any symfact.dat",
            dir.display()
        ));
    }
    let scale = reported.0.abs().max(f64::MIN_POSITIVE);
    if (sum - reported.0).abs() > CHANNEL_SUM_TOLERANCE * scale {
        return Err(format!(
            "{}: the channels sum to {sum:.6e}, results.dat reports {:.6e}: the kept channels are \
             not the ones MadEvent combined",
            dir.display(),
            reported.0
        ));
    }

    let mut by_lprup = BTreeMap::new();
    let init_path = dir.join("init.txt");
    if init_path.is_file() {
        let init = parse_init(&read(&init_path)?)?;
        if init.len() > 1 {
            let init_ids: BTreeSet<&String> = init.keys().collect();
            let channel_ids: BTreeSet<&String> = parts.keys().collect();
            if init_ids != channel_ids {
                return Err(format!(
                    "{}: <init> lists processes {init_ids:?}, the subprocess directories are \
                     {channel_ids:?}",
                    dir.display()
                ));
            }
            for (id, &(xsec, xerr)) in &init {
                let (m, v) = parts[id];
                by_lprup.insert(
                    id.clone(),
                    PartValues {
                        reported_pb: xsec,
                        reported_err_pb: xerr,
                        equal_weight_pb: m,
                        equal_weight_err_pb: v.sqrt(),
                    },
                );
            }
        }
    }

    Ok(RunValues {
        iseed,
        nevents,
        reported,
        equal_weight: (ew, ew_var.sqrt()),
        channels,
        iterations,
        by_lprup,
    })
}

/// The seed's kept directory: one run, or one run per `w<k>` window.
fn read_seed(row_dir: &Path, seed_dir: &Path) -> Result<SeedRun, String> {
    let name = seed_dir.file_name().unwrap().to_string_lossy().into_owned();
    let iseed: u64 = name
        .strip_prefix('s')
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("{}: not an s<iseed> directory", seed_dir.display()))?;
    let wall_s = read(&seed_dir.join("vg_seed_result.txt"))?
        .split_whitespace()
        .nth(2)
        .and_then(|w| w.parse::<f64>().ok())
        .ok_or_else(|| format!("{}: no wall time in vg_seed_result.txt", seed_dir.display()))?;

    let windows: Vec<PathBuf> = sorted_dirs(seed_dir)?
        .into_iter()
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .strip_prefix('w')
                .is_some_and(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    let runs: Vec<RunValues> = if windows.is_empty() {
        vec![read_run(seed_dir)?]
    } else {
        windows
            .iter()
            .map(|w| read_run(w))
            .collect::<Result<_, _>>()?
    };
    for run in &runs {
        if run.iseed != iseed {
            return Err(format!(
                "{}: the banner ran iseed {}, the directory is s{iseed}",
                seed_dir.display(),
                run.iseed
            ));
        }
        if run.nevents != runs[0].nevents {
            return Err(format!(
                "{}: windows ran different nevents",
                seed_dir.display()
            ));
        }
        if !windows.is_empty() && !run.by_lprup.is_empty() {
            return Err(format!(
                "{}: a partitioned seed with several processes is not supported",
                seed_dir.display()
            ));
        }
    }
    let quad =
        |f: &dyn Fn(&RunValues) -> f64| runs.iter().map(|r| f(r).powi(2)).sum::<f64>().sqrt();
    Ok(SeedRun {
        iseed,
        nevents: runs[0].nevents,
        reported_pb: runs.iter().map(|r| r.reported.0).sum(),
        reported_err_pb: quad(&|r| r.reported.1),
        equal_weight_pb: runs.iter().map(|r| r.equal_weight.0).sum(),
        equal_weight_err_pb: quad(&|r| r.equal_weight.1),
        wall_s,
        channels: runs.iter().map(|r| r.channels).sum(),
        iterations_min: runs.iter().map(|r| r.iterations.0).min().unwrap(),
        iterations_max: runs.iter().map(|r| r.iterations.1).max().unwrap(),
        files: seed_dir
            .strip_prefix(row_dir)
            .unwrap_or(seed_dir)
            .to_string_lossy()
            .into_owned(),
        windows: windows
            .iter()
            .map(|w| w.file_name().unwrap().to_string_lossy().into_owned())
            .collect(),
        by_lprup: runs[0].by_lprup.clone(),
    })
}

struct Args {
    row: String,
    kept: PathBuf,
    budget: u64,
    generator: String,
    tree: String,
    mg_version: String,
    validated_ew: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut map: BTreeMap<&str, &str> = BTreeMap::new();
    let mut validated_ew = false;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--validated-ew" => validated_ew = true,
            "--row" | "--kept" | "--budget" | "--generator" | "--tree" | "--mg-version" => {
                let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
                map.insert(flag.as_str(), value.as_str());
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    let get = |k: &str| {
        map.get(k)
            .map(|v| v.to_string())
            .ok_or_else(|| format!("{k} is required"))
    };
    Ok(Args {
        row: get("--row")?,
        kept: PathBuf::from(get("--kept")?),
        budget: get("--budget")?
            .parse()
            .map_err(|_| "--budget is an event count".to_string())?,
        generator: get("--generator")?,
        tree: get("--tree")?,
        mg_version: get("--mg-version")?,
        validated_ew,
    })
}

fn build_record(args: &Args) -> Result<SeedRecord, String> {
    let mut seeds = Vec::new();
    for dir in sorted_dirs(&args.kept)? {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with('s') && name[1..].chars().all(|c| c.is_ascii_digit()) {
            seeds.push(read_seed(&args.kept, &dir)?);
        }
    }
    seeds.sort_by_key(|s| (s.nevents, s.iseed));
    let mut record = SeedRecord {
        schema: SCHEMA,
        row: args.row.clone(),
        generator: args.generator.clone(),
        mg_version: args.mg_version.clone(),
        mg5_tree: args.tree.clone(),
        budget_nevents: args.budget,
        validated_ew: args.validated_ew,
        seeds,
        summary: None,
    };
    record.check()?;
    record.summary = record.summarise();
    Ok(record)
}

fn print_record(record: &SeedRecord) {
    let row = &record.row;
    for s in &record.seeds {
        println!(
            "[{row}] s{} nevents {}: reported {:.6e} +- {:.3e} | equal-weight {:.6e} +- {:.3e} \
             ({:+.3}%) | {} channels, {}-{} iterations | {:.0} s",
            s.iseed,
            s.nevents,
            s.reported_pb,
            s.reported_err_pb,
            s.equal_weight_pb,
            s.equal_weight_err_pb,
            100.0 * (s.equal_weight_pb / s.reported_pb - 1.0),
            s.channels,
            s.iterations_min,
            s.iterations_max,
            s.wall_s,
        );
    }
    let Some(summary) = &record.summary else {
        println!(
            "[{row}] fewer than two seeds at nevents {}: no seed-policy reading",
            record.budget_nevents
        );
        return;
    };
    for (label, m) in [
        ("reported", &summary.reported),
        ("equal-weight", &summary.equal_weight),
    ] {
        println!(
            "[{row}] {label} over {} seeds at nevents {}: {:.6e} +- {:.3e} (quoted {:.3e}, \
             spread {:.3e}, chi2/dof {:.2})",
            m.n,
            record.budget_nevents,
            m.mean_pb,
            m.err_pb,
            m.quoted_pb,
            m.spread_pb,
            m.chi2_per_dof
        );
    }
    println!(
        "[{row}] equal-weight - reported, paired: {:+.4e} +- {:.2e} ({:+.3}% +- {:.3}%)",
        summary.ew_minus_reported_pb,
        summary.ew_minus_reported_err_pb,
        100.0 * summary.ew_minus_reported_pb / summary.reported.mean_pb,
        100.0 * summary.ew_minus_reported_err_pb / summary.reported.mean_pb,
    );
    let screen = &summary.screen;
    println!(
        "SCREEN {row}: {} seed(s) at {}x against the {} mean of {}: pulls [{}]{} -> {}",
        screen.pulls.len(),
        seed_record::SCREEN_BUDGET_FACTOR,
        screen.estimator.label(),
        if screen.estimator == seed_record::Estimator::EqualWeight {
            summary.equal_weight.n
        } else {
            summary.reported.n
        },
        screen
            .pulls
            .iter()
            .map(|p| format!("{p:+.2}"))
            .collect::<Vec<_>>()
            .join(", "),
        screen
            .mean_pull
            .map(|p| format!(", their mean {p:+.2}"))
            .unwrap_or_default(),
        if screen.pulls.is_empty() {
            "no screening seeds"
        } else if screen.flagged {
            "FLAGGED: hold the row for a 20 + 20 budget ladder before banking it"
        } else {
            "within the screen"
        }
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("seed-record: {e}");
            return ExitCode::from(2);
        }
    };
    let record = match build_record(&args) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("seed-record [{}]: {e}", args.row);
            return ExitCode::FAILURE;
        }
    };
    print_record(&record);
    let path = args.kept.join(RECORD_FILE);
    let mut text = serde_json::to_string_pretty(&record).expect("record serialises");
    text.push('\n');
    if let Err(e) = std::fs::write(&path, text) {
        eprintln!("seed-record: {}: {e}", path.display());
        return ExitCode::FAILURE;
    }
    println!("[{}] wrote {}", record.row, path.display());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use seed_record::{Estimator, MIN_SEEDS};

    /// A pinned 3.7.1 run of `e+ e- > mu+ mu-` at the Z pole (iseed 7, 2000
    /// events): channel 1, and channel 2 split into `2.1`/`2.2`, plus the `Hel`
    /// helicity-filtering directory, which `symfact.dat` does not list.
    const TOTAL: &str = "0.20248E+04 0.18766E+01 0.00000E+01 49681 393 0 2867 0.00000E+01 0.00000E+01 0.20248E+04 0.00000E+01 0.00000E+01 0\n<lo_statistics>\n<cumulated_time> 4.54231 </cumulated_time>\n</lo_statistics>";
    const SYMFACT: &str = "            1            1\n2.1            1\n2.2            1\n";
    const G1: &str = " 0.10767E+02 0.28571E-01 0.00000E+00     7009      393    3      390 0.362E+02 0.26897E-03  0.10767E+02  0.27609E-01  0.27609E-01      390\n   1    0.10737E+02    0.10110E+00    0.29776E+00    0.16018E+02    0.10737E+02\n   2    0.10801E+02    0.58145E-01    0.24075E+00    0.17685E+02    0.10801E+02\n   3    0.10759E+02    0.34680E-01    0.20386E+00    0.13982E+02    0.10759E+02\n <lo_statistics> \n <cumulated_time>0.0743390000000000</cumulated_time>\n </lo_statistics>\n";
    const G21: &str = "0.20140E+04 0.18764E+01 0.00000E+01 42672 0 0 2476 0.12300E+01 0.00000E+01 0.20140E+04 0.00000E+01 0.00000E+01 0\n1 2016.1 8.2216 0.18383 2630.5 2016.1\n2 2011.7 6.2914 0.19937 2636.5 2011.7\n3 2014.7 4.4018 0.19698 2702.3 2014.7\n4 2016.9 8.4117 0.188 2650.2 2016.9\n5 2010.4 6.1375 0.19462 2647.4 2010.4\n6 2016.4 4.2325 0.18924 2658.4 2016.4\n7 2014.2 8.6477 0.19354 2597.9 2014.2\n8 2015.4 5.9873 0.18939 2634.6 2015.4\n9 2011.9 4.2749 0.19157 2639.3 2011.9\n<lo_statistics>\n<cumulated_time> 1.263923 </cumulated_time>\n</lo_statistics>";
    const G22: &str = " 0.00000E+00 0.00000E+00 0.00000E+00        0        0    0        1 0.000E+00 0.000E+00 0.000E+00 0.000E+00 0.000E+00        0\n   1    0.00000E+00    0.00000E+00    0.00000E+00    0.00000E+00    0.00000E+00\n <lo_statistics> \n <cumulated_time>2.3040479999999999</cumulated_time>\n </lo_statistics>\n";
    const HEL: &str = " 0.58341E-01 0.24468E-01 0.00000E+00       15        0    1        0 0.000E+00 0.29170E-04  0.58341E-01  0.28187E+01  0.00000E+00        0\n   1    0.58341E-01    0.24468E-01    0.10273E+01    0.84561E-01    0.58341E-01\n";
    const INIT_TWO: &str = "<init>\n2212 2212 6.500000e+03 6.500000e+03 0 0 247000 247000 -4 2\n5.132100e+02 4.896913e-01 1.085730e+03 0\n5.725205e+02 3.039602e+00 1.085730e+03 1\n<generator name='MadGraph5_aMC@NLO' version='3.7.1'>please cite 1405.0301 </generator>\n</init>\n";

    fn banner(iseed: u64, nevents: u64) -> String {
        format!(
            "<MGVersion>\n3.7.1\n</MGVersion>\n<MGRunCard>\n  {nevents}\t= nevents ! Number of unweighted events requested \n {iseed} = iseed ! rnd seed (0=assigned automatically=default))\n</MGRunCard>\n"
        )
    }

    /// A scratch directory unique to one test.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vg-seed-record-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// The pinned run's files under `dir`, as `bank_seeded_sigma.sh` keeps them.
    fn keep_run(dir: &Path, iseed: u64, nevents: u64) {
        write(&dir.join("SubProcesses/results.dat"), TOTAL);
        let p = dir.join("SubProcesses/P1_ll_ll");
        write(&p.join("symfact.dat"), SYMFACT);
        write(&p.join("G1/results.dat"), G1);
        write(&p.join("G2.1/results.dat"), G21);
        write(&p.join("G2.2/results.dat"), G22);
        write(&p.join("Hel/results.dat"), HEL);
        write(&dir.join("banner.txt"), &banner(iseed, nevents));
    }

    #[test]
    fn fortran_reals_in_every_spelling_madevent_writes() {
        assert_eq!(fortran_float("0.10767E+02"), Some(10.767));
        assert_eq!(fortran_float("1.5D-03"), Some(1.5e-3));
        assert_eq!(fortran_float("2016.1"), Some(2016.1));
        let three_digit = fortran_float("0.12345-100").unwrap();
        assert!((three_digit / 0.12345e-100 - 1.0).abs() < 1e-12);
        assert_eq!(fortran_float("-0.5E+01"), Some(-5.0));
        assert_eq!(fortran_float("abc"), None);
    }

    #[test]
    fn a_channel_reads_its_value_and_every_refine_iteration() {
        let c = Channel::parse(G1).unwrap();
        assert_eq!(c.value, 10.767);
        assert_eq!(
            c.iterations,
            vec![(10.737, 0.1011), (10.801, 0.058145), (10.759, 0.03468)]
        );
        let (m, e) = c.equal_weight();
        assert!((m - (10.737 + 10.801 + 10.759) / 3.0).abs() < 1e-12);
        let want = (0.1011f64.powi(2) + 0.058145f64.powi(2) + 0.03468f64.powi(2)).sqrt() / 3.0;
        assert!((e - want).abs() < 1e-15);
        // The plain-number layout the refine step writes parses the same way.
        assert_eq!(Channel::parse(G21).unwrap().iterations.len(), 9);
    }

    /// The equal-weight value is not the reported one: the reported channel value
    /// is the iterations' inverse-variance mean, and the test reproduces it from
    /// the iterations to pin which column is which.
    #[test]
    fn the_reported_channel_value_is_the_inverse_variance_mean_of_its_iterations() {
        let c = Channel::parse(G1).unwrap();
        let (num, den) = c.iterations.iter().fold((0.0, 0.0), |(n, d), (v, e)| {
            (n + v / (e * e), d + 1.0 / (e * e))
        });
        assert!((num / den - c.value).abs() < 5e-4);
        assert!((c.equal_weight().0 - c.value).abs() > 1e-3);
    }

    #[test]
    fn symfact_lists_the_channels_madevent_sums() {
        assert_eq!(parse_symfact(SYMFACT).unwrap(), vec!["1", "2.1", "2.2"]);
        assert_eq!(parse_symfact("1 1\n2 -1\n3 2\n").unwrap(), vec!["1", "3"]);
        assert!(parse_symfact("1\n").is_err());
    }

    #[test]
    fn the_banner_says_which_seed_and_budget_ran() {
        let b = banner(20261101, 10000);
        assert_eq!(banner_value(&b, "iseed").as_deref(), Some("20261101"));
        assert_eq!(banner_value(&b, "nevents").as_deref(), Some("10000"));
        assert_eq!(banner_value(&b, "ebeam1"), None);
    }

    #[test]
    fn the_init_block_gives_each_process_its_cross_section() {
        let init = parse_init(INIT_TWO).unwrap();
        assert_eq!(init["0"], (513.21, 0.4896913));
        assert_eq!(init["1"], (572.5205, 3.039602));
        assert_eq!(subprocess_id("P0_gg_ttx").as_deref(), Some("0"));
        assert_eq!(subprocess_id("P12_qq_ttxg").as_deref(), Some("12"));
        assert_eq!(subprocess_id("Hel"), None);
    }

    /// The pinned run read end to end: the channels `symfact.dat` lists sum to
    /// MadEvent's total, and the `Hel` directory is no channel.
    #[test]
    fn a_kept_run_sums_the_listed_channels_to_madevents_total() {
        let dir = scratch("run");
        keep_run(&dir, 7, 2000);
        let run = read_run(&dir).unwrap();
        assert_eq!((run.iseed, run.nevents), (7, 2000));
        assert_eq!(run.reported, (2024.8, 1.8766));
        assert_eq!(run.channels, 3);
        assert_eq!(run.iterations, (1, 9));
        let g21 = Channel::parse(G21).unwrap().equal_weight().0;
        let want = (10.737 + 10.801 + 10.759) / 3.0 + g21;
        assert!((run.equal_weight.0 - want).abs() < 1e-9);
        assert!(run.by_lprup.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A kept run missing one of its channels no longer sums to the total, and
    /// is refused rather than recorded low.
    #[test]
    fn a_run_missing_a_channel_is_refused() {
        let dir = scratch("missing");
        keep_run(&dir, 7, 2000);
        write(
            &dir.join("SubProcesses/P1_ll_ll/symfact.dat"),
            "            1            1\n2.2            1\n",
        );
        let err = read_run(&dir).unwrap_err();
        assert!(err.contains("not the ones MadEvent combined"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn args(kept: &Path, budget: u64) -> Args {
        Args {
            row: "ee_to_mumu".into(),
            kept: kept.to_path_buf(),
            budget,
            generator: "test".into(),
            tree: "pinned".into(),
            mg_version: "3.7.1".into(),
            validated_ew: false,
        }
    }

    #[test]
    fn a_record_carries_every_seed_and_screens_the_four_times_budget_ones() {
        let row = scratch("record");
        for iseed in 1..=5 {
            keep_run(&row.join(format!("s{iseed}")), iseed, 2000);
            write(
                &row.join(format!("s{iseed}/vg_seed_result.txt")),
                "2024.8 1.8766 11 abc\n",
            );
        }
        for iseed in [11, 12] {
            keep_run(&row.join(format!("s{iseed}")), iseed, 8000);
            write(
                &row.join(format!("s{iseed}/vg_seed_result.txt")),
                "2024.8 1.8766 30 abc\n",
            );
        }
        let record = build_record(&args(&row, 2000)).unwrap();
        assert_eq!(record.seeds.len(), 7);
        assert_eq!(record.at_budget().len(), 5);
        assert_eq!(record.screening().len(), 2);
        let summary = record.summary.as_ref().unwrap();
        // Identical seeds: no spread, so the policy's error is the quote over √n.
        assert_eq!(summary.reported.spread_pb, 0.0);
        assert!((summary.reported.err_pb - 1.8766 / 5f64.sqrt()).abs() < 1e-12);
        assert_eq!(summary.screen.pulls, vec![0.0, 0.0]);
        assert!(!summary.screen.flagged);

        // Round trip through the file the gates read.
        let text = serde_json::to_string_pretty(&record).unwrap();
        let back: SeedRecord = serde_json::from_str(&text).unwrap();
        assert_eq!(back, record);
        let reference = back.reference();
        assert_eq!(reference.estimator, Estimator::Reported);
        assert_eq!(reference.read.n, MIN_SEEDS);
        assert_eq!(reference.mean(), 2024.8);
        let _ = std::fs::remove_dir_all(&row);
    }

    #[test]
    fn a_partitioned_seed_sums_its_windows() {
        let row = scratch("windows");
        let seed = row.join("s3");
        keep_run(&seed.join("w1"), 3, 2000);
        keep_run(&seed.join("w2"), 3, 2000);
        write(&seed.join("vg_seed_result.txt"), "- - 40 abc\n");
        let s = read_seed(&row, &seed).unwrap();
        assert_eq!(s.windows, vec!["w1", "w2"]);
        assert_eq!(s.reported_pb, 2.0 * 2024.8);
        assert!((s.reported_err_pb - 2f64.sqrt() * 1.8766).abs() < 1e-12);
        assert_eq!(s.channels, 6);
        assert_eq!(s.wall_s, 40.0);

        // A window run under another seed is not this seed's.
        keep_run(&seed.join("w3"), 4, 2000);
        assert!(read_seed(&row, &seed).unwrap_err().contains("iseed 4"));
        let _ = std::fs::remove_dir_all(&row);
    }

    #[test]
    fn a_multi_process_run_splits_by_lprup() {
        let dir = scratch("lprup");
        keep_run(&dir, 7, 2000);
        // Two processes, @0 and @1, as the MLM rows have them.
        let p = dir.join("SubProcesses");
        std::fs::rename(p.join("P1_ll_ll"), p.join("P0_ll_ll")).unwrap();
        write(&p.join("P1_ll_llg/symfact.dat"), "1 1\n");
        write(&p.join("P1_ll_llg/G1/results.dat"), G22);
        write(&dir.join("init.txt"), INIT_TWO);
        let run = read_run(&dir).unwrap();
        assert_eq!(run.by_lprup.len(), 2);
        assert_eq!(run.by_lprup["0"].reported_pb, 513.21);
        assert!((run.by_lprup["0"].equal_weight_pb - run.equal_weight.0).abs() < 1e-9);
        assert_eq!(run.by_lprup["1"].equal_weight_pb, 0.0);

        // An <init> whose processes are not the subprocess directories' is refused.
        write(
            &dir.join("init.txt"),
            &INIT_TWO.replace(" 1\n<gen", " 2\n<gen"),
        );
        assert!(read_run(&dir).unwrap_err().contains("<init> lists"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn seed(iseed: u64, nevents: u64, reported: f64, ew: f64) -> SeedRun {
        SeedRun {
            iseed,
            nevents,
            reported_pb: reported,
            reported_err_pb: 1.0,
            equal_weight_pb: ew,
            equal_weight_err_pb: 1.0,
            wall_s: 1.0,
            channels: 1,
            iterations_min: 3,
            iterations_max: 3,
            files: format!("s{iseed}"),
            windows: Vec::new(),
            by_lprup: BTreeMap::new(),
        }
    }

    fn record(seeds: Vec<SeedRun>) -> SeedRecord {
        SeedRecord {
            schema: SCHEMA,
            row: "row".into(),
            generator: "test".into(),
            mg_version: "3.7.1".into(),
            mg5_tree: "pinned".into(),
            budget_nevents: 100,
            validated_ew: false,
            seeds,
            summary: None,
        }
    }

    /// Five seeds at the budget reading 100, 102, 98, 101, 99 reported and two
    /// points higher equal-weight, and two at four times the budget far off.
    fn five_and_two() -> SeedRecord {
        let base = [100.0, 102.0, 98.0, 101.0, 99.0];
        let mut seeds: Vec<SeedRun> = base
            .iter()
            .enumerate()
            .map(|(i, &v)| seed(i as u64 + 1, 100, v, v + 2.0))
            .collect();
        seeds.push(seed(11, 400, 150.0, 150.0));
        seeds.push(seed(12, 400, 151.0, 151.0));
        record(seeds)
    }

    #[test]
    fn a_gate_reads_the_budget_seeds_under_the_seed_policy() {
        let r = five_and_two();
        let read = r.reference();
        // The four-times-budget seeds are not averaged in.
        assert_eq!(read.mean(), 100.0);
        assert_eq!(read.read.n, 5);
        // spread/√n = √(10/4)/√5 = 0.7071 beats quoted √5/5 = 0.4472.
        assert!((read.err() - (2.5f64 / 5.0).sqrt()).abs() < 1e-12);
        assert_eq!(read.estimator, Estimator::Reported);
    }

    #[test]
    fn a_validated_row_reads_its_equal_weight_values() {
        let mut r = five_and_two();
        r.validated_ew = true;
        let read = r.reference();
        assert_eq!(read.estimator, Estimator::EqualWeight);
        assert_eq!(read.mean(), 102.0);
    }

    #[test]
    #[should_panic(expected = "the seed policy reads no fewer than 5")]
    fn fewer_than_five_budget_seeds_are_no_reference() {
        let mut r = five_and_two();
        r.seeds.remove(0);
        r.reference();
    }

    #[test]
    fn a_record_holding_a_seed_at_a_third_budget_or_twice_is_refused() {
        let mut r = five_and_two();
        r.seeds.push(seed(13, 200, 100.0, 100.0));
        assert!(r.check().unwrap_err().contains("neither the budget"));
        let mut r = five_and_two();
        r.seeds.push(seed(1, 100, 100.0, 100.0));
        assert!(r.check().unwrap_err().contains("distinct iseeds"));
        let mut r = five_and_two();
        r.schema = SCHEMA + 1;
        assert!(r.check().is_err());
    }

    #[test]
    fn the_screen_flags_four_times_budget_seeds_off_the_budget_mean() {
        let summary = five_and_two().summarise().unwrap();
        assert_eq!(summary.screen.pulls.len(), 2);
        assert!(summary.screen.mean_pull.unwrap() > 2.0);
        assert!(summary.screen.flagged);
        assert!((summary.ew_minus_reported_pb - 2.0).abs() < 1e-12);
        assert_eq!(summary.ew_minus_reported_err_pb, 0.0);

        // Screening seeds on the budget's mean pass.
        let mut r = five_and_two();
        for s in r.seeds.iter_mut().filter(|s| s.nevents == 400) {
            s.reported_pb = 100.0;
        }
        let summary = r.summarise().unwrap();
        assert!(!summary.screen.flagged);
        assert_eq!(summary.screen.mean_pull, Some(0.0));
    }

    #[test]
    fn one_process_is_read_from_its_own_share() {
        let mut r = five_and_two();
        for s in &mut r.seeds {
            s.by_lprup.insert(
                "1".into(),
                PartValues {
                    reported_pb: s.reported_pb / 2.0,
                    reported_err_pb: 0.5,
                    equal_weight_pb: s.equal_weight_pb / 2.0,
                    equal_weight_err_pb: 0.5,
                },
            );
        }
        assert_eq!(r.part_reference("1").mean(), 50.0);
        let missing = std::panic::catch_unwind(|| r.part_reference("2"));
        assert!(missing.is_err());
    }

    #[test]
    fn a_work_area_without_a_record_reads_none() {
        let dir = scratch("none");
        assert!(seed_record::read(&dir, "ee_to_mumu").is_none());
        let path = seed_record::record_path(&dir, "ee_to_mumu");
        write(&path, &serde_json::to_string(&five_and_two()).unwrap());
        // A record filed under another row's name is a broken bundle.
        let wrong = std::panic::catch_unwind(|| seed_record::read(&dir, "ee_to_mumu"));
        assert!(wrong.is_err());
        let mut r = five_and_two();
        r.row = "ee_to_mumu".into();
        write(&path, &serde_json::to_string(&r).unwrap());
        assert_eq!(
            seed_record::reference(&dir, "ee_to_mumu").unwrap().mean(),
            100.0
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
