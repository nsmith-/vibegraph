"""External parameters: five masses, three widths and one strength per vertex.

Values are distinct, because MadGraph's restriction merges external parameters
that share a value and drops the ones a card sets to zero. The masses keep the
banked 2 -> 2 rows at sqrt(s) = 500 GeV above every threshold and away from
every propagator pole.
"""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

MP3 = Parameter(name='MP3',
                nature='external',
                type='real',
                value=60.0,
                texname='M_{\\phi_3}',
                lhablock='MASS',
                lhacode=[9000051])

MR3 = Parameter(name='MR3',
                nature='external',
                type='real',
                value=70.0,
                texname='M_{\\rho_3}',
                lhablock='MASS',
                lhacode=[9000052])

MD3 = Parameter(name='MD3',
                nature='external',
                type='real',
                value=150.0,
                texname='M_{D_3}',
                lhablock='MASS',
                lhacode=[9000042])

MD6 = Parameter(name='MD6',
                nature='external',
                type='real',
                value=170.0,
                texname='M_{D_6}',
                lhablock='MASS',
                lhacode=[9000046])

MST = Parameter(name='MST',
                nature='external',
                type='real',
                value=200.0,
                texname='M_{S}',
                lhablock='MASS',
                lhacode=[9000025])

WD3 = Parameter(name='WD3',
                nature='external',
                type='real',
                value=1.2,
                texname='\\Gamma_{D_3}',
                lhablock='DECAY',
                lhacode=[9000042])

WD6 = Parameter(name='WD6',
                nature='external',
                type='real',
                value=0.8,
                texname='\\Gamma_{D_6}',
                lhablock='DECAY',
                lhacode=[9000046])

WST = Parameter(name='WST',
                nature='external',
                type='real',
                value=1.5,
                texname='\\Gamma_{S}',
                lhablock='DECAY',
                lhacode=[9000025])

gps = Parameter(name='gps',
                nature='external',
                type='real',
                value=29.0,
                texname='g_{\\phi S}',
                lhablock='TOYCOUP',
                lhacode=[1])

grs = Parameter(name='grs',
                nature='external',
                type='real',
                value=35.0,
                texname='g_{\\rho S}',
                lhablock='TOYCOUP',
                lhacode=[2])

geps = Parameter(name='geps',
                 nature='external',
                 type='real',
                 value=33.0,
                 texname='g_{\\epsilon}',
                 lhablock='TOYCOUP',
                 lhacode=[3])

gk6 = Parameter(name='gk6',
                nature='external',
                type='real',
                value=19.0,
                texname='g_{K_6}',
                lhablock='TOYCOUP',
                lhacode=[4])
