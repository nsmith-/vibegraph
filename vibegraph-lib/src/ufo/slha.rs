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
}

impl ParamCard {
    /// Parse a `param_card.dat` from a file.
    pub fn from_file(path: &Path) -> Result<Self, SlhaError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse SLHA content from a string.
    pub fn from_str(content: &str) -> Result<Self, SlhaError> {
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
                    .trim()
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
                // Decay blocks — skip for now
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

    /// Look up a parameter value.
    ///
    /// `block` is case-insensitive; `code` is the integer key list.
    pub fn get(&self, block: &str, code: &[i32]) -> Option<f64> {
        self.blocks
            .get(block.to_lowercase().as_str())
            .and_then(|b| b.get(code))
            .copied()
    }

    /// Returns `true` if the named block exists.
    pub fn has_block(&self, block: &str) -> bool {
        self.blocks.contains_key(block.to_lowercase().as_str())
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

    #[test]
    fn test_parse_sminputs() {
        let card = ParamCard::from_str(SAMPLE).unwrap();
        assert!((card.get("SMINPUTS", &[1]).unwrap() - 132.50698).abs() < 1e-5);
        assert!((card.get("sminputs", &[3]).unwrap() - 0.118).abs() < 1e-10);
        assert!((card.get("SMINPUTS", &[2]).unwrap() - 1.16639e-05).abs() < 1e-14);
    }

    #[test]
    fn test_parse_mass() {
        let card = ParamCard::from_str(SAMPLE).unwrap();
        assert!((card.get("MASS", &[23]).unwrap() - 91.188).abs() < 1e-5);
        assert!((card.get("mass", &[6]).unwrap() - 172.0).abs() < 1e-10);
    }

    #[test]
    fn test_missing_key() {
        let card = ParamCard::from_str(SAMPLE).unwrap();
        assert!(card.get("MASS", &[999]).is_none());
        assert!(card.get("NOSUCHBLOCK", &[1]).is_none());
    }

    #[test]
    fn test_decay_skipped() {
        let card = ParamCard::from_str(SAMPLE).unwrap();
        // Decay block should not be parsed into a mass-like block
        assert!(!card.has_block("decay"));
    }
}
