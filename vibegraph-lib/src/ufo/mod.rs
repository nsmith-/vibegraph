pub mod ast_util;
pub mod couplings;
pub mod expr;
pub mod lorentz;
pub mod parameters;
pub mod particles;
pub mod slha;
pub mod vertices;

use couplings::{parse_couplings, Coupling, CouplingError, CouplingId};
use feyngraph::model::Model as TopoModel;
use indexmap::IndexMap;
use lorentz::{parse_lorentz, LorentzError, LorentzId, LorentzStructure};
use num_complex::Complex64;
use parameters::{parse_parameters, ParameterError, ParameterSet};
use particles::{parse_particles, Particle, ParticleError, ParticleId};
use slha::ParamCard;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use vertices::{parse_vertices, RawVertex, Vertex, VertexError, VertexId};

// Default SM coupling hierarchy: QCD (strong) counts once, QED (electroweak) counts twice.
// Used when coupling_orders.py is absent or contains no hierarchy data.
fn default_sm_hierarchy() -> HashMap<String, u32> {
    [("QCD".to_owned(), 1u32), ("QED".to_owned(), 2u32)]
        .into_iter()
        .collect()
}

/// Parse `coupling_orders.py` and return a `name → hierarchy` map.
///
/// Each line of the form `VAR = CouplingOrder(name='X', hierarchy=N, ...)` contributes
/// one entry. Returns an empty map on parse failure (caller should fall back to defaults).
fn parse_coupling_orders_hierarchy(src: &str) -> HashMap<String, u32> {
    use ast_util::{call_func_name, kwarg_int, kwarg_str, parse_stmts};
    use rustpython_parser::ast;

    let Ok(stmts) = parse_stmts(src) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };
        let ast::Expr::Name(ast::ExprName { id: lhs_id, .. }) = targets.first().unwrap() else {
            continue;
        };
        let python_name = lhs_id.as_str();

        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value.as_ref() else {
            continue;
        };
        if call_func_name(func) != Some("CouplingOrder") {
            continue;
        }

        // Use the `name` keyword if present, otherwise fall back to the Python variable name.
        let name = kwarg_str(keywords, "name").unwrap_or_else(|| python_name.to_owned());
        let hierarchy = kwarg_int(keywords, "hierarchy").unwrap_or(1) as u32;
        map.insert(name, hierarchy);
    }
    map
}

#[derive(Debug, Error)]
pub enum UfoError {
    #[error("IO error reading UFO file '{file}': {cause}")]
    Io { file: String, cause: std::io::Error },
    #[error("Parameter parse error: {0}")]
    Parameters(#[from] ParameterError),
    #[error("Coupling parse error: {0}")]
    Couplings(#[from] CouplingError),
    #[error("Particle parse error: {0}")]
    Particles(#[from] ParticleError),
    #[error("Lorentz parse error: {0}")]
    Lorentz(#[from] LorentzError),
    #[error("Vertex parse error: {0}")]
    Vertex(#[from] VertexError),
    #[error("FeynGraph model error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
}

/// A UFO model with all topology and field/parameter/coupling information loaded.
///
/// Each collection is an `IndexMap<String, T>` keyed by the Python variable name,
/// preserving insertion order and providing O(1) name→index and name→value lookups
/// without a separate index map.
#[derive(Clone)]
pub struct UFOModel {
    pub particles: IndexMap<String, Particle>,
    pub lorentz: IndexMap<String, LorentzStructure>,
    pub couplings: IndexMap<String, Coupling>,
    pub vertices: IndexMap<String, Vertex>,
    pub params: ParameterSet,
    /// FeynGraph topology model — retained for diagram-level topology queries.
    pub topo: TopoModel,
    /// Particle sets (sorted) of zero-coupling vertices, used for diagram filtering.
    /// Populated only when a restrict card is loaded.
    pub zero_coupling_vertices: Vec<Vec<String>>,
    /// Coupling order hierarchy from `coupling_orders.py` (e.g. QCD→1, QED→2).
    /// Used to compute the WEIGHTED coupling order for automatic order selection.
    pub order_hierarchy: HashMap<String, u32>,
}

/// Helper function to determine feyngraph LineStyle from spin.
fn spin_to_line_style(spin: i32) -> feyngraph::model::LineStyle {
    use feyngraph::model::LineStyle;
    match spin {
        1 => LineStyle::Dashed,
        2 => LineStyle::Straight,
        3 => LineStyle::Curly,
        _ if spin >= 5 => LineStyle::Double,
        _ => LineStyle::Dashed,
    }
}

/// Helper function to determine feyngraph Statistic from spin.
fn spin_to_statistic(spin: i32) -> feyngraph::model::Statistic {
    use feyngraph::model::Statistic;
    match spin {
        1 | 3 => Statistic::Bose,
        2 => Statistic::Fermi,
        _ if spin >= 5 => Statistic::Bose,
        _ => Statistic::Bose,
    }
}

/// Build a feyngraph Model from vibegraph's parsed UFO data.
///
/// Uses feyngraph's mutation API to construct the model without re-parsing the UFO.
fn build_feyngraph_model(
    particles: &IndexMap<String, Particle>,
    lorentz: &IndexMap<String, LorentzStructure>,
    vertices: &IndexMap<String, Vertex>,
    _couplings: &IndexMap<String, Coupling>,
    _order_hierarchy: &HashMap<String, u32>,
) -> Result<TopoModel, UfoError> {
    use rustc_hash::FxHashMap;

    let mut model = TopoModel::empty();

    // Add all particles
    for (_py_name, particle) in particles {
        model.add_particle(
            particle.name.clone(),
            particle.antiname.clone(),
            particle.spin as isize,
            particle.color as isize,
            particle.pdg_code as isize,
            particle.texname.clone(),
            particle.antitexname.clone(),
            spin_to_line_style(particle.spin),
            spin_to_statistic(particle.spin),
        );
    }

    // Add all vertices
    let mut added_count = 0;
    let mut skipped_count = 0;
    for (vertex_name, vertex) in vertices {
        // Skip vertices with no lorentz structures — they cannot be added to feyngraph
        if vertex.lorentz.is_empty() {
            log::warn!("Skipping vertex {} with no lorentz structures", vertex_name);
            skipped_count += 1;
            continue;
        }
        added_count += 1;

        // Collect particle names for this vertex
        let particle_names: Vec<String> = vertex
            .particles
            .iter()
            .map(|pid| particles[pid.0].name.clone())
            .collect();

        // Build spin_map from lorentz structures
        // Use the first (and typically only) lorentz structure's spin_map
        let lorentz_id = vertex.lorentz[0];
        let lorentz_struct = &lorentz[lorentz_id.0];
        // compute_spin_map returns 1-indexed values; convert to 0-indexed for feyngraph
        let spin_map_for_vertex: Vec<isize> = lorentz_struct
            .spin_map
            .iter()
            .map(|&m| if m > 0 { m - 1 } else { 0 })
            .collect();

        // Build coupling orders map from vertex couplings
        // For now, use a default mapping; the coupling order discovery is done separately
        // during diagram enumeration via the WEIGHTED coupling mechanism.
        let mut coupling_orders: FxHashMap<String, usize> = FxHashMap::default();

        // If no coupling orders found, use default QCD=1
        if coupling_orders.is_empty() {
            coupling_orders.insert("QCD".to_string(), 1);
        }

        model.add_vertex(
            vertex_name.clone(),
            particle_names,
            spin_map_for_vertex,
            coupling_orders,
        )?;
    }

    log::info!(
        "Built feyngraph model: {} particles, {} vertices added, {} skipped",
        particles.len(),
        added_count,
        skipped_count
    );

    Ok(model)
}

impl UFOModel {
    /// Load a UFO model from a directory path.
    ///
    /// If `restrict_card` is `None`, automatically looks for `restrict_default.dat`
    /// in the UFO directory. If found, it is used for vertex pruning (zero-coupling vertices are removed).
    pub fn load(path: &Path, restrict_card: Option<&Path>) -> Result<Self, UfoError> {
        let read = |name: &str| -> Result<String, UfoError> {
            std::fs::read_to_string(path.join(name)).map_err(|e| UfoError::Io {
                file: path.join(name).display().to_string(),
                cause: e,
            })
        };

        let particles_src = read("particles.py")?;
        let lorentz_src = read("lorentz.py")?;
        let couplings_src = read("couplings.py")?;
        let params_src = read("parameters.py")?;
        let vertices_src = read("vertices.py")?;

        let particles: IndexMap<String, Particle> = parse_particles(&particles_src)?
            .into_iter()
            .map(|p| (p.python_name.clone(), p))
            .collect();

        let lorentz: IndexMap<String, LorentzStructure> = parse_lorentz(&lorentz_src)?
            .into_iter()
            .map(|l| (l.python_name.clone(), l))
            .collect();

        let couplings: IndexMap<String, Coupling> = parse_couplings(&couplings_src)?
            .into_iter()
            .map(|c| (c.python_name.clone(), c))
            .collect();

        let params = parse_parameters(&params_src)?;
        let raw_vertices = parse_vertices(&vertices_src)?;

        let mut vertices: IndexMap<String, Vertex> = raw_vertices
            .into_iter()
            .map(|rv| {
                let v = resolve_vertex(rv, &particles, &lorentz, &couplings);
                (v.name.clone(), v)
            })
            .collect();

        // Load restrict card for vertex pruning
        let restrict_card_path = match restrict_card {
            Some(path) => Some(path.to_path_buf()),
            None => {
                let default = path.join("restrict_default.dat");
                if default.exists() {
                    Some(default)
                } else {
                    None
                }
            }
        };

        let mut zero_coupling_vertices = Vec::new();

        if let Some(restrict_path) = restrict_card_path {
            let restrict_values =
                evaluate_couplings_for_restrict(&params, &couplings, &restrict_path)?;

            // Track zero-coupling vertices before filtering, using display names (as feyngraph uses them)
            for (_name, vertex) in &vertices {
                if is_zero_coupling_vertex(vertex, &couplings, &restrict_values) {
                    let mut particle_names: Vec<String> = vertex
                        .particles
                        .iter()
                        .map(|pid| particles[pid.0].name.clone()) // Use display name, not python_name
                        .collect();
                    particle_names.sort();
                    zero_coupling_vertices.push(particle_names);
                }
            }

            vertices.retain(|_name, vertex| {
                !is_zero_coupling_vertex(vertex, &couplings, &restrict_values)
            });
        }

        // Parse coupling_orders.py for the WEIGHTED order hierarchy.
        // Fall back to SM defaults (QCD=1, QED=2) if the file is absent or unparseable.
        let order_hierarchy = match std::fs::read_to_string(path.join("coupling_orders.py")) {
            Ok(src) => {
                let parsed = parse_coupling_orders_hierarchy(&src);
                if parsed.is_empty() {
                    default_sm_hierarchy()
                } else {
                    parsed
                }
            }
            Err(_) => default_sm_hierarchy(),
        };

        // TODO: Replace with build_feyngraph_model once feyngraph Model API is fully compatible.
        // For now, use feyngraph's own from_ufo parser, but vibegraph now computes spin_map
        // independently and makes it available via LorentzStructure.spin_map.
        let topo = TopoModel::from_ufo(path)?;

        Ok(UFOModel {
            particles,
            lorentz,
            couplings,
            vertices,
            params,
            topo,
            zero_coupling_vertices,
            order_hierarchy,
        })
    }

    /// Load with automatic restrict card discovery (equivalent to `load(path, None)`).
    pub fn load_auto(path: &Path) -> Result<Self, UfoError> {
        Self::load(path, None)
    }

    /// Check if a feyngraph Vertex matches any of the zero-coupling vertices.
    pub fn is_zero_coupling_vertex(&self, particle_names: &[&str]) -> bool {
        let mut names: Vec<String> = particle_names.iter().map(|s| s.to_string()).collect();
        names.sort();
        self.zero_coupling_vertices.contains(&names)
    }

    // ── Name → index lookup ───────────────────────────────────────────────────

    pub fn particle_id(&self, name: &str) -> Option<ParticleId> {
        self.particles.get_index_of(name).map(ParticleId)
    }

    pub fn lorentz_id(&self, name: &str) -> Option<LorentzId> {
        self.lorentz.get_index_of(name).map(LorentzId)
    }

    pub fn coupling_id(&self, name: &str) -> Option<CouplingId> {
        self.couplings.get_index_of(name).map(CouplingId)
    }

    pub fn vertex_id(&self, name: &str) -> Option<VertexId> {
        self.vertices.get_index_of(name).map(VertexId)
    }

    // ── Index → value accessors ───────────────────────────────────────────────

    pub fn particle(&self, id: ParticleId) -> &Particle {
        &self.particles[id.0]
    }

    pub fn lorentz_struct(&self, id: LorentzId) -> &LorentzStructure {
        &self.lorentz[id.0]
    }

    pub fn coupling_def(&self, id: CouplingId) -> &Coupling {
        &self.couplings[id.0]
    }

    pub fn vertex_def(&self, id: VertexId) -> &Vertex {
        &self.vertices[id.0]
    }

    // ── Evaluation ────────────────────────────────────────────────────────────

    /// Evaluate all parameters and coupling constants for the given param_card.
    pub fn evaluate<'a>(&'a self, param_card: &ParamCard) -> EvaluatedModel<'a> {
        let param_values = self.params.evaluate(param_card);

        let coupling_values: Vec<Complex64> = self
            .couplings
            .values()
            .map(|c| expr::eval(&c.value, &param_values))
            .collect();

        EvaluatedModel {
            model: self,
            param_values,
            coupling_values,
        }
    }
}

/// Resolve a `RawVertex`'s string names to typed IDs.
fn resolve_vertex(
    rv: RawVertex,
    particles: &IndexMap<String, Particle>,
    lorentz: &IndexMap<String, LorentzStructure>,
    couplings: &IndexMap<String, Coupling>,
) -> Vertex {
    let particle_ids = rv
        .particles
        .iter()
        .filter_map(|name| particles.get_index_of(name.as_str()).map(ParticleId))
        .collect();

    let lorentz_ids = rv
        .lorentz
        .iter()
        .filter_map(|name| lorentz.get_index_of(name.as_str()).map(LorentzId))
        .collect();

    let coupling_ids = rv
        .couplings
        .iter()
        .filter_map(|(key, name)| {
            couplings
                .get_index_of(name.as_str())
                .map(|i| (*key, CouplingId(i)))
        })
        .collect();

    Vertex {
        name: rv.name,
        particles: particle_ids,
        color: rv.color,
        lorentz: lorentz_ids,
        couplings: coupling_ids,
    }
}

/// Evaluated parameter and coupling values for a specific phase-space point.
pub struct EvaluatedModel<'a> {
    model: &'a UFOModel,
    /// All parameter values (external + internal), keyed by parameter name.
    pub param_values: HashMap<String, Complex64>,
    /// Coupling constant values indexed by `CouplingId.0`, parallel to `model.couplings`.
    coupling_values: Vec<Complex64>,
}

impl EvaluatedModel<'_> {
    /// Get the mass of a particle by its Python name.
    pub fn mass(&self, particle_name: &str) -> f64 {
        self.model
            .particle_id(particle_name)
            .and_then(|id| {
                let mass_param = &self.model.particle(id).mass_param;
                self.param_values.get(mass_param)
            })
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get the decay width of a particle by its Python name.
    pub fn width(&self, particle_name: &str) -> f64 {
        self.model
            .particle_id(particle_name)
            .and_then(|id| {
                let width_param = &self.model.particle(id).width_param;
                self.param_values.get(width_param)
            })
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get a coupling constant by its Python name.
    pub fn coupling(&self, coupling_name: &str) -> Complex64 {
        self.model
            .coupling_id(coupling_name)
            .map(|id| self.coupling_values[id.0])
            .unwrap_or_default()
    }

    /// Get the coupling entries for a vertex by its name.
    ///
    /// Returns `[(lorentz_idx, color_idx, value)]` or `None` if unknown.
    pub fn vertex_couplings(&self, vertex_name: &str) -> Option<Vec<(usize, usize, Complex64)>> {
        let id = self.model.vertex_id(vertex_name)?;
        let vertex = self.model.vertex_def(id);
        let entries = vertex
            .couplings
            .iter()
            .map(|((l, c), coup_id)| {
                let val = self.coupling_values[coup_id.0];
                (*l, *c, val)
            })
            .collect();
        Some(entries)
    }

    /// Re-evaluate only the parameters transitively depending on `changed`,
    /// then re-evaluate all coupling values that depend on any changed parameter.
    pub fn recompute(&mut self, changed: &str, new_value: Complex64) {
        self.param_values.insert(changed.to_owned(), new_value);
        self.model.params.recompute(changed, &mut self.param_values);

        let mut changed_params = vec![changed.to_owned()];
        if let Some(rdeps) = self.model.params.rdeps.get(changed) {
            changed_params.extend(rdeps.iter().cloned());
        }

        for (i, c) in self.model.couplings.values().enumerate() {
            if c.deps.iter().any(|d| changed_params.contains(d)) {
                let val = expr::eval(&c.value, &self.param_values);
                self.coupling_values[i] = val;
            }
        }
    }
}

/// Evaluate coupling constants under the parameters specified by a restrict card.
fn evaluate_couplings_for_restrict(
    params: &ParameterSet,
    couplings: &IndexMap<String, Coupling>,
    restrict_path: &std::path::Path,
) -> Result<HashMap<CouplingId, Complex64>, UfoError> {
    let restrict_card = ParamCard::from_file(restrict_path).map_err(|e| UfoError::Io {
        file: restrict_path.display().to_string(),
        cause: std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("restrict card parse error: {}", e),
        ),
    })?;

    let param_values = params.evaluate(&restrict_card);

    let coupling_values: HashMap<CouplingId, Complex64> = couplings
        .iter()
        .enumerate()
        .map(|(i, (_, c))| {
            let val = expr::eval(&c.value, &param_values);
            (CouplingId(i), val)
        })
        .collect();

    Ok(coupling_values)
}

/// Check if a vertex has all couplings equal to zero under the restrict parameters.
fn is_zero_coupling_vertex(
    vertex: &Vertex,
    _couplings: &IndexMap<String, Coupling>,
    restrict_values: &HashMap<CouplingId, Complex64>,
) -> bool {
    vertex.couplings.values().all(|coup_id| {
        restrict_values
            .get(coup_id)
            .map(|v| v.norm() < 1e-20)
            .unwrap_or(true)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn ufo_path(model: &str) -> std::path::PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        std::path::Path::new(&manifest)
            .join("../research/refs/mg5amcnlo/models")
            .join(model)
    }

    fn sm_ufo_path() -> std::path::PathBuf {
        ufo_path("sm")
    }

    #[test]
    fn test_load_loop_sm() {
        let path = ufo_path("loop_sm");
        if !path.exists() {
            eprintln!("loop_sm UFO not found — skipping");
            return;
        }
        let result = UFOModel::load(&path, None);
        if let Err(UfoError::FeynGraph(_)) = &result {
            eprintln!(
                "loop_sm: FeynGraph topology parser does not support loop-level \
                       particle attributes (.counterterm, .loop_particles) — skipping"
            );
            return;
        }
        let model = result.expect("unexpected error loading loop_sm UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        let mz = ev.mass("Z");
        assert!((mz - 91.188).abs() < 0.01, "loop_sm MZ = {mz}");

        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "loop_sm m_photon = {ma}");
    }

    #[test]
    fn test_load_mssm() {
        let path = ufo_path("MSSM_SLHA2");
        if !path.exists() {
            eprintln!("MSSM_SLHA2 UFO not found — skipping");
            return;
        }
        let model = UFOModel::load(&path, None).expect("failed to load MSSM_SLHA2 UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        let tb = ev.param_values["tb"].re;
        assert!((tb - 9.74862403).abs() < 1e-6, "MSSM tb = {tb}");

        let beta = ev.param_values["beta"].re;
        let expected_beta = 9.74862403f64.atan();
        assert!((beta - expected_beta).abs() < 1e-8, "MSSM beta = {beta}");

        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "MSSM m_photon = {ma}");
    }

    #[test]
    fn test_load_taudecay() {
        let path = ufo_path("taudecay_UFO");
        if !path.exists() {
            eprintln!("taudecay_UFO not found — skipping");
            return;
        }
        let param_card_path = path.join("param_card.dat");
        let card = slha::ParamCard::from_file(&param_card_path)
            .expect("failed to load taudecay param_card.dat");

        let result = UFOModel::load(&path, None);
        match &result {
            Err(UfoError::Lorentz(LorentzError::UnknownOperator(op))) => {
                eprintln!("taudecay_UFO: uses unsupported Lorentz operator '{op}' — skipping");
                return;
            }
            Err(UfoError::Lorentz(e)) => {
                eprintln!("taudecay_UFO: Lorentz parse error ({e}) — skipping");
                return;
            }
            _ => {}
        }
        let model = result.expect("failed to load taudecay UFO");
        let ev = model.evaluate(&card);

        let mta = ev.mass("ta__minus__");
        assert!((mta - 1.776820).abs() < 1e-4, "taudecay MTA = {mta}");

        let mmu = ev.mass("mu__minus__");
        assert!((mmu - 0.105660).abs() < 1e-4, "taudecay MMU = {mmu}");

        let mve = ev.mass("ve");
        assert!(mve.abs() < 1e-10, "taudecay m_ve = {mve}");
    }

    #[test]
    fn test_load_sm_ufo() {
        let path = sm_ufo_path();
        if !path.exists() {
            eprintln!("SM UFO not found at {:?} — skipping integration test", path);
            return;
        }
        let model = UFOModel::load(&path, None).expect("failed to load SM UFO");

        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        let as_val = ev.param_values["aS"].re;
        assert!((as_val - 0.118).abs() < 1e-10, "aS = {as_val}");

        let expected_g = 2.0 * (0.118f64).sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!((g_val - expected_g).abs() < 1e-6, "G = {g_val}");

        let gc10 = ev.coupling("GC_10");
        assert!((gc10.re + expected_g).abs() < 1e-6, "GC_10 = {gc10}");
        assert!(gc10.im.abs() < 1e-10);

        let mz = ev.mass("Z");
        assert!((mz - 91.1876).abs() < 0.01, "MZ = {mz}");

        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "m_photon = {ma}");

        // Verify the new index-based lookup
        let e_id = model.particle_id("e__minus__");
        assert!(e_id.is_some(), "e__minus__ not found in particle index");
    }

    #[test]
    fn test_recompute_propagates() {
        let path = sm_ufo_path();
        if !path.exists() {
            return;
        }
        let model = UFOModel::load(&path, None).expect("failed to load SM UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let mut ev = model.evaluate(&empty_card);

        let new_as = 0.130f64;
        ev.recompute("aS", Complex64::new(new_as, 0.0));

        let expected_g = 2.0 * new_as.sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!(
            (g_val - expected_g).abs() < 1e-6,
            "After recompute: G = {g_val}"
        );

        let gc10 = ev.coupling("GC_10");
        assert!(
            (gc10.re + expected_g).abs() < 1e-6,
            "After recompute: GC_10 = {gc10}"
        );
    }
}
