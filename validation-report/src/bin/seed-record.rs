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
//! * `Source/run_config.inc`, as the run left it: the survey mode's
//!   `min_events`/`max_events`;
//! * `events_written.txt`, the number of events the run's event file holds, as
//!   `madevent_seeds.sh`'s `mes_check_run` counted them;
//! * `vg_seed_result.txt`, `madevent_seeds.sh`'s line `value err wall_s card_sha`.
//!
//! A seed of a partitioned row keeps one such run per window, `s<iseed>/w<k>/`,
//! with the seed's own `vg_seed_result.txt` beside them.
//!
//! This tool reads those files, computes per seed the reported value and the
//! equal-weight recombination of each channel's iterations, and writes
//! `<kept>/seed_record.json` in the format `vibegraph-lib/tests/common/seed_record.rs`
//! defines and the σ gates read. It prints one line per seed, the seed-policy
//! readings, the screen of the four-times-budget seeds against the budget's
//! mean, the budget ladder's verdict (with 20 seeds on each rung), the survey
//! the runs made and the row's per-channel scatter test.
//!
//! `--rescreen` reads a record this tool wrote (at any version of the same
//! schema), checks it, and prints the same lines with the summary computed
//! afresh from its seeds. It writes nothing.
//!
//! A seed is refused, and no record written, when its run wrote fewer events than
//! it ran for, reports no positive finite cross section, or ran another
//! `hard_survey` than `--hard-survey`.
//!
//! Usage:
//!
//! ```text
//! seed-record --row ROW --kept DIR --budget NEVENTS --generator TEXT \
//!             --tree TEXT --mg-version VERSION --hard-survey N [--validated-ew]
//! seed-record --rescreen PATH/seed_record.json
//! ```

#[path = "../../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

#[path = "../../../vibegraph-lib/tests/common/seed_record.rs"]
mod seed_record;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use seed_record::{ChannelValues, PartValues, SeedRecord, SeedRun, RECORD_FILE, SCHEMA};

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
    /// The channel's combined value and error, the first two fields of its
    /// first line.
    value: f64,
    error: f64,
    /// Its refine iterations, `(value, error)`: the second and third fields of
    /// every six-field line after the first.
    iterations: Vec<(f64, f64)>,
}

impl Channel {
    fn parse(text: &str) -> Result<Channel, String> {
        let (value, error) = value_and_error(text)?;
        let mut iterations = Vec::new();
        for line in text.lines().skip(1) {
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
        Ok(Channel {
            value,
            error,
            iterations,
        })
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

/// The value and error on the first line of a `results.dat`: the combined
/// one's total, or one channel's.
fn value_and_error(text: &str) -> Result<(f64, f64), String> {
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

/// `min_events` and `max_events` of a `run_config.inc`: its
/// `parameter (min_events = N)` lines.
fn parse_run_config(text: &str) -> Result<(u64, u64), String> {
    let get = |key: &str| -> Result<u64, String> {
        let found: Vec<u64> = text
            .lines()
            .filter_map(|line| {
                let line = line.split('!').next()?;
                let inner = line.trim().strip_prefix("parameter")?.trim();
                let inner = inner.strip_prefix('(')?.split(')').next()?;
                let (name, value) = inner.split_once('=')?;
                (name.trim().eq_ignore_ascii_case(key)).then(|| value.trim().parse().ok())?
            })
            .collect();
        match found[..] {
            [v] => Ok(v),
            _ => Err(format!("run_config.inc sets {key} {} times", found.len())),
        }
    };
    Ok((get("min_events")?, get("max_events")?))
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
    hard_survey: u32,
    run_config: (u64, u64),
    events_written: u64,
    by_channel: BTreeMap<String, ChannelValues>,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The subdirectories of `dir`, sorted. An entry that cannot be read is an
/// error: skipping it would drop a seed or a channel without a word.
fn sorted_dirs(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// The last component of a path the walk itself produced.
fn dir_name(path: &Path) -> String {
    path.file_name()
        .expect("a directory listed by read_dir has a name")
        .to_string_lossy()
        .into_owned()
}

/// The digits of a `<prefix><digits>` name: `s<iseed>` for a seed, `w<k>` for a
/// window.
fn numbered(name: &str, prefix: char) -> Option<&str> {
    name.strip_prefix(prefix)
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// Every channel `symfact.dat` lists under a run's `SubProcesses/P*`, read from
/// its `G<tag>/results.dat`.
struct Channels {
    by_channel: BTreeMap<String, ChannelValues>,
    /// The channels' `results.dat` values, summed.
    sum: f64,
    /// The equal-weight values summed, and their variances summed.
    equal_weight: f64,
    equal_weight_var: f64,
    count: usize,
    /// The fewest and most refine iterations any channel recorded.
    iterations: (usize, usize),
    /// Per process (`P<N>_*`): its channels' equal-weight sum and variance.
    parts: BTreeMap<String, (f64, f64)>,
}

fn read_channels(dir: &Path) -> Result<Channels, String> {
    let mut c = Channels {
        by_channel: BTreeMap::new(),
        sum: 0.0,
        equal_weight: 0.0,
        equal_weight_var: 0.0,
        count: 0,
        iterations: (usize::MAX, 0),
        parts: BTreeMap::new(),
    };
    for pdir in sorted_dirs(&dir.join("SubProcesses"))? {
        let symfact = pdir.join("symfact.dat");
        if !symfact.is_file() {
            continue;
        }
        let name = dir_name(&pdir);
        let id = subprocess_id(&name)
            .ok_or_else(|| format!("{}: not a P<N>_ directory", pdir.display()))?;
        for tag in parse_symfact(&read(&symfact)?)? {
            let path = pdir.join(format!("G{tag}")).join("results.dat");
            let channel =
                Channel::parse(&read(&path)?).map_err(|e| format!("{}: {e}", path.display()))?;
            let (m, e) = channel.equal_weight();
            c.by_channel.insert(
                format!("{name}/G{tag}"),
                ChannelValues {
                    value_pb: channel.value,
                    err_pb: channel.error,
                    equal_weight_pb: m,
                    iterations: channel.iterations.len(),
                },
            );
            c.sum += channel.value;
            c.equal_weight += m;
            c.equal_weight_var += e * e;
            let part = c.parts.entry(id.clone()).or_default();
            part.0 += m;
            part.1 += e * e;
            c.count += 1;
            c.iterations.0 = c.iterations.0.min(channel.iterations.len());
            c.iterations.1 = c.iterations.1.max(channel.iterations.len());
        }
    }
    if c.count == 0 {
        return Err(format!(
            "{}: no channel listed by any symfact.dat",
            dir.display()
        ));
    }
    Ok(c)
}

/// Per process, where the run's `init.txt` lists several: the `<init>` cross
/// section as reported, and the equal-weight sum over that process's channels.
/// Empty for a single-process run or one that kept no `<init>` block.
fn split_by_lprup(
    dir: &Path,
    parts: &BTreeMap<String, (f64, f64)>,
) -> Result<BTreeMap<String, PartValues>, String> {
    let mut by_lprup = BTreeMap::new();
    let init_path = dir.join("init.txt");
    if !init_path.is_file() {
        return Ok(by_lprup);
    }
    let init = parse_init(&read(&init_path)?)?;
    if init.len() <= 1 {
        return Ok(by_lprup);
    }
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
    Ok(by_lprup)
}

fn read_run(dir: &Path) -> Result<RunValues, String> {
    let total_path = dir.join("SubProcesses/results.dat");
    let reported = value_and_error(&read(&total_path)?)
        .map_err(|e| format!("{}: {e}", total_path.display()))?;
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
    let hard_survey = u32::try_from(card_int("hard_survey")?)
        .map_err(|_| format!("{}: hard_survey out of range", dir.display()))?;
    let run_config_path = dir.join("Source/run_config.inc");
    let run_config = parse_run_config(&read(&run_config_path)?)
        .map_err(|e| format!("{}: {e}", run_config_path.display()))?;
    let events_path = dir.join("events_written.txt");
    let events_written: u64 = read(&events_path)?
        .trim()
        .parse()
        .map_err(|_| format!("{}: not an event count", events_path.display()))?;
    if !(reported.0.is_finite() && reported.0 > 0.0 && reported.1.is_finite()) {
        return Err(format!(
            "{}: results.dat reports {} +- {}, not a positive finite cross section",
            dir.display(),
            reported.0,
            reported.1
        ));
    }
    if events_written < nevents {
        return Err(format!(
            "{}: the event file holds {events_written} events, the run card asked for {nevents}",
            dir.display()
        ));
    }
    let channels = read_channels(dir)?;
    let scale = reported.0.abs().max(f64::MIN_POSITIVE);
    if (channels.sum - reported.0).abs() > CHANNEL_SUM_TOLERANCE * scale {
        return Err(format!(
            "{}: the channels sum to {:.6e}, results.dat reports {:.6e}: the kept channels are \
             not the ones MadEvent combined",
            dir.display(),
            channels.sum,
            reported.0
        ));
    }
    let by_lprup = split_by_lprup(dir, &channels.parts)?;

    Ok(RunValues {
        iseed,
        nevents,
        reported,
        equal_weight: (channels.equal_weight, channels.equal_weight_var.sqrt()),
        channels: channels.count,
        iterations: channels.iterations,
        by_lprup,
        hard_survey,
        run_config,
        events_written,
        by_channel: channels.by_channel,
    })
}

/// The seed's kept directory: one run, or one run per `w<k>` window.
fn read_seed(row_dir: &Path, seed_dir: &Path) -> Result<SeedRun, String> {
    let iseed: u64 = numbered(&dir_name(seed_dir), 's')
        .and_then(|digits| digits.parse().ok())
        .ok_or_else(|| format!("{}: not an s<iseed> directory", seed_dir.display()))?;
    let wall_s = read(&seed_dir.join("vg_seed_result.txt"))?
        .split_whitespace()
        .nth(2)
        .and_then(|w| w.parse::<f64>().ok())
        .ok_or_else(|| format!("{}: no wall time in vg_seed_result.txt", seed_dir.display()))?;

    let windows: Vec<PathBuf> = sorted_dirs(seed_dir)?
        .into_iter()
        .filter(|p| numbered(&dir_name(p), 'w').is_some())
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
        if run.hard_survey != runs[0].hard_survey || run.run_config != runs[0].run_config {
            return Err(format!(
                "{}: windows ran different surveys",
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
        windows: windows.iter().map(|w| dir_name(w)).collect(),
        by_lprup: runs[0].by_lprup.clone(),
        hard_survey: runs[0].hard_survey,
        min_events: runs[0].run_config.0,
        max_events: runs[0].run_config.1,
        events_written: runs.iter().map(|r| r.events_written).min().unwrap(),
        by_channel: if windows.is_empty() {
            runs[0].by_channel.clone()
        } else {
            windows
                .iter()
                .zip(&runs)
                .flat_map(|(w, r)| {
                    let w = dir_name(w);
                    r.by_channel
                        .iter()
                        .map(move |(k, v)| (format!("{w}/{k}"), *v))
                })
                .collect()
        },
    })
}

struct Args {
    row: String,
    kept: PathBuf,
    budget: u64,
    generator: String,
    tree: String,
    mg_version: String,
    hard_survey: u32,
    validated_ew: bool,
}

/// `Some` when the arguments ask to rescreen an existing record: its path, or
/// why the arguments do not name one.
fn rescreen_path(args: &[String]) -> Option<Result<PathBuf, String>> {
    if args.first().map(String::as_str) != Some("--rescreen") {
        return None;
    }
    Some(match &args[1..] {
        [path] => Ok(PathBuf::from(path)),
        _ => Err("--rescreen takes one seed_record.json and nothing else".to_string()),
    })
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut map: BTreeMap<&str, &str> = BTreeMap::new();
    let mut validated_ew = false;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--validated-ew" => validated_ew = true,
            "--row" | "--kept" | "--budget" | "--generator" | "--tree" | "--mg-version"
            | "--hard-survey" => {
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
        hard_survey: get("--hard-survey")?
            .parse()
            .map_err(|_| "--hard-survey is a run-card integer".to_string())?,
        validated_ew,
    })
}

fn build_record(args: &Args) -> Result<SeedRecord, String> {
    let mut seeds = Vec::new();
    for dir in sorted_dirs(&args.kept)? {
        if numbered(&dir_name(&dir), 's').is_some() {
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
        hard_survey: args.hard_survey,
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
    for s in &record.seeds {
        println!(
            "[{row}] s{} survey: hard_survey {}, run_config.inc min_events {} max_events {}, \
             {} events written",
            s.iseed, s.hard_survey, s.min_events, s.max_events, s.events_written
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
    println!("{}", ladder_line(row, &record.ladder()));
    print_survey(row, &summary.survey);
    println!("{}", scatter_line(row, summary.scatter.as_ref()));
}

fn print_survey(row: &str, survey: &seed_record::SurveySummary) {
    let configs = survey
        .run_config
        .iter()
        .map(|c| {
            format!(
                "{}/{} ({})",
                c.min_events,
                c.max_events,
                c.rate
                    .map(|r| format!("rate {r}"))
                    .unwrap_or_else(|| "no survey mode's".into())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "SURVEY {row}: hard_survey {} on every card; run_config.inc min/max_events {configs} -> {}",
        survey.hard_survey,
        if survey.as_set {
            "as set"
        } else {
            "INERT: the survey ran at another rate than hard_survey asks (mmjj > 0.01 (ebeam1 + \
             ebeam2) forces rate 1); list the row back"
        }
    );
}

/// The `LADDER` line: the budget ladder's rung means, the combined standard
/// error of their measured spreads, the separation in those and the verdict.
fn ladder_line(row: &str, ladder: &seed_record::Ladder) -> String {
    let (Some((m1, s1)), Some((m4, s4)), Some(sigma), Some(pull)) =
        (ladder.budget, ladder.screening, ladder.sigma, ladder.pull)
    else {
        return format!(
            "LADDER {row}: not run, it needs {} seeds at 1x and {} at {}x (this record has {} and {})",
            seed_record::LADDER_SEEDS,
            seed_record::LADDER_SEEDS,
            seed_record::SCREEN_BUDGET_FACTOR,
            ladder.n_budget,
            ladder.n_screening,
        );
    };
    let readings = format!(
        "{} seeds at 1x mean {m1:.6e} (sd {s1:.3e}), {} at {}x mean {m4:.6e} (sd {s4:.3e}): \
         sigma {sigma:.3e}, difference {pull:+.2} sigma",
        ladder.n_budget,
        ladder.n_screening,
        seed_record::SCREEN_BUDGET_FACTOR,
    );
    match ladder.verdict {
        seed_record::LadderVerdict::NotRun => format!(
            "LADDER {row}: not run, it needs {} seeds at 1x and {} at {}x; readings only: {readings}",
            seed_record::LADDER_SEEDS,
            seed_record::LADDER_SEEDS,
            seed_record::SCREEN_BUDGET_FACTOR,
        ),
        seed_record::LadderVerdict::Rebank => format!(
            "LADDER {row}: {readings} -> re-bank at {}x (beyond {} sigma)",
            seed_record::SCREEN_BUDGET_FACTOR,
            seed_record::LADDER_SIGMA_LIMIT,
        ),
        seed_record::LadderVerdict::Stays => format!(
            "LADDER {row}: {readings} -> stays at 1x (within {} sigma)",
            seed_record::LADDER_SIGMA_LIMIT,
        ),
    }
}

/// A p-value to three digits; `chi2_p_value` reads 0 only where the tail
/// underflows `f64`, below the smallest normal number.
fn p_text(p: f64) -> String {
    if p == 0.0 {
        "< 1e-300".to_string()
    } else {
        format!("{p:.2e}")
    }
}

/// The screen's `SCATTER` line: the look-elsewhere p-value, the channels
/// tested, the verdict, and the informational count above χ²/dof 3.
fn scatter_line(row: &str, scatter: Option<&seed_record::Scatter>) -> String {
    let Some((h, worst)) = scatter.and_then(|h| Some((h, h.worst.as_ref()?))) else {
        return format!("SCATTER {row}: no channel testable");
    };
    let above = h
        .above_info_limit
        .iter()
        .map(|c| {
            format!(
                "{} {:.2} ({:.1}% of sigma)",
                c.channel,
                c.chi2_per_dof,
                100.0 * c.share
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "SCATTER {row}: look-elsewhere p {} over {} channels tested (worst {} at seed chi2/dof \
         {:.2} on {} dof, p {}, {:.1}% of sigma) -> {}; for information: {} above chi2/dof {} \
         [{}], {:.2} expected by chance",
        p_text(h.corrected_p),
        h.channels_tested,
        worst.channel,
        worst.chi2_per_dof,
        h.dof,
        p_text(worst.p_value),
        100.0 * worst.share,
        match h.verdict {
            seed_record::ScatterVerdict::WithinChance => "within chance".to_string(),
            seed_record::ScatterVerdict::Flagged => format!(
                "FLAGGED: confirm with {} seeds",
                seed_record::SCATTER_CONFIRM_SEEDS
            ),
            seed_record::ScatterVerdict::Confirmed => "CONFIRMED".to_string(),
        },
        h.above_info_limit.len(),
        seed_record::SCATTER_INFO_CHI2_PER_DOF,
        above,
        h.expected_above_by_chance,
    )
}

/// An existing record, checked, with its summary computed afresh from its
/// seeds by this tool's rules.
fn rescreen(path: &Path) -> Result<SeedRecord, String> {
    let text = read(path)?;
    let mut record: SeedRecord = serde_json::from_str(&text)
        .map_err(|e| format!("{} does not parse: {e}", path.display()))?;
    record.check()?;
    record.summary = record.summarise();
    Ok(record)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(path) = rescreen_path(&args) {
        return match path.and_then(|p| rescreen(&p)) {
            Ok(record) => {
                print_record(&record);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("seed-record --rescreen: {e}");
                ExitCode::FAILURE
            }
        };
    }
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
    use seed_record::{
        chi2_p_value, Estimator, ScatterVerdict, MIN_SEEDS, SCATTER_INFO_CHI2_PER_DOF,
        SCATTER_P_LIMIT,
    };

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
            "<MGVersion>\n3.7.1\n</MGVersion>\n<MGRunCard>\n  {nevents}\t= nevents ! Number of unweighted events requested \n {iseed} = iseed ! rnd seed (0=assigned automatically=default))\n  2 = hard_survey\n</MGRunCard>\n"
        )
    }

    /// A scratch directory, removed when the test ends however it ends.
    fn scratch() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("vg-seed-record-")
            .tempdir()
            .unwrap()
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// The message of the panic `f` raises; fails when it does not panic.
    fn panic_message(f: impl FnOnce()) -> String {
        let payload =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).expect_err("the call panics");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .expect("a printable panic payload")
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
        write(&dir.join("Source/run_config.inc"), RUN_CONFIG_HS2);
        write(&dir.join("events_written.txt"), &format!("{nevents}\n"));
    }

    /// The lines of a pinned 3.7.1 run's `Source/run_config.inc` after a
    /// `hard_survey = 2` survey rewrote them.
    const RUN_CONFIG_HS2: &str = "      integer    min_events                 !Minimum number of events/iteration\n      parameter (min_events = 10000 )         !to refine a channel\n      integer    max_events                 !Maximum number of events/iteration\n      parameter (max_events = 30000 )        !to refine a channel\n      integer    MaxEventsPerJob            !Maximum number of events requested from a single job\n      parameter (MaxEventsPerJob=1000)\n";

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
        let err = parse_symfact("1\n").unwrap_err();
        assert!(err.contains("symfact.dat line"), "{err}");
        let err = parse_symfact("1 x\n").unwrap_err();
        assert!(err.contains("symfact.dat multiplicity"), "{err}");
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
        let tmp = scratch();
        let dir = tmp.path();
        keep_run(dir, 7, 2000);
        let run = read_run(dir).unwrap();
        assert_eq!((run.iseed, run.nevents), (7, 2000));
        assert_eq!(run.reported, (2024.8, 1.8766));
        assert_eq!(run.channels, 3);
        assert_eq!(run.iterations, (1, 9));
        let g21 = Channel::parse(G21).unwrap().equal_weight().0;
        let want = (10.737 + 10.801 + 10.759) / 3.0 + g21;
        assert!((run.equal_weight.0 - want).abs() < 1e-9);
        assert!(run.by_lprup.is_empty());
    }

    /// A kept run missing one of its channels no longer sums to the total, and
    /// is refused rather than recorded low.
    #[test]
    fn a_run_missing_a_channel_is_refused() {
        let tmp = scratch();
        let dir = tmp.path();
        keep_run(dir, 7, 2000);
        write(
            &dir.join("SubProcesses/P1_ll_ll/symfact.dat"),
            "            1            1\n2.2            1\n",
        );
        let err = read_run(dir).unwrap_err();
        assert!(err.contains("not the ones MadEvent combined"), "{err}");
        // One listing no channel at all says so.
        write(
            &dir.join("SubProcesses/P1_ll_ll/symfact.dat"),
            "            1           -1\n",
        );
        let err = read_run(dir).unwrap_err();
        assert!(
            err.contains("no channel listed by any symfact.dat"),
            "{err}"
        );
    }

    fn args(kept: &Path, budget: u64) -> Args {
        Args {
            row: "ee_to_mumu".into(),
            kept: kept.to_path_buf(),
            budget,
            generator: "test".into(),
            tree: "pinned".into(),
            mg_version: "3.7.1".into(),
            hard_survey: 2,
            validated_ew: false,
        }
    }

    #[test]
    fn a_record_carries_every_seed_and_screens_the_four_times_budget_ones() {
        let tmp = scratch();
        let row = tmp.path();
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
        let record = build_record(&args(row, 2000)).unwrap();
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
    }

    #[test]
    fn a_partitioned_seed_sums_its_windows() {
        let tmp = scratch();
        let row = tmp.path();
        let seed = row.join("s3");
        keep_run(&seed.join("w1"), 3, 2000);
        keep_run(&seed.join("w2"), 3, 2000);
        write(&seed.join("vg_seed_result.txt"), "- - 40 abc\n");
        let s = read_seed(row, &seed).unwrap();
        assert_eq!(s.windows, vec!["w1", "w2"]);
        assert_eq!(s.reported_pb, 2.0 * 2024.8);
        assert!((s.reported_err_pb - 2f64.sqrt() * 1.8766).abs() < 1e-12);
        assert_eq!(s.channels, 6);
        assert_eq!(s.wall_s, 40.0);

        // A window run under another seed is not this seed's.
        keep_run(&seed.join("w3"), 4, 2000);
        assert!(read_seed(row, &seed).unwrap_err().contains("iseed 4"));
    }

    /// The pinned run as two processes, `@0` and `@1`, as the MLM rows have them.
    fn keep_two_process_run(dir: &Path, iseed: u64, nevents: u64) {
        keep_run(dir, iseed, nevents);
        let p = dir.join("SubProcesses");
        std::fs::rename(p.join("P1_ll_ll"), p.join("P0_ll_ll")).unwrap();
        write(&p.join("P1_ll_llg/symfact.dat"), "1 1\n");
        write(&p.join("P1_ll_llg/G1/results.dat"), G22);
        write(&dir.join("init.txt"), INIT_TWO);
    }

    #[test]
    fn a_multi_process_run_splits_by_lprup() {
        let tmp = scratch();
        let dir = tmp.path();
        keep_two_process_run(dir, 7, 2000);
        let run = read_run(dir).unwrap();
        assert_eq!(run.by_lprup.len(), 2);
        assert_eq!(run.by_lprup["0"].reported_pb, 513.21);
        assert!((run.by_lprup["0"].equal_weight_pb - run.equal_weight.0).abs() < 1e-9);
        assert_eq!(run.by_lprup["1"].equal_weight_pb, 0.0);

        // An <init> whose processes are not the subprocess directories' is refused.
        write(
            &dir.join("init.txt"),
            &INIT_TWO.replace(" 1\n<gen", " 2\n<gen"),
        );
        assert!(read_run(dir).unwrap_err().contains("<init> lists"));
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
            hard_survey: 2,
            min_events: 10000,
            max_events: 30000,
            events_written: nevents,
            by_channel: BTreeMap::new(),
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
            hard_survey: 2,
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
        let err = r.check().unwrap_err();
        assert!(err.contains(&format!("schema {}", SCHEMA + 1)), "{err}");
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
        let missing = panic_message(|| {
            r.part_reference("2");
        });
        assert!(missing.contains("records no process 2"), "{missing}");
    }

    #[test]
    fn a_work_area_without_a_record_reads_none() {
        let tmp = scratch();
        let dir = tmp.path();
        assert!(seed_record::read(dir, "ee_to_mumu").is_none());
        let path = seed_record::record_path(dir, "ee_to_mumu");
        write(&path, &serde_json::to_string(&five_and_two()).unwrap());
        // A record filed under another row's name is a broken bundle.
        let wrong = panic_message(|| {
            seed_record::read(dir, "ee_to_mumu");
        });
        assert!(wrong.contains("records row row"), "{wrong}");
        let mut r = five_and_two();
        r.row = "ee_to_mumu".into();
        write(&path, &serde_json::to_string(&r).unwrap());
        assert_eq!(
            seed_record::reference(dir, "ee_to_mumu").unwrap().mean(),
            100.0
        );
        // So is one that fails its check, and the panic names the file.
        r.seeds[2].events_written = 99;
        write(&path, &serde_json::to_string(&r).unwrap());
        let broken = panic_message(|| {
            seed_record::read(dir, "ee_to_mumu");
        });
        assert!(
            broken.contains(&format!("{}: [ee_to_mumu] seed 3 wrote 99", path.display())),
            "{broken}"
        );
    }

    #[test]
    fn run_config_gives_the_survey_mode_the_run_made() {
        assert_eq!(parse_run_config(RUN_CONFIG_HS2).unwrap(), (10000, 30000));
        let template = RUN_CONFIG_HS2
            .replace("10000 )", "1000)")
            .replace("30000 )", "2000)");
        assert_eq!(parse_run_config(&template).unwrap(), (1000, 2000));
        let err = parse_run_config("      parameter (min_events = 10)\n").unwrap_err();
        assert!(err.contains("sets max_events 0 times"), "{err}");
        // MadEvent's pass_in_difficult_integration_mode: 2500 (2 + r), 10000 (1 + r).
        assert_eq!(seed_record::survey_rate(1000, 2000), Some(0));
        assert_eq!(seed_record::survey_rate(7500, 20000), Some(1));
        assert_eq!(seed_record::survey_rate(10000, 30000), Some(2));
        assert_eq!(seed_record::survey_rate(10000, 20000), None);
        assert_eq!(seed_record::survey_rate(5000, 10000), None);
    }

    #[test]
    fn a_survey_at_another_rate_than_the_card_asks_reads_inert() {
        let r = five_and_two();
        let s = r.survey();
        assert!(s.as_set);
        assert_eq!(s.run_config.len(), 1);
        assert_eq!(s.run_config[0].rate, Some(2));
        // The mmjj condition: rate 1 whatever hard_survey says, on one seed.
        let mut r = five_and_two();
        r.seeds[6].min_events = 7500;
        r.seeds[6].max_events = 20000;
        let s = r.survey();
        assert!(!s.as_set);
        assert_eq!(s.run_config.len(), 2);
        assert_eq!(s.run_config[0].rate, Some(1));
    }

    /// The forced failure: a kept run whose event file holds fewer events than its
    /// card asked for is refused, as is a run with no positive finite σ.
    #[test]
    fn a_short_event_file_or_a_non_numeric_sigma_is_refused() {
        let tmp = scratch();
        let dir = tmp.path();
        keep_run(dir, 7, 2000);
        assert!(read_run(dir).is_ok());
        write(&dir.join("events_written.txt"), "1999\n");
        let err = read_run(dir).unwrap_err();
        assert!(err.contains("holds 1999 events"), "{err}");
        write(&dir.join("events_written.txt"), "2000\n");
        for total in ["NaN 0.1 0 0\n", "0.00000E+00 0.0 0 0\n", "-0.1E+01 0.1 0\n"] {
            write(&dir.join("SubProcesses/results.dat"), total);
            let err = read_run(dir).unwrap_err();
            assert!(err.contains("not a positive finite"), "{total}: {err}");
        }
        std::fs::remove_file(dir.join("SubProcesses/results.dat")).unwrap();
        let err = read_run(dir).unwrap_err();
        assert!(err.contains("SubProcesses/results.dat: "), "{err}");
    }

    #[test]
    fn a_record_refuses_a_short_seed_and_another_hard_survey() {
        let mut r = five_and_two();
        r.seeds[2].events_written = 99;
        assert!(r.check().unwrap_err().contains("wrote 99 events"));
        let mut r = five_and_two();
        r.seeds[0].hard_survey = 0;
        assert!(r.check().unwrap_err().contains("hard_survey = 0"));
        let mut r = five_and_two();
        r.seeds[0].reported_pb = 0.0;
        assert!(r.check().unwrap_err().contains("not a positive"));
    }

    /// A NaN or an infinity in any of a seed's four values is refused: the
    /// positive-σ refusal cannot stand in, since `NaN <= 0` is false.
    #[test]
    fn a_record_refuses_a_non_finite_value() {
        type Field = fn(&mut SeedRun) -> &mut f64;
        let fields: [Field; 4] = [
            |s| &mut s.reported_pb,
            |s| &mut s.reported_err_pb,
            |s| &mut s.equal_weight_pb,
            |s| &mut s.equal_weight_err_pb,
        ];
        for (i, field) in fields.iter().enumerate() {
            for bad in [f64::NAN, f64::INFINITY] {
                let mut r = five_and_two();
                *field(&mut r.seeds[1]) = bad;
                let err = r.check().unwrap_err();
                assert!(
                    err.contains("seed 2 has a non-finite value"),
                    "field {i}, {bad}: {err}"
                );
            }
        }
    }

    /// `reference` checks the record itself, so a record built in memory is held
    /// to what a record read from the bundle is.
    #[test]
    fn a_record_that_fails_its_check_is_no_reference() {
        let mut r = five_and_two();
        r.seeds[2].events_written = 99;
        let err = panic_message(|| {
            r.reference();
        });
        assert!(err.contains("seed 3 wrote 99 events"), "{err}");
    }

    #[test]
    fn a_kept_run_records_every_channel_and_its_survey() {
        let tmp = scratch();
        let dir = tmp.path();
        keep_run(dir, 7, 2000);
        let run = read_run(dir).unwrap();
        assert_eq!(run.hard_survey, 2);
        assert_eq!(run.run_config, (10000, 30000));
        assert_eq!(run.events_written, 2000);
        let keys: Vec<&str> = run.by_channel.keys().map(String::as_str).collect();
        assert_eq!(keys, ["P1_ll_ll/G1", "P1_ll_ll/G2.1", "P1_ll_ll/G2.2"]);
        let g1 = run.by_channel["P1_ll_ll/G1"];
        assert_eq!(
            (g1.value_pb, g1.err_pb, g1.iterations),
            (10.767, 0.028571, 3)
        );
    }

    fn with_channels(mut s: SeedRun, channels: &[(&str, f64, f64)]) -> SeedRun {
        for &(name, value, err) in channels {
            s.by_channel.insert(
                name.into(),
                ChannelValues {
                    value_pb: value,
                    err_pb: err,
                    equal_weight_pb: value,
                    iterations: 3,
                },
            );
        }
        s
    }

    /// Two channels over five seeds: one scattering within its quote, one far
    /// outside it. The informational count names the second; a zero channel is
    /// not tested.
    #[test]
    fn the_informational_count_names_the_channel_past_chi2_per_dof_3() {
        let mut r = five_and_two();
        let calm = [50.0, 51.0, 49.0, 50.5, 49.5];
        let wild = [50.0, 58.0, 42.0, 51.0, 49.0];
        for (i, s) in r.seeds.iter_mut().enumerate() {
            let k = i.min(4);
            *s = with_channels(
                s.clone(),
                &[
                    ("P1_x/G1", calm[k], 1.0),
                    ("P1_x/G2", wild[k], 1.0),
                    ("P1_x/G3", 0.0, 0.0),
                ],
            );
        }
        let h = r.summarise().unwrap().scatter.unwrap();
        assert_eq!(h.channels_tested, 2);
        assert_eq!(h.dof, 4);
        let worst = h.worst.as_ref().unwrap();
        assert_eq!(worst.channel, "P1_x/G2");
        // Σ(v − 50)² = 0 + 64 + 64 + 1 + 1 = 130, over 4 dof.
        assert!((worst.chi2 - 130.0).abs() < 1e-10);
        assert!((worst.chi2_per_dof - 32.5).abs() < 1e-12);
        assert!((worst.share - 0.5).abs() < 1e-12);
        assert_eq!(h.above_info_limit.len(), 1);
        // P(χ²₄ > 12) = e⁻⁶ (1 + 6) per channel tested.
        let chance = 7.0 * (-6.0f64).exp();
        assert!((h.expected_above_by_chance - 2.0 * chance).abs() < 1e-9);

        // Within its quote, nothing is counted.
        for s in &mut r.seeds {
            let c = s.by_channel.get_mut("P1_x/G2").unwrap();
            c.err_pb = 10.0;
        }
        let h = r.summarise().unwrap().scatter.unwrap();
        assert!(h.above_info_limit.is_empty());
        assert!(h.worst.unwrap().chi2_per_dof <= SCATTER_INFO_CHI2_PER_DOF);
    }

    /// `n` seeds at the budget, each reading 100 pb; channel `name` reads
    /// `50 + deviations[i]` ± `err` in seed `i`.
    fn scatter_record(n: usize, channels: &[(String, Vec<f64>, f64)]) -> SeedRecord {
        let seeds = (0..n)
            .map(|i| {
                let values: Vec<(&str, f64, f64)> = channels
                    .iter()
                    .map(|(name, d, err)| (name.as_str(), 50.0 + d[i], *err))
                    .collect();
                with_channels(seed(i as u64 + 1, 100, 100.0, 100.0), &values)
            })
            .collect();
        record(seeds)
    }

    /// `count` channels `P1_x/G<k>`, each scattering by `deviations` with a
    /// unit quote, the one at `odd` by `odd_deviations` instead.
    fn channel_set(
        count: usize,
        deviations: &[f64],
        odd: usize,
        odd_deviations: &[f64],
    ) -> Vec<(String, Vec<f64>, f64)> {
        (1..=count)
            .map(|k| {
                let d = if k == odd { odd_deviations } else { deviations };
                (format!("P1_x/G{k}"), d.to_vec(), 1.0)
            })
            .collect()
    }

    /// Deviations about 50 giving χ² = 40 on 4 dof with a unit quote.
    const FAR: [f64; 5] = [4.0, -4.0, 2.0, -2.0, 0.0];

    /// Relative agreement asked of the upper incomplete gamma function against
    /// a closed form at the values these tests take.
    const P_TOLERANCE: f64 = 1e-9;

    fn scatter_of(r: &SeedRecord) -> seed_record::Scatter {
        r.summarise().unwrap().scatter.unwrap()
    }

    #[test]
    fn the_look_elsewhere_p_value_is_the_worst_tail_times_the_channels_tested() {
        // Q(2, 6) = e⁻⁶ (1 + 6) ≈ 0.0174; Q(1, 1) = e⁻¹.
        let q = 7.0 * (-6.0f64).exp();
        assert!((chi2_p_value(12.0, 4) / q - 1.0).abs() < P_TOLERANCE);
        assert!((chi2_p_value(2.0, 2) / (-1.0f64).exp() - 1.0).abs() < P_TOLERANCE);

        // Six channels over five seeds; G4 scatters by 3, −1, −1, −1, 0 about
        // 50 on a unit quote, χ² = 12 on 4 dof; the others do not move (p = 1).
        // Six times 0.0174 is within chance.
        let r = scatter_record(
            5,
            &channel_set(6, &[0.0; 5], 4, &[3.0, -1.0, -1.0, -1.0, 0.0]),
        );
        let h = scatter_of(&r);
        assert_eq!((h.channels_tested, h.dof), (6, 4));
        let worst = h.worst.as_ref().unwrap();
        assert_eq!(worst.channel, "P1_x/G4");
        assert!((worst.chi2 - 12.0).abs() < 1e-12);
        assert!((worst.p_value / q - 1.0).abs() < P_TOLERANCE);
        assert!((h.corrected_p / (6.0 * q) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(h.verdict, ScatterVerdict::WithinChance);
    }

    /// Fifty channels over five seeds, each at χ² = 4 on 4 dof (p = 3e⁻² ≈ 0.41)
    /// but one at χ² = 14 (p = 8e⁻⁷ ≈ 0.0073): about the smallest of fifty
    /// p-values from channels whose quotes are right. It is past χ²/dof 3 and
    /// below 1 % on its own, and within chance once the fifty are counted.
    #[test]
    fn many_channels_quoting_their_errors_do_not_flag() {
        let calm = [1.0, -1.0, 1.0, -1.0, 0.0];
        let r = scatter_record(5, &channel_set(50, &calm, 37, &[3.0, -2.0, -1.0, 0.0, 0.0]));
        let h = scatter_of(&r);
        assert_eq!(h.channels_tested, 50);
        let worst = h.worst.as_ref().unwrap();
        assert_eq!(worst.channel, "P1_x/G37");
        let p = 8.0 * (-7.0f64).exp();
        assert!((worst.p_value / p - 1.0).abs() < P_TOLERANCE);
        assert!(worst.p_value < SCATTER_P_LIMIT);
        assert!((h.corrected_p / (50.0 * p) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(h.verdict, ScatterVerdict::WithinChance);
        assert_eq!(h.above_info_limit.len(), 1);
    }

    /// The same fifty channels with G37 at 4, −4, 2, −2, 0: χ² = 40 on 4 dof.
    #[test]
    fn one_channel_far_outside_its_quote_flags() {
        let calm = [1.0, -1.0, 1.0, -1.0, 0.0];
        let r = scatter_record(5, &channel_set(50, &calm, 37, &FAR));
        let h = scatter_of(&r);
        let worst = h.worst.as_ref().unwrap();
        assert_eq!(worst.channel, "P1_x/G37");
        // Q(2, 20) = e⁻²⁰ (1 + 20) ≈ 4.3e-8.
        let p = 21.0 * (-20.0f64).exp();
        assert!((worst.p_value / p - 1.0).abs() < P_TOLERANCE);
        assert!((h.corrected_p / (50.0 * p) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(h.verdict, ScatterVerdict::Flagged);
    }

    /// Ten channels alternating ±1 about 50 over twenty seeds (χ² = 20 on 19
    /// dof), one of them ±3 (χ² = 180): confirmed on twenty seeds, a flag still
    /// to confirm on nineteen, and nothing on the ten calm ones alone.
    #[test]
    fn twenty_seeds_confirm_a_flag() {
        let alternate = |a: f64| {
            (0..20)
                .map(|i| if i % 2 == 0 { a } else { -a })
                .collect::<Vec<_>>()
        };
        let channels = channel_set(10, &alternate(1.0), 7, &alternate(3.0));
        let h = scatter_of(&scatter_record(20, &channels));
        assert_eq!(h.dof, 19);
        assert_eq!(h.worst.as_ref().unwrap().channel, "P1_x/G7");
        assert!((h.worst.as_ref().unwrap().chi2 - 180.0).abs() < 1e-9);
        assert_eq!(h.verdict, ScatterVerdict::Confirmed);

        let h = scatter_of(&scatter_record(19, &channels));
        assert_eq!(h.dof, 18);
        assert!(h.corrected_p < SCATTER_P_LIMIT);
        assert_eq!(h.verdict, ScatterVerdict::Flagged);

        let calm = channel_set(10, &alternate(1.0), 0, &[]);
        let h = scatter_of(&scatter_record(20, &calm));
        assert!(h.corrected_p > 0.5, "{}", h.corrected_p);
        assert_eq!(h.verdict, ScatterVerdict::WithinChance);
    }

    /// One channel over five seeds at 3, −1, −1, −1, 0 about 50, its quote set
    /// so χ² = 13 (p = 7.5 e⁻⁶·⁵ ≈ 0.0113) and then 13.5 (p = 7.75 e⁻⁶·⁷⁵ ≈
    /// 0.0091), either side of the 1 % limit.
    #[test]
    fn the_flag_falls_below_one_percent_not_above() {
        let devs = vec![3.0, -1.0, -1.0, -1.0, 0.0];
        let at = |chi2: f64| {
            let r = scatter_record(5, &[("P1_x/G1".into(), devs.clone(), (12.0 / chi2).sqrt())]);
            scatter_of(&r)
        };
        let above = at(13.0);
        assert!((above.corrected_p / (7.5 * (-6.5f64).exp()) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(above.verdict, ScatterVerdict::WithinChance);
        let below = at(13.5);
        assert!((below.corrected_p / (7.75 * (-6.75f64).exp()) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(below.verdict, ScatterVerdict::Flagged);
    }

    #[test]
    fn the_scatter_line_states_the_corrected_p_the_channels_and_the_verdict() {
        let calm = [1.0, -1.0, 1.0, -1.0, 0.0];
        let line = |r: &SeedRecord| scatter_line("row", Some(&scatter_of(r)));

        let flagged = line(&scatter_record(5, &channel_set(50, &calm, 37, &FAR)));
        assert_eq!(
            flagged,
            "SCATTER row: look-elsewhere p 2.16e-6 over 50 channels tested (worst P1_x/G37 at \
             seed chi2/dof 10.00 on 4 dof, p 4.33e-8, 50.0% of sigma) -> FLAGGED: confirm with \
             20 seeds; for information: 1 above chi2/dof 3 [P1_x/G37 10.00 (50.0% of sigma)], \
             0.87 expected by chance"
        );

        let within = line(&scatter_record(5, &channel_set(6, &[0.0; 5], 4, &calm)));
        assert!(
            within.contains("look-elsewhere p 1.00e0 over 6 channels"),
            "{within}"
        );
        assert!(within.contains("-> within chance;"), "{within}");

        let alternate = |a: f64| {
            (0..20)
                .map(|i| if i % 2 == 0 { a } else { -a })
                .collect::<Vec<_>>()
        };
        let confirmed = line(&scatter_record(
            20,
            &channel_set(3, &alternate(1.0), 2, &alternate(9.0)),
        ));
        assert!(
            confirmed.contains("look-elsewhere p < 1e-300 over 3 channels"),
            "{confirmed}"
        );
        assert!(confirmed.contains("-> CONFIRMED;"), "{confirmed}");

        assert_eq!(
            scatter_line("row", None),
            "SCATTER row: no channel testable"
        );
    }

    /// The `summary` a schema-2 writer before the scatter test wrote: a
    /// `heavy_tail` block and no `scatter`.
    const SUMMARY_HEAVY_TAIL: &str = r#"{
        "channels_tested": 6, "dof": 4,
        "worst": {"channel": "P1_x/G4", "chi2_per_dof": 10.0, "share": 0.5},
        "above_limit": [{"channel": "P1_x/G4", "chi2_per_dof": 10.0, "share": 0.5}],
        "expected_above_by_chance": 0.1, "flagged": true
    }"#;

    #[test]
    fn a_record_written_with_the_heavy_tail_summary_reads_and_rescreens() {
        let mut r = scatter_record(5, &channel_set(6, &[0.0; 5], 4, &FAR));
        r.row = "ee_to_mumu".into();
        r.summary = r.summarise();
        let mut json = serde_json::to_value(&r).unwrap();
        let summary = json["summary"].as_object_mut().unwrap();
        assert!(summary.remove("scatter").is_some());
        summary.insert(
            "heavy_tail".into(),
            serde_json::from_str(SUMMARY_HEAVY_TAIL).unwrap(),
        );

        let tmp = scratch();
        let path = seed_record::record_path(tmp.path(), "ee_to_mumu");
        write(&path, &serde_json::to_string_pretty(&json).unwrap());
        let read = seed_record::read(tmp.path(), "ee_to_mumu").unwrap();
        assert_eq!(read.summary.as_ref().unwrap().scatter, None);
        assert_eq!(read.reference().mean(), 100.0);

        let rescreened = rescreen(&path).unwrap();
        let h = rescreened.summary.unwrap().scatter.unwrap();
        assert_eq!(h.worst.unwrap().channel, "P1_x/G4");
        assert!((h.corrected_p / (6.0 * 21.0 * (-20.0f64).exp()) - 1.0).abs() < P_TOLERANCE);
        assert_eq!(h.verdict, ScatterVerdict::Flagged);
        assert_eq!(rescreened.seeds, r.seeds);

        // A record that fails its check is refused, not rescreened.
        json["seeds"][1]["events_written"] = 7.into();
        write(&path, &serde_json::to_string_pretty(&json).unwrap());
        let err = rescreen(&path).unwrap_err();
        assert!(err.contains("seed 2 wrote 7 events"), "{err}");
    }

    #[test]
    fn a_partitioned_seed_names_each_windows_channels() {
        let tmp = scratch();
        let row = tmp.path();
        let seed = row.join("s3");
        keep_run(&seed.join("w1"), 3, 2000);
        keep_run(&seed.join("w2"), 3, 2000);
        write(&seed.join("w2/events_written.txt"), "1990\n");
        write(&seed.join("vg_seed_result.txt"), "- - 40 abc\n");
        assert!(read_seed(row, &seed).unwrap_err().contains("holds 1990"));
        write(&seed.join("w2/events_written.txt"), "2000\n");
        let s = read_seed(row, &seed).unwrap();
        assert_eq!(s.by_channel.len(), 6);
        assert!(s.by_channel.contains_key("w2/P1_ll_ll/G2.1"));
        assert_eq!(
            (s.min_events, s.max_events, s.events_written),
            (10000, 30000, 2000)
        );
        write(
            &seed.join("w2/Source/run_config.inc"),
            &RUN_CONFIG_HS2
                .replace("10000 )", "7500)")
                .replace("30000 )", "20000)"),
        );
        assert!(read_seed(row, &seed)
            .unwrap_err()
            .contains("different surveys"));
    }

    /// A partitioned seed's windows are one seed: one budget, one process each.
    #[test]
    fn a_partitioned_seed_refuses_windows_at_other_budgets_or_with_several_processes() {
        let tmp = scratch();
        let row = tmp.path();
        let seed = row.join("s3");
        keep_run(&seed.join("w1"), 3, 2000);
        keep_run(&seed.join("w2"), 3, 4000);
        write(&seed.join("vg_seed_result.txt"), "- - 40 abc\n");
        let err = read_seed(row, &seed).unwrap_err();
        assert!(err.contains("windows ran different nevents"), "{err}");

        let seed = row.join("s4");
        keep_two_process_run(&seed.join("w1"), 4, 2000);
        keep_two_process_run(&seed.join("w2"), 4, 2000);
        write(&seed.join("vg_seed_result.txt"), "- - 40 abc\n");
        let err = read_seed(row, &seed).unwrap_err();
        assert!(
            err.contains("a partitioned seed with several processes is not supported"),
            "{err}"
        );
    }

    #[test]
    fn the_arguments_refuse_what_they_cannot_read() {
        let full: Vec<String> = [
            "--row",
            "r",
            "--kept",
            "k",
            "--budget",
            "2000",
            "--generator",
            "g",
            "--tree",
            "t",
            "--mg-version",
            "3.7.1",
            "--hard-survey",
            "2",
        ]
        .map(String::from)
        .to_vec();
        let a = parse_args(&full).unwrap();
        assert_eq!((a.budget, a.hard_survey, a.validated_ew), (2000, 2, false));
        let mut ew = full.clone();
        ew.push("--validated-ew".into());
        assert!(parse_args(&ew).unwrap().validated_ew);

        let refused = |args: &[String]| parse_args(args).err().expect("refused");
        let with = |i: usize, value: &str| {
            let mut args = full.clone();
            args[i] = value.into();
            refused(&args)
        };
        let err = with(5, "2k");
        assert!(err.contains("--budget is an event count"), "{err}");
        let err = with(13, "two");
        assert!(err.contains("--hard-survey is a run-card integer"), "{err}");
        let err = with(0, "--rows");
        assert!(err.contains("unknown argument \"--rows\""), "{err}");
        let err = refused(&full[2..]);
        assert!(err.contains("--row is required"), "{err}");
        let err = refused(&full[..13]);
        assert!(err.contains("--hard-survey needs a value"), "{err}");

        assert!(rescreen_path(&full).is_none());
        let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            rescreen_path(&words(&["--rescreen", "r/seed_record.json"])),
            Some(Ok(PathBuf::from("r/seed_record.json")))
        );
        for bad in [&["--rescreen"][..], &["--rescreen", "a", "b"]] {
            let err = rescreen_path(&words(bad)).unwrap().unwrap_err();
            assert!(
                err.contains("--rescreen takes one seed_record.json"),
                "{err}"
            );
        }
    }

    /// The screen reads the four-times-budget seeds' own mean against the
    /// budget's mean (100 ± √0.5 here): two seeds each within two standard errors
    /// whose mean is not are flagged, and one far seed its partner offsets is
    /// not.
    #[test]
    fn the_screen_reads_the_screening_seeds_mean_not_any_one_seed() {
        let mut r = five_and_two();
        for s in r.seeds.iter_mut().filter(|s| s.nevents == 400) {
            s.reported_pb = 102.2;
        }
        let screen = r.summarise().unwrap().screen;
        // Each: 2.2 / √(1 + 0.5) = 1.80. Their mean: 102.2 ± 1/√2, so
        // 2.2 / √(0.5 + 0.5) = 2.2.
        for p in &screen.pulls {
            assert!((p - 2.2 / 1.5f64.sqrt()).abs() < 1e-12, "{p}");
        }
        assert!((screen.mean_pull.unwrap() - 2.2).abs() < 1e-12);
        assert!(screen.flagged);

        let mut r = five_and_two();
        let mut screening = r.seeds.iter_mut().filter(|s| s.nevents == 400);
        screening.next().unwrap().reported_pb = 103.0;
        screening.next().unwrap().reported_pb = 97.6;
        let screen = r.summarise().unwrap().screen;
        // The first alone: 3 / √1.5 = 2.45. Their mean: 100.3 ± 2.7 (the
        // spread, 3.82/√2), so 0.3 / √(2.7² + 0.5) = 0.11.
        assert!(screen.pulls[0] > 2.0);
        let want = 0.3 / (2.7f64.powi(2) + 0.5).sqrt();
        assert!((screen.mean_pull.unwrap() - want).abs() < 1e-9);
        assert!(!screen.flagged);
    }

    /// Equal weight minus reported, paired per seed: its mean and the standard
    /// error of that mean.
    #[test]
    fn the_paired_shift_carries_the_standard_error_of_its_mean() {
        let mut r = five_and_two();
        for (s, d) in r.seeds.iter_mut().zip([1.0, 3.0, 2.0, 2.0, 2.0]) {
            s.equal_weight_pb = s.reported_pb + d;
        }
        let summary = r.summarise().unwrap();
        assert!((summary.ew_minus_reported_pb - 2.0).abs() < 1e-12);
        // Σ(d − 2)² = 2, over n − 1 = 4, over n = 5.
        assert!((summary.ew_minus_reported_err_pb - 0.1f64.sqrt()).abs() < 1e-12);
    }

    /// A record with `n1` seeds at the budget alternating ±1 about 100 and `n4`
    /// at four times it alternating ±`amp4` about `centre4` (even counts, so
    /// each rung's mean is its centre). Every seed quotes a huge error, which
    /// the ladder must not read.
    fn ladder_record_with(n1: usize, n4: usize, amp4: f64, centre4: f64) -> SeedRecord {
        let rung = |count: usize, nevents: u64, centre: f64, amp: f64, first: u64| {
            (0..count)
                .map(|i| {
                    let sign = if i % 2 == 0 { amp } else { -amp };
                    let mut s = seed(first + i as u64, nevents, centre + sign, centre + sign);
                    s.reported_err_pb = 1e6;
                    s
                })
                .collect::<Vec<SeedRun>>()
        };
        let mut seeds = rung(n1, 100, 100.0, 1.0, 1);
        seeds.extend(rung(n4, 400, centre4, amp4, 1000));
        record(seeds)
    }

    /// Both rungs at ±1.
    fn ladder_record(n1: usize, n4: usize, centre4: f64) -> SeedRecord {
        ladder_record_with(n1, n4, 1.0, centre4)
    }

    /// Twenty seeds a side at ±1: s² = 20/19 per rung, so
    /// σ = √(2 · (20/19) / 20) = √(2/19), whatever the quotes say.
    #[test]
    fn the_ladder_decides_on_the_measured_spread_either_side_of_two_sigma() {
        let sigma = (2.0f64 / 19.0).sqrt();
        let stays = ladder_record(20, 20, 100.0 + 1.9 * sigma).ladder();
        assert!((stays.sigma.unwrap() - sigma).abs() < 1e-12);
        assert!((stays.pull.unwrap() - 1.9).abs() < 1e-9);
        assert_eq!(stays.verdict, seed_record::LadderVerdict::Stays);

        let rebank = ladder_record(20, 20, 100.0 + 2.1 * sigma).ladder();
        assert_eq!(rebank.verdict, seed_record::LadderVerdict::Rebank);
        let down = ladder_record(20, 20, 100.0 - 2.1 * sigma).ladder();
        assert_eq!(down.verdict, seed_record::LadderVerdict::Rebank);
        assert!((down.pull.unwrap() + 2.1).abs() < 1e-9);
    }

    /// The ladder decides only with twenty seeds on each rung; the readings
    /// are still given below that.
    #[test]
    fn the_ladder_is_not_run_below_twenty_and_twenty() {
        for (n1, n4) in [(19, 20), (20, 19), (5, 2)] {
            let l = ladder_record(n1, n4, 110.0).ladder();
            assert_eq!(l.verdict, seed_record::LadderVerdict::NotRun, "{n1} + {n4}");
            assert_eq!((l.n_budget, l.n_screening), (n1, n4));
        }
        let line = ladder_line("row", &ladder_record(5, 2, 110.0).ladder());
        assert!(
            line.starts_with("LADDER row: not run, it needs 20 seeds at 1x and 20 at 4x"),
            "{line}"
        );
        let none = ladder_record(5, 0, 110.0).ladder();
        assert_eq!(none.sigma, None);
        assert_eq!(
            ladder_line("row", &none),
            "LADDER row: not run, it needs 20 seeds at 1x and 20 at 4x (this record has 5 and 0)"
        );
    }

    #[test]
    fn the_ladder_line_states_the_readings_and_the_verdict() {
        let sigma = (2.0f64 / 19.0).sqrt();
        let line = |c: f64| ladder_line("row", &ladder_record(20, 20, c).ladder());
        let stays = line(100.0 + 0.5 * sigma);
        assert!(
            stays.contains("20 seeds at 1x mean 1.000000e2 (sd 1.026e0)"),
            "{stays}"
        );
        assert!(
            stays.contains("difference +0.50 sigma -> stays at 1x"),
            "{stays}"
        );
        let rebank = line(100.0 + 3.0 * sigma);
        assert!(
            rebank.contains("difference +3.00 sigma -> re-bank at 4x"),
            "{rebank}"
        );
    }

    /// Rungs of different size and spread: 20 seeds at ±1 and 30 at ±3 give
    /// σ² = (20/19)/20 + 9 (30/29)/30 = 1/19 + 9/29. Swapping which rung's
    /// variance divides by which count, or reading one rung's spread for both,
    /// moves σ and, at 2.1 σ or 1.9 σ, the verdict.
    #[test]
    fn the_ladder_weights_each_rungs_spread_by_its_own_count() {
        let sigma = (1.0f64 / 19.0 + 9.0 / 29.0).sqrt();
        let swapped = (20.0f64 / 19.0 / 30.0 + 9.0 * 30.0 / 29.0 / 20.0).sqrt();
        let one_spread = (20.0f64 / 19.0 * (1.0 / 20.0 + 1.0 / 30.0)).sqrt();
        assert!(2.1 * sigma / swapped < 2.0 && 1.9 * sigma / one_spread > 2.0);
        let l = ladder_record_with(20, 30, 3.0, 100.0 + 2.1 * sigma).ladder();
        assert!((l.sigma.unwrap() - sigma).abs() < 1e-12, "{l:?}");
        assert!((l.pull.unwrap() - 2.1).abs() < 1e-9, "{l:?}");
        let (m1, s1) = l.budget.unwrap();
        let (m4, s4) = l.screening.unwrap();
        assert!((m1 - 100.0).abs() < 1e-12 && (m4 - (100.0 + 2.1 * sigma)).abs() < 1e-12);
        assert!((s1 * s1 - 20.0 / 19.0).abs() < 1e-12);
        assert!((s4 * s4 - 9.0 * 30.0 / 29.0).abs() < 1e-12);
        assert_eq!(l.verdict, seed_record::LadderVerdict::Rebank);
        let inside = ladder_record_with(20, 30, 3.0, 100.0 + 1.9 * sigma).ladder();
        assert_eq!(inside.verdict, seed_record::LadderVerdict::Stays);
    }

    /// The line names each rung's own mean, spread and count.
    #[test]
    fn the_ladder_line_keeps_the_two_rungs_apart() {
        let line = ladder_line("row", &ladder_record_with(20, 30, 3.0, 101.0).ladder());
        assert!(
            line.contains("20 seeds at 1x mean 1.000000e2 (sd 1.026e0), 30 at 4x mean 1.010000e2 (sd 3.051e0)"),
            "{line}"
        );
    }

    /// Two channels whose tails both underflow `f64` tie at p = 0; the worst
    /// is the one with the larger χ², not the first by name.
    #[test]
    fn a_tie_at_zero_p_names_the_channel_with_the_larger_chi_squared() {
        let alternate = |a: f64| {
            (0..20)
                .map(|i| if i % 2 == 0 { a } else { -a })
                .collect::<Vec<_>>()
        };
        let channels = vec![
            ("P1_x/G1".to_string(), alternate(9.0), 1.0),
            ("P1_x/G2".to_string(), alternate(20.0), 1.0),
            ("P1_x/G3".to_string(), alternate(12.0), 1.0),
        ];
        let h = scatter_of(&scatter_record(20, &channels));
        let worst = h.worst.as_ref().unwrap();
        assert_eq!(worst.p_value, 0.0);
        assert_eq!(worst.channel, "P1_x/G2");
        assert!((worst.chi2 - 8000.0).abs() < 1e-6);
        assert_eq!(h.verdict, ScatterVerdict::Confirmed);
        assert_eq!(h.above_info_limit[0].channel, "P1_x/G2");
    }

    /// Q(2, 65) = 66 e⁻⁶⁵ ≈ 3.9e-27, which a series truncated at a fixed
    /// number of terms reads as 0.
    #[test]
    fn a_deep_tail_p_value_is_not_truncated_to_zero() {
        let q = 66.0 * (-65.0f64).exp();
        let p = chi2_p_value(130.0, 4);
        assert!(
            p > 0.0 && (p / q - 1.0).abs() < P_TOLERANCE,
            "p = {p:e}, want {q:e}"
        );
        assert_eq!(chi2_p_value(0.0, 4), 1.0);
        assert_eq!(chi2_p_value(5.0, 0), 1.0);
        assert_eq!(p_text(p), format!("{p:.2e}"));
        assert_eq!(p_text(0.0), "< 1e-300");
    }
}
