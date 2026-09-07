"""One Lorentz structure: the scalar-scalar-scalar contact, which is the
constant 1. Everything this model is for lives in the colour strings of
`vertices.py`.
"""

from object_library import all_lorentz, Lorentz

SSS1 = Lorentz(name='SSS1',
               spins=[1, 1, 1],
               structure='1')
