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
}

/// Parsed SLHA parameter card.
#[derive(Debug, Default, Clone)]
pub struct ParamCard {
    /// block name (lower-cased) → { lha_code → value }
    blocks: HashMap<String, HashMap<Vec<i32>, f64>>,
    /// PDG codes whose `DECAY` line gives the width as `Auto` (MadWidth's
    /// placeholder, computed by MadGraph at launch), in the order they appear.
    auto_widths: Vec<i32>,
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

    /// Parse SLHA content from a string.
    fn from_str(content: &str) -> Result<Self, SlhaError> {
        let mut card = ParamCard::default();
        let mut current_block: Option<String> = None;

        for (lineno, raw) in content.lines().enumerate() {
            let lineno = lineno + 1;

            // Strip inline comments and trim.
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let lower = line.to_lowercase();

            if lower.starts_with("block") {
                // `BLOCK SMINPUTS`
                let block_name = lower
                    .strip_prefix("block")
                    .unwrap()
                    .split_whitespace()
                    .next()
                    .ok_or_else(|| SlhaError::Parse {
                        line: lineno,
                        msg: "BLOCK header missing name".into(),
                    })?
                    .to_owned();
                card.blocks.entry(block_name.clone()).or_default();
                current_block = Some(block_name);
            } else if lower.starts_with("decay") {
                // `DECAY <pdg> <total_width>` — store the total width under a
                // synthetic block "decay" keyed by [pdg], so external width
                // parameters (lhablock = DECAY, e.g. WZ) resolve. Any
                // branching-ratio rows that follow are ignored.
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() >= 3 {
                    if let Ok(code) = tokens[1].parse::<i32>() {
                        if tokens[2].eq_ignore_ascii_case("auto") {
                            card.auto_widths.push(code);
                        } else if let Ok(width) = tokens[2].parse::<f64>() {
                            card.blocks
                                .entry("decay".to_string())
                                .or_default()
                                .insert(vec![code], width);
                        }
                    }
                }
                current_block = None;
            } else if let Some(ref block) = current_block {
                // Data entry: one or more integers followed by a float
                // e.g. `   3   0.118`, `   1   2   0.5`
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() < 2 {
                    continue;
                }
                // Try to parse all but the last token as integers (the key)
                // and the last as a float.
                let value: f64 =
                    tokens[tokens.len() - 1]
                        .parse()
                        .map_err(|_| SlhaError::Parse {
                            line: lineno,
                            msg: format!("expected float, got '{}'", tokens[tokens.len() - 1]),
                        })?;
                let mut key: Vec<i32> = Vec::new();
                for tok in &tokens[..tokens.len() - 1] {
                    match tok.parse::<i32>() {
                        Ok(n) => key.push(n),
                        Err(_) => {
                            // Hit a non-integer token — stop parsing keys here
                            break;
                        }
                    }
                }
                if key.is_empty() {
                    continue;
                }
                card.blocks
                    .get_mut(block.as_str())
                    .unwrap()
                    .insert(key, value);
            }
        }

        Ok(card)
    }
}

impl ParamCard {
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
    pub fn insert(&mut self, block: &str, code: Vec<i32>, value: f64) {
        self.blocks
            .entry(block.to_lowercase())
            .or_default()
            .insert(code, value);
    }

    /// Every value the card sets, as `(block, code, value)` with the block
    /// lower-cased and a `DECAY` line's width under block `decay`; sorted by
    /// block, then code.
    pub fn entries(&self) -> Vec<(&str, &[i32], f64)> {
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
    pub fn auto_widths(&self) -> &[i32] {
        &self.auto_widths
    }
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
        // The branching-ratio row `1.0  2  5 24` must NOT be parsed as an entry.
        assert_eq!(card.get("decay", &[5, 24]), None);
        assert_eq!(card.get("decay", &[2, 5, 24]), None);
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
