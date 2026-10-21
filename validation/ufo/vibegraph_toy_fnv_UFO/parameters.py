"""Masses, widths and one coupling strength per vertex."""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

MAT = Parameter(name='MAT',
                nature='external',
                type='real',
                value=10.0,
                texname='M_{a}',
                lhablock='MASS',
                lhacode=[9000001])

MBT = Parameter(name='MBT',
                nature='external',
                type='real',
                value=20.0,
                texname='M_{b}',
                lhablock='MASS',
                lhacode=[9000003])

MVT = Parameter(name='MVT',
                nature='external',
                type='real',
                value=300.0,
                texname='M_{V}',
                lhablock='MASS',
                lhacode=[9000023])

MDQ = Parameter(name='MDQ',
                nature='external',
                type='real',
                value=400.0,
                texname='M_{D}',
                lhablock='MASS',
                lhacode=[9000006])

WVT = Parameter(name='WVT',
                nature='external',
                type='real',
                value=2.0,
                texname='\\Gamma_{V}',
                lhablock='DECAY',
                lhacode=[9000023])

WDQ = Parameter(name='WDQ',
                nature='external',
                type='real',
                value=3.0,
                texname='\\Gamma_{D}',
                lhablock='DECAY',
                lhacode=[9000006])

ga = Parameter(name='ga',
               nature='external',
               type='real',
               value=0.31,
               texname='g_{a}',
               lhablock='TOYCOUP',
               lhacode=[1])

gb = Parameter(name='gb',
               nature='external',
               type='real',
               value=0.21,
               texname='g_{b}',
               lhablock='TOYCOUP',
               lhacode=[2])

yd = Parameter(name='yd',
               nature='external',
               type='real',
               value=0.37,
               texname='y_{D}',
               lhablock='TOYCOUP',
               lhacode=[3])
