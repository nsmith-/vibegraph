//! LHAPDF6 `.info` and member `.dat` file parsing into raw grid data.
//!
//! This module only reads the on-disk `lhagrid1` format into [`SetInfo`] and
//! [`SubGrid`]; no interpolation happens here.

use std::collections::BTreeMap;

use thiserror::Error;
use yaml_serde::Value;

#[derive(Debug, Error)]
pub enum GridError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: not readable as YAML: {message}")]
    Yaml { path: String, message: String },
    #[error("{path}: missing key '{key}'")]
    MissingKey { path: String, key: String },
    #[error("{path}: key '{key}' is not a valid {expected}: '{value}'")]
    InvalidValue {
        path: String,
        key: String,
        expected: &'static str,
        value: String,
    },
    #[error("{path}: unsupported grid format '{format}' (only 'lhagrid1' is supported)")]
    UnsupportedFormat { path: String, format: String },
    #[error("{path}: no subgrid blocks found")]
    NoSubgrids { path: String },
    #[error("{path}: subgrid {index} is missing its x/Q/flavor header lines")]
    MissingSubgridHeader { path: String, index: usize },
    #[error(
        "{path}: subgrid {index} has {nx} x-knots x {nq} Q-knots x {nf} flavors, \
         but {got} xf rows were parsed"
    )]
    ShapeMismatch {
        path: String,
        index: usize,
        nx: usize,
        nq: usize,
        nf: usize,
        got: usize,
    },
    #[error("{path}: subgrid {index}, row {row}: expected {nf} values, got {got}")]
    RowLength {
        path: String,
        index: usize,
        row: usize,
        nf: usize,
        got: usize,
    },
    #[error("member {member} out of range: '{set}' has {num_members} members")]
    MemberOutOfRange {
        set: String,
        member: u32,
        num_members: u32,
    },
}

/// `AlphaS_*` metadata: the strong coupling a set, or one of its members,
/// was fitted at. An entry the metadata does not give reads as unset (`NaN`,
/// an empty list, an empty type).
#[derive(Debug, Clone, PartialEq)]
pub struct AlphaSInfo {
    pub(crate) mz: f64,
    pub(crate) order_qcd: i32,
    /// `AlphaS_Type`, e.g. `"ipol"`.
    pub(crate) kind: String,
    pub(crate) qs: Vec<f64>,
    pub vals: Vec<f64>,
    pub(crate) lambda4: f64,
    pub(crate) lambda5: f64,
}

/// Parsed `<set>.info` metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct SetInfo {
    pub(crate) set_desc: String,
    /// `SetIndex`, the LHAPDF id of member 0; `None` where the file has none.
    pub(crate) set_index: Option<i64>,
    pub(crate) format: String,
    pub(crate) num_members: u32,
    /// Beam particle PDG code the set is defined for (2212 = proton), where
    /// the `.info` gives one: the CT18 and MSHT20 sets do not, and LHAPDF
    /// reads neither the key nor a default for it when it opens a set.
    pub(crate) particle: Option<i32>,
    /// Flavors present somewhere in the set (union across subgrids); gluon is
    /// listed as 21 here even though `.dat` flavor lists may use either 0 or 21.
    pub flavors: Vec<i32>,
    pub(crate) order_qcd: i32,
    pub(crate) error_type: String,
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) q_min: f64,
    pub(crate) q_max: f64,
    pub alpha_s: AlphaSInfo,
    /// `ForcePositive` (LHAPDF's positivity-clamp level: `0` none, `1` clamp
    /// negatives to zero, `2` clamp below `1e-10`). Absent in a `.info` file
    /// resolves to `0`, matching the installed `lhapdf.conf` default.
    pub force_positive: i32,
    /// Every entry of the `.info`, as parsed: what a member's own header is
    /// laid over.
    pub(crate) raw: RawMap,
}

impl SetInfo {
    /// `SetDesc`, the set's description.
    pub fn description(&self) -> &str {
        &self.set_desc
    }

    /// LHAPDF's `PDFSet::description()`: `SetDesc` as LHAPDF stores it, the
    /// YAML value re-emitted by its bundled yaml-cpp (see [`lhapdf_entry`]).
    /// A description holding `": "` therefore comes back double-quoted.
    pub fn lhapdf_description(&self) -> String {
        self.raw
            .get("SetDesc")
            .map_or_else(|| self.set_desc.clone(), lhapdf_entry)
    }

    /// LHAPDF's `errorType`: `ErrorType` as LHAPDF stores it, lowercased,
    /// `unknown` when the `.info` gives none.
    pub fn error_type(&self) -> String {
        if self.raw.contains_key("ErrorType") {
            self.error_type.to_lowercase()
        } else {
            "unknown".to_owned()
        }
    }
}

/// A metadata entry as LHAPDF 6.5 keeps it: `Info::load` parses the block
/// with yaml-cpp and stores each value as `YAML::Emitter << node` writes it,
/// so a string comes back plain where yaml-cpp may write it plain and
/// double-quoted, with yaml-cpp's escapes, where it may not. The file's own
/// quoting is gone either way: a single-quoted CT18 `SetDesc` comes back
/// plain, and a double-quoted MSHT20 one holding `Web address: www` comes
/// back double-quoted. A number or flag is its plain spelling.
pub(crate) fn lhapdf_entry(value: &Value) -> String {
    match value {
        Value::String(s) if !yaml_cpp_plain(s) => yaml_cpp_double_quoted(s),
        other => scalar_text(other),
    }
}

/// yaml-cpp's `IsValidPlainScalar` in block context with non-ASCII allowed
/// (`emitterutils.cpp`, `exp.h` of the copy LHAPDF 6.5 bundles), byte by
/// byte as its `StringCharSource` reads.
fn yaml_cpp_plain(s: &str) -> bool {
    if matches!(s, "" | "~" | "null" | "Null" | "NULL") {
        return false;
    }
    let b = s.as_bytes();
    let at = |i: usize| b.get(i).copied();
    let blank = |i: usize| matches!(at(i), Some(b' ' | b'\t'));
    let brk = |i: usize| at(i) == Some(b'\n') || (at(i) == Some(b'\r') && at(i + 1) == Some(b'\n'));
    let brk_len = |i: usize| if at(i) == Some(b'\r') { 2 } else { 1 };
    // `Exp::PlainScalar()` at the first byte.
    if blank(0)
        || brk(0)
        || b",[]{}#&*!|>'\"%@`".contains(&b[0])
        || (b"-?:".contains(&b[0]) && (blank(1) || brk(1) || at(1).is_none()))
    {
        return false;
    }
    if s.ends_with(' ') {
        return false;
    }
    (0..b.len()).all(|i| {
        let c = b[i];
        let end_scalar = c == b':' && (blank(i + 1) || brk(i + 1) || at(i + 1).is_none());
        let comment =
            (blank(i) && at(i + 1) == Some(b'#')) || (brk(i) && at(i + brk_len(i)) == Some(b'#'));
        let not_printable = matches!(c, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F)
            || (c == 0xC2 && matches!(at(i + 1), Some(0x80..=0x84 | 0x86..=0x9F)));
        let bom = b[i..].starts_with(&[0xEF, 0xBB, 0xBF]);
        !(end_scalar || comment || not_printable || bom || brk(i) || c == b'\t')
    })
}

/// yaml-cpp's `WriteDoubleQuotedString`, non-ASCII written as is.
fn yaml_cpp_double_quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        let cp = u32::from(ch);
        // `GetNextCodePointAndAdvance` reads a non-character as U+FFFD.
        let ch = if (cp & 0xFFFE) == 0xFFFE || (0xFDD0..=0xFDEF).contains(&cp) {
            '\u{FFFD}'
        } else {
            ch
        };
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if u32::from(c) < 0x20 || (0x80..=0xA0).contains(&u32::from(c)) => {
                out.push_str(&format!("\\x{:02x}", u32::from(c)));
            }
            '\u{FEFF}' => out.push_str("\\ufeff"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl AlphaSInfo {
    /// `AlphaS_MZ`, `NaN` where the metadata gives none.
    pub fn mz(&self) -> f64 {
        self.mz
    }
}

/// One `lhagrid1` subgrid block: a rectangular (x, Q²) grid of x·f(x, Q²)
/// values for a fixed flavor list.
#[derive(Debug, Clone, PartialEq)]
pub struct SubGrid {
    /// Ascending momentum-fraction knots.
    pub x: Vec<f64>,
    /// Ascending squared factorization-scale knots (GeV²). The file stores Q;
    /// this stores Q² directly since evaluation happens in (ln x, ln Q²).
    pub q2: Vec<f64>,
    /// PDG flavor codes in file order (gluon may appear as either 0 or 21).
    pub flavors: Vec<i32>,
    /// Row-major x·f values: `xf[(ix * q2.len() + iq) * flavors.len() + ifl]`.
    pub xf: Vec<f64>,
}

impl SubGrid {
    pub fn nx(&self) -> usize {
        self.x.len()
    }

    pub fn nq(&self) -> usize {
        self.q2.len()
    }

    pub(crate) fn nf(&self) -> usize {
        self.flavors.len()
    }

    /// x·f(x, Q²) at grid indices `(ix, iq)` for flavor position `ifl`.
    pub fn xf_at(&self, ix: usize, iq: usize, ifl: usize) -> f64 {
        self.xf[(ix * self.nq() + iq) * self.nf() + ifl]
    }
}

// ── `.info` and member-header parsing ────────────────────────────────────────

/// A metadata block's entries, key → YAML value: a set's `.info`, or the
/// header a member's `.dat` opens with. LHAPDF reads both as YAML, so they are
/// parsed as YAML here: a quoted or multi-line `SetDesc` reads as its text.
pub(crate) type RawMap = BTreeMap<String, Value>;

/// Parse a metadata block as YAML into its top-level entries. An empty block
/// is an empty map; anything but a mapping at the top is refused.
pub(crate) fn parse_raw_map(content: &str, path: &str) -> Result<RawMap, GridError> {
    let yaml_error = |message: String| GridError::Yaml {
        path: path.to_owned(),
        message,
    };
    let value: Value = yaml_serde::from_str(content).map_err(|e| yaml_error(e.to_string()))?;
    match value {
        Value::Null => Ok(RawMap::new()),
        Value::Mapping(entries) => entries
            .into_iter()
            .map(|(key, value)| match key {
                Value::String(k) => Ok((k, value)),
                other => Ok((scalar_text(&other), value)),
            })
            .collect(),
        _ => Err(yaml_error("the top level is not a mapping".to_owned())),
    }
}

/// A scalar's text as Python's `str()` of the YAML value would give it for a
/// string, and the plain spelling for a number or flag; empty for null.
fn scalar_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => yaml_serde::to_string(other)
            .map(|s| s.trim_end().to_owned())
            .unwrap_or_default(),
    }
}

fn require<'a>(map: &'a RawMap, path: &str, key: &str) -> Result<&'a Value, GridError> {
    map.get(key).ok_or_else(|| GridError::MissingKey {
        path: path.to_owned(),
        key: key.to_owned(),
    })
}

fn invalid(path: &str, key: &str, expected: &'static str, value: &Value) -> GridError {
    GridError::InvalidValue {
        path: path.to_owned(),
        key: key.to_owned(),
        expected,
        value: scalar_text(value),
    }
}

/// A scalar entry as text.
fn text_of(value: &Value) -> String {
    scalar_text(value)
}

fn parse_num<T: std::str::FromStr>(
    path: &str,
    key: &str,
    value: &Value,
    expected: &'static str,
) -> Result<T, GridError> {
    match value {
        Value::Number(_) | Value::String(_) => scalar_text(value)
            .trim()
            .parse::<T>()
            .map_err(|_| invalid(path, key, expected, value)),
        other => Err(invalid(path, key, expected, other)),
    }
}

/// A YAML sequence of numbers; `[]` yields an empty list.
fn parse_list<T: std::str::FromStr>(
    path: &str,
    key: &str,
    value: &Value,
    expected: &'static str,
) -> Result<Vec<T>, GridError> {
    let Value::Sequence(items) = value else {
        return Err(invalid(path, key, expected, value));
    };
    items
        .iter()
        .map(|item| parse_num(path, key, item, expected))
        .collect()
}

/// `ForcePositive`, `0` when absent (the installed `lhapdf.conf` default).
fn parse_force_positive(map: &RawMap, path: &str) -> Result<i32, GridError> {
    match map.get("ForcePositive") {
        None => Ok(0),
        Some(v) => {
            let level: i32 = parse_num(path, "ForcePositive", v, "integer")?;
            if !(0..=2).contains(&level) {
                return Err(GridError::InvalidValue {
                    path: path.to_owned(),
                    key: "ForcePositive".to_owned(),
                    expected: "0, 1 or 2",
                    value: text_of(v),
                });
            }
            Ok(level)
        }
    }
}

/// The `AlphaS_*` entries. Each is read where present and left unset where
/// not, since a set may give them only in its members' headers
/// (`ABMP16als118_5_nnlo`, `MSHT20nnlo_as_largerange`); the interpolator
/// refuses a type or a table it cannot read when one is built from them.
fn parse_alpha_s(map: &RawMap, path: &str) -> Result<AlphaSInfo, GridError> {
    let float = |key: &str| -> Result<f64, GridError> {
        map.get(key)
            .map_or(Ok(f64::NAN), |v| parse_num(path, key, v, "float"))
    };
    let list = |key: &str| -> Result<Vec<f64>, GridError> {
        map.get(key)
            .map_or(Ok(Vec::new()), |v| parse_list(path, key, v, "float list"))
    };
    Ok(AlphaSInfo {
        mz: float("AlphaS_MZ")?,
        order_qcd: map
            .get("AlphaS_OrderQCD")
            .map_or(Ok(0), |v| parse_num(path, "AlphaS_OrderQCD", v, "integer"))?,
        kind: map.get("AlphaS_Type").map(text_of).unwrap_or_default(),
        qs: list("AlphaS_Qs")?,
        vals: list("AlphaS_Vals")?,
        lambda4: float("AlphaS_Lambda4")?,
        lambda5: float("AlphaS_Lambda5")?,
    })
}

/// A member's own metadata, its header's entries laid over the set's `.info`
/// as LHAPDF cascades them: what the member's strong coupling and positivity
/// clamp are read from.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MemberMeta {
    pub alpha_s: AlphaSInfo,
    pub force_positive: i32,
}

/// [`MemberMeta`] of a member whose header block is `header`, in a set whose
/// `.info` is `info`.
pub(crate) fn member_meta(
    info: &SetInfo,
    header: &RawMap,
    path: &str,
) -> Result<MemberMeta, GridError> {
    let mut merged = info.raw.clone();
    for (key, value) in header {
        merged.insert(key.clone(), value.clone());
    }
    Ok(MemberMeta {
        alpha_s: parse_alpha_s(&merged, path)?,
        force_positive: parse_force_positive(&merged, path)?,
    })
}

/// Parse a `.info` file's content (the caller supplies `path` only for error
/// messages).
pub(crate) fn parse_info(content: &str, path: &str) -> Result<SetInfo, GridError> {
    let map = parse_raw_map(content, path)?;
    let force_positive = parse_force_positive(&map, path)?;
    let alpha_s = parse_alpha_s(&map, path)?;
    let set_index = map
        .get("SetIndex")
        .map(|v| parse_num(path, "SetIndex", v, "integer"))
        .transpose()?;

    Ok(SetInfo {
        set_desc: text_of(require(&map, path, "SetDesc")?),
        set_index,
        format: text_of(require(&map, path, "Format")?),
        num_members: parse_num(
            path,
            "NumMembers",
            require(&map, path, "NumMembers")?,
            "integer",
        )?,
        particle: map
            .get("Particle")
            .map(|v| parse_num(path, "Particle", v, "integer"))
            .transpose()?,
        flavors: parse_list(
            path,
            "Flavors",
            require(&map, path, "Flavors")?,
            "integer list",
        )?,
        order_qcd: parse_num(
            path,
            "OrderQCD",
            require(&map, path, "OrderQCD")?,
            "integer",
        )?,
        error_type: map.get("ErrorType").map(lhapdf_entry).unwrap_or_default(),
        x_min: parse_num(path, "XMin", require(&map, path, "XMin")?, "float")?,
        x_max: parse_num(path, "XMax", require(&map, path, "XMax")?, "float")?,
        q_min: parse_num(path, "QMin", require(&map, path, "QMin")?, "float")?,
        q_max: parse_num(path, "QMax", require(&map, path, "QMax")?, "float")?,
        alpha_s,
        force_positive,
        raw: map,
    })
}

// ── member `.dat` parsing ───────────────────────────────────────────────────

/// Split `.dat` content on `---` separator lines. In the `lhagrid1` format
/// block 0 is the YAML header, the remaining non-empty blocks are subgrid
/// bodies, and a trailing empty block (from a closing `---` at EOF) is dropped.
fn split_dat_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in content.lines() {
        if line.trim() == "---" {
            blocks.push(std::mem::take(&mut current));
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    blocks.push(current);
    blocks
}

fn parse_float_row(
    line: &str,
    path: &str,
    index: usize,
    what: &'static str,
) -> Result<Vec<f64>, GridError> {
    line.split_whitespace()
        .map(|tok| {
            tok.parse::<f64>().map_err(|_| GridError::InvalidValue {
                path: path.to_owned(),
                key: format!("subgrid {index} {what}"),
                expected: "float",
                value: tok.to_owned(),
            })
        })
        .collect()
}

fn parse_int_row(line: &str, path: &str, index: usize) -> Result<Vec<i32>, GridError> {
    line.split_whitespace()
        .map(|tok| {
            tok.parse::<i32>().map_err(|_| GridError::InvalidValue {
                path: path.to_owned(),
                key: format!("subgrid {index} flavor list"),
                expected: "integer",
                value: tok.to_owned(),
            })
        })
        .collect()
}

fn parse_subgrid_block(block: &str, path: &str, index: usize) -> Result<SubGrid, GridError> {
    let lines: Vec<&str> = block
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 3 {
        return Err(GridError::MissingSubgridHeader {
            path: path.to_owned(),
            index,
        });
    }

    let x = parse_float_row(lines[0], path, index, "x-knot row")?;
    let q = parse_float_row(lines[1], path, index, "Q-knot row")?;
    let flavors = parse_int_row(lines[2], path, index)?;

    let nx = x.len();
    let nq = q.len();
    let nf = flavors.len();

    let data_lines = &lines[3..];
    if data_lines.len() != nx * nq {
        return Err(GridError::ShapeMismatch {
            path: path.to_owned(),
            index,
            nx,
            nq,
            nf,
            got: data_lines.len(),
        });
    }

    let mut xf = Vec::with_capacity(nx * nq * nf);
    for (row, line) in data_lines.iter().enumerate() {
        let values = parse_float_row(line, path, index, "xf row")?;
        if values.len() != nf {
            return Err(GridError::RowLength {
                path: path.to_owned(),
                index,
                row,
                nf,
                got: values.len(),
            });
        }
        xf.extend(values);
    }

    // The file stores Q; SubGrid stores Q².
    let q2 = q.iter().map(|v| v * v).collect();

    Ok(SubGrid { x, q2, flavors, xf })
}

/// Parse a member `.dat` file's content (the caller supplies `path` only for
/// error messages).
#[cfg(test)]
pub(crate) fn parse_member_dat(content: &str, path: &str) -> Result<Vec<SubGrid>, GridError> {
    parse_member(content, path).map(|(_, subgrids)| subgrids)
}

/// Parse a member `.dat` file's content into its metadata header (the YAML
/// block before the first `---`) and its subgrids.
pub(crate) fn parse_member(content: &str, path: &str) -> Result<(RawMap, Vec<SubGrid>), GridError> {
    let blocks = split_dat_blocks(content);
    if blocks.is_empty() {
        return Err(GridError::NoSubgrids {
            path: path.to_owned(),
        });
    }

    let header_map = parse_raw_map(&blocks[0], path)?;
    let format = header_map.get("Format").map(text_of).unwrap_or_default();
    if format != "lhagrid1" {
        return Err(GridError::UnsupportedFormat {
            path: path.to_owned(),
            format,
        });
    }

    let last_is_trailing = blocks.len() > 1 && blocks.last().unwrap().trim().is_empty();
    let subgrid_blocks = if last_is_trailing {
        &blocks[1..blocks.len() - 1]
    } else {
        &blocks[1..]
    };

    if subgrid_blocks.is_empty() {
        return Err(GridError::NoSubgrids {
            path: path.to_owned(),
        });
    }

    let subgrids = subgrid_blocks
        .iter()
        .enumerate()
        .map(|(index, block)| parse_subgrid_block(block, path, index))
        .collect::<Result<_, _>>()?;
    Ok((header_map, subgrids))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_INFO: &str = r#"
SetDesc: "LO QCD + LO QED PDF set, alphas(MZ)=0.130"
SetIndex: 247000
Authors: NNPDF Collaboration
Reference:
Format: lhagrid1
DataVersion: 1
NumMembers: 101
Particle: 2212
Flavors: [-6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 21, 22]
OrderQCD: 0
FlavorScheme: variable
NumFlavors: 6
ErrorType: replicas
XMin: 1e-09
XMax: 1
QMin: 1
QMax: 10000
MZ: 91.1876
AlphaS_MZ: 0.130003
AlphaS_OrderQCD: 0
AlphaS_Type: ipol
AlphaS_Qs: [1.0, 2.0, 3.0]
AlphaS_Vals: [0.5, 0.4, 0.3]
AlphaS_Lambda4: 0.276134
AlphaS_Lambda5: 0.165831
"#;

    #[test]
    fn parses_full_info() {
        let info = parse_info(SAMPLE_INFO, "test.info").unwrap();
        assert_eq!(info.num_members, 101);
        assert_eq!(info.set_index, Some(247000));
        assert_eq!(info.particle, Some(2212));
        assert_eq!(
            info.flavors,
            vec![-6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 21, 22]
        );
        assert_eq!(info.format, "lhagrid1");
        assert_eq!(info.error_type, "replicas");
        assert_eq!(info.x_min, 1e-9);
        assert_eq!(info.x_max, 1.0);
        assert_eq!(info.q_min, 1.0);
        assert_eq!(info.q_max, 10000.0);
        assert_eq!(info.alpha_s.qs, vec![1.0, 2.0, 3.0]);
        assert_eq!(info.alpha_s.vals, vec![0.5, 0.4, 0.3]);
        assert_eq!(info.alpha_s.kind, "ipol");
        assert!((info.alpha_s.mz - 0.130003).abs() < 1e-12);
    }

    /// `SetDesc` read as YAML: a double-quoted scalar folded over four lines
    /// (the shape of `HERAPDF20_NNLO_EIG`'s) reads as one line, and a
    /// single-quoted one (the CT18 sets') reads without its quotes, a doubled
    /// quote inside it as one.
    #[test]
    fn a_set_description_is_read_as_yaml() {
        let folded = SAMPLE_INFO.replace(
            "SetDesc: \"LO QCD + LO QED PDF set, alphas(MZ)=0.130\"",
            "SetDesc: \"HERAPDF2.0 NNLO EIG, alphas(MZ)=0.118,\n  mem=0 => central value; \
             mem=1-28 =>\n  experimental eigenvectors;\n  mem=29-42 => model and \
             parametrisation\"",
        );
        assert_eq!(
            parse_info(&folded, "t.info").unwrap().description(),
            "HERAPDF2.0 NNLO EIG, alphas(MZ)=0.118, mem=0 => central value; mem=1-28 => \
             experimental eigenvectors; mem=29-42 => model and parametrisation"
        );
        let quoted = SAMPLE_INFO.replace(
            "SetDesc: \"LO QCD + LO QED PDF set, alphas(MZ)=0.130\"",
            "SetDesc: 'CT18NNLO, the ''nominal'' fit'",
        );
        assert_eq!(
            parse_info(&quoted, "t.info").unwrap().description(),
            "CT18NNLO, the 'nominal' fit"
        );
        let info = parse_info(SAMPLE_INFO, "t.info").unwrap();
        assert_eq!(info.error_type(), "replicas");
        let upper = SAMPLE_INFO.replace("ErrorType: replicas", "ErrorType: Hessian");
        assert_eq!(
            parse_info(&upper, "t.info").unwrap().error_type(),
            "hessian"
        );
    }

    /// An `.info` without `Particle` (the CT18 and MSHT20 sets of CMS's
    /// run-3 list) or without `ErrorType` (`MSHT20nnlo_as_largerange`,
    /// `MSHT20nnlo_as_smallrange`) opens as LHAPDF opens it; the error type
    /// is then LHAPDF's `unknown`.
    #[test]
    fn an_info_without_particle_or_error_type_opens() {
        let text = SAMPLE_INFO
            .replace("Particle: 2212\n", "")
            .replace("ErrorType: replicas\n", "");
        let info = parse_info(&text, "t.info").unwrap();
        assert_eq!(info.particle, None);
        assert_eq!(info.error_type(), "unknown");
        assert_eq!(info.num_members, 101);
    }

    /// `SetDesc` as LHAPDF 6.5's `description()` returns it, on the
    /// `SetDesc` lines of real `.info` files of CMS's run-3 list (verbatim)
    /// and on the shapes yaml-cpp's emitter quotes. The expected strings are
    /// what MadGraph's `systematics` module wrote into its `<weightgroup>`
    /// lines with LHAPDF 6.5.6, before its `=>`/`>`/`<` edit.
    #[test]
    fn a_set_description_is_what_lhapdf_returns() {
        let desc = |line: &str| {
            let text = SAMPLE_INFO.replace(
                "SetDesc: \"LO QCD + LO QED PDF set, alphas(MZ)=0.130\"",
                line,
            );
            let info = parse_info(&text, "t.info").unwrap();
            (info.description().to_owned(), info.lhapdf_description())
        };
        // `MSHT20nnlo_as118.info`: double-quoted in the file, holding ": ",
        // so yaml-cpp writes it double-quoted.
        let (yaml, lhapdf) = desc(
            "SetDesc: \"MSHT 2020 NNLO (68% C.L>). mem=0 => central value; mem=1-64 => 32 \
             eigenvector sets (+/- directions). See Section 5.3.2 of arXiv:2012.04684 for error \
             calculation. Web address: www.hep.ucl.ac.uk/msht\"",
        );
        assert_eq!(lhapdf, format!("\"{yaml}\""));
        assert!(lhapdf.ends_with("Web address: www.hep.ucl.ac.uk/msht\""));
        // `PDF4LHC21_40_pdfas.info`: likewise (`41 and 42: alphas`).
        let (yaml, lhapdf) = desc(
            "SetDesc: \"The PDF4LHC21 Hessian 40 set, 3-loop QCD, alphas(MZ)=0.118, large-x \
             positivity. 1+40 symmetric eigenvector sets at alphas(MZ)=0.118; 41 and 42: \
             alphas(MZ)=0.117 and 0.119\"",
        );
        assert_eq!(lhapdf, format!("\"{yaml}\""));
        // `HERAPDF20_NNLO_EIG.info`: folded over five lines, no ": ", plain.
        let (_, lhapdf) = desc(
            "SetDesc: \"HERAPDF20 NNLO (Q2min=3.5) fit and experimental uncertainty variations.\n \
             The 28 error PDFs should be treated two by two as the up and down excursions along \
             14 eigenvectors,\n such that the symmetric error is calculated as the quadrature sum \
             of [Sum i=1,14 of (var i+1 - var i)/2 ],\n or if asymmetric errors are desired \
             according to Eqn 43 of Campbell, Huston and Stirling hep-ph/0611148.\n mem=0 => \
             central; mem=1-28 => error eigenvectors.\"",
        );
        assert_eq!(
            lhapdf,
            "HERAPDF20 NNLO (Q2min=3.5) fit and experimental uncertainty variations. The 28 error \
             PDFs should be treated two by two as the up and down excursions along 14 \
             eigenvectors, such that the symmetric error is calculated as the quadrature sum of \
             [Sum i=1,14 of (var i+1 - var i)/2 ], or if asymmetric errors are desired according \
             to Eqn 43 of Campbell, Huston and Stirling hep-ph/0611148. mem=0 => central; \
             mem=1-28 => error eigenvectors."
        );
        // `CT18NNLO.info`: single-quoted in the file, plain from LHAPDF.
        let (_, lhapdf) = desc(
            "SetDesc: 'CT18NNLO, 3-loop PDF evolution with HOPPET and HOPPET alphas_s running \
             solution. This set has 59 member PDFs. mem=0 --> central value; mem=1-58 --> \
             eigenvector sets 90%'",
        );
        assert_eq!(
            lhapdf,
            "CT18NNLO, 3-loop PDF evolution with HOPPET and HOPPET alphas_s running solution. \
             This set has 59 member PDFs. mem=0 --> central value; mem=1-58 --> eigenvector sets \
             90%"
        );
        // `NNPDF40_nnlo_hessian_pdfas.info` (plain in the file) and
        // `ABMP16als118_5_nnlo.info` (double-quoted in the file): plain.
        for (line, want) in [
            (
                "SetDesc: Hessian conversion of NNPDF40_nnlo_as_01180_1000; mem=0 central value \
                 => alphas(MZ)=0.1180; mem=1-50 => eigenvector sets; mem=51 => alphas(MZ)=0.1170; \
                 mem=52 => alphas(MZ)=0.1190",
                "Hessian conversion of NNPDF40_nnlo_as_01180_1000; mem=0 central value => \
                 alphas(MZ)=0.1180; mem=1-50 => eigenvector sets; mem=51 => alphas(MZ)=0.1170; \
                 mem=52 => alphas(MZ)=0.1190",
            ),
            (
                "SetDesc: \"NNLO 5-flavour PDFs with different alphas_s for the ABMP16 set\"",
                "NNLO 5-flavour PDFs with different alphas_s for the ABMP16 set",
            ),
        ] {
            assert_eq!(desc(line).1, want);
        }
        // The rest of yaml-cpp's plain-scalar rule, and its escapes.
        for (line, want) in [
            ("SetDesc: \"- a list marker\"", "\"- a list marker\""),
            ("SetDesc: \"-1 is fine\"", "-1 is fine"),
            ("SetDesc: \"a trailing space \"", "\"a trailing space \""),
            ("SetDesc: \"a #comment\"", "\"a #comment\""),
            ("SetDesc: \"a#b\"", "a#b"),
            ("SetDesc: \"ends in a colon:\"", "\"ends in a colon:\""),
            ("SetDesc: \"a:b\"", "a:b"),
            ("SetDesc: \"[bracketed] start\"", "\"[bracketed] start\""),
            ("SetDesc: \"say \\\"x\\\": y\"", "\"say \\\"x\\\": y\""),
            ("SetDesc: \"tab\\there\"", "\"tab\\there\""),
            ("SetDesc: \"two\\nlines\"", "\"two\\nlines\""),
            ("SetDesc: \"null\"", "\"null\""),
            ("SetDesc: \"caf\u{e9} \u{3b1}s\"", "caf\u{e9} \u{3b1}s"),
        ] {
            assert_eq!(desc(line).1, want, "{line}");
        }
    }

    /// A member's header overrides the set's `AlphaS_*` and `ForcePositive`,
    /// entry by entry, and a set whose `.info` gives no `AlphaS_*` at all
    /// takes them from its members.
    #[test]
    fn a_member_header_overrides_the_sets_coupling_and_clamp() {
        let info = parse_info(SAMPLE_INFO, "t.info").unwrap();
        let header = parse_raw_map(
            "AlphaS_MZ: 0.120\nAlphaS_Vals: [0.6, 0.5, 0.4]\nPdfType: central\n\
             Format: lhagrid1\nForcePositive: 2\n",
            "t_0102.dat",
        )
        .unwrap();
        let meta = member_meta(&info, &header, "t_0102.dat").unwrap();
        assert_eq!(meta.alpha_s.mz, 0.120);
        assert_eq!(meta.alpha_s.vals, [0.6, 0.5, 0.4]);
        assert_eq!(meta.alpha_s.qs, info.alpha_s.qs, "an entry not overridden");
        assert_eq!(meta.force_positive, 2);
        let plain = parse_raw_map("PdfType: central\nFormat: lhagrid1\n", "t_0000.dat").unwrap();
        let meta = member_meta(&info, &plain, "t_0000.dat").unwrap();
        assert_eq!(meta.alpha_s, info.alpha_s);
        assert_eq!(meta.force_positive, 0);

        let bare: String = SAMPLE_INFO
            .lines()
            .filter(|l| !l.starts_with("AlphaS_"))
            .map(|l| format!("{l}\n"))
            .collect();
        let bare = parse_info(&bare, "bare.info").unwrap();
        assert_eq!(bare.alpha_s.kind, "");
        let member = parse_raw_map(
            "AlphaS_Type: ipol\nAlphaS_Qs: [1.0, 2.0, 3.0]\nAlphaS_Vals: [0.5, 0.4, 0.3]\n\
             AlphaS_MZ: 0.118\n",
            "bare_0000.dat",
        )
        .unwrap();
        let meta = member_meta(&bare, &member, "bare_0000.dat").unwrap();
        assert_eq!(meta.alpha_s.kind, "ipol");
        assert_eq!(meta.alpha_s.qs, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn missing_key_is_typed_error() {
        let truncated = "Format: lhagrid1\nNumMembers: 1\n";
        let err = parse_info(truncated, "bad.info").unwrap_err();
        assert!(matches!(err, GridError::MissingKey { .. }), "{err:?}");
    }

    #[test]
    fn malformed_list_is_typed_error() {
        let bad = SAMPLE_INFO.replace(
            "Flavors: [-6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 21, 22]",
            "Flavors: -6, -5, 1",
        );
        let err = parse_info(&bad, "bad.info").unwrap_err();
        assert!(matches!(err, GridError::InvalidValue { .. }), "{err:?}");
    }

    /// `SAMPLE_INFO` carries no `ForcePositive` key, matching every fetched
    /// set's `.info` this crate has seen carry no clamp at all: the resolved
    /// level must be `0`, not merely "some default".
    #[test]
    fn an_info_without_forcepositive_reads_as_the_config_default() {
        let info = parse_info(SAMPLE_INFO, "test.info").unwrap();
        assert_eq!(info.force_positive, 0);
    }

    #[test]
    fn an_unknown_forcepositive_level_is_refused() {
        let bad = format!("{SAMPLE_INFO}ForcePositive: 3\n");
        let err = parse_info(&bad, "bad.info").unwrap_err();
        assert!(matches!(err, GridError::InvalidValue { .. }), "{err:?}");
    }

    /// A minimal two-flavor, 2x2 `lhagrid1` member with a single subgrid.
    const SAMPLE_DAT_SINGLE: &str = "PdfType: central\nFormat: lhagrid1\n---\n\
1.0 2.0\n\
10.0 20.0\n\
1 21\n\
0.1 0.2\n\
0.3 0.4\n\
0.5 0.6\n\
0.7 0.8\n\
---\n";

    /// `SetIndex` is read trimmed, so CVMFS's 325300 `.info`, which writes it
    /// with a trailing space, reads as written; a file without one reads as
    /// `None`, and one that is not an integer is refused.
    #[test]
    fn set_index_is_trimmed_optional_and_typed() {
        let with_space = SAMPLE_INFO.replace("SetIndex: 247000\n", "SetIndex: 325300 \n");
        assert_eq!(
            parse_info(&with_space, "t.info").unwrap().set_index,
            Some(325300)
        );
        let without = SAMPLE_INFO.replace("SetIndex: 247000\n", "");
        assert_eq!(parse_info(&without, "t.info").unwrap().set_index, None);
        let bad = SAMPLE_INFO.replace("SetIndex: 247000\n", "SetIndex: x\n");
        assert!(matches!(
            parse_info(&bad, "t.info"),
            Err(GridError::InvalidValue { .. })
        ));
    }

    #[test]
    fn parses_single_subgrid() {
        let subgrids = parse_member_dat(SAMPLE_DAT_SINGLE, "test.dat").unwrap();
        assert_eq!(subgrids.len(), 1);
        let sg = &subgrids[0];
        assert_eq!(sg.x, vec![1.0, 2.0]);
        assert_eq!(sg.q2, vec![100.0, 400.0]);
        assert_eq!(sg.flavors, vec![1, 21]);
        assert_eq!(sg.nx(), 2);
        assert_eq!(sg.nq(), 2);
        assert_eq!(sg.nf(), 2);
        // Row order is (ix, iq) outer-to-inner, matching the file's row order.
        assert_eq!(sg.xf_at(0, 0, 0), 0.1);
        assert_eq!(sg.xf_at(0, 0, 1), 0.2);
        assert_eq!(sg.xf_at(0, 1, 0), 0.3);
        assert_eq!(sg.xf_at(1, 0, 0), 0.5);
        assert_eq!(sg.xf_at(1, 1, 1), 0.8);
    }

    /// Two subgrids sharing a Q² seam, as real multi-threshold sets (e.g.
    /// flavor-scheme transitions) lay them out.
    const SAMPLE_DAT_MULTI: &str = "PdfType: central\nFormat: lhagrid1\n---\n\
1.0 2.0\n\
10.0 20.0\n\
1\n\
0.1\n\
0.2\n\
0.3\n\
0.4\n\
---\n\
1.0 2.0\n\
20.0 30.0\n\
1\n\
0.5\n\
0.6\n\
0.7\n\
0.8\n\
---\n";

    #[test]
    fn parses_multiple_subgrids() {
        let subgrids = parse_member_dat(SAMPLE_DAT_MULTI, "test.dat").unwrap();
        assert_eq!(subgrids.len(), 2);
        assert_eq!(subgrids[0].q2, vec![100.0, 400.0]);
        assert_eq!(subgrids[1].q2, vec![400.0, 900.0]);
    }

    #[test]
    fn row_length_mismatch_is_typed_error() {
        let bad = SAMPLE_DAT_SINGLE.replacen("0.1 0.2\n", "0.1\n", 1);
        let err = parse_member_dat(&bad, "bad.dat").unwrap_err();
        assert!(matches!(err, GridError::RowLength { .. }), "{err:?}");
    }

    #[test]
    fn shape_mismatch_is_typed_error() {
        // Drop one data row: nx*nq = 4 declared, only 3 rows present.
        let bad = SAMPLE_DAT_SINGLE.replacen("0.7 0.8\n", "", 1);
        let err = parse_member_dat(&bad, "bad.dat").unwrap_err();
        assert!(matches!(err, GridError::ShapeMismatch { .. }), "{err:?}");
    }

    #[test]
    fn unsupported_format_is_typed_error() {
        let bad = SAMPLE_DAT_SINGLE.replace("Format: lhagrid1", "Format: lhagrid2");
        let err = parse_member_dat(&bad, "bad.dat").unwrap_err();
        assert!(
            matches!(err, GridError::UnsupportedFormat { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn empty_content_is_typed_error() {
        let err = parse_member_dat("", "empty.dat").unwrap_err();
        assert!(
            matches!(err, GridError::UnsupportedFormat { .. }),
            "{err:?}"
        );
    }
}
