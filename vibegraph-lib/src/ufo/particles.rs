use super::ast_util::{
    call_func_name, extract_attr, kwarg_bool, named_assignments, parse_stmts, FieldError, Fields,
    UniqueNames,
};
use indexmap::IndexMap;
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::ops::Index;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParticleError {
    #[error("Failed to parse particles.py: {0}")]
    Parse(String),
    #[error(transparent)]
    Field(#[from] FieldError),
}

/// What a particle's `mass` and `width` must be.
const PARAM_REFERENCE: &str = "a `Param.<name>` reference";

/// What a particle's `charge` may be besides a number.
const CHARGE_FORM: &str = "a number, or the `value` of a parameter imported from parameters.py";

/// The default value of the external parameter whose Python variable is the
/// argument: what `<parameter>.value` is when `particles.py` is read. `Err`
/// carries why no number can be given (an unknown variable, an internal
/// parameter).
pub(crate) type ParameterValues<'a> = &'a dyn Fn(&str) -> Result<f64, String>;

/// Strongly typed index for [`Particle`] lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleId(usize);

impl From<usize> for ParticleId {
    fn from(value: usize) -> Self {
        ParticleId(value)
    }
}

impl ParticleId {
    /// The raw index this id wraps.
    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

impl std::fmt::Display for ParticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Particle {
    /// A Goldstone mode or a ghost field: absent from tree-level diagrams in
    /// unitary gauge, which is the gauge MadGraph's default `sm` import uses.
    pub fn is_unphysical(&self) -> bool {
        self.is_ghost() || self.is_goldstone()
    }

    /// A ghost field (non-zero ghost number).
    pub fn is_ghost(&self) -> bool {
        self.ghost_number != 0
    }

    /// A Goldstone mode of a massive gauge boson.
    pub fn is_goldstone(&self) -> bool {
        self.is_goldstone
    }
}

impl Index<ParticleId> for IndexMap<String, Particle> {
    type Output = Particle;

    fn index(&self, index: ParticleId) -> &Self::Output {
        self.index(index.0)
    }
}

/// A UFO particle with full field data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Particle {
    /// Python variable name, e.g. `"e__minus__"` or `"W__plus__"`.
    pub(crate) python_name: String,
    /// UFO `name` field, e.g. `"e-"`.
    pub name: String,
    /// UFO `antiname` field, e.g. `"e+"`.
    pub(crate) antiname: String,
    pub pdg_code: i64,
    /// 2s+1; negative values denote ghost fields.
    pub spin: i32,
    /// SU(3) representation: 1 = singlet, 3 = fundamental, 8 = adjoint.
    pub color: i32,
    /// Name of the mass `Parameter`, e.g. `"MZ"` or `"ZERO"`.
    pub(crate) mass_param: String,
    /// Name of the width `Parameter`, e.g. `"WZ"` or `"ZERO"`.
    pub(crate) width_param: String,
    pub(crate) charge: f64,
    pub(crate) texname: String,
    pub(crate) antitexname: String,
    pub(crate) ghost_number: i32,
    /// UFO `goldstoneboson`/`goldstone` flag: a Goldstone mode of a massive gauge
    /// boson. Excluded from diagram enumeration (unitary gauge, like MadGraph).
    pub(crate) is_goldstone: bool,
    /// True when `name == antiname` (self-conjugate, e.g. photon, Z).
    pub(crate) is_self_conjugate: bool,
    /// Optional line style override, e.g. `line = 'dashed'`.
    pub(crate) line_style: Option<String>,
    /// Python variable name of the `propagators.py` entry this particle
    /// propagates with (`propagator = Prop.Z1`), when the model overrides the
    /// default form. Rejected — see [`crate::ufo::propagators`] — only when such
    /// a particle actually propagates in a selected diagram.
    pub propagator: Option<String>,
}

impl Particle {
    /// The helicity states an on-shell external leg of this particle sums
    /// over, as MadGraph's `NHEL` values, or `None` for spin 3/2. Spin 2 lists
    /// MadGraph's five states although no wavefunction routine for it exists
    /// here; `AmplitudeEvaluator::compile` refuses a spin-2 line, and these
    /// states serve only the enumeration-level bookkeeping (polarization
    /// parsing, the duplicate-subprocess check). The UFO spin code is `2s+1`, negative for
    /// ghosts. A massless vector has no longitudinal mode (`vxxxxx`'s massless
    /// branch defines only ±1). The values are MadGraph's
    /// `Particle.get_helicity_states` (`base_objects.py:469`) as a set; the
    /// order it reverses for an antiparticle is not kept.
    pub(crate) fn helicity_states(&self) -> Option<Vec<i32>> {
        let massless = self.mass_param == "ZERO";
        match (self.spin.abs(), massless) {
            (1, _) => Some(vec![0]),
            (2, _) => Some(vec![-1, 1]),
            (3, false) => Some(vec![-1, 0, 1]),
            (3, true) => Some(vec![-1, 1]),
            (5, _) => Some(vec![-2, -1, 0, 1, 2]),
            _ => None,
        }
    }

    /// Return the antiparticle, assigning `python_name` as its variable name.
    ///
    /// The colour follows UFO's `Particle.anti()`: a singlet or an octet is its
    /// own conjugate and keeps its `color`, a triplet or sextet negates it.
    pub(crate) fn make_anti(&self, python_name: impl Into<String>) -> Particle {
        Particle {
            python_name: python_name.into(),
            name: self.antiname.clone(),
            antiname: self.name.clone(),
            pdg_code: -self.pdg_code,
            spin: self.spin,
            color: if matches!(self.color, 1 | 8) {
                self.color
            } else {
                -self.color
            },
            mass_param: self.mass_param.clone(),
            width_param: self.width_param.clone(),
            charge: -self.charge,
            texname: self.antitexname.clone(),
            antitexname: self.texname.clone(),
            ghost_number: self.ghost_number,
            is_goldstone: self.is_goldstone,
            is_self_conjugate: self.is_self_conjugate,
            line_style: self.line_style.clone(),
            propagator: self.propagator.clone(),
        }
    }
}

/// [`parse_particles_with`] with no `parameters.py`: a `charge` naming a
/// parameter is refused.
#[cfg(test)]
pub(crate) fn parse_particles(src: &str) -> Result<Vec<Particle>, ParticleError> {
    parse_particles_with(src, &|variable| {
        Err(format!("no parameters.py is available for `{variable}`"))
    })
}

/// Parse `particles.py` content into a list of [`Particle`]s, reading a
/// `charge` that is a parameter's `value` through `parameter_value`.
///
/// Handles both direct `Particle(...)` constructor assignments and the
/// `.anti()` shorthand (`u__tilde__ = u.anti()`).  Attribute assignments
/// (e.g. loop_sm's `.counterterm = ...`) are silently skipped.
///
/// Anti-particle entries are created so vertex resolution can look up all
/// particle python-names, including those defined via `.anti()`.
///
/// MadGraph's import sets the charge to the parameter object's `value` at the
/// time `particles.py` is read, which is the default `parameters.py` gives
/// (`import_ufo.py`, `add_particle`, `particle.set(key, value)`), so the
/// charge is that default, not a card's value.
pub(crate) fn parse_particles_with(
    src: &str,
    parameter_value: ParameterValues<'_>,
) -> Result<Vec<Particle>, ParticleError> {
    use std::collections::HashMap;

    let stmts = parse_stmts(src).map_err(|e| ParticleError::Parse(e.to_string()))?;
    let mut particles: Vec<Particle> = Vec::new();
    // Index base python_name → position in particles, for resolving .anti() calls.
    let mut by_python_name: HashMap<String, usize> = HashMap::new();
    // A repeated name or variable, an antiparticle's included, is refused
    // ([`UniqueNames`]): `.anti()` of a self-conjugate particle repeats its name,
    // which UFO's `Particle.anti` refuses too ("has no anti particle").
    let mut seen = UniqueNames::new("particle");
    // `from parameters import n [as m]`: local name -> the parameter's variable.
    let imported = imported_parameters(&stmts);

    for (python_name, value) in named_assignments(&stmts) {
        let python_name = python_name.to_owned();
        match value {
            // Direct constructor: `x = Particle(...)`
            ast::Expr::Call(ast::ExprCall { func, keywords, .. })
                if call_func_name(func) == Some("Particle") =>
            {
                // Every field of `Particle.require_args` in UFO's object library
                // (`mg5amcnlo/models/sm/object_library.py` L69) is required. The
                // defaults kept are its own (L74, `goldstoneboson = False`) and
                // those of fields it does not list (`GhostNumber`, `line`,
                // `propagator`).
                let fields = Fields::new("particle", &python_name, keywords);
                let name = fields.str("name")?;
                let antiname = fields.str("antiname")?;
                let pdg_code = fields.int("pdg_code")?;
                let spin = fields.int32("spin")?;
                let color = fields.int32("color")?;
                let charge = charge(&fields, &python_name, &imported, parameter_value)?;
                let texname = fields.str("texname")?;
                let antitexname = fields.str("antitexname")?;
                let mass_param = fields.reference("mass", "Param", PARAM_REFERENCE)?;
                let width_param = fields.reference("width", "Param", PARAM_REFERENCE)?;
                let ghost_number = fields.opt_int32("GhostNumber")?.unwrap_or(0);
                // Older UFO models (sm) spell it `goldstoneboson`, newer ones `goldstone`,
                // and some FeynRules exports `GoldstoneBoson`: MadGraph's importer accepts
                // all three and treats the particle as a Goldstone when any is true.
                let is_goldstone = ["goldstoneboson", "GoldstoneBoson", "goldstone"]
                    .iter()
                    .any(|spelling| kwarg_bool(keywords, spelling) == Some(true));
                let is_self_conjugate = name == antiname;
                let line_style = extract_param_ref(keywords, "line");
                let propagator = propagator_ref(keywords);

                seen.insert(&python_name, &name)?;
                by_python_name.insert(python_name.clone(), particles.len());
                particles.push(Particle {
                    python_name,
                    name,
                    antiname,
                    pdg_code,
                    spin,
                    color,
                    mass_param,
                    width_param,
                    charge,
                    texname,
                    antitexname,
                    ghost_number,
                    is_goldstone,
                    is_self_conjugate,
                    line_style,
                    propagator,
                });
            }

            // Anti-particle shorthand: `x = y.anti()`
            ast::Expr::Call(ast::ExprCall {
                func,
                args,
                keywords,
                ..
            }) if args.is_empty()
                && keywords.is_empty()
                && matches!(func.as_ref(),
                        ast::Expr::Attribute(ast::ExprAttribute { attr, .. })
                        if attr.as_str() == "anti") =>
            {
                let base_name = match func.as_ref() {
                    ast::Expr::Attribute(ast::ExprAttribute { value, .. }) => {
                        super::ast_util::extract_name(value)
                    }
                    _ => None,
                };
                match base_name.and_then(|n| by_python_name.get(n)) {
                    Some(base) => {
                        let anti = particles[*base].make_anti(python_name.clone());
                        seen.insert(&python_name, &anti.name)?;
                        by_python_name.insert(python_name, particles.len());
                        particles.push(anti);
                    }
                    None => {
                        return Err(ParticleError::Parse(format!(
                            "anti-particle '{python_name}' references an undefined base particle"
                        )));
                    }
                }
            }

            _ => {}
        }
    }

    Ok(particles)
}

/// The names `particles.py` imports from `parameters`, as `(local name, variable)`.
fn imported_parameters(stmts: &[ast::Stmt]) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for stmt in stmts {
        let ast::Stmt::ImportFrom(ast::StmtImportFrom {
            module: Some(module),
            names,
            level,
            ..
        }) = stmt
        else {
            continue;
        };
        if module.as_str() != "parameters" || level.is_some_and(|l| l.to_u32() != 0) {
            continue;
        }
        for alias in names {
            let local = alias.asname.as_ref().unwrap_or(&alias.name);
            out.insert(local.to_string(), alias.name.to_string());
        }
    }
    out
}

/// A particle's `charge`: a number, or `<name>.value` / `Param.<name>.value`
/// for a parameter.
fn charge(
    fields: &Fields<'_>,
    particle: &str,
    imported: &std::collections::HashMap<String, String>,
    parameter_value: ParameterValues<'_>,
) -> Result<f64, ParticleError> {
    use super::ast_util::{extract_float, extract_name};
    let expr = fields.required("charge")?;
    if let Some(x) = extract_float(expr) {
        return Ok(x);
    }
    let malformed = || fields.malformed("charge", CHARGE_FORM);
    let ast::Expr::Attribute(ast::ExprAttribute {
        value: owner, attr, ..
    }) = expr
    else {
        return Err(malformed().into());
    };
    if attr.as_str() != "value" {
        return Err(malformed().into());
    }
    let variable = match (extract_name(owner), extract_attr(owner)) {
        (Some(local), _) => imported.get(local).cloned(),
        (None, Some(("Param", variable))) => Some(variable.to_owned()),
        _ => None,
    }
    .ok_or_else(malformed)?;
    parameter_value(&variable)
        .map_err(|why| ParticleError::Parse(format!("particle '{particle}': `charge` {why}")))
}

/// The `propagator` keyword's `Prop.XXX` entry. MadGraph reads a list or a dict
/// `propagator` as one entry per gauge and takes the first, the unitary-gauge one
/// (`import_ufo.py`, `add_particle`: `value[0]`).
fn propagator_ref(keywords: &[ast::Keyword]) -> Option<String> {
    use super::ast_util::get_kwarg;
    let first = |expr: &ast::Expr| extract_attr(expr).map(|(_, attr)| attr.to_owned());
    match get_kwarg(keywords, "propagator")? {
        ast::Expr::List(ast::ExprList { elts, .. })
        | ast::Expr::Tuple(ast::ExprTuple { elts, .. }) => elts.first().and_then(first),
        ast::Expr::Dict(ast::ExprDict { keys, values, .. }) => keys
            .iter()
            .zip(values)
            .find(|(key, _)| {
                matches!(key, Some(ast::Expr::Constant(ast::ExprConstant {
                    value: ast::Constant::Int(i), ..
                })) if i.to_string() == "0")
            })
            .and_then(|(_, value)| first(value)),
        other => first(other),
    }
}

/// Extract `Param.XXX` reference from a keyword argument, returning `"XXX"`.
fn extract_param_ref(keywords: &[ast::Keyword], name: &str) -> Option<String> {
    use super::ast_util::get_kwarg;
    let val = get_kwarg(keywords, name)?;
    // `Param.XXX` → Attribute { value: Name("Param"), attr: "XXX" }
    let (_, attr) = extract_attr(val)?;
    Some(attr.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
from object_library import all_particles, Particle
import parameters as Param

e__minus__ = Particle(pdg_code = 11,
                      name = 'e-',
                      antiname = 'e+',
                      spin = 2,
                      color = 1,
                      mass = Param.ZERO,
                      width = Param.ZERO,
                      texname = 'e^-',
                      antitexname = 'e^+',
                      charge = -1,
                      GhostNumber = 0,
                      LeptonNumber = 1)

Z = Particle(pdg_code = 23,
             name = 'Z',
             antiname = 'Z',
             spin = 3,
             color = 1,
             mass = Param.MZ,
             width = Param.WZ,
             texname = 'Z',
             antitexname = 'Z',
             charge = 0,
             GhostNumber = 0,
             LeptonNumber = 0)

W__plus__ = Particle(pdg_code = 24,
                     name = 'W+',
                     antiname = 'W-',
                     spin = 3,
                     color = 1,
                     mass = Param.MW,
                     width = Param.WW,
                     texname = 'W+',
                     antitexname = 'W-',
                     charge = 1,
                     GhostNumber = 0,
                     LeptonNumber = 0)

W__minus__ = W__plus__.anti()
"#;

    #[test]
    fn test_basic_particle() {
        let ps = parse_particles(SAMPLE).unwrap();
        let e = ps.iter().find(|p| p.python_name == "e__minus__").unwrap();
        assert_eq!(e.pdg_code, 11);
        assert_eq!(e.spin, 2);
        assert_eq!(e.mass_param, "ZERO");
        assert_eq!(e.width_param, "ZERO");
        assert_eq!(e.charge, -1.0);
        assert!(!e.is_self_conjugate);
    }

    #[test]
    fn test_self_conjugate() {
        let ps = parse_particles(SAMPLE).unwrap();
        let z = ps.iter().find(|p| p.python_name == "Z").unwrap();
        assert!(z.is_self_conjugate);
        assert_eq!(z.mass_param, "MZ");
        assert_eq!(z.width_param, "WZ");
    }

    #[test]
    fn test_anti_resolved() {
        let ps = parse_particles(SAMPLE).unwrap();
        // W__minus__ = W__plus__.anti() should now be included as an anti-particle entry.
        assert_eq!(ps.len(), 4);
        let wm = ps.iter().find(|p| p.python_name == "W__minus__").unwrap();
        assert_eq!(wm.name, "W-");
        assert_eq!(wm.antiname, "W+");
        assert_eq!(wm.pdg_code, -24);
    }

    /// The antiparticle's colour is UFO's `anti()`: 1 and 8 unchanged, 3 and 6
    /// negated.
    #[test]
    fn an_antiparticle_conjugates_only_complex_colour() {
        let base = parse_particles(SAMPLE).unwrap().remove(0);
        for (color, anti) in [(1, 1), (8, 8), (3, -3), (-3, 3), (6, -6), (-6, 6)] {
            let p = Particle {
                color,
                ..base.clone()
            };
            assert_eq!(p.make_anti("x").color, anti, "colour {color}");
        }
    }

    /// Across the SM, a particle and its antiparticle carry the same singlet or
    /// octet colour and opposite triplet colour.
    #[test]
    fn sm_antiparticles_carry_ufos_anti_colour() {
        let model = crate::ufo::sm::sm_model(crate::ufo::sm::SMRestrict::Default);
        let mut pairs = 0;
        for p in model.particles.values().filter(|p| !p.is_self_conjugate) {
            let anti = &model.particles[p.antiname.as_str()];
            let want = if matches!(p.color.abs(), 1 | 8) {
                p.color
            } else {
                -p.color
            };
            assert_eq!(anti.color, want, "{} / {}", p.name, anti.name);
            assert!(matches!(p.color, 1 | 8 | 3 | -3), "{}: {}", p.name, p.color);
            pairs += 1;
        }
        assert!(pairs >= 20, "only {pairs} non-self-conjugate particles");
    }

    const LOOP_SM_FRAGMENT: &str = r#"
b = Particle(pdg_code = 5,
             name = 'b',
             antiname = 'b~',
             spin = 2,
             color = 3,
             mass = Param.MB,
             width = Param.ZERO,
             texname = 'b',
             antitexname = 'b',
             charge = -1/3,
             GhostNumber = 0,
             LeptonNumber = 0)

b.counterterm = {(0,0): -1, (1,0): -2}
b.loop_particles = [[['t']]]
"#;

    #[test]
    fn test_loop_sm_attributes_skipped() {
        let ps = parse_particles(LOOP_SM_FRAGMENT).unwrap();
        // Only the Particle(...) assignment should be included
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].python_name, "b");
    }

    #[test]
    fn test_charge_fraction() {
        // charge = -1/3 is a BinOp; extract_float now handles simple arithmetic.
        let ps = parse_particles(LOOP_SM_FRAGMENT).unwrap();
        let tol = 1e-9;
        assert!(
            (ps[0].charge - (-1.0 / 3.0)).abs() < tol,
            "expected charge ≈ -1/3, got {}",
            ps[0].charge
        );
    }

    const CHARGED: &str = r#"
from object_library import all_particles, Particle
import parameters as Param
from parameters import n
from parameters import m as mm

heco = Particle(pdg_code = 10000200, name = 'HECO', antiname = 'HECO~', spin = 2, color = 1,
                mass = Param.ZERO, width = Param.ZERO, texname = 'H', antitexname = 'H~',
                charge = n.value)
other = Particle(pdg_code = 10000300, name = 'OTH', antiname = 'OTH~', spin = 2, color = 1,
                 mass = Param.ZERO, width = Param.ZERO, texname = 'O', antitexname = 'O~',
                 charge = mm.value)
third = Particle(pdg_code = 10000400, name = 'THD', antiname = 'THD~', spin = 2, color = 1,
                 mass = Param.ZERO, width = Param.ZERO, texname = 'T', antitexname = 'T~',
                 charge = Param.n.value)
heco_bar = heco.anti()
"#;

    /// `charge = <parameter>.value` is that parameter's default, through an
    /// import, an aliased import and `Param.`, and the antiparticle's is
    /// the negative.
    #[test]
    fn a_charge_that_is_a_parameters_value_is_its_default() {
        let values = |variable: &str| match variable {
            "n" => Ok(39.0),
            "m" => Ok(-2.0),
            other => Err(format!(
                "names `{other}`, which parameters.py does not define"
            )),
        };
        let ps = parse_particles_with(CHARGED, &values).unwrap();
        let charges: Vec<(&str, f64)> = ps.iter().map(|p| (p.name.as_str(), p.charge)).collect();
        assert_eq!(
            charges,
            [
                ("HECO", 39.0),
                ("OTH", -2.0),
                ("THD", 39.0),
                ("HECO~", -39.0)
            ]
        );
    }

    /// A parameter the resolver cannot give a number for is refused naming the
    /// particle.
    #[test]
    fn a_charge_the_resolver_refuses_is_refused_naming_the_particle() {
        let err = parse_particles_with(CHARGED, &|v| Err(format!("has no value for {v}")))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("'heco'") && err.contains("has no value for n"),
            "{err}"
        );
    }

    #[test]
    fn every_spelling_of_the_goldstone_flag_marks_the_particle() {
        for spelling in ["goldstoneboson", "GoldstoneBoson", "goldstone"] {
            let src = format!(
                r#"
phi__plus__ = Particle(pdg_code = 251,
                       name = 'phi+',
                       antiname = 'phi-',
                       spin = 1,
                       color = 1,
                       mass = Param.MW,
                       width = Param.ZERO,
                       texname = 'p',
                       antitexname = 'p',
                       {spelling} = True,
                       charge = 1,
                       GhostNumber = 0)
"#
            );
            let particles = parse_particles(&src).unwrap();
            assert!(particles[0].is_unphysical(), "{spelling} not recognised");
        }
    }

    #[test]
    fn a_list_or_dict_propagator_is_read_as_its_unitary_gauge_entry() {
        for form in [
            "[Prop.U, Prop.F]",
            "(Prop.U, Prop.F)",
            "{0: Prop.U, 1: Prop.F}",
        ] {
            let src = format!(
                r#"
Z = Particle(pdg_code = 23, name = 'Z', antiname = 'Z', spin = 3, color = 1,
             mass = Param.MZ, width = Param.WZ, texname = 'Z', antitexname = 'Z',
             charge = 0, GhostNumber = 0, propagator = {form})
"#
            );
            let particles = parse_particles(&src).unwrap();
            assert_eq!(particles[0].propagator.as_deref(), Some("U"), "{form}");
        }
    }
}
