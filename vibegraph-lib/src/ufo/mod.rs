pub mod ast_util;
pub mod color;
pub mod couplings;
pub mod expr;
pub mod identity;
pub mod lorentz;
pub mod parameters;
pub mod particles;
pub mod slha;
pub mod sm;
pub mod topo;
pub mod vertices;

use couplings::{parse_couplings, Coupling, CouplingError, CouplingId};
use feyngraph::model::Model as TopoModel;
use identity::model_digest;
use indexmap::IndexMap;
use lorentz::{parse_lorentz, LorentzError, LorentzId, LorentzStructure};
use num_complex::Complex64;
use parameters::{parse_parameters, ParameterError, ParameterSet};
use particles::{parse_particles, Particle, ParticleError, ParticleId};
use serde::{Deserialize, Serialize};
use slha::ParamCard;
use std::path::Path;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use thiserror::Error;
use vertices::{parse_vertices, RawVertex, Vertex, VertexError, VertexId};

use topo::build_feyngraph_model;

/// UFO source files [`ParsedModel::parse`] requires, in read order.
pub const REQUIRED_SOURCE_FILES: [&str; 5] = [
    "particles.py",
    "lorentz.py",
    "couplings.py",
    "parameters.py",
    "vertices.py",
];

/// UFO source files [`ParsedModel::parse`] reads when present, falling back to a
/// built-in default when absent.
pub const OPTIONAL_SOURCE_FILES: [&str; 1] = ["coupling_orders.py"];

// Default SM coupling hierarchy: QCD (strong) counts once, QED (electroweak) counts twice.
// Used when coupling_orders.py is absent or contains no hierarchy data.
fn default_sm_hierarchy() -> BTreeMap<String, u32> {
    [("QCD".to_owned(), 1u32), ("QED".to_owned(), 2u32)]
        .into_iter()
        .collect()
}

/// Parse `coupling_orders.py` and return a `name → hierarchy` map.
///
/// Each line of the form `VAR = CouplingOrder(name='X', hierarchy=N, ...)` contributes
/// one entry. Returns an empty map on parse failure (caller should fall back to defaults).
fn parse_coupling_orders_hierarchy(src: &str) -> BTreeMap<String, u32> {
    use ast_util::{call_func_name, kwarg_int, kwarg_str, parse_stmts};
    use rustpython_parser::ast;

    let Ok(stmts) = parse_stmts(src) else {
        return BTreeMap::new();
    };

    let mut map = BTreeMap::new();
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
    FeynGraph(#[from] topo::TopoError),
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
    /// Coupling order hierarchy from `coupling_orders.py` (e.g. QCD→1, QED→2).
    /// Used to compute the WEIGHTED coupling order for automatic order selection.
    pub order_hierarchy: BTreeMap<String, u32>,
}

/// The parsed, pre-restriction UFO model data.
///
/// Holds everything that is independent of a specific restrict card and is
/// serializable — i.e. everything in [`UFOModel`] except the feyngraph `topo`
/// model (which is rebuilt by [`ParsedModel::into_model`]). Vertices are the
/// full unpruned set; parameters have not yet had a restriction baked in.
#[derive(Clone, Serialize, Deserialize)]
pub struct ParsedModel {
    pub particles: IndexMap<String, Particle>,
    pub lorentz: IndexMap<String, LorentzStructure>,
    pub couplings: IndexMap<String, Coupling>,
    pub vertices: IndexMap<String, Vertex>,
    pub params: ParameterSet,
    pub order_hierarchy: BTreeMap<String, u32>,
}

impl ParsedModel {
    /// Parse a UFO model directory into its pre-restriction form: no vertex
    /// pruning, no topology model.
    pub fn parse(path: &Path) -> Result<Self, UfoError> {
        let read = |name: &str| -> Result<String, UfoError> {
            std::fs::read_to_string(path.join(name)).map_err(|e| UfoError::Io {
                file: path.join(name).display().to_string(),
                cause: e,
            })
        };

        let [particles_src, lorentz_src, couplings_src, params_src, vertices_src] =
            REQUIRED_SOURCE_FILES.map(read);
        let particles_src = particles_src?;
        let lorentz_src = lorentz_src?;
        let couplings_src = couplings_src?;
        let params_src = params_src?;
        let vertices_src = vertices_src?;

        let particles: IndexMap<String, Particle> = parse_particles(&particles_src)?
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        let lorentz: IndexMap<String, LorentzStructure> = parse_lorentz(&lorentz_src)?
            .into_iter()
            .map(|l| (l.name.clone(), l))
            .collect();

        let couplings: IndexMap<String, Coupling> = parse_couplings(&couplings_src)?
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        let params = parse_parameters(&params_src)?;
        let raw_vertices = parse_vertices(&vertices_src)?;
        let vertices: IndexMap<String, Vertex> =
            resolve_vertices(raw_vertices, &particles, &lorentz, &couplings)?;

        // Parse coupling_orders.py for the WEIGHTED order hierarchy.
        // Fall back to SM defaults (QCD=1, QED=2) if the file is absent or unparseable.
        let order_hierarchy = match std::fs::read_to_string(path.join(OPTIONAL_SOURCE_FILES[0])) {
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

        Ok(ParsedModel {
            particles,
            lorentz,
            couplings,
            vertices,
            params,
            order_hierarchy,
        })
    }

    /// Bake a restrict card into the parsed model: lock the parameters it zeroes
    /// and drop the vertices whose couplings vanish under them.
    ///
    /// The zeroed parameters (light masses/Yukawas, CKM mixing) are locked rather
    /// than merely set — MadGraph does this on `import model` and prunes
    /// vertices/diagrams against it, so a later param card must not revive them.
    /// See `apply_restrict`.
    pub fn apply_restriction(&mut self, restrict_card: &ParamCard) {
        self.params.apply_restrict(restrict_card);

        let restrict_values =
            evaluate_couplings_for_restrict(&self.params, &self.couplings, restrict_card);

        self.vertices.retain(|_name, vertex| {
            !is_zero_coupling_vertex(vertex, &self.couplings, &restrict_values)
        });
    }

    /// Apply a restrict card and build the feyngraph topology model.
    ///
    /// With `restrict = None`, no pruning happens — the full vertex set is kept.
    pub fn into_model(mut self, restrict: Option<&ParamCard>) -> Result<UFOModel, UfoError> {
        if let Some(restrict_card) = restrict {
            self.apply_restriction(restrict_card);
        }

        // Build the feyngraph model using vibegraph's parsed UFO data
        let topo = build_feyngraph_model(
            &self.particles,
            &self.lorentz,
            &self.couplings,
            &self.vertices,
        )?;

        Ok(UFOModel {
            particles: self.particles,
            lorentz: self.lorentz,
            couplings: self.couplings,
            vertices: self.vertices,
            params: self.params,
            topo,
            order_hierarchy: self.order_hierarchy,
        })
    }
}

impl UFOModel {
    /// Load a UFO model from a directory path.
    ///
    /// If `restrict_card` is `None`, automatically looks for `restrict_default.dat`
    /// in the UFO directory. If found, it is used for vertex pruning (zero-coupling vertices are removed).
    pub fn load(path: &Path, restrict_card: Option<&Path>) -> Result<Arc<Self>, UfoError> {
        Self::load_with_digest(path, restrict_card).map(|(model, _)| model)
    }

    /// [`load`](Self::load), also returning the [`model_digest`] of the restricted
    /// model it built.
    ///
    /// Loading and identifying share this one path so the digest can never
    /// describe a different model than the one returned.
    pub fn load_with_digest(
        path: &Path,
        restrict_card: Option<&Path>,
    ) -> Result<(Arc<Self>, String), UfoError> {
        let mut parsed = ParsedModel::parse(path)?;

        // Resolve the restrict card path: explicit, else restrict_default.dat if present.
        let restrict_card_path = match restrict_card {
            Some(path) => Some(path.to_path_buf()),
            None => {
                let default = path.join("restrict_default.dat");
                default.exists().then_some(default)
            }
        };

        let card = match restrict_card_path {
            Some(restrict_path) => {
                Some(
                    ParamCard::from_file(&restrict_path).map_err(|e| UfoError::Io {
                        file: restrict_path.display().to_string(),
                        cause: std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("restrict card parse error: {}", e),
                        ),
                    })?,
                )
            }
            None => None,
        };

        if let Some(card) = card.as_ref() {
            parsed.apply_restriction(card);
        }
        let digest = model_digest(&parsed);
        // The restriction is already baked in, so `into_model` only builds topology.
        let model = parsed.into_model(None)?;
        Ok((Arc::new(model), digest))
    }

    /// Load with automatic restrict card discovery (equivalent to `load(path, None)`).
    pub fn load_auto(path: &Path) -> Result<Arc<Self>, UfoError> {
        Self::load(path, None)
    }

    // ── Name → index lookup ───────────────────────────────────────────────────

    /// Get a ParticleId by its name
    pub fn particle_id(&self, name: &str) -> Option<ParticleId> {
        self.particles.get_index_of(name).map(ParticleId::from)
    }

    /// Get a LorentzId by its name
    pub fn lorentz_id(&self, name: &str) -> Option<LorentzId> {
        self.lorentz.get_index_of(name).map(LorentzId::from)
    }

    /// Get a CouplingId by its name
    pub fn coupling_id(&self, name: &str) -> Option<CouplingId> {
        self.couplings.get_index_of(name).map(CouplingId::from)
    }

    /// Get a VertexId by its name
    pub fn vertex_id(&self, name: &str) -> Option<VertexId> {
        self.vertices.get_index_of(name).map(VertexId)
    }

    // ── Index → value accessors ───────────────────────────────────────────────

    pub fn particle(&self, id: ParticleId) -> &Particle {
        &self.particles[id]
    }

    pub fn lorentz_struct(&self, id: LorentzId) -> &LorentzStructure {
        &self.lorentz[id]
    }

    pub fn coupling_def(&self, id: CouplingId) -> &Coupling {
        &self.couplings[id]
    }

    pub fn vertex_def(&self, id: VertexId) -> &Vertex {
        &self.vertices[id.0]
    }
}

/// Resolve raw vertex definitions into `Vertex`es
///
/// Returns values with proper IDs, validating that all referenced particles,
/// Lorentz structures, and couplings exist in the provided maps.
/// Errors if any referenced particle, Lorentz structure, or coupling is missing from the provided maps.
fn resolve_vertices(
    input: impl IntoIterator<Item = RawVertex>,
    particles: &IndexMap<String, Particle>,
    lorentz: &IndexMap<String, LorentzStructure>,
    couplings: &IndexMap<String, Coupling>,
) -> Result<IndexMap<String, Vertex>, VertexError> {
    // Vertices link to their particles, structures, and couplings by their Python variable names
    // The rest of the library uses the object names (e.g. "P.e__minus__" → "e-"), so we resolve here to get the correct names and IDs.
    let particle_id_map = particles
        .values()
        .enumerate()
        .map(|(i, p)| (p.python_name.clone(), ParticleId::from(i)))
        .collect::<HashMap<_, _>>();
    let lorentz_id_map = lorentz
        .values()
        .enumerate()
        .map(|(i, l)| (l.python_name.clone(), LorentzId::from(i)))
        .collect::<HashMap<_, _>>();
    let coupling_id_map = couplings
        .values()
        .enumerate()
        .map(|(i, c)| (c.python_name.clone(), CouplingId::from(i)))
        .collect::<HashMap<_, _>>();

    input
        .into_iter()
        .map(|rv| {
            let particle_ids: Vec<ParticleId> = rv
                .particles
                .iter()
                .map(|py_name| {
                    particle_id_map
                        .get(py_name.as_str())
                        .copied()
                        .ok_or(VertexError::Parse(format!(
                            "vertex references nonexistent particle '{py_name}'"
                        )))
                })
                .collect::<Result<_, _>>()?;

            let lorentz_ids = rv
                .lorentz
                .iter()
                .map(|py_name| {
                    lorentz_id_map
                        .get(py_name.as_str())
                        .copied()
                        .ok_or_else(|| {
                            VertexError::Parse(format!(
                                "vertex references nonexistent Lorentz structure '{py_name}'"
                            ))
                        })
                })
                .collect::<Result<_, _>>()?;

            let coupling_ids = rv
                .couplings
                .iter()
                .map(|(key, py_name)| {
                    coupling_id_map
                        .get(py_name.as_str())
                        .map(|&i| (*key, i))
                        .ok_or(VertexError::Parse(format!(
                            "vertex references nonexistent coupling '{py_name}'"
                        )))
                })
                .collect::<Result<_, _>>()?;

            let particle_colors: Vec<i32> =
                particle_ids.iter().map(|&id| particles[id].color).collect();
            let color: Vec<color::ColorExpr> = rv
                .color
                .iter()
                .map(|s| {
                    color::parse_and_resolve(s, &particle_colors)
                        .map_err(|e| VertexError::Parse(format!("vertex '{}': {e}", rv.name)))
                })
                .collect::<Result<_, _>>()?;

            let vertex = Vertex {
                name: rv.name,
                particles: particle_ids,
                color,
                lorentz: lorentz_ids,
                couplings: coupling_ids,
            };
            Ok((vertex.name.clone(), vertex))
        })
        .collect::<Result<_, _>>()
}

/// Evaluated parameter and coupling values for a specific parameter set (e.g. from a param_card).
///
/// `Clone` is cheap in the model itself (an `Arc` bump) and copies the parameter and
/// coupling value tables, so a thread that moves parameters per event owns its own
/// copy rather than sharing one.
#[derive(Clone)]
pub struct EvaluatedModel {
    model: Arc<UFOModel>,
    /// All parameter values (external + internal), keyed by parameter name.
    ///
    /// TODO: intern parameter names into UFOModel and use ParameterId here instead of string keys
    /// (then, as for the other Ids, we allow to expect the parameter exists)
    pub param_values: HashMap<String, Complex64>,
    /// Coupling constant values indexed by [`CouplingId`], parallel to `model.couplings`.
    coupling_values: Vec<Complex64>,
}

impl EvaluatedModel {
    /// Construct an `EvaluatedModel` from a `UFOModel` and a parameter card
    ///
    /// Evaluates all parameter values and coupling constants according to the given parameter card.
    pub fn from_model_card(model: Arc<UFOModel>, param_card: &ParamCard) -> Self {
        let param_values = model.params.evaluate(param_card);

        let coupling_values: Vec<Complex64> = model
            .couplings
            .values()
            .map(|c| expr::eval(&c.value, &param_values))
            .collect();

        EvaluatedModel {
            model,
            param_values,
            coupling_values,
        }
    }

    /// Construct an `EvaluatedModel` from a `UFOModel` with default parameters
    pub fn from_model(model: Arc<UFOModel>) -> Self {
        let empty_card = ParamCard::default();
        Self::from_model_card(model, &empty_card)
    }

    /// The model these values were evaluated from.
    pub fn model(&self) -> &Arc<UFOModel> {
        &self.model
    }

    /// The strong coupling these values were evaluated at, or `None` for a model
    /// with no `aS` parameter.
    pub fn alpha_s(&self) -> Option<f64> {
        self.param_values.get("aS").map(|v| v.re)
    }

    /// Move the strong coupling to `alpha_s`, re-evaluating every parameter and
    /// coupling that depends on it.
    ///
    /// The model-level half of MadGraph's per-event `update_as_param`: `G` and the
    /// couplings built from it follow `aS` through the UFO expressions themselves, so
    /// the result is exact for any model, at the cost of walking the parameter graph.
    pub fn set_alpha_s(&mut self, alpha_s: f64) {
        self.recompute("aS", Complex64::new(alpha_s, 0.0));
    }

    /// Get the mass of a particle
    pub fn mass(&self, id: ParticleId) -> f64 {
        self.param_values
            .get(&self.model.particle(id).mass_param)
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get the decay width of a particle
    pub fn width(&self, id: ParticleId) -> f64 {
        self.param_values
            .get(&self.model.particle(id).width_param)
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get a coupling constant
    pub fn coupling(&self, id: CouplingId) -> Complex64 {
        self.coupling_values[id]
    }

    /// Get the coupling entries for a vertex by its name.
    ///
    /// Returns `[(color_idx, lorentz_idx, value)]` or `None` if unknown.
    pub fn vertex_couplings(&self, id: VertexId) -> Option<Vec<(usize, usize, Complex64)>> {
        let vertex = self.model.vertex_def(id);
        let entries = vertex
            .couplings
            .iter()
            .map(|((c, l), &coup_id)| {
                let val = self.coupling_values[coup_id];
                (*c, *l, val)
            })
            .collect();
        Some(entries)
    }

    /// Re-evaluate only the parameters transitively depending on `changed`,
    /// then re-evaluate all coupling values that depend on any changed parameter.
    pub fn recompute(&mut self, changed: &str, new_value: Complex64) {
        // Update the changed parameter value, allowing error if it doesn't exist (caller should only call with known parameters)
        self.param_values
            .get_mut(changed)
            .map(|v| *v = new_value)
            .unwrap_or_else(|| panic!("attempted to recompute unknown parameter '{changed}'"));
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

/// Evaluate coupling constants under the parameters specified by a restrict card,
/// for the purpose of pruning zero-coupling vertices.
fn evaluate_couplings_for_restrict(
    params: &ParameterSet,
    couplings: &IndexMap<String, Coupling>,
    restrict_card: &ParamCard,
) -> HashMap<CouplingId, Complex64> {
    let param_values = params.evaluate(restrict_card);

    couplings
        .iter()
        .enumerate()
        .map(|(i, (_, c))| {
            let val = expr::eval(&c.value, &param_values);
            (CouplingId::from(i), val)
        })
        .collect()
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

    #[test]
    fn test_load_sm_ufo() {
        let model = sm::sm_model(sm::SMRestrict::Default);
        let ev = EvaluatedModel::from_model(model.clone());

        let as_val = ev.param_values["aS"].re;
        assert!((as_val - 0.118).abs() < 1e-10, "aS = {as_val}");

        let expected_g = 2.0 * (0.118f64).sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!((g_val - expected_g).abs() < 1e-6, "G = {g_val}");

        let gc10 = ev.coupling(model.coupling_id("GC_10").expect("no GC_10 in model"));
        assert!((gc10.re + expected_g).abs() < 1e-6, "GC_10 = {gc10}");
        assert!(gc10.im.abs() < 1e-10);

        let mz = ev.mass(model.particle_id("Z").expect("missing param"));
        assert!((mz - 91.1876).abs() < 0.01, "MZ = {mz}");

        let ma = ev.mass(model.particle_id("a").expect("missing param"));
        assert!(ma.abs() < 1e-10, "m_photon = {ma}");

        // Verify the new index-based lookup
        let e_id = model.particle_id("e-");
        assert!(e_id.is_some(), "e- not found in particle index");
    }

    #[test]
    fn test_recompute_propagates() {
        let model = sm::sm_model(sm::SMRestrict::Default);
        let mut ev = EvaluatedModel::from_model(model.clone());

        let new_as = 0.130f64;
        ev.recompute("aS", Complex64::new(new_as, 0.0));

        let expected_g = 2.0 * new_as.sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!(
            (g_val - expected_g).abs() < 1e-6,
            "After recompute: G = {g_val}"
        );

        let gc10 = ev.coupling(model.coupling_id("GC_10").expect("missing coupling"));
        assert!(
            (gc10.re + expected_g).abs() < 1e-6,
            "After recompute: GC_10 = {gc10}"
        );
    }
}
