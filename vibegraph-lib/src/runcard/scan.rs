//! A run card's text as the parser reads it, keeping what the parser would stop
//! on: [`RunCard::parse`](super::RunCard::parse) returns the first of these
//! errors, and the report lists every one.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::{canonical_name, parse_value, split_line, ParamValue, RunCardError, PARAM_DEFAULTS};

/// Why a card line did not read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Problem {
    /// A name MadGraph's template does not have.
    UnknownName,
    /// A known name whose value does not parse.
    MalformedValue,
    /// A `$`-led template line, which the parser refuses.
    Placeholder,
}

/// A card line this build does not read.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UnknownKey {
    pub name: String,
    pub line: usize,
    pub value: String,
    pub problem: Problem,
}

impl UnknownKey {
    /// The parser's refusal of a [`Problem::Placeholder`] line; `None` for any
    /// other problem.
    pub(super) fn placeholder_refusal(&self) -> Option<RunCardError> {
        (self.problem == Problem::Placeholder).then(|| RunCardError::UnfilledPlaceholder {
            name: self.name.clone(),
            placeholder: self.value.clone(),
            line: self.line,
        })
    }
}

/// The card's lines as the parser reads them: values by canonical name, the
/// lines they came from, the names written, and every line that did not read.
pub(super) struct Scanned {
    pub(super) values: BTreeMap<String, ParamValue>,
    pub(super) lines: BTreeMap<&'static str, usize>,
    pub(super) written: BTreeSet<String>,
    pub(super) unknown: Vec<UnknownKey>,
    /// The first error in line order, the one the parser returns.
    pub(super) first_error: Option<RunCardError>,
}

pub(super) fn scan(text: &str) -> Scanned {
    let mut values: BTreeMap<String, ParamValue> = PARAM_DEFAULTS
        .iter()
        .map(|(name, default)| (name.to_string(), default.to_value()))
        .collect();
    let mut lines = BTreeMap::new();
    let mut written = BTreeSet::new();
    let mut unknown = Vec::new();
    let mut first_error = None;
    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let Some((value_tok, name)) = split_line(raw) else {
            let body = raw.split(['#', '!']).next().unwrap_or("").trim();
            if let Some((value, name)) = body.rsplit_once('=').filter(|_| body.starts_with('$')) {
                let key = UnknownKey {
                    name: name.trim().to_string(),
                    line: line_no,
                    value: value.trim().to_string(),
                    problem: Problem::Placeholder,
                };
                first_error = first_error.or_else(|| key.placeholder_refusal());
                unknown.push(key);
            }
            continue;
        };
        let Some(canonical) = canonical_name(name) else {
            first_error.get_or_insert(RunCardError::UnknownParam {
                name: name.to_string(),
                line: line_no,
            });
            unknown.push(UnknownKey {
                name: name.to_string(),
                line: line_no,
                value: value_tok.to_string(),
                problem: Problem::UnknownName,
            });
            continue;
        };
        let kind = values[canonical].kind();
        let Some(parsed) = parse_value(value_tok, kind) else {
            first_error.get_or_insert(RunCardError::BadValue {
                name: name.to_string(),
                value: value_tok.to_string(),
                line: line_no,
            });
            unknown.push(UnknownKey {
                name: name.to_string(),
                line: line_no,
                value: value_tok.to_string(),
                problem: Problem::MalformedValue,
            });
            continue;
        };
        let auto_list = matches!(&parsed, ParamValue::Opaque(v) if v.eq_ignore_ascii_case("auto"));
        if auto_list {
            written.remove(canonical);
        } else {
            written.insert(canonical.to_string());
        }
        lines.insert(canonical, line_no);
        values.insert(canonical.to_string(), parsed);
    }
    Scanned {
        values,
        lines,
        written,
        unknown,
        first_error,
    }
}
