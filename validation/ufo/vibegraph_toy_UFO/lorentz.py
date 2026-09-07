"""The Lorentz structures this model exists to isolate.

`FFVD` and `FFFFT` write `Sigma` literally, which is what no FeynRules-generated
model does: FeynRules expands sigma^{mu nu} into gamma^mu gamma^nu chains before
it writes the UFO, so `Sigma` appears in ALOHA's own object library and in no
model file. `FFFFG` is that expansion, written out for the same operator
`FFFFT` writes with `Sigma`, so the two spellings can be compared against each
other diagram by diagram inside one process. It is the textbook expansion, for

  sigma^{mu nu} = (i/2)(gamma^mu gamma^nu - gamma^nu gamma^mu)

  sigma^{mu nu} (x) sigma_{mu nu}
      = -(1/2)[ (gamma^mu gamma^nu) (x) (gamma_mu gamma_nu)
              - (gamma^mu gamma^nu) (x) (gamma_nu gamma_mu) ]

and ALOHA's `Sigma` is half of that sigma: the banked amplitude table of the
`ll_to_qqx_toy_tensor` row measures AMP(FFFFG)/AMP(FFFFT) at exactly four times
the ratio of the two couplings, to 4.7e-14 over every helicity of every detail
point. (`aloha/aloha_object.py`'s `L_Sigma.sigma` carries +/-0.5 and +/-0.5j
where the textbook matrix carries +/-1 and +/-1j.) The factor is left as a
measurement rather than absorbed into `FFFFG`'s coefficient, so an evaluator
that gives its own `Sigma` kernel the textbook normalisation disagrees with the
reference by a number this row already knows. Squaring is blind to a global sign
on `Sigma`; what pins that is `FFVD`, which is linear in it and interferes with
a plain gauge coupling.

`FFFFG`'s index graph is a 4-cycle: its two fermion lines share two summed
Lorentz indices, so no rooting of a contraction tree can walk it.
"""

from object_library import all_lorentz, Lorentz

SSS1 = Lorentz(name='SSS1',
               spins=[1, 1, 1],
               structure='1')

FFS1 = Lorentz(name='FFS1',
               spins=[2, 2, 1],
               structure='Identity(2,1)')

FFS5 = Lorentz(name='FFS5',
               spins=[2, 2, 1],
               structure='Gamma5(2,1)')

FFV1 = Lorentz(name='FFV1',
               spins=[2, 2, 3],
               structure='Gamma(3,2,1)')

FFVD = Lorentz(name='FFVD',
               spins=[2, 2, 3],
               structure='Sigma(3,-1,2,-2)*P(-1,3)*ProjM(-2,1)')

FFFFT = Lorentz(name='FFFFT',
                spins=[2, 2, 2, 2],
                structure='Sigma(-1,-2,2,1)*Sigma(-1,-2,4,3)')

FFFFG = Lorentz(name='FFFFG',
                spins=[2, 2, 2, 2],
                structure='-1/2*Gamma(-1,2,-5)*Gamma(-2,-5,1)*Gamma(-1,4,-6)*Gamma(-2,-6,3)'
                          ' + 1/2*Gamma(-1,2,-5)*Gamma(-2,-5,1)*Gamma(-2,4,-6)*Gamma(-1,-6,3)')
