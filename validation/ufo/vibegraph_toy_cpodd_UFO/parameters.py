"""Masses and coupling strengths, every value distinct.

Every particle is stable and every propagator in the banked processes is a
fermion exchanged in the t or u channel, so the model has no widths. The masses
keep every banked 2 -> 2 process at sqrt(s) = 600 GeV above threshold.

`cd` and `cdI` are the CP-even and CP-odd parts of the `ut` dipole, `cc` and
`ccI` those of the charged contact. Each CP-odd strength is the imaginary part
of a complex Wilson coefficient, which is what puts a real number on the
dipole's `gamma^5` term and an imaginary part on the contact's coupling.
"""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

MUT = Parameter(name='MUT',
                nature='external',
                type='real',
                value=170.0,
                texname='M_{u}',
                lhablock='MASS',
                lhacode=[9000002])

MDT = Parameter(name='MDT',
                nature='external',
                type='real',
                value=20.0,
                texname='M_{d}',
                lhablock='MASS',
                lhacode=[9000001])

MWT = Parameter(name='MWT',
                nature='external',
                type='real',
                value=80.0,
                texname='M_{W}',
                lhablock='MASS',
                lhacode=[9000024])

MZT = Parameter(name='MZT',
                nature='external',
                type='real',
                value=90.0,
                texname='M_{Z}',
                lhablock='MASS',
                lhacode=[9000023])

gzu = Parameter(name='gzu',
                nature='external',
                type='real',
                value=0.31,
                texname='g_{u}',
                lhablock='TOYCOUP',
                lhacode=[1])

gzd = Parameter(name='gzd',
                nature='external',
                type='real',
                value=0.17,
                texname='g_{d}',
                lhablock='TOYCOUP',
                lhacode=[2])

gw = Parameter(name='gw',
               nature='external',
               type='real',
               value=0.43,
               texname='g_{W}',
               lhablock='TOYCOUP',
               lhacode=[3])

cd = Parameter(name='cd',
               nature='external',
               type='real',
               value=0.0011,
               texname='c_{d}',
               lhablock='TOYCOUP',
               lhacode=[4])

cdI = Parameter(name='cdI',
                nature='external',
                type='real',
                value=0.0017,
                texname='c_{d}^{I}',
                lhablock='TOYCOUP',
                lhacode=[5])

cc = Parameter(name='cc',
               nature='external',
               type='real',
               value=0.0013,
               texname='c_{c}',
               lhablock='TOYCOUP',
               lhacode=[6])

ccI = Parameter(name='ccI',
                nature='external',
                type='real',
                value=0.0019,
                texname='c_{c}^{I}',
                lhablock='TOYCOUP',
                lhacode=[7])
