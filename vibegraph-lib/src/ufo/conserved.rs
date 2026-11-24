//! The additive quantum numbers every interaction of a model conserves.
//!
//! A number `q` gives each particle an integer, its antiparticle the opposite
//! one and a self-conjugate particle zero. It is conserved when the particles
//! of every vertex diagrams are built from sum to zero. In a tree diagram
//! every internal line joins a particle at one vertex to its antiparticle at
//! another, so summing the vertex totals leaves the external legs alone: a
//! subprocess whose outgoing total differs from its incoming total, in any
//! conserved number, has no diagram.
//!
//! The numbers are read off the vertex table rather than listed: they are the
//! integer null space of the vertex-by-particle-pair matrix, solved exactly
//! over the rationals. With a diagonal CKM matrix the Standard Model conserves
//! electric charge, the three lepton flavours and the three quark-generation
//! numbers; a CKM restriction leaves charge, the lepton flavours and baryon
//! number; a model whose vertices violate any of them conserves only what its
//! vertices leave. Nothing about the Standard Model is assumed.
//!
//! The vertices read are the ones diagram enumeration builds its topology
//! model from ([`Vertex::is_physical`]), after the restrict card has pruned
//! the vanishing ones. A Goldstone mode or ghost takes no part: it carries
//! zero in every number.

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

use super::particles::{Particle, ParticleId};
use super::vertices::Vertex;

/// The conserved additive numbers of a model, as a basis of the space they
/// span: each row is one number's value on every particle, indexed by
/// [`ParticleId`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ConservedNumbers {
    values: Vec<Vec<i64>>,
}

impl ConservedNumbers {
    /// The numbers `vertices` conserve over `particles`.
    pub(crate) fn of_vertices(
        particles: &IndexMap<String, Particle>,
        vertices: &IndexMap<String, Vertex>,
    ) -> Self {
        // One unknown per particle-antiparticle pair; a self-conjugate or
        // unphysical particle has none. `sign` is +1 on the pair's first
        // particle in model order and -1 on the other.
        let mut pair: Vec<Option<(usize, i64)>> = vec![None; particles.len()];
        let mut pairs = 0usize;
        for (index, p) in particles.values().enumerate() {
            if p.is_unphysical() || pair[index].is_some() || p.antiname == p.name {
                continue;
            }
            let Some(anti) = particles.get_index_of(&p.antiname) else {
                continue;
            };
            pair[index] = Some((pairs, 1));
            pair[anti] = Some((pairs, -1));
            pairs += 1;
        }
        let mut rows: Vec<Vec<i64>> = Vec::new();
        for vertex in vertices.values().filter(|v| v.is_physical(particles)) {
            let mut row = vec![0i64; pairs];
            for &id in &vertex.particles {
                if let Some((k, sign)) = pair[id.index()] {
                    row[k] += sign;
                }
            }
            if row.iter().any(|&v| v != 0) && !rows.contains(&row) {
                rows.push(row);
            }
        }
        let basis = integer_null_space(&rows, pairs);
        let values = basis
            .into_iter()
            .map(|b| {
                pair.iter()
                    .map(|slot| slot.map_or(0, |(k, sign)| sign * b[k]))
                    .collect()
            })
            .collect();
        ConservedNumbers { values }
    }

    /// The numbers `values`, each one's value on every particle indexed by
    /// [`ParticleId`], whether or not the model conserves them.
    #[cfg(test)]
    pub(crate) fn from_values(values: Vec<Vec<i64>>) -> Self {
        ConservedNumbers { values }
    }

    /// Whether no number is conserved.
    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Whether every conserved number totals the same over the incoming legs
    /// `initial` as over the outgoing legs `final_state`: false means the model
    /// gives the subprocess no diagram.
    pub(crate) fn balanced(&self, initial: &[ParticleId], final_state: &[ParticleId]) -> bool {
        self.values.iter().all(|q| {
            let total = |legs: &[ParticleId]| -> i128 {
                legs.iter().map(|&id| i128::from(q[id.index()])).sum()
            };
            total(initial) == total(final_state)
        })
    }
}

/// A basis of the rational null space of `rows` (each of length `columns`),
/// each vector scaled to coprime integers. A vector with an entry outside
/// `i64` is left out, which only drops a number, never adds one.
fn integer_null_space(rows: &[Vec<i64>], columns: usize) -> Vec<Vec<i64>> {
    let mut m: Vec<Vec<BigRational>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|&v| BigRational::from_integer(BigInt::from(v)))
                .collect()
        })
        .collect();
    let pivots = reduce(&mut m, columns);
    let mut basis = Vec::new();
    for free in (0..columns).filter(|c| !pivots.contains(c)) {
        let mut v = vec![BigRational::zero(); columns];
        v[free] = BigRational::one();
        for (row, &p) in pivots.iter().enumerate() {
            v[p] = -m[row][free].clone();
        }
        if let Some(integer) = to_coprime_integers(&v) {
            basis.push(integer);
        }
    }
    basis
}

/// Reduce `m` in place to reduced row-echelon form over its first `columns`
/// columns and return the pivot column of each leading row, in order.
fn reduce(m: &mut [Vec<BigRational>], columns: usize) -> Vec<usize> {
    let mut pivots = Vec::new();
    let mut row = 0;
    for col in 0..columns {
        let Some(found) = (row..m.len()).find(|&r| !m[r][col].is_zero()) else {
            continue;
        };
        m.swap(row, found);
        let lead = m[row][col].clone();
        for v in m[row].iter_mut() {
            *v = &*v / &lead;
        }
        for r in 0..m.len() {
            if r != row && !m[r][col].is_zero() {
                let factor = m[r][col].clone();
                let (pivot_row, other) = if r < row {
                    let (a, b) = m.split_at_mut(row);
                    (&b[0], &mut a[r])
                } else {
                    let (a, b) = m.split_at_mut(r);
                    (&a[row], &mut b[0])
                };
                for (o, p) in other.iter_mut().zip(pivot_row) {
                    *o = &*o - &factor * p;
                }
            }
        }
        pivots.push(col);
        row += 1;
        if row == m.len() {
            break;
        }
    }
    pivots
}

/// `v` times the least common multiple of its denominators, divided by the
/// greatest common divisor of the result, with its first non-zero entry
/// positive; `None` if an entry does not fit in `i64`.
fn to_coprime_integers(v: &[BigRational]) -> Option<Vec<i64>> {
    let lcm = v.iter().fold(BigInt::one(), |acc, x| acc.lcm(x.denom()));
    let scaled: Vec<BigInt> = v.iter().map(|x| x.numer() * (&lcm / x.denom())).collect();
    let gcd = scaled.iter().fold(BigInt::zero(), |acc, x| acc.gcd(x));
    if gcd.is_zero() {
        return None;
    }
    let sign = match scaled.iter().find(|x| !x.is_zero()) {
        Some(first) if first.is_negative() => -BigInt::one(),
        _ => BigInt::one(),
    };
    scaled.iter().map(|x| (x * &sign / &gcd).to_i64()).collect()
}

/// The rank of `rows` over the rationals.
#[cfg(test)]
pub(crate) fn rank(rows: &[Vec<i64>]) -> usize {
    let Some(columns) = rows.first().map(Vec::len) else {
        return 0;
    };
    let mut m: Vec<Vec<BigRational>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|&v| BigRational::from_integer(BigInt::from(v)))
                .collect()
        })
        .collect();
    reduce(&mut m, columns).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::UFOModel;

    /// A number on every physical particle of `model` from its PDG code;
    /// antiparticles (negative codes) take the opposite value, and a Goldstone
    /// mode or ghost zero.
    fn by_pdg(model: &UFOModel, value: impl Fn(i64) -> i64) -> Vec<i64> {
        model
            .particles
            .values()
            .map(|p| {
                if p.is_unphysical() {
                    0
                } else {
                    p.pdg_code.signum() * value(p.pdg_code.abs())
                }
            })
            .collect()
    }

    fn lepton(model: &UFOModel, generation: i64) -> Vec<i64> {
        by_pdg(model, |pdg| {
            i64::from(pdg == 9 + 2 * generation || pdg == 10 + 2 * generation)
        })
    }

    fn quark_generation(model: &UFOModel, generation: i64) -> Vec<i64> {
        by_pdg(model, |pdg| {
            i64::from(pdg == 2 * generation - 1 || pdg == 2 * generation)
        })
    }

    fn three_charge(model: &UFOModel) -> Vec<i64> {
        model
            .particles
            .values()
            .map(|p| {
                if p.is_unphysical() {
                    0
                } else {
                    (3.0 * p.charge).round() as i64
                }
            })
            .collect()
    }

    /// Whether the derived numbers span exactly the space `expected` spans.
    fn spans(derived: &ConservedNumbers, expected: &[Vec<i64>]) -> bool {
        let mut both = derived.values.clone();
        both.extend_from_slice(expected);
        let r = rank(expected);
        rank(&derived.values) == r && rank(&both) == r
    }

    /// With a diagonal CKM matrix the vertex table conserves electric charge, the
    /// three lepton flavours and the three quark-generation numbers, and nothing
    /// else: seven numbers, spanning exactly those. It cannot see a number the
    /// model conserves that is not additive (a parity).
    #[test]
    fn the_diagonal_standard_model_conserves_charge_lepton_flavour_and_quark_generation() {
        let model = sm_model(SMRestrict::NoBMass);
        let derived = model.conserved_numbers();
        let mut expected = vec![three_charge(&model)];
        expected.extend((1..=3).map(|g| lepton(&model, g)));
        expected.extend((1..=3).map(|g| quark_generation(&model, g)));
        assert_eq!(derived.values.len(), 7, "{:?}", derived.values);
        assert!(spans(derived, &expected), "{:?}", derived.values);
    }

    /// The CKM restriction's W vertices mix the quark generations, so only
    /// baryon number survives of the three generation numbers: five numbers,
    /// fewer than the diagonal model's seven, from the vertex table alone.
    #[test]
    fn quark_mixing_leaves_baryon_number_and_no_generation_number() {
        let model = sm_model(SMRestrict::Ckm);
        let derived = model.conserved_numbers();
        let baryon = by_pdg(&model, |pdg| i64::from((1..=6).contains(&pdg)));
        let mut expected = vec![three_charge(&model), baryon];
        expected.extend((1..=3).map(|g| lepton(&model, g)));
        assert_eq!(derived.values.len(), 5, "{:?}", derived.values);
        assert!(spans(derived, &expected), "{:?}", derived.values);
        for g in 1..=3 {
            assert!(!spans(derived, &[quark_generation(&model, g)]));
        }
    }

    /// Every derived number sums to zero over every vertex enumeration reads.
    #[test]
    fn every_derived_number_is_conserved_at_every_vertex() {
        for variant in [SMRestrict::NoBMass, SMRestrict::Ckm, SMRestrict::Default] {
            let model = sm_model(variant);
            for q in &model.conserved_numbers().values {
                for (_, vertex) in model.unitary_gauge_vertices() {
                    let total: i64 = vertex.particles.iter().map(|&id| q[id.index()]).sum();
                    assert_eq!(total, 0, "{variant:?}: {}", vertex.name);
                }
            }
        }
    }

    /// The null space of a small system, exactly: `x₀ = x₁` and `x₁ = x₂` leave
    /// `(1, 1, 1)`, and a fractional pivot is scaled to coprime integers.
    #[test]
    fn the_null_space_is_exact_and_integer() {
        assert_eq!(
            integer_null_space(&[vec![1, -1, 0], vec![0, 1, -1]], 3),
            vec![vec![1, 1, 1]]
        );
        assert_eq!(integer_null_space(&[vec![2, 3]], 2), vec![vec![3, -2]]);
        assert_eq!(integer_null_space(&[], 2), vec![vec![1, 0], vec![0, 1]]);
        assert!(integer_null_space(&[vec![1, 0], vec![0, 1]], 2).is_empty());
    }

    /// An assignment balances when every number totals alike on both sides.
    #[test]
    fn balance_compares_incoming_with_outgoing_totals() {
        let model = sm_model(SMRestrict::NoBMass);
        let ids = |names: &[&str]| -> Vec<ParticleId> {
            names
                .iter()
                .map(|n| model.particle_id(n).unwrap())
                .collect()
        };
        let numbers = model.conserved_numbers();
        assert!(numbers.balanced(&ids(&["u", "d~"]), &ids(&["e+", "ve"])));
        assert!(!numbers.balanced(&ids(&["u", "d~"]), &ids(&["e+", "vm"])));
        assert!(!numbers.balanced(&ids(&["u", "s~"]), &ids(&["e+", "ve"])));
        assert!(sm_model(SMRestrict::Ckm)
            .conserved_numbers()
            .balanced(&ids(&["u", "s~"]), &ids(&["e+", "ve"])));
        assert!(ConservedNumbers::default().balanced(&ids(&["u"]), &ids(&["e+"])));
    }
}
