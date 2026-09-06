"""The UFO object model: the classes a UFO's own files instantiate.

Every UFO ships a copy of this file, because a model directory is a Python
package that MadGraph imports on its own -- there is no library to depend on.
The attribute names below are the interface MadGraph's `models/import_ufo.py`
reads, so they are fixed; the implementation is this repository's.

Instantiating any of these classes appends it to the module-level `all_*` list
that MadGraph collects through the package's `__init__.py`.
"""

import cmath  # noqa: F401  (a coupling or parameter value may call into it)


class UFOBaseClass(object):
    """Positional arguments named by `require_args`, keywords set verbatim."""

    require_args = []

    def __init__(self, *args, **options):
        assert len(self.require_args) == len(args)
        for name, value in zip(self.require_args, args):
            setattr(self, name, value)
        for option, value in options.items():
            setattr(self, option, value)

    def get(self, name):
        return getattr(self, name)

    def set(self, name, value):
        setattr(self, name, value)

    def get_all(self):
        return self.__dict__

    def __str__(self):
        return self.name

    def nice_string(self):
        return '\n'.join('%s \t: %s' % item for item in self.__dict__.items())

    def __repr__(self):
        # The Python identifier MadGraph writes into generated sources: the
        # display name with every character illegal in an identifier spelled out.
        text = self.name
        for orig, sub in (('+', '__plus__'), ('-', '__minus__'), ('@', '__at__'),
                          ('!', '__exclam__'), ('?', '__quest__'), ('*', '__star__'),
                          ('~', '__tilde__')):
            text = text.replace(orig, sub)
        return text


all_particles = []


class Particle(UFOBaseClass):
    """One field. `spin` is 2S+1 (1 scalar, 2 fermion, 3 vector), `color` the
    SU(3) representation as a signed dimension (1, 3, -3, 6, -6, 8)."""

    require_args = ['pdg_code', 'name', 'antiname', 'spin', 'color', 'mass',
                    'width', 'texname', 'antitexname', 'charge']
    require_args_all = require_args + ['line', 'propagating', 'goldstoneboson']

    def __init__(self, pdg_code, name, antiname, spin, color, mass, width,
                 texname, antitexname, charge, line=None, propagating=True,
                 goldstoneboson=False, **options):
        UFOBaseClass.__init__(self, pdg_code, name, antiname, spin, color, mass,
                              width, texname, antitexname, float(charge), **options)
        all_particles.append(self)
        self.propagating = propagating
        self.goldstoneboson = goldstoneboson
        self.selfconjugate = (name == antiname)
        self.line = line if line else self.find_line_type()

    def find_line_type(self):
        """How the diagram drawer joins this field's legs."""
        if self.spin == 1:
            return 'dashed'
        if self.spin == 2:
            if not self.selfconjugate:
                return 'straight'
            return 'swavy' if self.color == 1 else 'scurly'
        if self.spin == 3:
            return 'wavy' if self.color == 1 else 'curly'
        if self.spin == 5:
            return 'double'
        if self.spin == -1:
            return 'dotted'
        return 'dashed'

    def anti(self):
        """The conjugate field: names and tex swapped, additive quantum numbers
        and the colour representation negated (1 and 8 are self-conjugate)."""
        if self.selfconjugate:
            raise Exception('%s has no anti particle.' % self.name)
        extra = {key: -value for key, value in self.__dict__.items()
                 if key not in self.require_args_all}
        color = self.color if self.color in (1, 8) else -self.color
        return Particle(-self.pdg_code, self.antiname, self.name, self.spin,
                        color, self.mass, self.width, self.antitexname,
                        self.texname, -self.charge, self.line, self.propagating,
                        self.goldstoneboson, **extra)


all_parameters = []


class Parameter(UFOBaseClass):
    """An `external` parameter is read from a param card at `lhablock`/`lhacode`;
    an `internal` one is the Python expression in `value`."""

    require_args = ['name', 'nature', 'type', 'value', 'texname']

    def __init__(self, name, nature, type, value, texname, lhablock=None,
                 lhacode=None):
        UFOBaseClass.__init__(self, name, nature, type, value, texname)
        all_parameters.append(self)
        if nature == 'external' and (lhablock is None or lhacode is None):
            raise Exception('Need LHA information for external parameter "%s".' % name)
        self.lhablock = lhablock
        self.lhacode = lhacode


all_vertices = []


class Vertex(UFOBaseClass):
    """`couplings` is keyed `(color index, lorentz index)` into the `color` and
    `lorentz` lists, so one vertex may carry several structures at once."""

    require_args = ['name', 'particles', 'color', 'lorentz', 'couplings']

    def __init__(self, name, particles, color, lorentz, couplings, **opt):
        UFOBaseClass.__init__(self, name, particles, color, lorentz, couplings, **opt)
        all_vertices.append(self)


all_couplings = []


class Coupling(UFOBaseClass):
    """`order` is the coupling-order tuple MadGraph splits interactions by."""

    require_args = ['name', 'value', 'order']

    def __init__(self, name, value, order, **opt):
        UFOBaseClass.__init__(self, name, value, order, **opt)
        all_couplings.append(self)


all_lorentz = []


class Lorentz(UFOBaseClass):
    """`spins` is 2S+1 per leg; `structure` the ALOHA expression over the leg
    indices (positive, 1-based) and summed indices (negative)."""

    require_args = ['name', 'spins', 'structure']

    def __init__(self, name, spins, structure='external', **opt):
        UFOBaseClass.__init__(self, name, spins, structure, **opt)
        all_lorentz.append(self)


all_functions = []


class Function(object):
    """A named expression usable inside parameter and coupling values."""

    def __init__(self, name, arguments, expression):
        all_functions.append(self)
        self.name = name
        self.arguments = arguments
        self.expr = expression

    def __call__(self, *opt):
        return eval(self.expr, dict(cmath.__dict__),
                    dict(zip(self.arguments, opt)))


all_orders = []


class CouplingOrder(object):
    """`hierarchy` ranks the order in MadGraph's WEIGHTED search;
    `expansion_order` caps it, and only a value strictly between 0 and 99 caps
    anything."""

    def __init__(self, name, expansion_order, hierarchy, perturbative_expansion=0):
        all_orders.append(self)
        self.name = name
        self.expansion_order = expansion_order
        self.hierarchy = hierarchy
        self.perturbative_expansion = perturbative_expansion


all_decays = []


class Decay(UFOBaseClass):
    require_args = ['particle', 'partial_widths']

    def __init__(self, particle, partial_widths, **opt):
        UFOBaseClass.__init__(self, particle, partial_widths, **opt)
        all_decays.append(self)
        particle.partial_widths = partial_widths


all_form_factors = []


class FormFactor(UFOBaseClass):
    require_args = ['name', 'type', 'value']

    def __init__(self, name, type, value, **opt):
        UFOBaseClass.__init__(self, name, type, value, **opt)
        all_form_factors.append(self)
