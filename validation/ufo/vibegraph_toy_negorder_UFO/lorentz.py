from object_library import all_lorentz, Lorentz

SSS1 = Lorentz(name='SSS1',
               spins=[1, 1, 1],
               structure='1')

SSSS1 = Lorentz(name='SSSS1',
                spins=[1, 1, 1, 1],
                structure='1')

FFS1 = Lorentz(name='FFS1',
               spins=[2, 2, 1],
               structure='Identity(2,1)')

FFV1 = Lorentz(name='FFV1',
               spins=[2, 2, 3],
               structure='Gamma(3,2,1)')

VVS1 = Lorentz(name='VVS1',
               spins=[3, 3, 1],
               structure='Metric(1,2)')
