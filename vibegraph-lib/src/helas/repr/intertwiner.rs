//! Spin(1,3)-equivariant maps between representation fibers (intertwiners).
//!
//! ## What is an intertwiner?
//!
//! A Feynman rule for a vertex factor is not just a matrix — it is an
//! *intertwiner*: a Spin(1,3)-equivariant linear map between the fibers of the
//! bundles attached to each leg of the vertex.  Equivariance means the map
//! commutes with the group action, i.e. it preserves Lorentz covariance.
//!
//! For example, the QED vertex `ieγ^μ` intertwines:
//!
//! \`\`\`text
//! γ^μ : S* ⊗ S → T*M        (fermion current → photon leg)
//! \`\`\`
//!
//! where `S` is the Dirac spinor bundle and `T*M` is the cotangent bundle
//! (spin-(½,½)).
//!
//! ## Orientations
//!
//! Each intertwiner has multiple *orientations* depending on which legs are
//! incoming vs. outgoing and which particle species are flowing through them:
//!
//! - **Both fermions incoming** (e.g. two on-shell fermion lines in a current):
//!   produces an off-shell vector.
//! - **One fermion in, one fermion out** (e.g. off-shell fermion propagation):
//!   produces an off-shell spinor.
//! - **Both fermions outgoing** (e.g. building a fermion-loop contribution):
//!   produces an off-shell vector with different normalization.
//!
//! The same coupling constant appears in all orientations; only the map between
//! fibers changes.
//!
//! ## Where the vertex factors live
//!
//! The vertex factors are methods on the representation types in
//! [`lorentz`](crate::helas::repr::lorentz), which is where each has a concrete
//! basis to be written in:
//!
//! | Vertex factor | Map | `(j_L,j_R)` chain | Where |
//! |---------------|-----|-------------------|-------|
//! | `ψ̄ γ^μ P_L ψ` | S\* ⊗ S → T\*M | `(½,0)×(½,0)→(½,½)` | [`SpinorRepr::left_current`](crate::helas::repr::lorentz::SpinorRepr::left_current) |
//! | `ψ̄ γ^μ P_R ψ` | S\* ⊗ S → T\*M | `(0,½)×(0,½)→(½,½)` | [`SpinorRepr::right_current`](crate::helas::repr::lorentz::SpinorRepr::right_current) |
//! | `ψ̄ γ^μ Γ ψ` | S\* ⊗ S → T\*M | `(½,0)×(0,½)→(½,½)` | [`SpinorRepr::vector_bilinear`](crate::helas::repr::lorentz::SpinorRepr::vector_bilinear) |
//! | `ψ̄ σ^{μν} Γ ψ` | S\* ⊗ S → Λ²T\*M | `(½,0)×(0,½)→(1,0)⊕(0,1)` | [`SpinorRepr::tensor_bilinear`](crate::helas::repr::lorentz::SpinorRepr::tensor_bilinear) |
//! | all sixteen `ψ̄ Γ_A ψ` | S\* ⊗ S → Cl(1,3)⊗ℂ | — | [`SpinorRepr::fierz_coefficients`](crate::helas::repr::lorentz::SpinorRepr::fierz_coefficients) |
//! | `ε^{μνρσ}` | (T\*M)³ → T\*M | `(½,½)³→(½,½)` | [`epsilon_vector`](crate::helas::repr::lorentz::epsilon_vector), [`epsilon4`](crate::helas::repr::lorentz::epsilon4) |
//!
//! An arbitrary Clifford element acts on a spinor through [`SpinorRepr::apply`](crate::helas::repr::lorentz::SpinorRepr::apply),
//! and two of them compose through [`Multivector::clifford_product`](crate::helas::repr::lorentz::Multivector::clifford_product), so a
//! γ-chain of any length needs no intertwiner type of its own.
