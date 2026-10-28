//! The variation request: MadGraph's systematics options parsed into one typed
//! value with every default filled in, from whichever entry point produced
//! them.
//!
//! MadGraph's `call_systematics` (`systematics.py:1311-1420`) reads a run card
//! one of two ways: the `systematics_arguments` list when the card writes that
//! line (`'systematics_arguments' in card.user_set`), and otherwise the
//! `sys_*` fields. A command line (`generate --systematics`) is split as a
//! shell splits it and read like the arguments. All three land in
//! [`VariationRequest`], so nothing downstream knows which one produced it.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Which entry point a request was read from; it only shapes messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestSource {
    /// The run card's `systematics_arguments` list.
    Arguments,
    /// The run card's `sys_scalefact`, `sys_alpsfact` and `sys_pdf`.
    SysFields,
    /// `generate --systematics`.
    CommandLine,
}

impl RequestSource {
    fn describe(self) -> &'static str {
        match self {
            RequestSource::Arguments => "systematics_arguments",
            RequestSource::SysFields => "the run card's sys_* fields",
            RequestSource::CommandLine => "--systematics",
        }
    }
}

/// `--dyn`: the scale a variation is evaluated at. The labels are
/// `write_banner`'s (`systematics.py:679`), `sum pt` included, though choice
/// `1` is `Σ E_T`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynScale {
    /// `-1`: the event's own scales, as its `<mgrwt>` record holds them.
    Nominal,
    /// `1`: `Σ E·pT/√(pT² + pz²)` over the final state (`get_et_scale`).
    SumEt,
    /// `2`: `Σ √(m² + pT²)` over the final state (`get_ht_scale`).
    Ht,
    /// `3`: half of [`DynScale::Ht`].
    HalfHt,
    /// `4`: `√ŝ` of the incoming partons (`get_sqrts_scale`).
    Sqrts,
}

impl DynScale {
    /// MadGraph's integer for the choice.
    pub fn code(self) -> i32 {
        match self {
            DynScale::Nominal => -1,
            DynScale::SumEt => 1,
            DynScale::Ht => 2,
            DynScale::HalfHt => 3,
            DynScale::Sqrts => 4,
        }
    }

    fn from_code(code: i64) -> Option<Self> {
        Some(match code {
            -1 => DynScale::Nominal,
            1 => DynScale::SumEt,
            2 => DynScale::Ht,
            3 => DynScale::HalfHt,
            4 => DynScale::Sqrts,
            _ => return None,
        })
    }

    /// `write_banner`'s description of the choice.
    pub fn label(self) -> &'static str {
        match self {
            DynScale::Nominal => "",
            DynScale::SumEt => "sum pt",
            DynScale::Ht => "HT",
            DynScale::HalfHt => "HT/2",
            DynScale::Sqrts => "sqrts",
        }
    }
}

/// One axis of the variation grid, as `--together` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    Mur,
    Muf,
    Alps,
    Dyn,
    Pdf,
}

impl Axis {
    /// The axes in `get_all_fct`'s position order (`pos`).
    pub const ALL: [Axis; 5] = [Axis::Mur, Axis::Muf, Axis::Alps, Axis::Dyn, Axis::Pdf];

    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "mur" => Axis::Mur,
            "muf" => Axis::Muf,
            "alps" => Axis::Alps,
            "dyn" => Axis::Dyn,
            "pdf" => Axis::Pdf,
            _ => return None,
        })
    }
}

/// A set named by its LHAPDF id or by its name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfRef {
    /// Any LHAPDF id of the set: a plain id naming member k ≠ 0 still means
    /// the whole set (`mkPDF(id).set()`).
    Id(i64),
    Name(String),
}

/// One member, by its LHAPDF id or by set name and member index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRef {
    /// `325402@0`: the member whose LHAPDF id this is.
    Id(i64),
    /// `NNPDF31_nnlo_as_0118_mc@7`; `name@0` is member 0.
    Named { set: String, member: u32 },
}

/// One `--pdf` entry, unresolved: which set or member it names is decided at
/// generate, where the sets are read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfEntry {
    /// `errorset`: every member of the set holding the run's own lhaid.
    ErrorSet,
    /// `central`: the run's own lhaid.
    Central,
    /// Every member of a set.
    Set(PdfRef),
    /// One member.
    Member(MemberRef),
}

/// MadGraph's systematics options, every default filled in.
///
/// Values are kept as `f64` and written with Python's `str(float)`, so `0.50`
/// and `0.5` on a card write the same header, as in MadGraph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationRequest {
    /// Where it was read from; messages only.
    pub source: RequestSource,
    /// `--mur`: renormalisation-scale factors, each finite and positive.
    pub mur: Vec<f64>,
    /// `--muf`: factorisation-scale factors.
    pub muf: Vec<f64>,
    /// `--alps`: factors on the MLM emission scales.
    pub alps: Vec<f64>,
    /// `--dyn`.
    pub dynamical: Vec<DynScale>,
    /// `--together` blocks, in order: the axes varied jointly, the first
    /// outermost. A request that gives none carries MadGraph's default.
    pub together: Vec<Vec<Axis>>,
    /// `--pdf` entries, unresolved.
    pub pdfs: Vec<PdfEntry>,
    /// `--start_id`: the first weight id.
    pub start_id: Option<i64>,
}

/// Why a request cannot be honoured. Each names the option or field and why;
/// the caller says at which step (integrate or generate) it was refused.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum RequestError {
    #[error(
        "{source_}: option {option} is not implemented: vibegraph writes MadGraph's default \
         weight names and descriptions and removes no weight"
    )]
    NotImplemented {
        source_: &'static str,
        option: String,
    },
    #[error(
        "{source_}: option {option} is not one of the honoured options (--mur, --muf, --alps, \
         --dyn, --together, --pdf, --start_id)"
    )]
    NotHonoured {
        source_: &'static str,
        option: String,
    },
    #[error(
        "{source_}: '{argument}' is not an option=value pair; MadGraph's parser splits each \
         argument at its one '=' and reads the value after it"
    )]
    Malformed {
        source_: &'static str,
        argument: String,
    },
    #[error("{source_}: {option} value '{value}' is not a {expected}")]
    BadValue {
        source_: &'static str,
        option: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error(
        "{source_}: {option} value {value} is not finite and positive; alpha_s and the parton \
         densities are undefined there"
    )]
    NonPositiveFactor {
        source_: &'static str,
        option: &'static str,
        value: String,
    },
    #[error(
        "{source_}: --dyn value {value} is outside -1, 1, 2, 3, 4, the choices MadGraph's \
         write_banner labels"
    )]
    UnknownDyn { source_: &'static str, value: i64 },
    #[error("{source_}: --together names '{name}', which is not one of mur, muf, alps, dyn, pdf")]
    UnknownAxis { source_: &'static str, name: String },
    #[error(
        "{source_}: PDF entry '{entry}' names member {member} by LHAPDF id; name the set \
         instead ('<set name>@{member}'): MadGraph adds the member to the id, which names a \
         different set's member wherever the id is not the set's first"
    )]
    MemberById {
        source_: &'static str,
        entry: String,
        member: String,
    },
    #[error(
        "{source_}: PDF entry '{entry}' is not a set name, an LHAPDF id, 'errorset' or \
         'central'{why}"
    )]
    BadPdfEntry {
        source_: &'static str,
        entry: String,
        why: &'static str,
    },
    #[error(
        "sys_pdf = '{value}' starts with the member count {count}, which follows no set name: \
         MadGraph's call_systematics indexes an empty list there"
    )]
    SysPdfLeadingCount { value: String, count: String },
    #[error("systematics_arguments = '{value}' does not split into arguments: {why}")]
    BadArgumentList { value: String, why: &'static str },
    #[error("--systematics '{value}' does not split as a shell command line")]
    BadCommandLine { value: String },
}

/// MadGraph's defaults when the arguments name none (`Systematics.__init__`,
/// `systematics.py:53-70`).
fn argument_defaults(source: RequestSource) -> VariationRequest {
    VariationRequest {
        source,
        mur: vec![0.5, 1.0, 2.0],
        muf: vec![0.5, 1.0, 2.0],
        alps: vec![1.0],
        dynamical: vec![
            DynScale::Nominal,
            DynScale::SumEt,
            DynScale::Ht,
            DynScale::HalfHt,
            DynScale::Sqrts,
        ],
        together: vec![vec![Axis::Mur, Axis::Muf, Axis::Dyn]],
        pdfs: vec![PdfEntry::ErrorSet],
        start_id: None,
    }
}

/// Options MadGraph's module reads that this crate refuses as unimplemented.
const NOT_IMPLEMENTED: &[&str] = &["weight_format", "weight_info", "remove_wgts", "keep_wgts"];

/// Python's `str.isdigit` on an ASCII token.
fn is_digit(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// Python's `float(text)` for the spellings a factor takes; refused unless
/// finite and positive.
fn factor(source: RequestSource, option: &'static str, text: &str) -> Result<f64, RequestError> {
    let value: f64 = text.trim().parse().map_err(|_| RequestError::BadValue {
        source_: source.describe(),
        option,
        value: text.to_string(),
        expected: "number",
    })?;
    if !(value.is_finite() && value > 0.0) {
        return Err(RequestError::NonPositiveFactor {
            source_: source.describe(),
            option,
            value: text.trim().to_string(),
        });
    }
    Ok(value)
}

/// One `--pdf` value, as `Systematics.__init__` reads it (`systematics.py:248-274`).
fn pdf_entry(source: RequestSource, data: &str) -> Result<PdfEntry, RequestError> {
    let bad = |why: &'static str| RequestError::BadPdfEntry {
        source_: source.describe(),
        entry: data.to_string(),
        why,
    };
    if data == "errorset" {
        return Ok(PdfEntry::ErrorSet);
    }
    if data == "central" {
        return Ok(PdfEntry::Central);
    }
    if data.chars().any(char::is_whitespace) {
        return Err(bad(
            ": it holds whitespace, which MadGraph passes on to LHAPDF, where no set is named so",
        ));
    }
    if data.is_empty() {
        return Err(bad(": it is empty"));
    }
    if data.contains('/') {
        return Err(bad(": a set name holds no '/'"));
    }
    if let Some((name, arg)) = data.rsplit_once('@') {
        let member: i64 = arg
            .parse()
            .map_err(|_| bad(": the member after '@' is not an integer"))?;
        if is_digit(name) {
            if member != 0 {
                return Err(RequestError::MemberById {
                    source_: source.describe(),
                    entry: data.to_string(),
                    member: arg.to_string(),
                });
            }
            let id = name.parse().map_err(|_| bad(": the id does not fit"))?;
            return Ok(PdfEntry::Member(MemberRef::Id(id)));
        }
        if name.is_empty() {
            return Err(bad(": no set name precedes '@'"));
        }
        let member = u32::try_from(member).map_err(|_| bad(": the member index is negative"))?;
        return Ok(PdfEntry::Member(MemberRef::Named {
            set: name.to_string(),
            member,
        }));
    }
    if is_digit(data) {
        let id = data.parse().map_err(|_| bad(": the id does not fit"))?;
        return Ok(PdfEntry::Set(PdfRef::Id(id)));
    }
    Ok(PdfEntry::Set(PdfRef::Name(data.to_string())))
}

impl VariationRequest {
    /// The request MadGraph's default card makes: `--mur=0.5,1,2
    /// --muf=0.5,1,2 --pdf=errorset` read as arguments.
    pub fn madgraph_default() -> Self {
        argument_defaults(RequestSource::Arguments)
    }

    /// Parse MadGraph systematics arguments as `call_systematics` reads them
    /// (`systematics.py:1324-1347`): each `option=value`, the option's dashes
    /// dropped, the value's outer quotes stripped and split at commas, repeated
    /// options accumulating and repeated `--together` blocks kept apart.
    pub fn from_arguments(
        arguments: &[String],
        source: RequestSource,
    ) -> Result<Self, RequestError> {
        let mut request = argument_defaults(source);
        let describe = source.describe();
        let mut mur: Option<Vec<f64>> = None;
        let mut muf: Option<Vec<f64>> = None;
        let mut alps: Option<Vec<f64>> = None;
        let mut dynamical: Option<Vec<DynScale>> = None;
        let mut together: Option<Vec<Vec<Axis>>> = None;
        let mut pdfs: Option<Vec<PdfEntry>> = None;
        for argument in arguments {
            let malformed = || RequestError::Malformed {
                source_: describe,
                argument: argument.clone(),
            };
            let parts: Vec<&str> = argument.split('=').collect();
            let [key, values] = parts.as_slice() else {
                return Err(malformed());
            };
            let key = key.replace('-', "");
            let mut values = values.trim();
            if values.is_empty() {
                return Err(malformed());
            }
            let bytes = values.as_bytes();
            if (bytes[0] == b'\'' || bytes[0] == b'"') && bytes[bytes.len() - 1] == bytes[0] {
                values = if values.len() >= 2 {
                    &values[1..values.len() - 1]
                } else {
                    ""
                };
            }
            let values: Vec<&str> = values.split(',').collect();
            match key.as_str() {
                "mur" | "muf" | "alps" => {
                    let option = match key.as_str() {
                        "mur" => "--mur",
                        "muf" => "--muf",
                        _ => "--alps",
                    };
                    let parsed = values
                        .iter()
                        .map(|v| factor(source, option, v))
                        .collect::<Result<Vec<_>, _>>()?;
                    let slot = match key.as_str() {
                        "mur" => &mut mur,
                        "muf" => &mut muf,
                        _ => &mut alps,
                    };
                    slot.get_or_insert_with(Vec::new).extend(parsed);
                }
                "dyn" => {
                    for v in &values {
                        let code: i64 = v.trim().parse().map_err(|_| RequestError::BadValue {
                            source_: describe,
                            option: "--dyn",
                            value: v.to_string(),
                            expected: "integer",
                        })?;
                        let choice = DynScale::from_code(code).ok_or(RequestError::UnknownDyn {
                            source_: describe,
                            value: code,
                        })?;
                        dynamical.get_or_insert_with(Vec::new).push(choice);
                    }
                }
                "together" => {
                    let block = values
                        .iter()
                        .map(|name| {
                            Axis::from_name(name).ok_or_else(|| RequestError::UnknownAxis {
                                source_: describe,
                                name: name.to_string(),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    together.get_or_insert_with(Vec::new).push(block);
                }
                "pdf" => {
                    for v in &values {
                        pdfs.get_or_insert_with(Vec::new)
                            .push(pdf_entry(source, v)?);
                    }
                }
                "start_id" => {
                    if request.start_id.is_none() {
                        let first = values[0];
                        let id: i64 = first.trim().parse().map_err(|_| RequestError::BadValue {
                            source_: describe,
                            option: "--start_id",
                            value: first.to_string(),
                            expected: "integer",
                        })?;
                        request.start_id = Some(id);
                    }
                }
                name if NOT_IMPLEMENTED.contains(&name) => {
                    return Err(RequestError::NotImplemented {
                        source_: describe,
                        option: format!("--{name}"),
                    })
                }
                name => {
                    return Err(RequestError::NotHonoured {
                        source_: describe,
                        option: format!("--{name}"),
                    })
                }
            }
        }
        if let Some(v) = mur {
            request.mur = v;
        }
        if let Some(v) = muf {
            request.muf = v;
        }
        if let Some(v) = alps {
            request.alps = v;
        }
        if let Some(v) = dynamical {
            request.dynamical = v;
        }
        if let Some(v) = together {
            request.together = v;
        }
        if let Some(v) = pdfs {
            request.pdfs = v;
        }
        Ok(request)
    }

    /// Parse a command line as `generate --systematics` receives it: split as
    /// a shell splits it, then read as arguments.
    pub fn from_command_line(text: &str) -> Result<Self, RequestError> {
        let arguments = shlex::split(text).ok_or_else(|| RequestError::BadCommandLine {
            value: text.to_string(),
        })?;
        Self::from_arguments(&arguments, RequestSource::CommandLine)
    }

    /// Parse the card's `systematics_arguments` payload: the list split as
    /// `banner.py`'s list reader splits it ([`split_card_list`]), then read
    /// as arguments.
    pub fn from_card_arguments(payload: &str) -> Result<Self, RequestError> {
        let arguments = split_card_list(payload);
        Self::from_arguments(&arguments, RequestSource::Arguments)
    }

    /// The request the `sys_*` fields make (`systematics.py:1374-1410`):
    /// `mur = muf = sys_scalefact`, `alps = sys_alpsfact` (`None` → `[1]`),
    /// every dynamical choice, the four scale axes varied together, and
    /// `sys_pdf` tokenised as `call_systematics` does.
    pub fn from_sys_fields(
        scalefact: &str,
        alpsfact: &str,
        pdf: &str,
    ) -> Result<Self, RequestError> {
        let source = RequestSource::SysFields;
        let mut request = argument_defaults(source);
        request.mur = scalefact
            .split_whitespace()
            .map(|v| factor(source, "sys_scalefact", v))
            .collect::<Result<_, _>>()?;
        request.muf = request.mur.clone();
        request.alps = if alpsfact == "None" {
            vec![1.0]
        } else {
            alpsfact
                .split_whitespace()
                .map(|v| factor(source, "sys_alpsfact", v))
                .collect::<Result<_, _>>()?
        };
        request.together = vec![vec![Axis::Mur, Axis::Muf, Axis::Alps, Axis::Dyn]];
        let groups: Vec<String> = if pdf.contains("&&") {
            pdf.split("&&").map(str::to_string).collect()
        } else {
            let mut groups: Vec<String> = Vec::new();
            for token in pdf.split_whitespace() {
                if !is_digit(token) || token.parse::<u64>().map_or(true, |n| n > 500) {
                    groups.push(token.to_string());
                } else {
                    let Some(last) = groups.last_mut() else {
                        return Err(RequestError::SysPdfLeadingCount {
                            value: pdf.to_string(),
                            count: token.to_string(),
                        });
                    };
                    *last = format!("{last} {token}");
                }
            }
            groups
        };
        let mut entries = Vec::new();
        for group in &groups {
            let split: Vec<&str> = group.split_whitespace().collect();
            match split.as_slice() {
                [_] => entries.push(pdf_entry(source, group)?),
                [name, count] => {
                    let n: u32 = count.parse().map_err(|_| RequestError::BadValue {
                        source_: source.describe(),
                        option: "sys_pdf",
                        value: group.clone(),
                        expected: "set name followed by a member count",
                    })?;
                    for i in 0..n {
                        entries.push(pdf_entry(source, &format!("{name}@{i}"))?);
                    }
                }
                _ => {
                    return Err(RequestError::BadPdfEntry {
                        source_: source.describe(),
                        entry: group.clone(),
                        why: ": a sys_pdf group is a set name, optionally followed by a member \
                              count",
                    })
                }
            }
        }
        request.pdfs = if entries.is_empty() {
            vec![PdfEntry::Central]
        } else {
            entries
        };
        Ok(request)
    }
}

/// `banner.py`'s reader for a list-valued card field (`ConfigFile.__setitem__`,
/// `banner.py:1140-1163`, then `format_variable`'s quote stripping): outer
/// brackets dropped, the text split at unescaped whitespace and commas except
/// inside quotes, a quoted run glued to the token just before it, and each
/// item's outer quotes stripped.
pub fn split_card_list(payload: &str) -> Vec<String> {
    let mut value = payload.trim();
    if value.starts_with('[') && value.ends_with(']') && value.len() >= 2 {
        value = &value[1..value.len() - 1];
    }
    let chars: Vec<char> = value.chars().collect();
    // `re.split` on `((?<![\\])['"])((?:.(?!(?<![\\])\1))*.?)\1`: unquoted
    // chunks interleaved with (quote, content) pairs.
    let mut chunks: Vec<String> = Vec::new();
    let mut quoted: Vec<(char, String)> = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let escaped = i > 0 && chars[i - 1] == '\\';
        if (c == '\'' || c == '"') && !escaped {
            let close = (i + 1..chars.len()).find(|&j| chars[j] == c && chars[j - 1] != '\\');
            if let Some(close) = close {
                chunks.push(std::mem::take(&mut current));
                quoted.push((c, chars[i + 1..close].iter().collect()));
                i = close + 1;
                continue;
            }
        }
        current.push(c);
        i += 1;
    }
    chunks.push(current);
    let split_chunk = |chunk: &str| -> Vec<String> {
        let cs: Vec<char> = chunk.chars().collect();
        let mut out = Vec::new();
        let mut token = String::new();
        for (k, &c) in cs.iter().enumerate() {
            let separator = c == ',' || (c.is_whitespace() && !(k > 0 && cs[k - 1] == '\\'));
            if separator {
                if !token.is_empty() {
                    out.push(std::mem::take(&mut token));
                }
            } else {
                token.push(c);
            }
        }
        if !token.is_empty() {
            out.push(token);
        }
        out
    };
    let mut items: Vec<String> = Vec::new();
    for (k, chunk) in chunks.iter().enumerate() {
        let mut current = split_chunk(chunk);
        if let Some((q, content)) = quoted.get(k) {
            let piece = format!("{q}{content}{q}");
            match current.last_mut() {
                Some(last) => last.push_str(&piece),
                None => current.push(piece),
            }
        }
        items.extend(current);
    }
    items
        .into_iter()
        .map(|item| {
            let t = item.trim();
            let stripped = if t.len() >= 2
                && ((t.starts_with('\'') && t.ends_with('\''))
                    || (t.starts_with('"') && t.ends_with('"')))
            {
                &t[1..t.len() - 1]
            } else {
                t
            };
            stripped.to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    /// The banked card's line splits into MadGraph's three arguments, and an
    /// unquoted list splits at every comma.
    #[test]
    fn the_card_list_splits_as_banner_py_splits_it() {
        assert_eq!(
            split_card_list("['--mur=0.5,1,2', '--muf=0.5,1,2', '--pdf=errorset']"),
            ["--mur=0.5,1,2", "--muf=0.5,1,2", "--pdf=errorset"]
        );
        assert_eq!(
            split_card_list("--mur=1,2,0.5 --pdf=a,b"),
            ["--mur=1", "2", "0.5", "--pdf=a", "b"]
        );
        assert_eq!(split_card_list("--pdf='a,b'"), ["--pdf='a,b'"]);
        assert_eq!(split_card_list("[]"), Vec::<String>::new());
        assert_eq!(split_card_list("['']"), [""]);
    }

    #[test]
    fn the_default_card_reads_as_madgraphs_defaults() {
        let request = VariationRequest::from_card_arguments(
            "['--mur=0.5,1,2', '--muf=0.5,1,2', '--pdf=errorset']",
        )
        .unwrap();
        assert_eq!(request, VariationRequest::madgraph_default());
        let empty = VariationRequest::from_card_arguments("").unwrap();
        assert_eq!(empty, VariationRequest::madgraph_default());
    }

    /// CMS's call parses unchanged, `--together` replacing the default.
    #[test]
    fn the_cms_command_line_parses() {
        let request = VariationRequest::from_command_line(
            "--start_id=1001 --pdf=325300,316200,306000@0 --mur=1,2,0.5 --muf=1,2,0.5 \
             --together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2",
        )
        .unwrap();
        assert_eq!(request.start_id, Some(1001));
        assert_eq!(request.mur, [1.0, 2.0, 0.5]);
        assert_eq!(request.alps, [0.5, 1.0, 2.0]);
        assert_eq!(request.together, [vec![Axis::Muf, Axis::Mur, Axis::Dyn]]);
        assert_eq!(
            request.pdfs,
            [
                PdfEntry::Set(PdfRef::Id(325300)),
                PdfEntry::Set(PdfRef::Id(316200)),
                PdfEntry::Member(MemberRef::Id(306000)),
            ]
        );
    }

    #[test]
    fn repeated_options_accumulate() {
        let request = VariationRequest::from_arguments(
            &args(&[
                "--mur=0.5",
                "--mur=2",
                "--together=mur",
                "--together=muf,dyn",
                "mur=4",
                "--pdf='NNPDF31_lo_as_0130@3'",
            ]),
            RequestSource::Arguments,
        )
        .unwrap();
        assert_eq!(request.mur, [0.5, 2.0, 4.0]);
        assert_eq!(
            request.together,
            [vec![Axis::Mur], vec![Axis::Muf, Axis::Dyn]]
        );
        assert_eq!(
            request.pdfs,
            [PdfEntry::Member(MemberRef::Named {
                set: "NNPDF31_lo_as_0130".into(),
                member: 3
            })]
        );
    }

    fn refused(arguments: &[&str]) -> RequestError {
        VariationRequest::from_arguments(&args(arguments), RequestSource::CommandLine)
            .expect_err("refused")
    }

    /// Refusal table: the custom naming and weight removal options.
    #[test]
    fn the_naming_and_removal_options_are_refused_as_unimplemented() {
        for option in ["weight_format", "weight_info", "remove_wgts", "keep_wgts"] {
            let e = refused(&[&format!("--{option}=x")]);
            assert!(
                matches!(&e, RequestError::NotImplemented { option: o, .. } if *o == format!("--{option}")),
                "{option}: {e}"
            );
        }
    }

    /// Refusal table: every other option, and a positional token.
    #[test]
    fn options_outside_the_honoured_set_are_refused() {
        for option in [
            "only_beam",
            "ion_scaling",
            "start_event",
            "stop_event",
            "lhapdf_config",
            "result",
            "write_banner",
            "from_card",
            "weight_info_format",
            "frobnicate",
        ] {
            let e = refused(&[&format!("--{option}=1")]);
            assert!(
                matches!(&e, RequestError::NotHonoured { option: o, .. } if *o == format!("--{option}")),
                "{option}: {e}"
            );
        }
        assert!(matches!(
            refused(&["--mur=1", "positional"]),
            RequestError::Malformed { .. }
        ));
    }

    /// Refusal table: an argument without `=`, with two, or with an empty
    /// value.
    #[test]
    fn a_malformed_argument_is_refused() {
        for bad in ["--mur", "--mur=1=2", "--mur=", "--mur=  ", ""] {
            assert!(
                matches!(refused(&[bad]), RequestError::Malformed { .. }),
                "{bad:?}"
            );
        }
    }

    /// Refusal table: a scale factor that is not finite and positive.
    #[test]
    fn a_factor_that_is_not_finite_and_positive_is_refused() {
        for option in ["mur", "muf", "alps"] {
            for value in ["0", "-1", "inf", "nan", "-0.5"] {
                let e = refused(&[&format!("--{option}=1,{value}")]);
                assert!(
                    matches!(e, RequestError::NonPositiveFactor { .. }),
                    "{option}={value}: {e}"
                );
            }
            let e = refused(&[&format!("--{option}=half")]);
            assert!(matches!(e, RequestError::BadValue { .. }), "{e}");
        }
    }

    /// Refusal table: a dynamical choice outside `-1, 1–4`.
    #[test]
    fn a_dynamical_choice_without_a_label_is_refused() {
        for value in ["0", "5", "-2"] {
            assert!(
                matches!(
                    refused(&[&format!("--dyn={value}")]),
                    RequestError::UnknownDyn { .. }
                ),
                "{value}"
            );
        }
        assert!(matches!(
            refused(&["--dyn=1.0"]),
            RequestError::BadValue { .. }
        ));
    }

    /// Refusal table: a `--together` name outside the five axes.
    #[test]
    fn an_unknown_together_axis_is_refused() {
        assert!(matches!(
            refused(&["--together=mur,scale"]),
            RequestError::UnknownAxis { name, .. } if name == "scale"
        ));
    }

    /// Refusal table: a `--start_id` that is not an integer.
    #[test]
    fn a_start_id_that_is_not_an_integer_is_refused() {
        assert!(matches!(
            refused(&["--start_id=10a"]),
            RequestError::BadValue {
                option: "--start_id",
                ..
            }
        ));
    }

    /// Refusal table: a member named by `id@N`, N ≠ 0, and the `sys_pdf`
    /// `id N` that expands to it.
    #[test]
    fn a_member_by_id_and_offset_is_refused() {
        assert!(matches!(
            refused(&["--pdf=247000@3"]),
            RequestError::MemberById { .. }
        ));
        assert!(matches!(
            VariationRequest::from_sys_fields("1", "None", "247000 3"),
            Err(RequestError::MemberById { .. })
        ));
        assert!(VariationRequest::from_command_line("--pdf=247000@0").is_ok());
    }

    /// Refusal table: an entry holding whitespace, from `--pdf` or from a
    /// `sys_pdf` piece between `&&`.
    #[test]
    fn a_pdf_entry_with_whitespace_is_refused() {
        assert!(matches!(
            refused(&["--pdf=NNPDF23_lo_as_0130_qed, CT14lo"]),
            RequestError::BadPdfEntry { .. }
        ));
        assert!(matches!(
            VariationRequest::from_sys_fields("1", "None", "CT14lo 3 && NNPDF23_lo"),
            Err(RequestError::BadPdfEntry { .. })
        ));
    }

    /// Refusal table: `sys_pdf` starting with a member count.
    #[test]
    fn a_sys_pdf_starting_with_a_count_is_refused() {
        assert!(matches!(
            VariationRequest::from_sys_fields("1", "None", "3 CT14lo"),
            Err(RequestError::SysPdfLeadingCount { .. })
        ));
    }

    /// The `sys_*` path: scales together with `alps` and `dyn`, and `sys_pdf`
    /// tokenised as `call_systematics` does.
    #[test]
    fn the_sys_fields_read_as_call_systematics_reads_them() {
        let request = VariationRequest::from_sys_fields("0.5 1 2", "None", "errorset").unwrap();
        assert_eq!(request.mur, [0.5, 1.0, 2.0]);
        assert_eq!(request.muf, request.mur);
        assert_eq!(request.alps, [1.0]);
        assert_eq!(
            request.together,
            [vec![Axis::Mur, Axis::Muf, Axis::Alps, Axis::Dyn]]
        );
        assert_eq!(request.pdfs, [PdfEntry::ErrorSet]);
        let request =
            VariationRequest::from_sys_fields("1", "0.5 1", "CT14lo 3 &&NNPDF23_lo").unwrap();
        assert_eq!(request.alps, [0.5, 1.0]);
        assert_eq!(request.pdfs.len(), 4);
        assert_eq!(
            request.pdfs[2],
            PdfEntry::Member(MemberRef::Named {
                set: "CT14lo".into(),
                member: 2
            })
        );
        let request = VariationRequest::from_sys_fields("1", "None", "").unwrap();
        assert_eq!(request.pdfs, [PdfEntry::Central]);
        let request = VariationRequest::from_sys_fields("1", "None", "247000 1 CT14lo").unwrap();
        assert_eq!(
            request.pdfs,
            [
                PdfEntry::Member(MemberRef::Id(247000)),
                PdfEntry::Set(PdfRef::Name("CT14lo".into()))
            ]
        );
    }
}
