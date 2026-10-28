//! The variation list and the `<initrwgt>` block: MadGraph's `get_all_fct`
//! (`systematics.py:798-833`) and `write_banner` (`:603-738`), ported as a
//! state machine over the expanded variations.
//!
//! A PDF is identified by its *slot* in the resolved list, never by its
//! LHAPDF id: MadGraph compares PDF objects by identity, so
//! `--pdf=errorset,central` writes a second, non-nominal weight for the run's
//! own member, and a single PDF not holding the nominal is compared against a
//! fresh nominal object no list entry equals.

use std::collections::BTreeMap;

use itertools::Itertools;

use super::format::py_float;
use super::request::{Axis, DynScale, VariationRequest};

/// One resolved PDF slot, as the header reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotId {
    /// `pdf.lhapdfID`.
    pub lhapdf_id: i64,
    /// `pdf.memberID`.
    pub member: u32,
}

impl SlotId {
    /// The set's `SetIndex`, `lhapdfID − memberID`.
    pub fn set_index(self) -> i64 {
        self.lhapdf_id - i64::from(self.member)
    }
}

/// A whole set the request named (`self.pdfsets`), keyed by its `SetIndex`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetHeader {
    pub name: String,
    /// LHAPDF's `errorType`: the `.info` `ErrorType` lowercased, `unknown`
    /// when absent.
    pub error_type: String,
    /// `SetDesc`.
    pub description: String,
}

/// One variation: the scale factors, the dynamical choice and the PDF slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arg {
    pub mur: f64,
    pub muf: f64,
    pub alps: f64,
    pub dyn_scale: DynScale,
    pub pdf: usize,
}

#[derive(Clone, Copy)]
enum AxisValue {
    Factor(f64),
    Dyn(DynScale),
    Pdf(usize),
}

fn assign(arg: &mut Arg, axis: Axis, value: AxisValue) {
    match (axis, value) {
        (Axis::Mur, AxisValue::Factor(v)) => arg.mur = v,
        (Axis::Muf, AxisValue::Factor(v)) => arg.muf = v,
        (Axis::Alps, AxisValue::Factor(v)) => arg.alps = v,
        (Axis::Dyn, AxisValue::Dyn(d)) => arg.dyn_scale = d,
        (Axis::Pdf, AxisValue::Pdf(p)) => arg.pdf = p,
        _ => unreachable!("an axis takes values of its own kind"),
    }
}

fn axis_values(request: &VariationRequest, axis: Axis, list_slots: usize) -> Vec<AxisValue> {
    match axis {
        Axis::Mur => request.mur.iter().map(|&v| AxisValue::Factor(v)).collect(),
        Axis::Muf => request.muf.iter().map(|&v| AxisValue::Factor(v)).collect(),
        Axis::Alps => request.alps.iter().map(|&v| AxisValue::Factor(v)).collect(),
        Axis::Dyn => request
            .dynamical
            .iter()
            .map(|&d| AxisValue::Dyn(d))
            .collect(),
        Axis::Pdf => (0..list_slots).map(AxisValue::Pdf).collect(),
    }
}

/// `get_all_fct`: the nominal first, then every `--together` block's product
/// (first axis outermost) and every axis in no block on its own, each
/// variation equal to the nominal dropped, and the nominal inserted again
/// before the first variation of the run's `lhaid + 1`.
///
/// `slots[..list_slots]` are the `--pdf` list's members in order and
/// `nominal` is the slot the nominal reads.
pub fn expand(
    request: &VariationRequest,
    slots: &[SlotId],
    list_slots: usize,
    nominal: usize,
) -> Result<Vec<Arg>, String> {
    let default = Arg {
        mur: 1.0,
        muf: 1.0,
        alps: 1.0,
        dyn_scale: DynScale::Nominal,
        pdf: nominal,
    };
    let mut all = Vec::new();
    let mut done: Vec<Axis> = Vec::new();
    for block in &request.together {
        done.extend(block.iter().copied());
        let values: Vec<Vec<AxisValue>> = block
            .iter()
            .map(|&axis| axis_values(request, axis, list_slots))
            .collect();
        for combination in values.into_iter().multi_cartesian_product() {
            let mut arg = default;
            for (&axis, value) in block.iter().zip(combination) {
                assign(&mut arg, axis, value);
            }
            all.push(arg);
        }
    }
    for axis in Axis::ALL {
        if done.contains(&axis) {
            continue;
        }
        for value in axis_values(request, axis, list_slots) {
            let mut arg = default;
            assign(&mut arg, axis, value);
            all.push(arg);
        }
    }
    let mut args = vec![default];
    args.extend(all.into_iter().filter(|a| *a != default));

    let nominal_id = slots[nominal].lhapdf_id;
    if let Some(plus_one) = (0..list_slots).find(|&s| slots[s].lhapdf_id == nominal_id + 1) {
        let target = Arg {
            pdf: plus_one,
            ..default
        };
        let index = args.iter().position(|a| *a == target).ok_or_else(|| {
            format!(
                "the request varies PDF {} only together with other axes, never alone at the \
                 nominal scales, so MadGraph's get_all_fct finds no weight to place the nominal \
                 before and fails",
                nominal_id + 1
            )
        })?;
        args.insert(index, default);
    }
    Ok(args)
}

/// `write_banner`'s description edit: `=>` to `;`, `>` to `.gt.`, `<` to
/// `.lt.`.
fn description(text: &str) -> String {
    text.replace("=>", ";")
        .replace('>', ".gt.")
        .replace('<', ".lt.")
}

/// `get_wgt_tag`: each attribute followed by a space, joined by a space.
fn tag(arg: &Arg, slot: SlotId) -> String {
    let mut tags = vec![
        format!("MUR=\"{}\" ", py_float(arg.mur)),
        format!("MUF=\"{}\" ", py_float(arg.muf)),
    ];
    if arg.alps != 1.0 {
        tags.push(format!("ALPSFACT=\"{}\" ", py_float(arg.alps)));
    }
    if arg.dyn_scale != DynScale::Nominal {
        tags.push(format!("DYN_SCALE=\"{}\" ", arg.dyn_scale.code()));
    }
    tags.push(format!("PDF=\"{}\" ", slot.lhapdf_id));
    tags.join(" ")
}

/// `get_wgt_info`.
fn info(arg: &Arg, slot: SlotId, nominal: usize) -> String {
    let mut info = String::new();
    if arg.mur != 1.0 {
        info += &format!("MUR={} ", py_float(arg.mur));
    }
    if arg.muf != 1.0 {
        info += &format!("MUF={} ", py_float(arg.muf));
    }
    if arg.alps != 1.0 {
        info += &format!("alpsfact={} ", py_float(arg.alps));
    }
    if arg.dyn_scale != DynScale::Nominal {
        info += &format!("dyn_scale_choice={} ", arg.dyn_scale.label());
    }
    if arg.pdf != nominal {
        info += &format!("PDF={} MemberID={}", slot.set_index(), slot.member);
    }
    info
}

/// `write_banner`'s weight groups and weights for `args[1..]`, ids from
/// `first_id`, as the lines between `<initrwgt>` and `</initrwgt>`.
pub fn banner_lines(
    args: &[Arg],
    slots: &[SlotId],
    sets: &BTreeMap<i64, SetHeader>,
    nominal: usize,
    first_id: i64,
) -> Vec<String> {
    let default = args[0];
    let mut lines = Vec::new();
    let mut in_scale = false;
    let mut in_alps = false;
    let mut in_pdf: Option<i64> = None;
    for (k, arg) in args[1..].iter().enumerate() {
        let slot = slots[arg.pdf];
        let at_nominal_pdf = arg.pdf == nominal;
        if at_nominal_pdf
            && arg.alps == 1.0
            && (arg.mur != 1.0 || arg.muf != 1.0 || arg.dyn_scale != DynScale::Nominal)
        {
            if !in_scale {
                lines.push(
                    "<weightgroup name=\"Central scale variation\" combine=\"envelope\">"
                        .to_string(),
                );
                in_scale = true;
            }
        } else if in_scale && !(at_nominal_pdf && arg.alps == 1.0 && *arg != default) {
            lines.push("</weightgroup> # scale".to_string());
            in_scale = false;
        }

        if at_nominal_pdf
            && arg.mur == 1.0
            && arg.muf == 1.0
            && arg.dyn_scale == DynScale::Nominal
            && arg.alps != 1.0
        {
            if !in_alps {
                lines.push(
                    "<weightgroup name=\"Emission scale variation\" combine=\"envelope\">"
                        .to_string(),
                );
                in_alps = true;
            }
        } else if in_alps {
            lines.push("</weightgroup> # ALPS".to_string());
            in_alps = false;
        }

        if arg.mur == 1.0 && arg.muf == 1.0 && arg.dyn_scale == DynScale::Nominal && arg.alps == 1.0
        {
            if let Some(set) = sets.get(&slot.lhapdf_id) {
                if in_pdf.is_some() {
                    lines.push("</weightgroup> # PDFSET to PDFSET".to_string());
                }
                lines.push(format!(
                    "<weightgroup name=\"{}\" combine=\"{}\"> # {}: {}",
                    set.name,
                    set.error_type,
                    slot.lhapdf_id,
                    description(&set.description)
                ));
                in_pdf = Some(slot.lhapdf_id);
            } else if in_pdf.is_some_and(|open| slot.set_index() != open) {
                lines.push("</weightgroup> # PDFSET to PDF".to_string());
                in_pdf = None;
            }
        } else if in_pdf.is_some() {
            lines.push("</weightgroup> PDF ".to_string());
            in_pdf = None;
        }

        lines.push(format!(
            "<weight id=\"{}\" {}> {} </weight>",
            first_id + k as i64,
            tag(arg, slot),
            info(arg, slot, nominal)
        ));
    }
    if in_scale || in_alps || in_pdf.is_some() {
        lines.push("</weightgroup>".to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errorset_slots(set_index: i64, n: u32) -> Vec<SlotId> {
        (0..n)
            .map(|m| SlotId {
                lhapdf_id: set_index + i64::from(m),
                member: m,
            })
            .collect()
    }

    /// The default request on a 101-member set: 145 weights, the nominal
    /// re-inserted before member 1, the scale group's first and last lines
    /// as MadGraph's own `write_banner` writes them.
    #[test]
    fn the_default_request_expands_to_145_weights() {
        let request = VariationRequest::madgraph_default();
        let slots = errorset_slots(247000, 101);
        let args = expand(&request, &slots, 101, 0).unwrap();
        assert_eq!(args.len() - 1, 145);
        assert_eq!(args[45], args[0], "the re-inserted nominal");
        let mut sets = BTreeMap::new();
        sets.insert(
            247000,
            SetHeader {
                name: "NNPDF23_lo_as_0130_qed".into(),
                error_type: "replicas".into(),
                description: "a => b <c>".into(),
            },
        );
        let lines = banner_lines(&args, &slots, &sets, 0, 1);
        assert_eq!(
            lines[0],
            "<weightgroup name=\"Central scale variation\" combine=\"envelope\">"
        );
        assert_eq!(
            lines[1],
            "<weight id=\"1\" MUR=\"0.5\"  MUF=\"0.5\"  PDF=\"247000\" > MUR=0.5 MUF=0.5  </weight>"
        );
        assert_eq!(
            lines[2],
            "<weight id=\"2\" MUR=\"0.5\"  MUF=\"0.5\"  DYN_SCALE=\"1\"  PDF=\"247000\" > MUR=0.5 \
             MUF=0.5 dyn_scale_choice=sum pt  </weight>"
        );
        assert_eq!(lines[45], "</weightgroup> # scale");
        assert_eq!(
            lines[46],
            "<weightgroup name=\"NNPDF23_lo_as_0130_qed\" combine=\"replicas\"> # 247000: a ; b \
             .lt.c.gt."
        );
        assert_eq!(
            lines[47],
            "<weight id=\"45\" MUR=\"1.0\"  MUF=\"1.0\"  PDF=\"247000\" >  </weight>"
        );
        assert_eq!(
            lines[48],
            "<weight id=\"46\" MUR=\"1.0\"  MUF=\"1.0\"  PDF=\"247001\" > PDF=247000 MemberID=1 \
             </weight>"
        );
        assert_eq!(lines.last().unwrap(), "</weightgroup>");
        assert_eq!(lines.len(), 145 + 4);
    }

    /// `--pdf=errorset,central`: the second reading of the run's own member is
    /// a different slot, so it is not the nominal (quirk J).
    #[test]
    fn a_second_slot_of_the_nominal_member_is_not_the_nominal() {
        let mut request = VariationRequest::madgraph_default();
        request.together.clear();
        request.mur = vec![1.0];
        request.muf = vec![1.0];
        request.dynamical = vec![DynScale::Nominal];
        let mut slots = errorset_slots(247000, 3);
        slots.push(SlotId {
            lhapdf_id: 247000,
            member: 0,
        });
        let args = expand(&request, &slots, 4, 0).unwrap();
        // nominal, re-inserted nominal, members 1 and 2, the second reading.
        assert_eq!(args.len(), 5);
        assert_eq!(args[4].pdf, 3);
        let lines = banner_lines(&args, &slots, &BTreeMap::new(), 0, 1);
        assert_eq!(
            lines.last().unwrap(),
            "<weight id=\"4\" MUR=\"1.0\"  MUF=\"1.0\"  PDF=\"247000\" > PDF=247000 MemberID=0 \
             </weight>"
        );
    }
}
