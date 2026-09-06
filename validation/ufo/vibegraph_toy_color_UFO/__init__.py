import particles
import couplings
import lorentz
import parameters
import vertices
import coupling_orders
import function_library
import object_library

all_particles = particles.all_particles
all_vertices = vertices.all_vertices
all_couplings = couplings.all_couplings
all_lorentz = lorentz.all_lorentz
all_parameters = parameters.all_parameters
all_orders = coupling_orders.all_orders
all_functions = function_library.all_functions

# Unitary gauge only: the massive vector has no Goldstone partner, so there is
# no Feynman-gauge form of this model to select.
gauge = [0]
