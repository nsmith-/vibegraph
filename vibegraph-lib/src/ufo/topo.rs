//! Utilities for building feyngraph Models from vibegraph's parsed UFO data.
//!
//! This module provides functionality to construct a feyngraph Model from vibegraph's parsed UFO data.

use std::collections::HashSet;

use crate::ufo::{
    couplings::Coupling, lorentz::LorentzStructure, particles::Particle, vertices::Vertex,
};
use feyngraph::model::{LineStyle, Model as TopoModel, Statistic};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopoError {
    #[error("Error building feyngraph model: {0}")]
    BuildError(String),
    #[error("FeynGraph model evaluation error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
}

/// Helper function to determine feyngraph LineStyle
///
/// Follows particle rule in feyngraph/src/model/ufo_parser.rs
fn map_line_style(particle: &Particle) -> LineStyle {
    match particle.line_style {
        Some(ref style) => match style.as_str() {
            "dashed" => LineStyle::Dashed,
            "dotted" => LineStyle::Dotted,
            "straight" => LineStyle::Straight,
            "wavy" => LineStyle::Wavy,
            "curly" => LineStyle::Curly,
            "scurly" => LineStyle::Scurly,
            "swavy" => LineStyle::Swavy,
            "double" => LineStyle::Double,
            _ => LineStyle::None,
        },
        None => {
            match (particle.spin, particle.color) {
                (1, 1) => LineStyle::Dashed,   // Scalar boson
                (2, _) => LineStyle::Straight, // Fermion
                (3, 1) => LineStyle::Wavy,     // Vector boson
                (3, 8) => LineStyle::Curly,    // Gluon-like boson
                _ => LineStyle::None,
            }
        }
    }
}

/// Helper function to determine feyngraph Statistic from spin.
fn spin_to_statistic(spin: i32) -> Statistic {
    // 2s+1
    match spin.rem_euclid(2) {
        1 => Statistic::Bose,
        0 => Statistic::Fermi,
        _ => unreachable!(),
    }
}

/// Build a feyngraph Model from vibegraph's parsed UFO data.
///
/// Uses feyngraph's mutation API to construct the model without re-parsing the UFO.
pub fn build_feyngraph_model(
    particles: &IndexMap<String, Particle>,
    lorentz: &IndexMap<String, LorentzStructure>,
    couplings: &IndexMap<String, Coupling>,
    vertices: &IndexMap<String, Vertex>,
) -> Result<TopoModel, TopoError> {
    let mut model_builder = TopoModel::empty();

    // Add all particles but skip antiparticles since feyngraph's add_particle
    // automatically adds the antiparticle. If we add both, diagram generation
    // misbehaves
    let mut seen_anti = HashSet::new();
    for particle in particles.values() {
        if seen_anti.contains(&particle.name) {
            continue; // skip if we've already added the antiparticle
        }
        seen_anti.insert(&particle.antiname);
        model_builder.add_particle(
            particle.name.clone(),
            particle.antiname.clone(),
            (particle.spin - 1) as isize, // feyngraph uses 2s for spin
            particle.color as isize,
            particle.pdg_code as isize,
            particle.texname.clone(),
            particle.antitexname.clone(),
            map_line_style(particle),
            spin_to_statistic(particle.spin),
        );
    }

    for (vertex_name, vertex) in vertices {
        if vertex.lorentz.is_empty() {
            return Err(TopoError::BuildError(format!(
                "Vertex '{}' has no Lorentz structures defined",
                vertex_name
            )));
        }

        // Collect particle names for this vertex
        let particle_names: Vec<String> = vertex
            .particles
            .iter()
            .map(|&pid| particles[pid].name.clone())
            .collect();

        // Build spin_map from lorentz structures
        // Use the first (and typically only) lorentz structure's spin_map
        let lorentz_id = vertex.lorentz[0];
        let lorentz_struct = &lorentz[lorentz_id];
        let spin_map_for_vertex: Vec<isize> = lorentz_struct.spin_map.clone();

        // Build coupling orders map from vertex couplings
        let mut coupling_orders: FxHashMap<String, usize> = FxHashMap::default();

        // Process all couplings associated with this vertex
        for (_coupling_key, &coupling_id) in &vertex.couplings {
            let coupling_def = &couplings[coupling_id];

            // Extract coupling order information from the coupling definition
            // The coupling definition includes the order information in the `orders` field
            for (order_name, order_value) in &coupling_def.orders {
                coupling_orders.insert(order_name.clone(), *order_value);
            }
        }

        // the vertex should have been pruned already
        if coupling_orders.is_empty() {
            return Err(TopoError::BuildError(format!(
                "Vertex '{}' has no coupling orders defined",
                vertex_name
            )));
        }

        model_builder.add_vertex(
            vertex_name.clone(),
            particle_names,
            spin_map_for_vertex,
            coupling_orders,
        )?;
    }

    Ok(model_builder)
}
