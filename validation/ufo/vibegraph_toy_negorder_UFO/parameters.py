"""External parameters: three masses, two widths and one coupling strength
per vertex, every value distinct so a restriction merges nothing."""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

MFW = Parameter(name='MFW',
                nature='external',
                type='real',
                value=10.0,
                texname='M_{f_w}',
                lhablock='MASS',
                lhacode=[9000111])

MVW = Parameter(name='MVW',
                nature='external',
                type='real',
                value=90.0,
                texname='M_{V_w}',
                lhablock='MASS',
                lhacode=[9000122])

MXW = Parameter(name='MXW',
                nature='external',
                type='real',
                value=100.0,
                texname='M_{X_w}',
                lhablock='MASS',
                lhacode=[9000135])

WVW = Parameter(name='WVW',
                nature='external',
                type='real',
                value=2.0,
                texname='\\Gamma_{V_w}',
                lhablock='DECAY',
                lhacode=[9000122])

WXW = Parameter(name='WXW',
                nature='external',
                type='real',
                value=1.5,
                texname='\\Gamma_{X_w}',
                lhablock='DECAY',
                lhacode=[9000135])

gfv = Parameter(name='gfv',
                nature='external',
                type='real',
                value=0.31,
                texname='g_{fV}',
                lhablock='TOYCOUP',
                lhacode=[1])

gfx = Parameter(name='gfx',
                nature='external',
                type='real',
                value=0.27,
                texname='g_{fX}',
                lhablock='TOYCOUP',
                lhacode=[2])

gvx = Parameter(name='gvx',
                nature='external',
                type='real',
                value=41.0,
                texname='g_{VX}',
                lhablock='TOYCOUP',
                lhacode=[3])

gx3 = Parameter(name='gx3',
                nature='external',
                type='real',
                value=33.0,
                texname='g_{X^3}',
                lhablock='TOYCOUP',
                lhacode=[4])

gx3np = Parameter(name='gx3np',
                  nature='external',
                  type='real',
                  value=7.0,
                  texname='g_{X^3}^{NP}',
                  lhablock='TOYCOUP',
                  lhacode=[5])

gx4 = Parameter(name='gx4',
                nature='external',
                type='real',
                value=0.43,
                texname='g_{X^4}',
                lhablock='TOYCOUP',
                lhacode=[6])
