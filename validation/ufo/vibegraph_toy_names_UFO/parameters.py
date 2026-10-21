"""One coupling strength per vertex; every field is massless."""

from object_library import all_parameters, Parameter

ZERO = Parameter(name='ZERO',
                 nature='internal',
                 type='real',
                 value='0.0',
                 texname='0')

gm = Parameter(name='gm',
               nature='external',
               type='real',
               value=0.31,
               texname='g_{\\mu}',
               lhablock='TOYCOUP',
               lhacode=[1])

gu = Parameter(name='gu',
               nature='external',
               type='real',
               value=0.21,
               texname='g_{u}',
               lhablock='TOYCOUP',
               lhacode=[2])
