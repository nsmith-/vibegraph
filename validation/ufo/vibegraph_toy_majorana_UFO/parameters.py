"""Masses, widths and one coupling strength per vertex."""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

MLT = Parameter(name='MLT',
                nature='external',
                type='real',
                value=10.0,
                texname='M_{\\ell_t}',
                lhablock='MASS',
                lhacode=[9000011])

MNT = Parameter(name='MNT',
                nature='external',
                type='real',
                value=100.0,
                texname='M_{\\chi_t}',
                lhablock='MASS',
                lhacode=[9000022])

MVT = Parameter(name='MVT',
                nature='external',
                type='real',
                value=300.0,
                texname='M_{V}',
                lhablock='MASS',
                lhacode=[9000023])

MSC = Parameter(name='MSC',
                nature='external',
                type='real',
                value=150.0,
                texname='M_{\\tilde\\ell_t}',
                lhablock='MASS',
                lhacode=[9000015])

WVT = Parameter(name='WVT',
                nature='external',
                type='real',
                value=2.0,
                texname='\\Gamma_{V}',
                lhablock='DECAY',
                lhacode=[9000023])

WSC = Parameter(name='WSC',
                nature='external',
                type='real',
                value=1.0,
                texname='\\Gamma_{\\tilde\\ell_t}',
                lhablock='DECAY',
                lhacode=[9000015])

gl = Parameter(name='gl',
               nature='external',
               type='real',
               value=0.31,
               texname='g_{\\ell}',
               lhablock='TOYCOUP',
               lhacode=[1])

gn = Parameter(name='gn',
               nature='external',
               type='real',
               value=0.21,
               texname='g_{\\chi}',
               lhablock='TOYCOUP',
               lhacode=[2])

ys = Parameter(name='ys',
               nature='external',
               type='real',
               value=0.37,
               texname='y_{s}',
               lhablock='TOYCOUP',
               lhacode=[3])
