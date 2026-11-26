"""The Lorentz structures, copied from the dim6top model's `lorentz.py`
(FeynRules 2.3.36) under the same names: its dipoles and contacts write their
chirality with `ProjM` and `ProjP` rather than with `Gamma5`.

`FFV11` is `[gamma^mu, pslash]`, `FFV9` is `[gamma^mu, pslash](P_L - P_R)`,
`FFVV1` and `FFVV3` are `[gamma^mu, gamma^nu]` times `P_L` and `P_R`, each with
the boson momentum and indices as the dim6top vertices read them.
`FFVS4` and `FFVS6` are `FFV9` and `FFV11` beside a scalar, as dim6top's
`t~ t g H` contact writes them, and `FFVS2` and `FFVS5` the one-chirality
momentum chains `[gamma^mu, pslash] P_L` and `P_R` beside a scalar, as its
`t b W H` contacts do; `FFS1` and `FFS5` are the Higgs-characterisation
model's scalar and pseudoscalar Yukawas.
"""

from object_library import all_lorentz, Lorentz

FFV1 = Lorentz(name='FFV1',
               spins=[2, 2, 3],
               structure='Gamma(3,2,1)')

FFV2 = Lorentz(name='FFV2',
               spins=[2, 2, 3],
               structure='Gamma(3,2,-1)*ProjM(-1,1)')

FFV9 = Lorentz(name='FFV9',
               spins=[2, 2, 3],
               structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjM(-2,1) + P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjP(-2,1) - P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjP(-2,1)')

FFV11 = Lorentz(name='FFV11',
                spins=[2, 2, 3],
                structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjM(-2,1) - P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjP(-2,1) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjP(-2,1)')

FFVV1 = Lorentz(name='FFVV1',
                spins=[2, 2, 3, 3],
                structure='Gamma(3,2,-2)*Gamma(4,-2,-1)*ProjM(-1,1) - Gamma(3,-2,-1)*Gamma(4,2,-2)*ProjM(-1,1)')

FFVV3 = Lorentz(name='FFVV3',
                spins=[2, 2, 3, 3],
                structure='Gamma(3,2,-2)*Gamma(4,-2,-1)*ProjP(-1,1) - Gamma(3,-2,-1)*Gamma(4,2,-2)*ProjP(-1,1)')

FFS1 = Lorentz(name='FFS1',
               spins=[2, 2, 1],
               structure='Identity(2,1)')

FFS5 = Lorentz(name='FFS5',
               spins=[2, 2, 1],
               structure='Gamma5(2,1)')

FFVS4 = Lorentz(name='FFVS4',
                spins=[2, 2, 3, 1],
                structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjM(-2,1) + P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjP(-2,1) - P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjP(-2,1)')

FFVS6 = Lorentz(name='FFVS6',
                spins=[2, 2, 3, 1],
                structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjM(-2,1) - P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjP(-2,1) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjP(-2,1)')

FFVS2 = Lorentz(name='FFVS2',
                spins=[2, 2, 3, 1],
                structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjM(-2,1)')

FFVS5 = Lorentz(name='FFVS5',
                spins=[2, 2, 3, 1],
                structure='-(P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjP(-2,1)) + P(-1,3)*Gamma(-1,-3,-2)*Gamma(3,2,-3)*ProjP(-2,1)')
