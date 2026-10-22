"""External parameters: four masses, three widths and one coupling strength
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

MYW = Parameter(name='MYW',
                nature='external',
                type='real',
                value=150.0,
                texname='M_{Y_w}',
                lhablock='MASS',
                lhacode=[9000136])

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

WYW = Parameter(name='WYW',
                nature='external',
                type='real',
                value=2.5,
                texname='\\Gamma_{Y_w}',
                lhablock='DECAY',
                lhacode=[9000136])

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

gx4 = Parameter(name='gx4',
                nature='external',
                type='real',
                value=0.43,
                texname='g_{X^4}',
                lhablock='TOYCOUP',
                lhacode=[3])

gxy = Parameter(name='gxy',
                nature='external',
                type='real',
                value=37.0,
                texname='g_{XY}',
                lhablock='TOYCOUP',
                lhacode=[4])

gy4 = Parameter(name='gy4',
                nature='external',
                type='real',
                value=0.19,
                texname='g_{Y^4}',
                lhablock='TOYCOUP',
                lhacode=[5])
