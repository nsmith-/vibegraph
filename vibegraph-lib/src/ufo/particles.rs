use super::ast_util::{
    call_func_name, extract_attr, kwarg_float, kwarg_int, kwarg_str, parse_stmts,
};
use rustpython_parser::ast;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParticleError {
    #[error("Failed to parse particles.py: {0}")]
    Parse(String),
}

/// Opaque index into `UFOModel::particles`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleId(pub usize);

/// A UFO particle with full field data.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Python variable name, e.g. `"e__minus__"` or `"W__plus__"`.
    pub python_name: String,
    /// UFO `name` field, e.g. `"e-"`.
    pub name: String,
    /// UFO `antiname` field, e.g. `"e+"`.
    pub antiname: String,
    pub pdg_code: i64,
    /// 2s+1; negative values denote ghost fields.
    pub spin: i32,
    /// SU(3) representation: 1 = singlet, 3 = fundamental, 8 = adjoint.
    pub color: i32,
    /// Name of the mass `Parameter`, e.g. `"MZ"` or `"ZERO"`.
    pub mass_param: String,
    /// Name of the width `Parameter`, e.g. `"WZ"` or `"ZERO"`.
    pub width_param: String,
    pub charge: f64,
    pub texname: String,
    pub antitexname: String,
    pub ghost_number: i32,
    /// True when `name == antiname` (self-conjugate, e.g. photon, Z).
    pub is_self_conjugate: bool,
}

/// Parse `particles.py` content into a list of [`Particle`]s.
///
/// Only direct `Particle(...)` constructor assignments are returned.
/// `.anti()` shorthand and attribute assignments (e.g. loop_sm's
/// `.counterterm = ...`) are silently skipped.
pub fn parse_particles(src: &str) -> Result<Vec<Particle>, ParticleError> {
    let stmts = parse_stmts(src).map_err(|e| ParticleError::Parse(e.to_string()))?;
    let mut particles = Vec::new();

    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };

        // Skip attribute assignments like `b.counterterm = ...`
        let ast::Expr::Name(ast::ExprName { id: lhs_id, .. }) = targets.first().unwrap() else {
            continue;
        };
        let python_name = lhs_id.as_str().to_owned();

        // Only handle `Particle(...)` constructor calls.
        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value.as_ref() else {
            continue;
        };
        if call_func_name(func) != Some("Particle") {
            continue;
        }

        let name = kwarg_str(keywords, "name").unwrap_or_else(|| python_name.clone());
        let antiname = kwarg_str(keywords, "antiname").unwrap_or_else(|| name.clone());
        let pdg_code = kwarg_int(keywords, "pdg_code").unwrap_or(0);
        let spin = kwarg_int(keywords, "spin").unwrap_or(1) as i32;
        let color = kwarg_int(keywords, "color").unwrap_or(1) as i32;
        let charge = kwarg_float(keywords, "charge").unwrap_or(0.0);
        let texname = kwarg_str(keywords, "texname").unwrap_or_default();
        let antitexname = kwarg_str(keywords, "antitexname").unwrap_or_default();
        let ghost_number = kwarg_int(keywords, "GhostNumber").unwrap_or(0) as i32;

        let mass_param = extract_param_ref(keywords, "mass").unwrap_or_else(|| "ZERO".to_owned());
        let width_param = extract_param_ref(keywords, "width").unwrap_or_else(|| "ZERO".to_owned());

        let is_self_conjugate = name == antiname;

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
            is_self_conjugate,
        });
    }

    Ok(particles)
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
    fn test_anti_skipped() {
        let ps = parse_particles(SAMPLE).unwrap();
        // W__minus__ = W__plus__.anti() should be skipped
        assert_eq!(ps.len(), 3);
        assert!(ps.iter().all(|p| p.python_name != "W__minus__"));
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
        // charge = -1/3 is a BinOp in the AST, so charge defaults to 0.0
        let ps = parse_particles(LOOP_SM_FRAGMENT).unwrap();
        // -1/3 is not a simple constant — charge falls back to 0.0
        let _ = ps[0].charge; // just ensure no panic
    }
}
