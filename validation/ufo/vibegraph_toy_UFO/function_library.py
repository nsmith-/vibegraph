"""Functions a parameter or coupling value may call, beyond `cmath`'s own.

MadGraph evaluates every value expression in a namespace built from `cmath`
plus these; a UFO that uses none of them still has to define the module, since
`__init__.py` collects `all_functions` from it.
"""

from object_library import all_functions, Function

complexconjugate = Function(name='complexconjugate',
                            arguments=('z',),
                            expression='z.conjugate()')

re = Function(name='re',
              arguments=('z',),
              expression='z.real')

im = Function(name='im',
              arguments=('z',),
              expression='z.imag')

sec = Function(name='sec',
               arguments=('z',),
               expression='1./cos(z)')

asec = Function(name='asec',
                arguments=('z',),
                expression='acos(1./z)')

csc = Function(name='csc',
               arguments=('z',),
               expression='1./sin(z)')

acsc = Function(name='acsc',
                arguments=('z',),
                expression='asin(1./z)')
