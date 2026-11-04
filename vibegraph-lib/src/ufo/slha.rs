//! SLHA-format `param_card.dat` reader.
//!
//! The Standard Les Houches Accord (SLHA) defines a text format for passing
//! physics parameters between tools. A typical entry looks like:
//!
//! ```text
//! Block SMINPUTS
//!     1   132.50698   # aEWM1
//!     2   1.16639e-05 # Gf
//!     3   0.118       # aS
//! ```
//!
//! We parse this into `blocks: HashMap<String, HashMap<Vec<i32>, f64>>`.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlhaError {
    #[error("IO error reading param_card: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error on line {line}: {msg}")]
    Parse { line: usize, msg: String },
    /// More than one line of the card is refused; each is listed.
    #[error("{}", several(.0))]
    Several(Vec<SlhaError>),
}

fn several(errors: &[SlhaError]) -> String {
    errors
        .iter()
        .map(SlhaError::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Parsed SLHA parameter card.
#[derive(Debug, Default, Clone)]
pub struct ParamCard {
    /// block name (lower-cased) → { lha_code → value }
    blocks: HashMap<String, HashMap<Vec<i32>, f64>>,
    /// PDG codes whose `DECAY` line gives the width as `Auto` (MadWidth's
    /// placeholder, computed by MadGraph at launch), in the order they appear.
    auto_widths: Vec<i32>,
    /// PDG codes whose `DECAY` line is followed by a decay table (branching-ratio
    /// rows), in the order they appear.
    decay_tables: Vec<i32>,
}

/// What the lines below the last header belong to.
enum Section {
    /// Nothing yet: MadGraph's reader skips such a line, and so does this one.
    None,
    Block(String),
    /// Below `DECAY <pdg>`: a line is a row of the particle's decay table.
    Decay(i32),
    /// Below `XSECTION`: SLHA cross-section rows, which MadGraph keeps unparsed
    /// up to the next `BLOCK` or `DECAY`.
    CrossSections,
}

impl ParamCard {
    /// Parse a `param_card.dat` from a file.
    pub fn from_file(path: &Path) -> Result<Self, SlhaError> {
        let content = std::fs::read_to_string(path)?;
        content.parse()
    }
}

impl FromStr for ParamCard {
    type Err = SlhaError;

    /// Parse SLHA content from a string, reporting every line refused.
    ///
    /// The constructs follow MadGraph's reader (`ParamCard.read`,
    /// `models/check_param_card.py`): lines are matched without case, a decay
    /// table's rows are recorded by particle ([`ParamCard::decay_tables`]) and
    /// an `XSECTION` block's rows are skipped up to the next header. A value
    /// MadGraph would keep as a string (a `scan:` value, a `DECAY` width
    /// followed by more fields) is refused.
    fn from_str(content: &str) -> Result<Self, SlhaError> {
        let mut card = ParamCard::default();
        let mut section = Section::None;
        let mut seen: HashMap<(String, Vec<i32>), (CardValue, usize)> = HashMap::new();
        let mut headers: HashMap<String, usize> = HashMap::new();
        let mut errors: Vec<SlhaError> = Vec::new();

        for (lineno, raw) in content.lines().enumerate() {
            let lineno = lineno + 1;

            // Strip inline comments and trim.
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let lower = line.to_lowercase();
            let parse_error = |msg: String| SlhaError::Parse { line: lineno, msg };

            if let Some(header) = lower.strip_prefix("block") {
                // `BLOCK SMINPUTS`, `BLOCK MSOFT Q= 4.6e+02`. A `QNUMBERS` block
                // is one per new state, named by its PDG code (`BLOCK QNUMBERS
                // 9000005`), and is kept as `qnumbers <pdg>`, as MadGraph's
                // reader keeps it. A block's scale is not read.
                section = Section::None;
                let mut words = header.split_whitespace();
                let Some(name) = words.next() else {
                    errors.push(parse_error("BLOCK header missing name".into()));
                    continue;
                };
                let mut block_name = name.to_owned();
                if block_name == "qnumbers" {
                    if let Some(pdg) = words.next() {
                        block_name = format!("qnumbers {pdg}");
                    }
                }
                // MadGraph's reader replaces a block by a later header of the
                // same name, dropping the earlier entries; this one would
                // merge them. Neither is what a card means, so it is refused.
                if let Some(first_line) = headers.insert(block_name.clone(), lineno) {
                    errors.push(parse_error(format!(
                        "block {block_name} is opened again (first on line {first_line})"
                    )));
                    continue;
                }
                card.blocks.entry(block_name.clone()).or_default();
                section = Section::Block(block_name);
            } else if lower.starts_with("decay") {
                // `DECAY <pdg> <total_width>` — store the total width under a
                // synthetic block "decay" keyed by [pdg], so external width
                // parameters (lhablock = DECAY, e.g. WZ) resolve.
                section = Section::None;
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() < 3 {
                    errors.push(parse_error(format!(
                        "a DECAY line needs a PDG code and a width, got '{line}'"
                    )));
                    continue;
                }
                let Ok(code) = tokens[1].parse::<i32>() else {
                    errors.push(parse_error(format!(
                        "expected an integer PDG code after DECAY, got '{}'",
                        tokens[1]
                    )));
                    continue;
                };
                section = Section::Decay(code);
                if tokens.len() > 3 {
                    errors.push(parse_error(format!(
                        "the DECAY line of {code} has fields after the width ('{}'), \
                         which MadGraph reads as a width that is not a number",
                        tokens[3..].join(" ")
                    )));
                    continue;
                }
                let width = if tokens[2].eq_ignore_ascii_case("auto") {
                    CardValue::Auto
                } else if let Ok(width) = tokens[2].parse::<f64>() {
                    CardValue::Value(width)
                } else {
                    errors.push(parse_error(format!(
                        "expected a number or Auto as the width of {code}, got '{}'",
                        tokens[2]
                    )));
                    continue;
                };
                if let Err(e) = card.record("decay", vec![code], width, lineno, &mut seen) {
                    errors.push(e);
                }
            } else if lower.starts_with("xsection") {
                section = Section::CrossSections;
            } else {
                match &section {
                    Section::None | Section::CrossSections => {}
                    Section::Decay(pdg) => {
                        if !card.decay_tables.contains(pdg) {
                            card.decay_tables.push(*pdg);
                        }
                    }
                    Section::Block(block) => {
                        if let Err(e) = card.read_entry(block, line, lineno, &mut seen) {
                            errors.push(e);
                        }
                    }
                }
            }
        }

        match errors.len() {
            0 => Ok(card),
            1 => Err(errors.remove(0)),
            _ => Err(SlhaError::Several(errors)),
        }
    }
}

/// A value as a card line gives it: a number, or (for a `DECAY` width only)
/// `Auto`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum CardValue {
    Value(f64),
    Auto,
}

impl ParamCard {
    /// Read one data line of `block`: integer codes followed by a float, e.g.
    /// `   3   0.118`, `   1   2   0.5`; a lone value (SLHA's `Block ALPHA`)
    /// has the empty code, as MadGraph reads it.
    fn read_entry(
        &mut self,
        block: &str,
        line: &str,
        lineno: usize,
        seen: &mut HashMap<(String, Vec<i32>), (CardValue, usize)>,
    ) -> Result<(), SlhaError> {
        let parse_error = |msg: String| SlhaError::Parse { line: lineno, msg };
        let tokens: Vec<&str> = line.split_whitespace().collect();
        // MadGraph's scan marker (`Parameter.load_str`), one run per value.
        if tokens.iter().any(|t| t.to_lowercase().starts_with("scan")) {
            return Err(parse_error(format!(
                "block {block}: '{line}' is a scan value, which MadGraph runs once per \
                 value; give one card per value"
            )));
        }
        let Some((last, codes)) = tokens.split_last() else {
            return Ok(());
        };
        let value: f64 = last
            .parse()
            .map_err(|_| parse_error(format!("expected float, got '{last}'")))?;
        let key = codes
            .iter()
            .map(|tok| {
                tok.parse::<i32>().map_err(|_| {
                    parse_error(format!(
                        "block {block}: expected an integer code, got '{tok}'"
                    ))
                })
            })
            .collect::<Result<Vec<i32>, _>>()?;
        self.record(block, key, CardValue::Value(value), lineno, seen)
    }

    /// Store one parsed entry, refusing a `(block, code)` an earlier line set to
    /// another value. A repeat of the same value is accepted, as MadGraph's
    /// `check_param_card.Block.append` accepts it; a different one is the error
    /// MadGraph raises ("is already define").
    fn record(
        &mut self,
        block: &str,
        code: Vec<i32>,
        value: CardValue,
        lineno: usize,
        seen: &mut HashMap<(String, Vec<i32>), (CardValue, usize)>,
    ) -> Result<(), SlhaError> {
        let key = (block.to_owned(), code);
        if let Some(&(first, first_line)) = seen.get(&key) {
            if first == value {
                return Ok(());
            }
            let show = |w: CardValue| match w {
                CardValue::Value(v) => v.to_string(),
                CardValue::Auto => "Auto".to_owned(),
            };
            return Err(SlhaError::Parse {
                line: lineno,
                msg: format!(
                    "block {block} code {} is set to {} here and to {} on line {first_line}",
                    code_text(&key.1),
                    show(value),
                    show(first),
                ),
            });
        }
        seen.insert(key.clone(), (value, lineno));
        match value {
            CardValue::Value(v) => {
                self.blocks.entry(key.0).or_default().insert(key.1, v);
            }
            CardValue::Auto => self.auto_widths.push(key.1[0]),
        }
        Ok(())
    }

    /// Look up a parameter value.
    ///
    /// `block` is case-insensitive; `code` is the integer key list.
    pub fn get(&self, block: &str, code: &[i32]) -> Option<f64> {
        self.blocks
            .get(block.to_lowercase().as_str())
            .and_then(|b| b.get(code))
            .copied()
    }

    /// Set the value at `block` (case-insensitive) and `code`, replacing any
    /// value already there.
    pub(crate) fn insert(&mut self, block: &str, code: Vec<i32>, value: f64) {
        self.blocks
            .entry(block.to_lowercase())
            .or_default()
            .insert(code, value);
    }

    /// Every value the card sets, as `(block, code, value)` with the block
    /// lower-cased and a `DECAY` line's width under block `decay`; sorted by
    /// block, then code.
    pub(crate) fn entries(&self) -> Vec<(&str, &[i32], f64)> {
        let mut out: Vec<(&str, &[i32], f64)> = self
            .blocks
            .iter()
            .flat_map(|(block, values)| {
                values
                    .iter()
                    .map(move |(code, &value)| (block.as_str(), code.as_slice(), value))
            })
            .collect();
        out.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        out
    }

    /// The PDG codes whose width the card leaves to MadGraph to compute
    /// (`DECAY <pdg> Auto`), in card order.
    pub(crate) fn auto_widths(&self) -> &[i32] {
        &self.auto_widths
    }

    /// The PDG codes whose `DECAY` line the card follows with a decay table, in
    /// card order. Nothing here reads a branching ratio.
    pub(crate) fn decay_tables(&self) -> &[i32] {
        &self.decay_tables
    }
}

/// An LHA code as a card line writes it: the integers separated by spaces.
pub(crate) fn code_text(code: &[i32]) -> String {
    code.iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"
# Sample param_card
Block SMINPUTS
    1   132.50698   # aEWM1 (1/alpha_EM)
    2   1.16639e-05 # Gf
    3   0.118       # aS

Block MASS
    6   172.0       # MT
   23    91.188     # MZ
   24    80.419     # MW
    25  125.0       # MH

DECAY 6 1.49
   1.0  2  5 24    # t -> b W+
";

    /// A block header with no name and a block entry whose value is not a
    /// number are errors at their own line.
    #[test]
    fn malformed_lines_are_parse_errors_at_their_line() {
        for (text, want_line) in [
            ("Block MASS\n  6 173.0\nBlock\n", 3),
            ("Block MASS\n  6 173.0\n  23 heavy # MZ\n", 3),
        ] {
            let result = text.parse::<ParamCard>();
            assert!(
                matches!(result, Err(SlhaError::Parse { line, .. }) if line == want_line),
                "{text:?}: {result:?}"
            );
        }
    }

    /// The lines a card cannot honour are errors at their own line: a `DECAY`
    /// width that is neither a number nor `Auto`, a `DECAY` line with no width
    /// or a non-integer code, a block entry with a non-integer code, a `(block, code)` set twice to different values
    /// (MadGraph's "is already define"), `DECAY` widths and `Auto` included, and
    /// a block opened twice.
    #[test]
    fn lines_the_card_cannot_honour_are_parse_errors_at_their_line() {
        for (text, want_line, want) in [
            ("Block MASS\n 6 173.0\nDECAY 6 wide\n", 3, "number or Auto"),
            ("DECAY 6\n", 1, "a PDG code and a width"),
            ("DECAY t 1.49\n", 1, "integer PDG code"),
            (
                "Block MASS\n 6 173.0\n mt 173.0\n",
                3,
                "integer code, got 'mt'",
            ),
            ("Block YUKAWA\n 6 x 173.0\n", 2, "integer code, got 'x'"),
            (
                "Block MASS\n 23 91.188\n 6 173\n 23 95 # MZ\n",
                4,
                "on line 2",
            ),
            (
                "Block MASS\n 23 91.188\nBlock mass\n 6 173\n",
                3,
                "first on line 1",
            ),
            ("DECAY 23 2.44\nDECAY 23 2.5\n", 2, "on line 1"),
            ("DECAY 25 Auto\nDECAY 25 4e-3\n", 2, "on line 1"),
        ] {
            match text.parse::<ParamCard>() {
                Err(SlhaError::Parse { line, msg }) => {
                    assert_eq!(line, want_line, "{text:?}: {msg}");
                    assert!(msg.contains(want), "{text:?}: {msg}");
                }
                other => panic!("{text:?} parsed: {other:?}"),
            }
        }
    }

    /// A line that repeats a `(block, code)` at the same value is accepted, as
    /// MadGraph accepts it, and a lone value in a block is read under the empty
    /// code, as MadGraph reads SLHA's `Block ALPHA`.
    #[test]
    fn a_repeat_at_the_same_value_and_a_lone_value_are_read() {
        let card = "Block MASS\n 23 91.188\n 23 9.118800e+01\nDECAY 25 Auto\nDECAY 25 auto\n\
                    Block ALPHA\n -1.1e-01 # alpha\n"
            .parse::<ParamCard>()
            .unwrap();
        assert_eq!(card.get("MASS", &[23]), Some(91.188));
        assert_eq!(card.auto_widths(), &[25]);
        assert_eq!(card.get("ALPHA", &[]), Some(-0.11));
    }

    /// Two new states' `QNUMBERS` blocks are two blocks, as MadGraph names
    /// them, not one block set twice.
    #[test]
    fn each_new_state_has_its_own_qnumbers_block() {
        let card =
            "Block QNUMBERS 9000005 # x\n 1 0\n 2 1\nBlock QNUMBERS 9000006 # y\n 1 3\n 2 2\n"
                .parse::<ParamCard>()
                .unwrap();
        assert_eq!(card.get("QNUMBERS 9000005", &[1]), Some(0.0));
        assert_eq!(card.get("qnumbers 9000006", &[1]), Some(3.0));
    }

    #[test]
    fn test_parse_sminputs() {
        let card = SAMPLE.parse::<ParamCard>().unwrap();
        assert!((card.get("SMINPUTS", &[1]).unwrap() - 132.50698).abs() < 1e-5);
        assert!((card.get("sminputs", &[3]).unwrap() - 0.118).abs() < 1e-10);
        assert!((card.get("SMINPUTS", &[2]).unwrap() - 1.16639e-05).abs() < 1e-14);
    }

    #[test]
    fn test_parse_mass() {
        let card = SAMPLE.parse::<ParamCard>().unwrap();
        assert!((card.get("MASS", &[23]).unwrap() - 91.188).abs() < 1e-5);
        assert!((card.get("mass", &[6]).unwrap() - 172.0).abs() < 1e-10);
    }

    #[test]
    fn test_missing_key() {
        let card = SAMPLE.parse::<ParamCard>().unwrap();
        assert!(card.get("MASS", &[999]).is_none());
        assert!(card.get("NOSUCHBLOCK", &[1]).is_none());
    }

    #[test]
    fn test_decay_width_parsed() {
        let card = SAMPLE.parse::<ParamCard>().unwrap();
        // `DECAY 6 1.49` → total width under block "decay", key [6].
        assert_eq!(card.get("DECAY", &[6]), Some(1.49));
        // The branching-ratio row `1.0  2  5 24` is not an entry; it is recorded
        // as the top's decay table.
        assert_eq!(card.get("decay", &[5, 24]), None);
        assert_eq!(card.get("decay", &[2, 5, 24]), None);
        assert_eq!(card.decay_tables(), &[6]);
    }

    /// `DECAY <pdg> Auto`, in any case, is recorded as an automatic width and
    /// sets no value; a numeric width beside it is unaffected.
    #[test]
    fn auto_widths_are_recorded_and_set_nothing() {
        let card = "DECAY 25 Auto\nDECAY 6 auto # WT\nDECAY 23 2.4\n"
            .parse::<ParamCard>()
            .unwrap();
        assert_eq!(card.auto_widths(), &[25, 6]);
        assert_eq!(card.get("DECAY", &[25]), None);
        assert_eq!(card.get("DECAY", &[6]), None);
        assert_eq!(card.get("DECAY", &[23]), Some(2.4));
    }

    /// `entries` lists every value once, sorted, under lower-cased blocks; an
    /// inserted value replaces the parsed one.
    #[test]
    fn entries_are_sorted_and_insert_replaces() {
        let mut card = SAMPLE.parse::<ParamCard>().unwrap();
        card.insert("MASS", vec![23], 91.0);
        let entries = card.entries();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0], ("decay", &[6][..], 1.49));
        assert!(entries.contains(&("mass", &[23][..], 91.0)));
        let keys: Vec<_> = entries.iter().map(|e| (e.0, e.1)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }
}
