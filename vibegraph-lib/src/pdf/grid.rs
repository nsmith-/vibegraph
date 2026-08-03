//! LHAPDF6 `.info` and member `.dat` file parsing into raw grid data.
//!
//! This module only reads the on-disk `lhagrid1` format into [`SetInfo`] and
//! [`SubGrid`]; no interpolation happens here.

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GridError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
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

/// `AlphaS_*` metadata from the `.info` file.
///
/// Parsed but not consumed at LO: Drell-Yan uses fixed electroweak couplings
/// from the param card, not a running α_s from the PDF set.
#[derive(Debug, Clone, PartialEq)]
pub struct AlphaSInfo {
    pub mz: f64,
    pub order_qcd: i32,
    /// `AlphaS_Type`, e.g. `"ipol"`.
    pub kind: String,
    pub qs: Vec<f64>,
    pub vals: Vec<f64>,
    pub lambda4: f64,
    pub lambda5: f64,
}

/// Parsed `<set>.info` metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct SetInfo {
    pub set_desc: String,
    pub format: String,
    pub num_members: u32,
    /// Beam particle PDG code the set is defined for (2212 = proton).
    pub particle: i32,
    /// Flavors present somewhere in the set (union across subgrids); gluon is
    /// listed as 21 here even though `.dat` flavor lists may use either 0 or 21.
    pub flavors: Vec<i32>,
    pub order_qcd: i32,
    pub error_type: String,
    pub x_min: f64,
    pub x_max: f64,
    pub q_min: f64,
    pub q_max: f64,
    pub alpha_s: AlphaSInfo,
    /// `ForcePositive` (LHAPDF's positivity-clamp level: `0` none, `1` clamp
    /// negatives to zero, `2` clamp below `1e-10`). Absent in a `.info` file
    /// resolves to `0`, matching the installed `lhapdf.conf` default.
    pub force_positive: i32,
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

    pub fn nf(&self) -> usize {
        self.flavors.len()
    }

    /// x·f(x, Q²) at grid indices `(ix, iq)` for flavor position `ifl`.
    pub fn xf_at(&self, ix: usize, iq: usize, ifl: usize) -> f64 {
        self.xf[(ix * self.nq() + iq) * self.nf() + ifl]
    }
}

// ── `.info` parsing ─────────────────────────────────────────────────────────

/// key → raw (untyped) value string, one entry per non-blank, non-comment
/// `key: value` line.
type RawMap = HashMap<String, String>;

fn parse_raw_map(content: &str) -> RawMap {
    let mut map = RawMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        map.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    map
}

fn require<'a>(map: &'a RawMap, path: &str, key: &str) -> Result<&'a str, GridError> {
    map.get(key)
        .map(String::as_str)
        .ok_or_else(|| GridError::MissingKey {
            path: path.to_owned(),
            key: key.to_owned(),
        })
}

/// Strip a matching pair of surrounding double quotes, if present.
fn unquote(value: &str) -> String {
    let v = value.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        v[1..v.len() - 1].to_owned()
    } else {
        v.to_owned()
    }
}

fn parse_num<T: std::str::FromStr>(
    path: &str,
    key: &str,
    value: &str,
    expected: &'static str,
) -> Result<T, GridError> {
    value
        .trim()
        .parse::<T>()
        .map_err(|_| GridError::InvalidValue {
            path: path.to_owned(),
            key: key.to_owned(),
            expected,
            value: value.to_owned(),
        })
}

/// Parse a flow-style `[a, b, c]` list; `[]` yields an empty list.
fn parse_list<T>(
    path: &str,
    key: &str,
    value: &str,
    expected: &'static str,
) -> Result<Vec<T>, GridError>
where
    T: std::str::FromStr,
{
    let v = value.trim();
    let inner = v
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| GridError::InvalidValue {
            path: path.to_owned(),
            key: key.to_owned(),
            expected,
            value: value.to_owned(),
        })?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|tok| {
            tok.trim()
                .parse::<T>()
                .map_err(|_| GridError::InvalidValue {
                    path: path.to_owned(),
                    key: key.to_owned(),
                    expected,
                    value: tok.trim().to_owned(),
                })
        })
        .collect()
}

/// Parse a `.info` file's content (the caller supplies `path` only for error
/// messages).
pub fn parse_info(content: &str, path: &str) -> Result<SetInfo, GridError> {
    let map = parse_raw_map(content);

    let force_positive = match map.get("ForcePositive") {
        None => 0,
        Some(v) => {
            let level: i32 = parse_num(path, "ForcePositive", v, "integer")?;
            if !(0..=2).contains(&level) {
                return Err(GridError::InvalidValue {
                    path: path.to_owned(),
                    key: "ForcePositive".to_owned(),
                    expected: "0, 1 or 2",
                    value: v.to_owned(),
                });
            }
            level
        }
    };

    let alpha_s = AlphaSInfo {
        mz: parse_num(
            path,
            "AlphaS_MZ",
            require(&map, path, "AlphaS_MZ")?,
            "float",
        )?,
        order_qcd: parse_num(
            path,
            "AlphaS_OrderQCD",
            require(&map, path, "AlphaS_OrderQCD")?,
            "integer",
        )?,
        kind: unquote(require(&map, path, "AlphaS_Type")?),
        qs: parse_list(
            path,
            "AlphaS_Qs",
            require(&map, path, "AlphaS_Qs")?,
            "float list",
        )?,
        vals: parse_list(
            path,
            "AlphaS_Vals",
            require(&map, path, "AlphaS_Vals")?,
            "float list",
        )?,
        lambda4: parse_num(
            path,
            "AlphaS_Lambda4",
            require(&map, path, "AlphaS_Lambda4")?,
            "float",
        )?,
        lambda5: parse_num(
            path,
            "AlphaS_Lambda5",
            require(&map, path, "AlphaS_Lambda5")?,
            "float",
        )?,
    };

    Ok(SetInfo {
        set_desc: unquote(require(&map, path, "SetDesc")?),
        format: unquote(require(&map, path, "Format")?),
        num_members: parse_num(
            path,
            "NumMembers",
            require(&map, path, "NumMembers")?,
            "integer",
        )?,
        particle: parse_num(
            path,
            "Particle",
            require(&map, path, "Particle")?,
            "integer",
        )?,
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
        error_type: unquote(require(&map, path, "ErrorType")?),
        x_min: parse_num(path, "XMin", require(&map, path, "XMin")?, "float")?,
        x_max: parse_num(path, "XMax", require(&map, path, "XMax")?, "float")?,
        q_min: parse_num(path, "QMin", require(&map, path, "QMin")?, "float")?,
        q_max: parse_num(path, "QMax", require(&map, path, "QMax")?, "float")?,
        alpha_s,
        force_positive,
    })
}

pub fn parse_info_file(path: &Path) -> Result<SetInfo, GridError> {
    let path_str = path.display().to_string();
    let content = std::fs::read_to_string(path).map_err(|e| GridError::Io {
        path: path_str.clone(),
        source: e,
    })?;
    parse_info(&content, &path_str)
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

fn parse_member_dat_str(content: &str, path: &str) -> Result<Vec<SubGrid>, GridError> {
    let blocks = split_dat_blocks(content);
    if blocks.is_empty() {
        return Err(GridError::NoSubgrids {
            path: path.to_owned(),
        });
    }

    let header_map = parse_raw_map(&blocks[0]);
    let format = header_map
        .get("Format")
        .map(|s| unquote(s))
        .unwrap_or_default();
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

    subgrid_blocks
        .iter()
        .enumerate()
        .map(|(index, block)| parse_subgrid_block(block, path, index))
        .collect()
}

pub fn parse_member_dat(path: &Path) -> Result<Vec<SubGrid>, GridError> {
    let path_str = path.display().to_string();
    let content = std::fs::read_to_string(path).map_err(|e| GridError::Io {
        path: path_str.clone(),
        source: e,
    })?;
    parse_member_dat_str(&content, &path_str)
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
        assert_eq!(info.particle, 2212);
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

    #[test]
    fn parses_single_subgrid() {
        let subgrids = parse_member_dat_str(SAMPLE_DAT_SINGLE, "test.dat").unwrap();
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
        let subgrids = parse_member_dat_str(SAMPLE_DAT_MULTI, "test.dat").unwrap();
        assert_eq!(subgrids.len(), 2);
        assert_eq!(subgrids[0].q2, vec![100.0, 400.0]);
        assert_eq!(subgrids[1].q2, vec![400.0, 900.0]);
    }

    #[test]
    fn row_length_mismatch_is_typed_error() {
        let bad = SAMPLE_DAT_SINGLE.replacen("0.1 0.2\n", "0.1\n", 1);
        let err = parse_member_dat_str(&bad, "bad.dat").unwrap_err();
        assert!(matches!(err, GridError::RowLength { .. }), "{err:?}");
    }

    #[test]
    fn shape_mismatch_is_typed_error() {
        // Drop one data row: nx*nq = 4 declared, only 3 rows present.
        let bad = SAMPLE_DAT_SINGLE.replacen("0.7 0.8\n", "", 1);
        let err = parse_member_dat_str(&bad, "bad.dat").unwrap_err();
        assert!(matches!(err, GridError::ShapeMismatch { .. }), "{err:?}");
    }

    #[test]
    fn unsupported_format_is_typed_error() {
        let bad = SAMPLE_DAT_SINGLE.replace("Format: lhagrid1", "Format: lhagrid2");
        let err = parse_member_dat_str(&bad, "bad.dat").unwrap_err();
        assert!(
            matches!(err, GridError::UnsupportedFormat { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn empty_content_is_typed_error() {
        let err = parse_member_dat_str("", "empty.dat").unwrap_err();
        assert!(
            matches!(err, GridError::UnsupportedFormat { .. }),
            "{err:?}"
        );
    }
}
