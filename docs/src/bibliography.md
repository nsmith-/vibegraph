# Bibliography

Every paper cited in the guided tour, grouped by the stage it informs. The
project's research notes under `research/notes/` summarise most of these in
more detail, and `research/refs/fetch-papers.sh` fetches the arXiv ones.

## The pipeline as a whole

- J. Alwall, R. Frederix, S. Frixione, V. Hirschi, F. Maltoni, O. Mattelaer,
  H.-S. Shao, T. Stelzer, P. Torrielli, M. Zaro, *The automated computation
  of tree-level and next-to-leading order differential cross sections, and
  their matching to parton shower simulations*, JHEP 07 (2014) 079,
  [arXiv:1405.0301](https://arxiv.org/abs/1405.0301). The MadGraph5\_aMC@NLO
  paper; its LO half is the pipeline this generator follows.
- J. Alwall, M. Herquet, F. Maltoni, O. Mattelaer, T. Stelzer, *MadGraph 5:
  Going Beyond*, JHEP 06 (2011) 128,
  [arXiv:1106.0522](https://arxiv.org/abs/1106.0522). Diagram-level
  wavefunction reuse, the process grammar, the UFO/ALOHA integration.
- F. Maltoni, T. Stelzer, *MadEvent: Automatic event generation with
  MadGraph*, JHEP 02 (2003) 027,
  [hep-ph/0208156](https://arxiv.org/abs/hep-ph/0208156). Single-diagram-enhanced
  multichannel integration and the survey/refine/generate structure.
- A. Buckley et al., *General-purpose event generators for LHC physics*,
  Phys. Rept. 504 (2011) 145,
  [arXiv:1101.2599](https://arxiv.org/abs/1101.2599). Where a fixed-order
  matrix-element generator sits in the simulation chain.

## Models

- C. Degrande, C. Duhr, B. Fuks, D. Grellscheid, O. Mattelaer, T. Reiter,
  *UFO – The Universal FeynRules Output*, Comput. Phys. Commun. 183 (2012)
  1201, [arXiv:1108.2040](https://arxiv.org/abs/1108.2040).
- N. Christensen, C. Duhr, *FeynRules – Feynman rules made easy*,
  Comput. Phys. Commun. 180 (2009) 1614,
  [arXiv:0806.4194](https://arxiv.org/abs/0806.4194); A. Alloul,
  N. Christensen, C. Degrande, C. Duhr, B. Fuks, *FeynRules 2.0*,
  Comput. Phys. Commun. 185 (2014) 2250,
  [arXiv:1310.1921](https://arxiv.org/abs/1310.1921). The tool that writes
  UFO models.
- P. Skands et al., *SUSY Les Houches Accord: Interfacing SUSY Spectrum
  Calculators, Decay Packages, and Event Generators*, JHEP 07 (2004) 036,
  [hep-ph/0311123](https://arxiv.org/abs/hep-ph/0311123). The `param_card.dat`
  block format.
- J. Alwall, C. Duhr, B. Fuks, O. Mattelaer, D. G. Öztürk, C.-H. Shen,
  *Computing decay rates for new physics theories with FeynRules and
  MadGraph5\_aMC@NLO*, Comput. Phys. Commun. 197 (2015) 312,
  [arXiv:1402.1178](https://arxiv.org/abs/1402.1178). MadWidth, and the
  `decay.py` extension of UFO.

## Diagrams

- T. Stelzer, W. F. Long, *Automatic generation of tree level helicity
  amplitudes*, Comput. Phys. Commun. 81 (1994) 357,
  [hep-ph/9401258](https://arxiv.org/abs/hep-ph/9401258). The original
  MadGraph: topology-first diagram enumeration and HELAS call generation.
- P. Nogueira, *Automatic Feynman graph generation*, J. Comput. Phys. 105
  (1993) 279. QGRAF: orderly enumeration of graph topologies over vertex-degree
  partitions, the family feyngraph's topology stage belongs to.
- J. Braun, *FeynGraph*, <https://github.com/Jens-Braun/FeynGraph>. The Rust
  diagram generator vibegraph drives.

## Helicity amplitudes

- H. Murayama, I. Watanabe, K. Hagiwara, *HELAS: HELicity Amplitude
  Subroutines for Feynman diagram evaluations*, KEK Report 91-11 (1992),
  [KEK preprint](https://lib-extopc.kek.jp/preprints/PDF/1991/9124/9124011.pdf).
  The wavefunction and vertex routines, and the conventions in Appendix A.
- P. de Aquino, W. Link, F. Maltoni, O. Mattelaer, T. Stelzer, *ALOHA:
  Automatic Libraries Of Helicity Amplitudes for Feynman diagram
  computations*, Comput. Phys. Commun. 183 (2012) 2254,
  [arXiv:1108.2041](https://arxiv.org/abs/1108.2041). Generating HELAS-style
  routines from UFO Lorentz structures.
- D. Buarque Franzosi, O. Mattelaer, R. Ruiz, S. Shil, *Automated predictions
  from polarized matrix elements*, JHEP 04 (2020) 082,
  [arXiv:1912.01725](https://arxiv.org/abs/1912.01725). MadGraph's helicity
  conventions, stated explicitly.
- T. Gleisberg, S. Höche, *Comix, a new matrix element generator*, JHEP 12
  (2008) 039, [arXiv:0808.3674](https://arxiv.org/abs/0808.3674).
  Berends–Giele recursion, the alternative to summing diagrams.

## Colour

- F. Maltoni, K. Paul, T. Stelzer, S. Willenbrock, *Color-flow decomposition
  of QCD amplitudes*, Phys. Rev. D 67 (2003) 014026,
  [hep-ph/0209271](https://arxiv.org/abs/hep-ph/0209271). The leading-colour
  flow picture behind Les Houches colour tags.

## The amplitude compiler

- O. Mattelaer, K. Ostrolenk, *Speeding up MadGraph5\_aMC@NLO*, Eur. Phys.
  J. C 81 (2021) 435,
  [arXiv:2102.00773](https://arxiv.org/abs/2102.00773). Helicity recycling:
  common-subexpression elimination across the unrolled helicity loop.
- M. Willsey, C. Nandi, Y. R. Wang, O. Flatt, Z. Tatlock, P. Panchekha,
  *egg: Fast and extensible equality saturation*, POPL 2021,
  [arXiv:2004.03082](https://arxiv.org/abs/2004.03082).
- Y. Zhang, Y. R. Wang, O. Flatt, D. Cao, P. Zucker, E. Rosenthal, Z. Tatlock,
  M. Willsey, *Better Together: Unifying Datalog and Equality Saturation*,
  PLDI 2023, [arXiv:2304.04332](https://arxiv.org/abs/2304.04332). The egglog
  system the crate's e-graph stage is built on.
- J. T. Schwartz, *Fast probabilistic algorithms for verification of
  polynomial identities*, J. ACM 27 (1980) 701; R. Zippel, *Probabilistic
  algorithms for sparse polynomials*, EUROSAM 1979. Why probing a rational
  function at random points decides whether it vanishes identically.

## Phase space and integration

- R. Kleiss, W. J. Stirling, S. D. Ellis, *A new Monte Carlo treatment of
  multiparticle phase space at high energies*, Comput. Phys. Commun. 40
  (1986) 359. RAMBO.
- E. Byckling, K. Kajantie, *Particle Kinematics*, Wiley (1973). The
  recursive two-body decomposition of $n$-body phase space.
- G. P. Lepage, *A new algorithm for adaptive multidimensional integration*,
  J. Comput. Phys. 27 (1978) 192. VEGAS.
- G. P. Lepage, *Adaptive multidimensional integration: VEGAS enhanced*,
  J. Comput. Phys. 439 (2021) 110386,
  [arXiv:2009.05112](https://arxiv.org/abs/2009.05112). A full restatement of
  the classic algorithm, plus the stratified extension.
- R. Kleiss, R. Pittau, *Weight optimization in multichannel Monte Carlo*,
  Comput. Phys. Commun. 83 (1994) 141,
  [hep-ph/9405257](https://arxiv.org/abs/hep-ph/9405257). Adapting the channel
  weights $\alpha_j$.
- V. Hirschi, O. Mattelaer, *Automated event generation for loop-induced
  processes*, JHEP 10 (2015) 146,
  [arXiv:1507.00020](https://arxiv.org/abs/1507.00020). Its appendix describes
  MadGraph's phase-space channel parametrisations.
- J. Neyman, *On the two different aspects of the representative method*,
  J. R. Stat. Soc. 97 (1934) 558. Variance-minimising allocation across
  strata, applied here across channels.
- Particle Data Group, *Review of Particle Physics*, introduction, the
  scale-factor treatment of inconsistent measurements.

## Proton beams

- A. Buckley, J. Ferrando, S. Lloyd, K. Nordström, B. Page, M. Rüfenacht,
  M. Schönherr, G. Watt, *LHAPDF6: parton density access in the LHC
  precision era*, Eur. Phys. J. C 75 (2015) 132,
  [arXiv:1412.7420](https://arxiv.org/abs/1412.7420). The grid format and the
  log-bicubic interpolator reproduced here.
- S. Catani, F. Krauss, R. Kuhn, B. R. Webber, *QCD matrix elements + parton
  showers*, JHEP 11 (2001) 063,
  [hep-ph/0109231](https://arxiv.org/abs/hep-ph/0109231). The kT-clustering
  measure MadGraph's dynamical scale is built on.

## Event files

- E. Boos et al., *Generic user process interface for event generators*,
  Les Houches 2001, [hep-ph/0109068](https://arxiv.org/abs/hep-ph/0109068).
  The Les Houches Accord: `IDWTUP`, the `HEPRUP`/`HEPEUP` records.
- J. Alwall et al., *A standard format for Les Houches Event Files*,
  Comput. Phys. Commun. 176 (2007) 300,
  [hep-ph/0609017](https://arxiv.org/abs/hep-ph/0609017). The `.lhe` file
  format.

## Further reading

- S. Catani, M. H. Seymour, *A general algorithm for calculating jet cross
  sections in NLO QCD*, Nucl. Phys. B 485 (1997) 291,
  [hep-ph/9605323](https://arxiv.org/abs/hep-ph/9605323). Not used at LO;
  the reference for what an NLO extension would need.
