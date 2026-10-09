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
//! Everything else MadGraph's reweight module understands — `change model`,
//! `change process`, `change helicity`, a param-card path in place of `set` lines,
//! `scan:` values — is refused with the line it appeared on rather than skipped:
//! a skipped directive would write weights for a hypothesis other than the one the
//! card describes.

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
}

impl FromStr for ReweightCard {
    type Err = CardError;

    fn from_str(text: &str) -> Result<Self, CardError> {
        let mut launches: Vec<LaunchSpec> = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw.split('#').next().unwrap_or("").trim();
            if content.is_empty() {
                continue;
            }
            let tokens: Vec<&str> = content.split_whitespace().collect();
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
            match tokens[0].to_ascii_lowercase().as_str() {
                "launch" => launches.push(parse_launch(&tokens[1..], line).map_err(malformed)?),
                "set" => {
                    let launch = launches
                        .last_mut()
                        .ok_or(CardError::SetBeforeLaunch { line })?;
                    if content.contains("scan") {
                        return Err(unsupported(
                            "scan values; write one `launch` block per value",
                        ));
                    }
                    launch
                        .changes
                        .push(parse_set(&tokens[1..], line).map_err(malformed)?);
                }
                "change" => {
                    return Err(unsupported(
                        "reweighting evaluates the generated process in the generated model; \
                         a different model, process, helicity treatment or mode is not \
                         available",
                    ))
                }
                "done" | "exit" | "quit" => break,
                _ if tokens[0].contains('/') || tokens[0].ends_with(".dat") => {
                    return Err(unsupported(
                        "a param card in place of `set` lines; give the changes as `set` lines",
                    ))
                }
                _ => return Err(malformed("unknown directive".to_string())),
            }
        }
        if launches.is_empty() {
            return Err(CardError::Empty);
        }
        Ok(ReweightCard { launches })
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
set ignored 1
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
            ("launch\n set ymt scan:[1,2]\n", 2),
            ("launch\n ./param_card.dat\n", 2),
        ] {
            match text.parse::<ReweightCard>() {
                Err(CardError::Unsupported { line: l, .. }) => assert_eq!(l, line, "{text}"),
                other => panic!("{text}: {other:?}"),
            }
        }
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
