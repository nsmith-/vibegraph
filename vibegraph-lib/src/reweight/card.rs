//! MadGraph's `reweight_card.dat`, parsed into data.
//!
//! The card is a list of `launch` blocks, each followed by the parameter changes
//! that define one alternative hypothesis:
//!
//! ```text
//! launch --rwgt_name=ymt_150
//!   set yukawa 6 150.0
//! launch
//!   set ymt 180.0
//!   set param_card mass 25 130.0
//! ```
//!
//! `set` takes either an LHA address (`<block> <code…> <value>`, the optional
//! `param_card` keyword first) or a parameter name (`<name> <value>`). What the
//! address names is resolved against a model later ([`super::resolve`]); this layer
//! only reads the card.
//!
//! Of MadGraph's `change` options, the ones that select what this reweighting
//! already does (`change helicity False`, `change output default`) or that only
//! direct MadGraph's own working files (`change rwgt_dir`, `change multicore`)
//! are accepted and read no further. Everything else MadGraph's reweight module
//! understands — the other `change` options, a param-card path in place of
//! `set` lines, `scan:` values, a command after `done` — is refused with the
//! line it appeared on rather than skipped: a skipped directive would write
//! weights for a hypothesis other than the one the card describes. Every line
//! refused is reported at once.

use std::fmt;
use std::str::FromStr;

/// One parameter change of a `launch` block.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Change {
    /// `set <block> <code…> <value>`: the external parameter at that LHA address.
    Lha {
        block: String,
        code: Vec<i32>,
        value: f64,
        line: usize,
    },
    /// `set <name> <value>`: the external parameter of that name.
    Name {
        name: String,
        value: f64,
        line: usize,
    },
}

impl Change {
    /// The card line the change was read from, 1-based.
    pub(crate) fn line(&self) -> usize {
        match self {
            Change::Lha { line, .. } | Change::Name { line, .. } => *line,
        }
    }

    /// The value the change sets.
    pub(crate) fn value(&self) -> f64 {
        match self {
            Change::Lha { value, .. } | Change::Name { value, .. } => *value,
        }
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Change::Lha {
                block, code, value, ..
            } => {
                write!(f, "set param_card {block}")?;
                for c in code {
                    write!(f, " {c}")?;
                }
                write!(f, " {value}")
            }
            Change::Name { name, value, .. } => write!(f, "set {name} {value}"),
        }
    }
}

/// One `launch` block: an optional name and description, and its changes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LaunchSpec {
    /// `--rwgt_name=…`, if given.
    pub(crate) name: Option<String>,
    /// `--rwgt_info=…`, if given.
    pub(crate) info: Option<String>,
    /// The card line of the `launch` itself, 1-based.
    pub(crate) line: usize,
    pub(crate) changes: Vec<Change>,
}

/// A parsed reweight card: its `launch` blocks in card order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReweightCard {
    pub(crate) launches: Vec<LaunchSpec>,
}

/// Why a reweight card was refused.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum CardError {
    #[error("line {line}: `{text}` is not supported: {reason}")]
    Unsupported {
        line: usize,
        text: String,
        reason: &'static str,
    },
    #[error("line {line}: `{text}`: {reason}")]
    Malformed {
        line: usize,
        text: String,
        reason: String,
    },
    #[error("line {line}: `set` before the first `launch`")]
    SetBeforeLaunch { line: usize },
    #[error("the card has no `launch` block")]
    Empty,
    #[error(
        "line {line}: a command after `{end}` on line {end_line}; MadGraph reads a reweight card \
         to its end and would run it, so remove `{end}` or the lines after it"
    )]
    AfterEnd {
        line: usize,
        end: String,
        end_line: usize,
    },
    /// More than one line of the card is refused; each is listed.
    #[error("{}", several(.0))]
    Several(Vec<CardError>),
}

fn several(errors: &[CardError]) -> String {
    errors
        .iter()
        .map(CardError::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// What a `change` line asks for, by MadGraph's option name
/// (`ReweightInterface.do_change`, `madgraph/interface/reweight_interface.py`).
fn change(args: &[&str]) -> Result<(), ChangeRefusal> {
    use ChangeRefusal::{Malformed, Unsupported};
    let Some(option) = args.first().map(|o| o.to_ascii_lowercase()) else {
        return Err(Malformed("`change` needs an option and a value".into()));
    };
    let value = args.get(1).map(|v| v.to_ascii_lowercase());
    let Some(value) = value else {
        return Err(Malformed(format!("`change {option}` needs a value")));
    };
    match option.as_str() {
        "helicity" => match value.as_str() {
            "false" | "f" | "0" | ".false." | "off" | "no" | "n" => Ok(()),
            "true" | "t" | "1" | ".true." | "on" | "yes" | "y" => Err(Unsupported(
                "the ratio in each event's own helicity, MadGraph's default; only the \
                 helicity-summed ratio (`change helicity False`) is written",
            )),
            _ => Err(Malformed(format!("`{value}` is not a boolean"))),
        },
        "output" if value == "default" => Ok(()),
        "output" => Err(Unsupported(
            "one event file per hypothesis; only weights added to the events are written",
        )),
        // MadGraph's own working directory and parallelism: neither reaches a weight.
        "rwgt_dir" | "multicore" => Ok(()),
        "model" | "process" => Err(Unsupported(
            "reweighting evaluates the generated process in the generated model; a \
             different model or process is not available",
        )),
        "mode" => Err(Unsupported("only the leading-order ratio is computed")),
        "systematics" => Err(Unsupported(
            "systematics on each hypothesis need MadGraph's per-hypothesis output",
        )),
        "keep_ordering" | "use_eventid" | "allow_missing_finalstate" => Err(Unsupported(
            "each event is evaluated in the subprocess it was generated in",
        )),
        "boost" => Err(Unsupported("the events are evaluated as written")),
        "tree_path" | "virtual_path" => Err(Unsupported(
            "the matrix element is this crate's, not a library's",
        )),
        "soft_threshold" => Err(Unsupported("no soft threshold is applied")),
        "identical_particle_in_prod_and_decay" => Err(Unsupported(
            "decayed events with identical particles are not matched specially",
        )),
        "include_sudakov" => Err(Unsupported("no Sudakov factor is applied")),
        _ => Err(Unsupported(
            "MadGraph does not know this option and discards the line",
        )),
    }
}

/// Why a `change` line is refused.
enum ChangeRefusal {
    Unsupported(&'static str),
    Malformed(String),
}

impl FromStr for ReweightCard {
    type Err = CardError;

    fn from_str(text: &str) -> Result<Self, CardError> {
        let mut launches: Vec<LaunchSpec> = Vec::new();
        let mut errors: Vec<CardError> = Vec::new();
        // `done`, `exit` or `quit` and its line, once seen.
        let mut ended: Option<(String, usize)> = None;
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw.split('#').next().unwrap_or("").trim();
            let tokens: Vec<&str> = content.split_whitespace().collect();
            let Some(&command) = tokens.first() else {
                continue;
            };
            if let Some((end, end_line)) = &ended {
                errors.push(CardError::AfterEnd {
                    line,
                    end: end.clone(),
                    end_line: *end_line,
                });
                continue;
            }
            let malformed = |reason: String| CardError::Malformed {
                line,
                text: content.to_string(),
                reason,
            };
            let unsupported = |reason: &'static str| CardError::Unsupported {
                line,
                text: content.to_string(),
                reason,
            };
            match command.to_ascii_lowercase().as_str() {
                "launch" => match parse_launch(&tokens[1..], line) {
                    Ok(launch) => launches.push(launch),
                    Err(reason) => errors.push(malformed(reason)),
                },
                "set" => {
                    let Some(launch) = launches.last_mut() else {
                        errors.push(CardError::SetBeforeLaunch { line });
                        continue;
                    };
                    if tokens[1..].iter().any(|t| is_scan_value(t)) {
                        errors.push(unsupported(
                            "scan values; write one `launch` block per value",
                        ));
                        continue;
                    }
                    match parse_set(&tokens[1..], line) {
                        Ok(change) => launch.changes.push(change),
                        Err(reason) => errors.push(malformed(reason)),
                    }
                }
                "change" => match change(&tokens[1..]) {
                    Ok(()) => {}
                    Err(ChangeRefusal::Unsupported(reason)) => errors.push(unsupported(reason)),
                    Err(ChangeRefusal::Malformed(reason)) => errors.push(malformed(reason)),
                },
                end @ ("done" | "exit" | "quit") => ended = Some((end.to_string(), line)),
                _ if command.contains('/') || command.ends_with(".dat") => {
                    errors.push(unsupported(
                        "a param card in place of `set` lines; give the changes as `set` lines",
                    ))
                }
                _ => errors.push(malformed("unknown directive".to_string())),
            }
        }
        match errors.len() {
            0 if launches.is_empty() => Err(CardError::Empty),
            0 => Ok(ReweightCard { launches }),
            1 => Err(errors.remove(0)),
            _ => Err(CardError::Several(errors)),
        }
    }
}

/// `launch [--rwgt_name=NAME] [--rwgt_info=INFO]`.
fn parse_launch(args: &[&str], line: usize) -> Result<LaunchSpec, String> {
    let mut name = None;
    let mut info = None;
    // An `--rwgt_info` value may contain spaces, so it takes every token up to the
    // next option.
    let mut i = 0;
    while i < args.len() {
        let arg = args[i];
        if let Some(v) = arg.strip_prefix("--rwgt_name=") {
            name = Some(unquote(v).to_string());
            i += 1;
        } else if let Some(v) = arg.strip_prefix("--rwgt_info=") {
            let mut words = vec![v];
            i += 1;
            while i < args.len() && !args[i].starts_with("--") {
                words.push(args[i]);
                i += 1;
            }
            info = Some(unquote(&words.join(" ")).to_string());
        } else {
            return Err(format!("unknown launch option `{arg}`"));
        }
    }
    Ok(LaunchSpec {
        name,
        info,
        line,
        changes: Vec::new(),
    })
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    for q in ['"', '\''] {
        if let Some(inner) = s.strip_prefix(q).and_then(|t| t.strip_suffix(q)) {
            return inner;
        }
    }
    s
}

/// `set [param_card] <block> <code…> <value>` or `set <name> <value>`.
fn parse_set(args: &[&str], line: usize) -> Result<Change, String> {
    let args = match args.first() {
        Some(first) if first.eq_ignore_ascii_case("param_card") => &args[1..],
        _ => args,
    };
    let (value, address) = match args.split_last() {
        Some((v, address)) if !address.is_empty() => (*v, address),
        _ => return Err("expected `set <block> <code…> <value>` or `set <name> <value>`".into()),
    };
    let value: f64 = parse_value(value).ok_or_else(|| format!("`{value}` is not a number"))?;
    if !value.is_finite() {
        return Err(format!("`{value}` is not a finite number"));
    }
    if address.len() == 1 {
        return Ok(Change::Name {
            name: address[0].to_string(),
            value,
            line,
        });
    }
    let code = address[1..]
        .iter()
        .map(|t| {
            t.parse::<i32>()
                .map_err(|_| format!("`{t}` is not an integer LHA code"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Change::Lha {
        block: address[0].to_ascii_lowercase(),
        code,
        value,
        line,
    })
}

/// MadGraph's scan value, `scan:[...]` or `scanN:[...]` (any case): the
/// prefix `reweight_interface.py` looks for at the start of a card value.
fn is_scan_value(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    lower
        .strip_prefix("scan")
        .map(|rest| rest.trim_start_matches(|c: char| c.is_ascii_digit()))
        .is_some_and(|rest| rest.starts_with(':'))
}

/// A number as a param card spells it, Fortran's `d` exponent included.
fn parse_value(token: &str) -> Option<f64> {
    token
        .parse()
        .ok()
        .or_else(|| token.replace(['d', 'D'], "e").parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launches_collect_their_set_lines() {
        let card: ReweightCard = "\
# a comment
launch --rwgt_name=heavy
  set yukawa 6 180.0   # trailing comment
  set param_card DECAY 25 5.0d-03
launch --rwgt_info=two words --rwgt_name='light'
  set ymt 1.5e2
done
# nothing after the end
"
        .parse()
        .unwrap();
        assert_eq!(card.launches.len(), 2);
        let heavy = &card.launches[0];
        assert_eq!(heavy.name.as_deref(), Some("heavy"));
        assert_eq!(heavy.line, 2);
        assert_eq!(
            heavy.changes,
            vec![
                Change::Lha {
                    block: "yukawa".into(),
                    code: vec![6],
                    value: 180.0,
                    line: 3
                },
                Change::Lha {
                    block: "decay".into(),
                    code: vec![25],
                    value: 5.0e-3,
                    line: 4
                },
            ]
        );
        let light = &card.launches[1];
        assert_eq!(light.name.as_deref(), Some("light"));
        assert_eq!(light.info.as_deref(), Some("two words"));
        assert_eq!(
            light.changes,
            vec![Change::Name {
                name: "ymt".into(),
                value: 150.0,
                line: 6
            }]
        );
    }

    #[test]
    fn a_launch_may_change_nothing() {
        let card: ReweightCard = "launch\nlaunch\n set ymt 1\n".parse().unwrap();
        assert!(card.launches[0].changes.is_empty());
        assert_eq!(card.launches[1].changes.len(), 1);
    }

    /// Every directive the card cannot honour is a refusal naming its line, never a
    /// skipped line.
    #[test]
    fn unsupported_directives_are_refused_with_their_line() {
        for (text, line) in [
            ("change model sm-full\nlaunch\n", 1),
            ("launch\n change process p p > e+ e-\n", 2),
            ("change helicity True\nlaunch\n", 1),
            ("change output 2.0\nlaunch\n", 1),
            ("change mode LO\nlaunch\n", 1),
            ("change nosuch 1\nlaunch\n", 1),
            ("launch\n set ymt scan:[1,2]\n", 2),
            ("launch\n set mass 6 SCAN1:[170, 175]\n", 2),
            ("launch\n ./param_card.dat\n", 2),
        ] {
            match text.parse::<ReweightCard>() {
                Err(CardError::Unsupported { line: l, .. }) => assert_eq!(l, line, "{text}"),
                other => panic!("{text}: {other:?}"),
            }
        }
    }

    /// Only a `scan:` value is a scan; a parameter whose name contains `scan`
    /// is set like any other.
    #[test]
    fn a_name_containing_scan_is_not_a_scan() {
        let card: ReweightCard = "launch\n set mscan 1\n set scanblock 3 2\n"
            .parse()
            .unwrap();
        assert_eq!(
            card.launches[0].changes,
            vec![
                Change::Name {
                    name: "mscan".into(),
                    value: 1.0,
                    line: 2
                },
                Change::Lha {
                    block: "scanblock".into(),
                    code: vec![3],
                    value: 2.0,
                    line: 3
                },
            ]
        );
    }

    #[test]
    fn malformed_lines_are_refused() {
        for text in [
            "set ymt 1\n",
            "launch\n set ymt\n",
            "launch\n set ymt abc\n",
            "launch\n set yukawa six 1\n",
            "launch\n set ymt inf\n",
            "launch --bogus\n",
            "frobnicate\n",
            "# nothing\n",
        ] {
            assert!(text.parse::<ReweightCard>().is_err(), "{text}");
        }
    }
}
