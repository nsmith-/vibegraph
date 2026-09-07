"""External parameters: five masses, three widths and one coupling strength per
Lorentz or colour structure the model exists to isolate.

Every value is distinct. MadGraph's restriction merges external parameters that
share a value and drops the ones a card sets to zero, so distinct values keep
the per-structure cards free to switch exactly one structure off without
also collapsing two others into one.

The masses and widths are chosen to keep the banked 2 -> 2 processes at
sqrt(s) = 500 GeV well above every threshold and far from every propagator pole.
"""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

# ── masses ───────────────────────────────────────────────────────────────────

MLT = Parameter(name='MLT',
                nature='external',
                type='real',
                value=10.0,
                texname='M_{\\ell_t}',
                lhablock='MASS',
                lhacode=[9000011])

MQT = Parameter(name='MQT',
                nature='external',
                type='real',
                value=50.0,
                texname='M_{q_t}',
                lhablock='MASS',
                lhacode=[9000004])

MST = Parameter(name='MST',
                nature='external',
                type='real',
                value=200.0,
                texname='M_{S}',
                lhablock='MASS',
                lhacode=[9000025])

MVT = Parameter(name='MVT',
                nature='external',
                type='real',
                value=300.0,
                texname='M_{V}',
                lhablock='MASS',
                lhacode=[9000023])

MO8 = Parameter(name='MO8',
                nature='external',
                type='real',
                value=120.0,
                texname='M_{O_8}',
                lhablock='MASS',
                lhacode=[9000021])

# ── widths ───────────────────────────────────────────────────────────────────
#
# Non-zero on the three fields that appear as internal propagators, so every
# banked amplitude carries a complex propagator denominator rather than a real
# one, and the comparison sees the imaginary part.

WST = Parameter(name='WST',
                nature='external',
                type='real',
                value=1.5,
                texname='\\Gamma_{S}',
                lhablock='DECAY',
                lhacode=[9000025])

WVT = Parameter(name='WVT',
                nature='external',
                type='real',
                value=2.0,
                texname='\\Gamma_{V}',
                lhablock='DECAY',
                lhacode=[9000023])

WO8 = Parameter(name='WO8',
                nature='external',
                type='real',
                value=1.0,
                texname='\\Gamma_{O_8}',
                lhablock='DECAY',
                lhacode=[9000021])

# ── one strength per structure ───────────────────────────────────────────────

gl = Parameter(name='gl',
               nature='external',
               type='real',
               value=0.31,
               texname='g_{\\ell}',
               lhablock='TOYCOUP',
               lhacode=[1])

gq = Parameter(name='gq',
               nature='external',
               type='real',
               value=0.21,
               texname='g_{q}',
               lhablock='TOYCOUP',
               lhacode=[2])

gdip = Parameter(name='gdip',
                 nature='external',
                 type='real',
                 value=0.013,
                 texname='c_{\\sigma}',
                 lhablock='TOYCOUP',
                 lhacode=[3])

gtens = Parameter(name='gtens',
                  nature='external',
                  type='real',
                  value=0.000023,
                  texname='c_{TT}',
                  lhablock='TOYCOUP',
                  lhacode=[4])

ggam = Parameter(name='ggam',
                 nature='external',
                 type='real',
                 value=0.000017,
                 texname='c_{\\gamma\\gamma}',
                 lhablock='TOYCOUP',
                 lhacode=[5])

ysl = Parameter(name='ysl',
                nature='external',
                type='real',
                value=0.37,
                texname='y^{S}_{\\ell}',
                lhablock='TOYCOUP',
                lhacode=[6])

ysq = Parameter(name='ysq',
                nature='external',
                type='real',
                value=0.27,
                texname='y^{S}_{q}',
                lhablock='TOYCOUP',
                lhacode=[7])

ypl = Parameter(name='ypl',
                nature='external',
                type='real',
                value=0.19,
                texname='y^{P}_{\\ell}',
                lhablock='TOYCOUP',
                lhacode=[8])

ypq = Parameter(name='ypq',
                nature='external',
                type='real',
                value=0.23,
                texname='y^{P}_{q}',
                lhablock='TOYCOUP',
                lhacode=[9])

gqo = Parameter(name='gqo',
                nature='external',
                type='real',
                value=0.41,
                texname='g_{qO}',
                lhablock='TOYCOUP',
                lhacode=[10])

go3 = Parameter(name='go3',
                nature='external',
                type='real',
                value=45.0,
                texname='g_{O^3}',
                lhablock='TOYCOUP',
                lhacode=[11])
