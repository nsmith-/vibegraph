pub mod ast_util;
pub mod color;
pub mod couplings;
pub mod expr;
pub mod identity;
pub mod lorentz;
pub mod parameters;
pub mod particles;
pub mod propagators;
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
use propagators::{parse_propagators, Propagator, PropagatorError};
use serde::{Deserialize, Serialize};
use slha::ParamCard;
use std::path::Path;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
pub const OPTIONAL_SOURCE_FILES: [&str; 2] = ["coupling_orders.py", "propagators.py"];

// Default SM coupling hierarchy: QCD (strong) counts once, QED (electroweak) counts twice.
// Used when coupling_orders.py is absent or contains no hierarchy data.
fn default_sm_hierarchy() -> BTreeMap<String, u32> {
    [("QCD".to_owned(), 1u32), ("QED".to_owned(), 2u32)]
        .into_iter()
        .collect()
}

/// `coupling_orders.py` read as MadGraph reads it: a `name → hierarchy` map and a
/// `name → expansion_order` map.
///
/// Each line of the form `VAR = CouplingOrder(name='X', expansion_order=M, hierarchy=N)`
/// contributes one entry to each. MadGraph sets the model's `expansion_order`
/// only when *every* declared order carries the attribute
/// (`models/import_ufo.py`, the `try`/`except AttributeError` around the
/// `for order in all_orders` loop), so one order missing it leaves the whole map
/// empty here too. Returns empty maps on parse failure (the caller falls back to
/// defaults).
fn parse_coupling_orders(src: &str) -> (BTreeMap<String, u32>, BTreeMap<String, i64>) {
    use ast_util::{call_func_name, kwarg_int, kwarg_str, parse_stmts};
    use rustpython_parser::ast;

    let Ok(stmts) = parse_stmts(src) else {
        return (BTreeMap::new(), BTreeMap::new());
    };

    let mut hierarchy = BTreeMap::new();
    let mut expansion = BTreeMap::new();
    let mut expansion_complete = true;
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
        hierarchy.insert(
            name.clone(),
            kwarg_int(keywords, "hierarchy").unwrap_or(1) as u32,
        );
        match kwarg_int(keywords, "expansion_order") {
            Some(v) => {
                expansion.insert(name, v);
            }
            None => expansion_complete = false,
        }
    }
    if !expansion_complete {
        expansion.clear();
    }
    (hierarchy, expansion)
}

/// The coupling-order caps a model imposes on any process, keyed by order name.
///
/// MadGraph's `Process.check_expansion_orders`
/// (`madgraph/core/base_objects.py`, `tmp = [(k,v) for (k,v) in
/// expansion_orders.items() if 0 < v < 99]`) uses only the orders whose
/// `expansion_order` is **strictly** between 0 and 99: an order declared 0, 99 or
/// negative caps nothing at all.
pub fn expansion_order_caps(expansion_order: &BTreeMap<String, i64>) -> BTreeMap<String, u32> {
    expansion_order
        .iter()
        .filter(|(_, &v)| 0 < v && v < 99)
        .map(|(k, &v)| (k.clone(), v as u32))
        .collect()
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
    #[error("Propagator parse error: {0}")]
    Propagators(#[from] PropagatorError),
    #[error("particle '{particle}' names propagator '{propagator}', absent from propagators.py")]
    UnknownPropagator {
        particle: String,
        propagator: String,
    },
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
    /// `expansion_order` from `coupling_orders.py`, the model's own cap on how
    /// far a process may go in each order. Read through
    /// [`expansion_order_caps`], which applies MadGraph's `0 < v < 99` rule.
    pub expansion_order: BTreeMap<String, i64>,
    /// Custom propagator forms from `propagators.py`, keyed by Python variable
    /// name — what [`Particle::propagator`] refers to.
    pub propagators: IndexMap<String, Propagator>,
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
    pub expansion_order: BTreeMap<String, i64>,
    pub propagators: IndexMap<String, Propagator>,
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
        let vertices = resolve_vertices(raw_vertices, &particles, &lorentz, &couplings)?;
        let vertices = split_vertices_by_coupling_order(vertices, &couplings);

        // Parse coupling_orders.py for the WEIGHTED order hierarchy and the
        // model's own per-order expansion caps. Fall back to SM defaults
        // (QCD=1, QED=2) if the file is absent or unparseable.
        let (order_hierarchy, expansion_order) =
            match std::fs::read_to_string(path.join(OPTIONAL_SOURCE_FILES[0])) {
                Ok(src) => {
                    let (hierarchy, expansion) = parse_coupling_orders(&src);
                    if hierarchy.is_empty() {
                        (default_sm_hierarchy(), expansion)
                    } else {
                        (hierarchy, expansion)
                    }
                }
                Err(_) => (default_sm_hierarchy(), BTreeMap::new()),
            };

        let propagators: IndexMap<String, Propagator> =
            match std::fs::read_to_string(path.join(OPTIONAL_SOURCE_FILES[1])) {
                Ok(src) => parse_propagators(&src)?
                    .into_iter()
                    .map(|p| (p.python_name.clone(), p))
                    .collect(),
                Err(_) => IndexMap::new(),
            };
        for particle in particles.values() {
            if let Some(name) = &particle.propagator {
                if !propagators.contains_key(name) {
                    return Err(UfoError::UnknownPropagator {
                        particle: particle.name.clone(),
                        propagator: name.clone(),
                    });
                }
            }
        }

        Ok(ParsedModel {
            particles,
            lorentz,
            couplings,
            vertices,
            params,
            order_hierarchy,
            expansion_order,
            propagators,
        })
    }

    /// Bake a restrict card into the parsed model: lock the parameters it zeroes
    /// and drop what vanishes under them.
    ///
    /// The zeroed parameters (light masses/Yukawas, CKM mixing) are locked rather
    /// than merely set — MadGraph does this on `import model` and prunes
    /// vertices/diagrams against it, so a later param card must not revive them.
    /// See `apply_restrict`.
    ///
    /// The pruning follows the order of MadGraph's
    /// `import_ufo.RestrictModel.remove_interactions`: the vanishing *couplings*
    /// go first, then the vertices left with none, then the Lorentz structures no
    /// surviving coupling references (with the remaining coupling keys reindexed
    /// onto the shortened list). Colour structures are left alone, as MadGraph
    /// leaves them: an unreferenced one is never read, since every consumer
    /// enumerates colour structures through the coupling keys.
    pub fn apply_restriction(&mut self, restrict_card: &ParamCard) {
        self.params.apply_restrict(restrict_card);

        let restrict_values =
            evaluate_couplings_for_restrict(&self.params, &self.couplings, restrict_card);

        for vertex in self.vertices.values_mut() {
            vertex
                .couplings
                .retain(|_key, coup_id| !is_zero_coupling(*coup_id, &restrict_values));
        }
        self.vertices
            .retain(|_name, vertex| !vertex.couplings.is_empty());
        for vertex in self.vertices.values_mut() {
            prune_unreferenced_lorentz(vertex);
        }
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
            expansion_order: self.expansion_order,
            propagators: self.propagators,
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

/// Whether one coupling vanishes under the restrict parameters.
fn is_zero_coupling(coup_id: CouplingId, restrict_values: &HashMap<CouplingId, Complex64>) -> bool {
    restrict_values
        .get(&coup_id)
        .map(|v| v.norm() < 1e-20)
        .unwrap_or(true)
}

/// Drop the Lorentz structures no coupling of `vertex` refers to, renumbering the
/// surviving coupling keys onto the shortened list (MadGraph's tail of
/// `RestrictModel.remove_interactions`).
fn prune_unreferenced_lorentz(vertex: &mut Vertex) {
    let used: BTreeSet<usize> = vertex.couplings.keys().map(|&(_, l)| l).collect();
    if used.len() == vertex.lorentz.len() {
        return;
    }
    let renumber: HashMap<usize, usize> = used.iter().enumerate().map(|(i, &l)| (l, i)).collect();
    vertex.lorentz = used.iter().map(|&l| vertex.lorentz[l]).collect();
    vertex.couplings = std::mem::take(&mut vertex.couplings)
        .into_iter()
        .map(|((c, l), id)| ((c, renumber[&l]), id))
        .collect();
}

/// Split each UFO vertex into one [`Vertex`] per distinct coupling-order tuple,
/// each carrying only that tuple's `(color, lorentz)` couplings.
///
/// This is MadGraph's `import_ufo.UFOMG5Converter.add_interaction`, whose
/// `order_to_int` dictionary keys the interactions it emits on
/// `tuple(coupling.order.items())`. Without it a vertex bundling couplings of
/// different orders — every SMEFTsim `FFV`, which carries the SM current
/// alongside dipole and current-shift operators — would present the union of
/// their orders, and a coupling-order constraint would judge the SM current by
/// the dipole's `NP` power.
///
/// Splits are named `<vertex>#<n>`, 1-based in the order their tuples first
/// appear; a vertex with a single order tuple keeps its name and content
/// unchanged, which is every vertex of every model in `mg5amcnlo/models/`.
///
/// One deliberate difference from MadGraph: the key is the *sorted* order map,
/// where MadGraph keys on the UFO file's dict insertion order, so two couplings
/// writing the same orders in a different sequence would split apart there and
/// merge here. No model read by this loader contains such a pair.
/// One coupling-order tuple and the vertex coupling entries carrying it.
type OrderGroup<'a> = (
    &'a BTreeMap<String, usize>,
    BTreeMap<(usize, usize), CouplingId>,
);

fn split_vertices_by_coupling_order(
    vertices: IndexMap<String, Vertex>,
    couplings: &IndexMap<String, Coupling>,
) -> IndexMap<String, Vertex> {
    let mut out: IndexMap<String, Vertex> = IndexMap::with_capacity(vertices.len());
    for (name, vertex) in vertices {
        // One entry per distinct order tuple, in first-appearance order, holding
        // that tuple's `(color, lorentz) -> coupling` entries.
        let mut groups: Vec<OrderGroup<'_>> = Vec::new();
        for (&key, &coup_id) in &vertex.couplings {
            let orders = &couplings[coup_id].orders;
            match groups.iter_mut().find(|(o, _)| *o == orders) {
                Some((_, members)) => {
                    members.insert(key, coup_id);
                }
                None => groups.push((orders, [(key, coup_id)].into_iter().collect())),
            }
        }

        let single = groups.len() <= 1;
        for (n, (_, members)) in groups.into_iter().enumerate() {
            let split_name = if single {
                name.clone()
            } else {
                format!("{name}#{}", n + 1)
            };
            let mut split = Vertex {
                name: split_name.clone(),
                particles: vertex.particles.clone(),
                color: vertex.color.clone(),
                lorentz: vertex.lorentz.clone(),
                couplings: members,
            };
            prune_unreferenced_lorentz(&mut split);
            let clash = out.insert(split_name, split);
            assert!(
                clash.is_none(),
                "split interaction name collides with an existing vertex in '{name}'"
            );
        }
    }
    out
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

    /// A model defining custom propagator forms loads; the refusal moved to the
    /// point where one of them would actually be used
    /// (`ConvertError::CustomPropagator`, pinned in `diagrams::diagram`). The
    /// forms are kept verbatim and the particle keeps its reference to one.
    #[test]
    fn propagators_py_present_loads_and_attaches() {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-ufo-propagators-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for name in REQUIRED_SOURCE_FILES.iter() {
            std::fs::write(dir.join(name), "\n").unwrap();
        }
        std::fs::write(
            dir.join("particles.py"),
            "Z1 = Particle(pdg_code = 9000005, name = 'Z1', antiname = 'Z1', spin = 3,\n\
             color = 1, propagator = Prop.Z1, texname = 'Z1', antitexname = 'Z1', charge = 0)\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("propagators.py"),
            "Z1 = Propagator(name = 'Z1', numerator = '- Metric(1, 2)', denominator = 'X')\n",
        )
        .unwrap();

        let result = ParsedModel::parse(&dir);
        std::fs::remove_dir_all(&dir).unwrap();

        let parsed = result.expect("a model with propagators.py must load");
        assert_eq!(
            parsed.particles["Z1"].propagator.as_deref(),
            Some("Z1"),
            "the particle must keep its propagator reference"
        );
        assert_eq!(parsed.propagators["Z1"].numerator, "- Metric(1, 2)");
        assert_eq!(parsed.propagators["Z1"].denominator, "X");
    }

    /// A particle naming a propagator the model never defines is a broken model,
    /// caught while parsing rather than when a diagram happens to use it.
    #[test]
    fn dangling_propagator_reference_is_rejected() {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-ufo-dangling-propagator-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for name in REQUIRED_SOURCE_FILES.iter() {
            std::fs::write(dir.join(name), "\n").unwrap();
        }
        std::fs::write(
            dir.join("particles.py"),
            "Z1 = Particle(pdg_code = 9000005, name = 'Z1', antiname = 'Z1', spin = 3,\n\
             color = 1, propagator = Prop.Nope, texname = 'Z1', antitexname = 'Z1', charge = 0)\n",
        )
        .unwrap();

        let result = ParsedModel::parse(&dir);
        std::fs::remove_dir_all(&dir).unwrap();

        assert!(
            matches!(result, Err(UfoError::UnknownPropagator { .. })),
            "expected UnknownPropagator"
        );
    }

    #[test]
    fn propagators_py_absent_still_parses() {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-ufo-no-propagators-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for name in REQUIRED_SOURCE_FILES.iter() {
            std::fs::write(dir.join(name), "\n").unwrap();
        }

        let result = ParsedModel::parse(&dir);
        std::fs::remove_dir_all(&dir).unwrap();

        assert!(
            result.is_ok(),
            "a model without propagators.py must still load"
        );
        assert!(result.unwrap().propagators.is_empty());
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

    /// MadGraph's `0 < v < 99` window: an order declared `expansion_order = 0`
    /// (SMEFTsim's `NPprop`), 99, or negative caps nothing at all — only a value
    /// strictly inside the window becomes a process-order bound.
    #[test]
    fn expansion_order_caps_use_madgraphs_window() {
        let declared: BTreeMap<String, i64> = [
            ("NPprop".to_owned(), 0),
            ("QCD".to_owned(), 99),
            ("HIG".to_owned(), 1),
            ("HIW".to_owned(), 98),
            ("PERT".to_owned(), -1),
        ]
        .into_iter()
        .collect();
        let caps = expansion_order_caps(&declared);
        assert_eq!(
            caps,
            [("HIG".to_owned(), 1u32), ("HIW".to_owned(), 98)]
                .into_iter()
                .collect::<BTreeMap<_, _>>()
        );
    }

    /// Neither model this loader is gated on declares a cap, so folding
    /// `expansion_order` into diagram generation cannot move a single SM diagram.
    /// If a bumped submodule ever changes that, this is where it shows.
    #[test]
    fn the_interned_sm_declares_no_expansion_order_cap() {
        let model = sm::sm_model(sm::SMRestrict::Default);
        assert!(
            expansion_order_caps(&model.expansion_order).is_empty(),
            "SM expansion_order: {:?}",
            model.expansion_order
        );
    }

    /// The claim the interned SM blob rests on: every SM vertex carries exactly
    /// one coupling-order tuple, so interaction splitting is the identity on it —
    /// and every Lorentz structure it lists is referenced by a coupling, so the
    /// pruning is the identity too.
    #[test]
    fn splitting_is_the_identity_on_the_standard_model() {
        let parsed = sm::sm_parsed_model();
        for (name, vertex) in &parsed.vertices {
            assert!(
                !name.contains('#'),
                "vertex '{name}' was split, so it mixed coupling orders"
            );
            let mut tuples: Vec<&BTreeMap<String, usize>> = vertex
                .couplings
                .values()
                .map(|&id| &parsed.couplings[id].orders)
                .collect();
            tuples.dedup();
            assert_eq!(tuples.len(), 1, "vertex '{name}' mixes coupling orders");

            let used: BTreeSet<usize> = vertex.couplings.keys().map(|&(_, l)| l).collect();
            assert_eq!(
                used.len(),
                vertex.lorentz.len(),
                "vertex '{name}' lists a Lorentz structure no coupling references"
            );
        }
    }

    /// A vertex whose couplings carry different orders becomes one interaction per
    /// order tuple, each with that tuple's couplings and only the Lorentz
    /// structures they reference — MadGraph's `add_interaction`/`order_to_int`.
    /// This is what stops an SM photon current bundled with a dipole operator from
    /// reading as `NP = 1`.
    #[test]
    fn mixed_order_vertex_splits_per_order_tuple() {
        use couplings::Coupling;
        use expr::parse_expr;
        use vertices::Vertex;

        let coupling = |name: &str, orders: &[(&str, usize)]| Coupling {
            python_name: name.to_owned(),
            name: name.to_owned(),
            value: parse_expr("1").unwrap(),
            orders: orders.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
            deps: vec![],
        };
        let couplings: IndexMap<String, Coupling> = [
            coupling("GC_1", &[("QED", 1)]),
            coupling("GC_2", &[("NP", 1), ("QED", 1)]),
            coupling("GC_3", &[("QED", 1)]),
        ]
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect();

        // Three Lorentz structures; the SM current on 0, the dipole on 1, a second
        // SM structure on 2.
        let vertex = Vertex {
            name: "V_1".to_owned(),
            particles: vec![],
            color: vec![color::ColorExpr {
                coeff: 1,
                atoms: vec![],
            }],
            lorentz: vec![
                LorentzId::from(10),
                LorentzId::from(11),
                LorentzId::from(12),
            ],
            couplings: [
                ((0, 0), CouplingId::from(0)),
                ((0, 1), CouplingId::from(1)),
                ((0, 2), CouplingId::from(2)),
            ]
            .into_iter()
            .collect(),
        };
        let input: IndexMap<String, Vertex> = [("V_1".to_owned(), vertex)].into_iter().collect();

        let split = split_vertices_by_coupling_order(input, &couplings);
        assert_eq!(split.keys().collect::<Vec<_>>(), ["V_1#1", "V_1#2"]);

        // The QED-only split keeps both of its structures, renumbered onto 0 and 1.
        let qed = &split["V_1#1"];
        assert_eq!(qed.lorentz, vec![LorentzId::from(10), LorentzId::from(12)]);
        assert_eq!(qed.couplings.keys().collect::<Vec<_>>(), [&(0, 0), &(0, 1)]);

        // The NP split keeps only the dipole structure, renumbered onto 0.
        let np = &split["V_1#2"];
        assert_eq!(np.lorentz, vec![LorentzId::from(11)]);
        assert_eq!(np.couplings.keys().collect::<Vec<_>>(), [&(0, 0)]);
        assert_eq!(np.couplings[&(0, 0)], CouplingId::from(1));
    }
}
