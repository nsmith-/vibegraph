//! UFO vertex color-factor strings: grammar and load-time `Identity` resolution.
//!
//! A vertex's `color` list holds one string per color structure, e.g. `'1'`,
//! `'T(3,2,1)'`, `'f(-1,1,2)*f(3,4,-1)'`. Each string is a product (`*`) of
//! atoms with signed integer indices: positive indices are 1-based positions
//! in the vertex's particle list; negative indices are "summed" (only ever
//! introduced later, during diagram colorization — never present in a raw
//! UFO model file).
//!
//! `Identity(m,n)` is representation-dependent and is resolved here, at
//! parse time, using the color reps of the particles at slots `m`/`n`
//! (mirrors MadGraph's `import_ufo.treat_color`):
//! - a 3/3̄ pair becomes `T(i,j)` with the fundamental slot first;
//! - an 8/8 pair becomes `2·Tr(m,n)` (`Tr[T^aT^b] = δ^{ab}/2`);
//! - a sextet pair is an explicit unsupported error.
//!
//! The color-algebra engine (simplification, canonical forms, the CF matrix)
//! is a separate, later concern; this module only produces the parsed,
//! Identity-resolved per-vertex expression.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One factor in a color-factor product, after `Identity` resolution.
///
/// Integer indices are either 1-based vertex-particle slots (positive) or
/// summed indices (negative) — see the module docs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorAtom {
    /// `T(a1,...,an,i,j)`: `n` adjoint (octet) indices, then the fundamental
    /// index `i`, then the antifundamental index `j`.
    T(Vec<i32>, i32, i32),
    /// `Tr(a1,...,an)`: a trace over adjoint indices only, no fundamental
    /// legs. Never appears in a raw UFO color string; only produced by
    /// resolving an octet-octet `Identity`.
    Tr(Vec<i32>),
    /// `f(a,b,c)`: the totally antisymmetric SU(3) structure constant.
    F(i32, i32, i32),
    /// `d(a,b,c)`: the totally symmetric SU(3) structure constant.
    D(i32, i32, i32),
}

impl std::fmt::Display for ColorAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = |head: &str, indices: &[i32]| {
            let args: Vec<String> = indices.iter().map(i32::to_string).collect();
            format!("{head}({})", args.join(","))
        };
        match self {
            ColorAtom::T(adj, i, j) => {
                let mut indices = adj.clone();
                indices.push(*i);
                indices.push(*j);
                write!(f, "{}", joined("T", &indices))
            }
            ColorAtom::Tr(adj) => write!(f, "{}", joined("Tr", adj)),
            ColorAtom::F(a, b, c) => write!(f, "{}", joined("f", &[*a, *b, *c])),
            ColorAtom::D(a, b, c) => write!(f, "{}", joined("d", &[*a, *b, *c])),
        }
    }
}

/// A parsed, `Identity`-resolved vertex color factor: an integer coefficient
/// times a product of [`ColorAtom`]s.
///
/// The coefficient is `1` except when an octet-octet `Identity` contributed
/// its exact factor of `2`; an empty atom list represents the colorless `'1'`
/// factor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorExpr {
    pub coeff: i64,
    pub atoms: Vec<ColorAtom>,
}

impl std::fmt::Display for ColorExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.atoms.is_empty() {
            return write!(f, "{}", self.coeff);
        }
        if self.coeff != 1 {
            write!(f, "{}*", self.coeff)?;
        }
        let parts: Vec<String> = self.atoms.iter().map(ColorAtom::to_string).collect();
        write!(f, "{}", parts.join("*"))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ColorError {
    #[error("failed to parse color string '{string}': {message}")]
    Parse { string: String, message: String },
    #[error("Identity index {0} must be a positive (1-based) vertex-particle slot")]
    IdentitySlotNotPositive(i32),
    #[error("color slot {0} out of range for a vertex with {1} particle(s)")]
    SlotOutOfRange(i32, usize),
    #[error("Identity({m},{n}) pairs incompatible color representations {cm} and {cn}")]
    IdentityRepMismatch { m: i32, n: i32, cm: i32, cn: i32 },
    #[error(
        "Identity({m},{n}) touches an SU(3) sextet representation (color {rep}); \
         sextets are not supported"
    )]
    SextetUnsupported { m: i32, n: i32, rep: i32 },
}

/// One factor as parsed, before `Identity` resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RawAtom {
    Identity(i32, i32),
    T(Vec<i32>, i32, i32),
    F(i32, i32, i32),
    D(i32, i32, i32),
}

peg::parser! {
    /// PEG grammar for UFO vertex color-factor strings.
    grammar color_grammar() for str {
        pub rule color_string() -> Vec<RawAtom>
            = terms:(term() ** "*") { terms.into_iter().flatten().collect() }

        rule term() -> Option<RawAtom>
            = "1" { None }
            / a:atom() { Some(a) }

        rule atom() -> RawAtom
            = "Identity(" m:int() "," n:int() ")" { RawAtom::Identity(m, n) }
            / "T(" args:int_list() ")" {?
                if args.len() < 2 {
                    Err("T(...) needs at least two indices")
                } else {
                    let j = args[args.len() - 1];
                    let i = args[args.len() - 2];
                    Ok(RawAtom::T(args[..args.len() - 2].to_vec(), i, j))
                }
            }
            / "f(" a:int() "," b:int() "," c:int() ")" { RawAtom::F(a, b, c) }
            / "d(" a:int() "," b:int() "," c:int() ")" { RawAtom::D(a, b, c) }

        rule int_list() -> Vec<i32> = int() ** ","

        rule int() -> i32
            = n:$("-"? ['0'..='9']+) { n.parse().expect("digit-only literal parses as i32") }
    }
}

/// Parse a raw UFO color-factor string into its atom product (no `Identity`
/// resolution — that needs the vertex's particle color reps).
fn parse_color_string(s: &str) -> Result<Vec<RawAtom>, ColorError> {
    color_grammar::color_string(s.trim()).map_err(|e| ColorError::Parse {
        string: s.to_owned(),
        message: e.to_string(),
    })
}

/// The particle color rep at a 1-based vertex-particle slot.
fn slot_color(slot: i32, particle_colors: &[i32]) -> Result<i32, ColorError> {
    if slot <= 0 {
        return Err(ColorError::IdentitySlotNotPositive(slot));
    }
    particle_colors
        .get((slot - 1) as usize)
        .copied()
        .ok_or(ColorError::SlotOutOfRange(slot, particle_colors.len()))
}

/// Resolve one `Identity(m,n)` atom given the color reps at its two slots.
///
/// Mirrors MadGraph's `import_ufo.treat_color`: a 3/3̄ pair becomes `T(i,j)`
/// with the fundamental slot first; an 8/8 pair becomes `2·Tr(m,n)`; sextets
/// are rejected.
fn resolve_identity(
    m: i32,
    n: i32,
    particle_colors: &[i32],
) -> Result<(i64, ColorAtom), ColorError> {
    let cm = slot_color(m, particle_colors)?;
    let cn = slot_color(n, particle_colors)?;
    match (cm, cn) {
        (3, -3) => Ok((1, ColorAtom::T(vec![], m, n))),
        (-3, 3) => Ok((1, ColorAtom::T(vec![], n, m))),
        (8, 8) => Ok((2, ColorAtom::Tr(vec![m, n]))),
        (6, _) | (_, 6) | (-6, _) | (_, -6) => {
            let rep = if cm == 6 || cm == -6 { cm } else { cn };
            Err(ColorError::SextetUnsupported { m, n, rep })
        }
        _ => Err(ColorError::IdentityRepMismatch { m, n, cm, cn }),
    }
}

/// Resolve every `Identity` atom in a parsed color string, given the color
/// reps of all of the vertex's particles (indexed by 1-based slot).
fn resolve(atoms: Vec<RawAtom>, particle_colors: &[i32]) -> Result<ColorExpr, ColorError> {
    let mut coeff: i64 = 1;
    let mut resolved = Vec::with_capacity(atoms.len());
    for atom in atoms {
        match atom {
            RawAtom::Identity(m, n) => {
                let (c, a) = resolve_identity(m, n, particle_colors)?;
                coeff *= c;
                resolved.push(a);
            }
            RawAtom::T(adj, i, j) => resolved.push(ColorAtom::T(adj, i, j)),
            RawAtom::F(a, b, c) => resolved.push(ColorAtom::F(a, b, c)),
            RawAtom::D(a, b, c) => resolved.push(ColorAtom::D(a, b, c)),
        }
    }
    Ok(ColorExpr {
        coeff,
        atoms: resolved,
    })
}

/// Parse a vertex color-factor string and resolve any `Identity` atom
/// against the vertex's particle color reps (indexed by 1-based slot).
pub(crate) fn parse_and_resolve(s: &str, particle_colors: &[i32]) -> Result<ColorExpr, ColorError> {
    let atoms = parse_color_string(s)?;
    resolve(atoms, particle_colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<RawAtom> {
        parse_color_string(s).unwrap_or_else(|e| panic!("failed to parse '{s}': {e}"))
    }

    // ── §1d SM color vocabulary: raw-grammar goldens ──────────────────────

    #[test]
    fn golden_colorless() {
        assert_eq!(parse("1"), vec![]);
    }

    #[test]
    fn golden_identity() {
        assert_eq!(parse("Identity(1,2)"), vec![RawAtom::Identity(1, 2)]);
    }

    #[test]
    fn golden_t_qqg() {
        assert_eq!(parse("T(3,2,1)"), vec![RawAtom::T(vec![3], 2, 1)]);
    }

    #[test]
    fn golden_f_ggg() {
        assert_eq!(parse("f(1,2,3)"), vec![RawAtom::F(1, 2, 3)]);
    }

    #[test]
    fn golden_gggg_chain_and_perms() {
        assert_eq!(
            parse("f(-1,1,2)*f(3,4,-1)"),
            vec![RawAtom::F(-1, 1, 2), RawAtom::F(3, 4, -1)]
        );
        assert_eq!(
            parse("f(-1,1,3)*f(2,4,-1)"),
            vec![RawAtom::F(-1, 1, 3), RawAtom::F(2, 4, -1)]
        );
        assert_eq!(
            parse("f(-1,1,4)*f(2,3,-1)"),
            vec![RawAtom::F(-1, 1, 4), RawAtom::F(2, 3, -1)]
        );
    }

    #[test]
    fn d_atom_parses() {
        assert_eq!(parse("d(1,2,3)"), vec![RawAtom::D(1, 2, 3)]);
    }

    #[test]
    fn unknown_atom_is_hard_error() {
        let err = parse_color_string("K6(1,2)").unwrap_err();
        assert!(matches!(err, ColorError::Parse { .. }));
    }

    #[test]
    fn t_needs_at_least_two_indices() {
        assert!(parse_color_string("T(1)").is_err());
    }

    // ── §1d Identity resolution, mirroring MG's treat_color ───────────────

    /// V_71-like: particles = [d~ (-3), d (3), a (1)], color = 'Identity(1,2)'.
    /// MG resolves this to `T(2,1)` (fundamental slot 2 first).
    #[test]
    fn identity_33bar_fundamental_second_slot() {
        let resolved = parse_and_resolve("Identity(1,2)", &[-3, 3, 1]).unwrap();
        assert_eq!(
            resolved,
            ColorExpr {
                coeff: 1,
                atoms: vec![ColorAtom::T(vec![], 2, 1)],
            }
        );
        assert_eq!(resolved.to_string(), "T(2,1)");
    }

    /// Same pair, slots swapped: fundamental now in the first slot.
    #[test]
    fn identity_33bar_fundamental_first_slot() {
        let resolved = parse_and_resolve("Identity(1,2)", &[3, -3, 1]).unwrap();
        assert_eq!(
            resolved,
            ColorExpr {
                coeff: 1,
                atoms: vec![ColorAtom::T(vec![], 1, 2)],
            }
        );
        assert_eq!(resolved.to_string(), "T(1,2)");
    }

    /// V_74-like: particles = [d~ (-3), d (3), g (8)], color = 'T(3,2,1)'
    /// (already a raw `T`, not an `Identity` — passes through unchanged).
    #[test]
    fn raw_t_passes_through() {
        let resolved = parse_and_resolve("T(3,2,1)", &[-3, 3, 8]).unwrap();
        assert_eq!(
            resolved,
            ColorExpr {
                coeff: 1,
                atoms: vec![ColorAtom::T(vec![3], 2, 1)],
            }
        );
        assert_eq!(resolved.to_string(), "T(3,2,1)");
    }

    /// Octet-octet `Identity` becomes `2*Tr(m,n)` (`Tr[T^aT^b] = δ^{ab}/2`).
    #[test]
    fn identity_octet_pair_gets_trace_and_factor_of_two() {
        let resolved = parse_and_resolve("Identity(1,2)", &[8, 8, 1]).unwrap();
        assert_eq!(
            resolved,
            ColorExpr {
                coeff: 2,
                atoms: vec![ColorAtom::Tr(vec![1, 2])],
            }
        );
        assert_eq!(resolved.to_string(), "2*Tr(1,2)");
    }

    #[test]
    fn identity_sextet_is_unsupported() {
        let err = parse_and_resolve("Identity(1,2)", &[6, -6, 1]).unwrap_err();
        assert!(matches!(err, ColorError::SextetUnsupported { .. }));
    }

    #[test]
    fn identity_mismatched_reps_errors() {
        let err = parse_and_resolve("Identity(1,2)", &[3, 3, 1]).unwrap_err();
        assert!(matches!(err, ColorError::IdentityRepMismatch { .. }));
    }

    #[test]
    fn identity_nonpositive_slot_errors() {
        let err = parse_and_resolve("Identity(0,1)", &[3, -3]).unwrap_err();
        assert!(matches!(err, ColorError::IdentitySlotNotPositive(0)));
    }

    #[test]
    fn identity_slot_out_of_range_errors() {
        let err = parse_and_resolve("Identity(1,5)", &[3, -3]).unwrap_err();
        assert!(matches!(err, ColorError::SlotOutOfRange(5, 2)));
    }

    #[test]
    fn colorless_round_trips() {
        let resolved = parse_and_resolve("1", &[1, 1]).unwrap();
        assert_eq!(
            resolved,
            ColorExpr {
                coeff: 1,
                atoms: vec![]
            }
        );
        assert_eq!(resolved.to_string(), "1");
    }
}
