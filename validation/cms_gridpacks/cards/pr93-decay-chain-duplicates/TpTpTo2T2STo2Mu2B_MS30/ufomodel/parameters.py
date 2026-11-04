# This file was automatically created by FeynRules 2.3.47
# Mathematica version: 13.1.0 for Linux x86 (64-bit) (June 16, 2022)
# Date: Wed 27 Jul 2022 15:42:16



from object_library import all_parameters, Parameter


from function_library import complexconjugate, re, im, csc, sec, acsc, asec, cot

# This is a default parameter object representing 0.
ZERO = Parameter(name = 'ZERO',
                 nature = 'internal',
                 type = 'real',
                 value = '0.0',
                 texname = '0')

# User-defined parameters.
CKM1x1 = Parameter(name = 'CKM1x1',
                   nature = 'external',
                   type = 'complex',
                   value = 0.97434,
                   texname = '\\text{CKM1x1}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 1, 1 ])

CKM1x2 = Parameter(name = 'CKM1x2',
                   nature = 'external',
                   type = 'complex',
                   value = 0.22506,
                   texname = '\\text{CKM1x2}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 1, 2 ])

CKM1x3 = Parameter(name = 'CKM1x3',
                   nature = 'external',
                   type = 'complex',
                   value = 0.00357,
                   texname = '\\text{CKM1x3}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 1, 3 ])

CKM2x1 = Parameter(name = 'CKM2x1',
                   nature = 'external',
                   type = 'complex',
                   value = 0.22492,
                   texname = '\\text{CKM2x1}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 2, 1 ])

CKM2x2 = Parameter(name = 'CKM2x2',
                   nature = 'external',
                   type = 'complex',
                   value = 0.97351,
                   texname = '\\text{CKM2x2}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 2, 2 ])

CKM2x3 = Parameter(name = 'CKM2x3',
                   nature = 'external',
                   type = 'complex',
                   value = 0.0411,
                   texname = '\\text{CKM2x3}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 2, 3 ])

CKM3x1 = Parameter(name = 'CKM3x1',
                   nature = 'external',
                   type = 'complex',
                   value = 0.00875,
                   texname = '\\text{CKM3x1}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 3, 1 ])

CKM3x2 = Parameter(name = 'CKM3x2',
                   nature = 'external',
                   type = 'complex',
                   value = 0.0403,
                   texname = '\\text{CKM3x2}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 3, 2 ])

CKM3x3 = Parameter(name = 'CKM3x3',
                   nature = 'external',
                   type = 'complex',
                   value = 0.999152,
                   texname = '\\text{CKM3x3}',
                   lhablock = 'CKMBlock',
                   lhacode = [ 3, 3 ])

KHTT = Parameter(name = 'KHTT',
                 nature = 'external',
                 type = 'real',
                 value = 1,
                 texname = '\\text{KHTT}',
                 lhablock = 'HQQSMMOD',
                 lhacode = [ 1 ])

KHBB = Parameter(name = 'KHBB',
                 nature = 'external',
                 type = 'real',
                 value = 1,
                 texname = '\\text{KHBB}',
                 lhablock = 'HQQSMMOD',
                 lhacode = [ 2 ])

KAWS10S11 = Parameter(name = 'KAWS10S11',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KAWS10S11}',
                      lhablock = 'KAWSS',
                      lhacode = [ 1 ])

KAWS11S12 = Parameter(name = 'KAWS11S12',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KAWS11S12}',
                      lhablock = 'KAWSS',
                      lhacode = [ 2 ])

KAZSS11 = Parameter(name = 'KAZSS11',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KAZSS11}',
                    lhablock = 'KAZSS',
                    lhacode = [ 1 ])

KAZSS12 = Parameter(name = 'KAZSS12',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KAZSS12}',
                    lhablock = 'KAZSS',
                    lhacode = [ 2 ])

KBLh1 = Parameter(name = 'KBLh1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLh1}',
                  lhablock = 'KBLH',
                  lhacode = [ 1 ])

KBLh2 = Parameter(name = 'KBLh2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLh2}',
                  lhablock = 'KBLH',
                  lhacode = [ 2 ])

KBLh3 = Parameter(name = 'KBLh3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLh3}',
                  lhablock = 'KBLH',
                  lhacode = [ 3 ])

KBLw1 = Parameter(name = 'KBLw1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLw1}',
                  lhablock = 'KBLW',
                  lhacode = [ 1 ])

KBLw2 = Parameter(name = 'KBLw2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLw2}',
                  lhablock = 'KBLW',
                  lhacode = [ 2 ])

KBLw3 = Parameter(name = 'KBLw3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLw3}',
                  lhablock = 'KBLW',
                  lhacode = [ 3 ])

KBLz1 = Parameter(name = 'KBLz1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLz1}',
                  lhablock = 'KBLZ',
                  lhacode = [ 1 ])

KBLz2 = Parameter(name = 'KBLz2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLz2}',
                  lhablock = 'KBLZ',
                  lhacode = [ 2 ])

KBLz3 = Parameter(name = 'KBLz3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBLz3}',
                  lhablock = 'KBLZ',
                  lhacode = [ 3 ])

KBRh1 = Parameter(name = 'KBRh1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRh1}',
                  lhablock = 'KBRH',
                  lhacode = [ 1 ])

KBRh2 = Parameter(name = 'KBRh2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRh2}',
                  lhablock = 'KBRH',
                  lhacode = [ 2 ])

KBRh3 = Parameter(name = 'KBRh3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRh3}',
                  lhablock = 'KBRH',
                  lhacode = [ 3 ])

KBRw1 = Parameter(name = 'KBRw1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRw1}',
                  lhablock = 'KBRW',
                  lhacode = [ 1 ])

KBRw2 = Parameter(name = 'KBRw2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRw2}',
                  lhablock = 'KBRW',
                  lhacode = [ 2 ])

KBRw3 = Parameter(name = 'KBRw3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRw3}',
                  lhablock = 'KBRW',
                  lhacode = [ 3 ])

KBRz1 = Parameter(name = 'KBRz1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRz1}',
                  lhablock = 'KBRZ',
                  lhacode = [ 1 ])

KBRz2 = Parameter(name = 'KBRz2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRz2}',
                  lhablock = 'KBRZ',
                  lhacode = [ 2 ])

KBRz3 = Parameter(name = 'KBRz3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KBRz3}',
                  lhablock = 'KBRZ',
                  lhacode = [ 3 ])

KHDS10DS10 = Parameter(name = 'KHDS10DS10',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KHDS10DS10}',
                       lhablock = 'KHDS10DS10',
                       lhacode = [ 1 ])

KHWW = Parameter(name = 'KHWW',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KHWW}',
                 lhablock = 'KHVV',
                 lhacode = [ 1 ])

KHZZ = Parameter(name = 'KHZZ',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KHZZ}',
                 lhablock = 'KHVV',
                 lhacode = [ 2 ])

KP10D1x1 = Parameter(name = 'KP10D1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10D1x1}',
                     lhablock = 'KP10D',
                     lhacode = [ 1, 1 ])

KP10D2x2 = Parameter(name = 'KP10D2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10D2x2}',
                     lhablock = 'KP10D',
                     lhacode = [ 2, 2 ])

KP10D3x3 = Parameter(name = 'KP10D3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10D3x3}',
                     lhablock = 'KP10D',
                     lhacode = [ 3, 3 ])

KP10E1x1 = Parameter(name = 'KP10E1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10E1x1}',
                     lhablock = 'KP10E',
                     lhacode = [ 1, 1 ])

KP10E2x2 = Parameter(name = 'KP10E2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10E2x2}',
                     lhablock = 'KP10E',
                     lhacode = [ 2, 2 ])

KP10E3x3 = Parameter(name = 'KP10E3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10E3x3}',
                     lhablock = 'KP10E',
                     lhacode = [ 3, 3 ])

KP10N1x1 = Parameter(name = 'KP10N1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10N1x1}',
                     lhablock = 'KP10N',
                     lhacode = [ 1, 1 ])

KP10N2x2 = Parameter(name = 'KP10N2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10N2x2}',
                     lhablock = 'KP10N',
                     lhacode = [ 2, 2 ])

KP10N3x3 = Parameter(name = 'KP10N3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10N3x3}',
                     lhablock = 'KP10N',
                     lhacode = [ 3, 3 ])

KP10TT = Parameter(name = 'KP10TT',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10TT}',
                   lhablock = 'KP10QQ',
                   lhacode = [ 1 ])

KP10BB = Parameter(name = 'KP10BB',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10BB}',
                   lhablock = 'KP10QQ',
                   lhacode = [ 2 ])

KP10XX = Parameter(name = 'KP10XX',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10XX}',
                   lhablock = 'KP10QQ',
                   lhacode = [ 3 ])

KP10YY = Parameter(name = 'KP10YY',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10YY}',
                   lhablock = 'KP10QQ',
                   lhacode = [ 4 ])

KP10U1x1 = Parameter(name = 'KP10U1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10U1x1}',
                     lhablock = 'KP10U',
                     lhacode = [ 1, 1 ])

KP10U2x2 = Parameter(name = 'KP10U2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10U2x2}',
                     lhablock = 'KP10U',
                     lhacode = [ 2, 2 ])

KP10U3x3 = Parameter(name = 'KP10U3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KP10U3x3}',
                     lhablock = 'KP10U',
                     lhacode = [ 3, 3 ])

KP10G = Parameter(name = 'KP10G',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KP10G}',
                  lhablock = 'KP10VV',
                  lhacode = [ 1 ])

KP10W = Parameter(name = 'KP10W',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KP10W}',
                  lhablock = 'KP10VV',
                  lhacode = [ 2 ])

KP10ZZ = Parameter(name = 'KP10ZZ',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10ZZ}',
                   lhablock = 'KP10VV',
                   lhacode = [ 3 ])

KP10ZA = Parameter(name = 'KP10ZA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10ZA}',
                   lhablock = 'KP10VV',
                   lhacode = [ 4 ])

KP10AA = Parameter(name = 'KP10AA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP10AA}',
                   lhablock = 'KP10VV',
                   lhacode = [ 5 ])

KP11WA = Parameter(name = 'KP11WA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP11WA}',
                   lhablock = 'KP11VV',
                   lhacode = [ 1 ])

KP11WZ = Parameter(name = 'KP11WZ',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KP11WZ}',
                   lhablock = 'KP11VV',
                   lhacode = [ 2 ])

KS10BL1 = Parameter(name = 'KS10BL1',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BL1}',
                    lhablock = 'KS10BL',
                    lhacode = [ 1 ])

KS10BL2 = Parameter(name = 'KS10BL2',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BL2}',
                    lhablock = 'KS10BL',
                    lhacode = [ 2 ])

KS10BL3 = Parameter(name = 'KS10BL3',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BL3}',
                    lhablock = 'KS10BL',
                    lhacode = [ 3 ])

KS10BR1 = Parameter(name = 'KS10BR1',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BR1}',
                    lhablock = 'KS10BR',
                    lhacode = [ 1 ])

KS10BR2 = Parameter(name = 'KS10BR2',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BR2}',
                    lhablock = 'KS10BR',
                    lhacode = [ 2 ])

KS10BR3 = Parameter(name = 'KS10BR3',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10BR3}',
                    lhablock = 'KS10BR',
                    lhacode = [ 3 ])

KS10D1x1 = Parameter(name = 'KS10D1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10D1x1}',
                     lhablock = 'KS10D',
                     lhacode = [ 1, 1 ])

KS10D2x2 = Parameter(name = 'KS10D2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10D2x2}',
                     lhablock = 'KS10D',
                     lhacode = [ 2, 2 ])

KS10D3x3 = Parameter(name = 'KS10D3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10D3x3}',
                     lhablock = 'KS10D',
                     lhacode = [ 3, 3 ])

KS10E1x1 = Parameter(name = 'KS10E1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10E1x1}',
                     lhablock = 'KS10E',
                     lhacode = [ 1, 1 ])

KS10E2x2 = Parameter(name = 'KS10E2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10E2x2}',
                     lhablock = 'KS10E',
                     lhacode = [ 2, 2 ])

KS10E3x3 = Parameter(name = 'KS10E3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10E3x3}',
                     lhablock = 'KS10E',
                     lhacode = [ 3, 3 ])

KHS10S10 = Parameter(name = 'KHS10S10',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KHS10S10}',
                     lhablock = 'KS10H',
                     lhacode = [ 1 ])

KHHS10 = Parameter(name = 'KHHS10',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KHHS10}',
                   lhablock = 'KS10H',
                   lhacode = [ 2 ])

KHS10Z = Parameter(name = 'KHS10Z',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KHS10Z}',
                   lhablock = 'KS10H',
                   lhacode = [ 3 ])

KS10N1x1 = Parameter(name = 'KS10N1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10N1x1}',
                     lhablock = 'KS10N',
                     lhacode = [ 1, 1 ])

KS10N2x2 = Parameter(name = 'KS10N2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10N2x2}',
                     lhablock = 'KS10N',
                     lhacode = [ 2, 2 ])

KS10N3x3 = Parameter(name = 'KS10N3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10N3x3}',
                     lhablock = 'KS10N',
                     lhacode = [ 3, 3 ])

KS10TT = Parameter(name = 'KS10TT',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10TT}',
                   lhablock = 'KS10QQ',
                   lhacode = [ 1 ])

KS10BB = Parameter(name = 'KS10BB',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10BB}',
                   lhablock = 'KS10QQ',
                   lhacode = [ 2 ])

KS10XX = Parameter(name = 'KS10XX',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10XX}',
                   lhablock = 'KS10QQ',
                   lhacode = [ 3 ])

KS10YY = Parameter(name = 'KS10YY',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10YY}',
                   lhablock = 'KS10QQ',
                   lhacode = [ 4 ])

KS10TL1 = Parameter(name = 'KS10TL1',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TL1}',
                    lhablock = 'KS10TL',
                    lhacode = [ 1 ])

KS10TL2 = Parameter(name = 'KS10TL2',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TL2}',
                    lhablock = 'KS10TL',
                    lhacode = [ 2 ])

KS10TL3 = Parameter(name = 'KS10TL3',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TL3}',
                    lhablock = 'KS10TL',
                    lhacode = [ 3 ])

KS10TR1 = Parameter(name = 'KS10TR1',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TR1}',
                    lhablock = 'KS10TR',
                    lhacode = [ 1 ])

KS10TR2 = Parameter(name = 'KS10TR2',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TR2}',
                    lhablock = 'KS10TR',
                    lhacode = [ 2 ])

KS10TR3 = Parameter(name = 'KS10TR3',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS10TR3}',
                    lhablock = 'KS10TR',
                    lhacode = [ 3 ])

KS10U1x1 = Parameter(name = 'KS10U1x1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10U1x1}',
                     lhablock = 'KS10U',
                     lhacode = [ 1, 1 ])

KS10U2x2 = Parameter(name = 'KS10U2x2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10U2x2}',
                     lhablock = 'KS10U',
                     lhacode = [ 2, 2 ])

KS10U3x3 = Parameter(name = 'KS10U3x3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS10U3x3}',
                     lhablock = 'KS10U',
                     lhacode = [ 3, 3 ])

KS10G = Parameter(name = 'KS10G',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KS10G}',
                  lhablock = 'KS10VV',
                  lhacode = [ 1 ])

KS10W = Parameter(name = 'KS10W',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KS10W}',
                  lhablock = 'KS10VV',
                  lhacode = [ 2 ])

KS10ZZ = Parameter(name = 'KS10ZZ',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10ZZ}',
                   lhablock = 'KS10VV',
                   lhacode = [ 3 ])

KS10ZA = Parameter(name = 'KS10ZA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10ZA}',
                   lhablock = 'KS10VV',
                   lhacode = [ 4 ])

KS10AA = Parameter(name = 'KS10AA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS10AA}',
                   lhablock = 'KS10VV',
                   lhacode = [ 5 ])

KS11BuL1 = Parameter(name = 'KS11BuL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuL1}',
                     lhablock = 'KS11BuL',
                     lhacode = [ 1 ])

KS11BuL2 = Parameter(name = 'KS11BuL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuL2}',
                     lhablock = 'KS11BuL',
                     lhacode = [ 2 ])

KS11BuL3 = Parameter(name = 'KS11BuL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuL3}',
                     lhablock = 'KS11BuL',
                     lhacode = [ 3 ])

KS11BuR1 = Parameter(name = 'KS11BuR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuR1}',
                     lhablock = 'KS11BuR',
                     lhacode = [ 1 ])

KS11BuR2 = Parameter(name = 'KS11BuR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuR2}',
                     lhablock = 'KS11BuR',
                     lhacode = [ 2 ])

KS11BuR3 = Parameter(name = 'KS11BuR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11BuR3}',
                     lhablock = 'KS11BuR',
                     lhacode = [ 3 ])

KS11BYL = Parameter(name = 'KS11BYL',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11BYL}',
                    lhablock = 'KS11BY',
                    lhacode = [ 1 ])

KS11BYR = Parameter(name = 'KS11BYR',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11BYR}',
                    lhablock = 'KS11BY',
                    lhacode = [ 2 ])

KS11H1 = Parameter(name = 'KS11H1',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11H1}',
                   lhablock = 'KS11H',
                   lhacode = [ 1 ])

KS11H2 = Parameter(name = 'KS11H2',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11H2}',
                   lhablock = 'KS11H',
                   lhacode = [ 2 ])

KS11HH = Parameter(name = 'KS11HH',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11HH}',
                   lhablock = 'KS11H',
                   lhacode = [ 3 ])

KS11HW = Parameter(name = 'KS11HW',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11HW}',
                   lhablock = 'KS11H',
                   lhacode = [ 4 ])

KS11ll1x1 = Parameter(name = 'KS11ll1x1',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KS11ll1x1}',
                      lhablock = 'KS11ll',
                      lhacode = [ 1, 1 ])

KS11ll2x2 = Parameter(name = 'KS11ll2x2',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KS11ll2x2}',
                      lhablock = 'KS11ll',
                      lhacode = [ 2, 2 ])

KS11ll3x3 = Parameter(name = 'KS11ll3x3',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KS11ll3x3}',
                      lhablock = 'KS11ll',
                      lhacode = [ 3, 3 ])

KS11qqL1x1 = Parameter(name = 'KS11qqL1x1',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqL1x1}',
                       lhablock = 'KS11qqL',
                       lhacode = [ 1, 1 ])

KS11qqL2x2 = Parameter(name = 'KS11qqL2x2',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqL2x2}',
                       lhablock = 'KS11qqL',
                       lhacode = [ 2, 2 ])

KS11qqL3x3 = Parameter(name = 'KS11qqL3x3',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqL3x3}',
                       lhablock = 'KS11qqL',
                       lhacode = [ 3, 3 ])

KS11qqR1x1 = Parameter(name = 'KS11qqR1x1',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqR1x1}',
                       lhablock = 'KS11qqR',
                       lhacode = [ 1, 1 ])

KS11qqR2x2 = Parameter(name = 'KS11qqR2x2',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqR2x2}',
                       lhablock = 'KS11qqR',
                       lhacode = [ 2, 2 ])

KS11qqR3x3 = Parameter(name = 'KS11qqR3x3',
                       nature = 'external',
                       type = 'real',
                       value = 0,
                       texname = '\\text{KS11qqR3x3}',
                       lhablock = 'KS11qqR',
                       lhacode = [ 3, 3 ])

KS11TBL = Parameter(name = 'KS11TBL',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11TBL}',
                    lhablock = 'KS11TB',
                    lhacode = [ 1 ])

KS11TBR = Parameter(name = 'KS11TBR',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11TBR}',
                    lhablock = 'KS11TB',
                    lhacode = [ 2 ])

KS11TdL1 = Parameter(name = 'KS11TdL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdL1}',
                     lhablock = 'KS11TdL',
                     lhacode = [ 1 ])

KS11TdL2 = Parameter(name = 'KS11TdL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdL2}',
                     lhablock = 'KS11TdL',
                     lhacode = [ 2 ])

KS11TdL3 = Parameter(name = 'KS11TdL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdL3}',
                     lhablock = 'KS11TdL',
                     lhacode = [ 3 ])

KS11TdR1 = Parameter(name = 'KS11TdR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdR1}',
                     lhablock = 'KS11TdR',
                     lhacode = [ 1 ])

KS11TdR2 = Parameter(name = 'KS11TdR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdR2}',
                     lhablock = 'KS11TdR',
                     lhacode = [ 2 ])

KS11TdR3 = Parameter(name = 'KS11TdR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11TdR3}',
                     lhablock = 'KS11TdR',
                     lhacode = [ 3 ])

KS11WA = Parameter(name = 'KS11WA',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11WA}',
                   lhablock = 'KS11VV',
                   lhacode = [ 1 ])

KS11WZ = Parameter(name = 'KS11WZ',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS11WZ}',
                   lhablock = 'KS11VV',
                   lhacode = [ 2 ])

KS11XTL = Parameter(name = 'KS11XTL',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11XTL}',
                    lhablock = 'KS11XT',
                    lhacode = [ 1 ])

KS11XTR = Parameter(name = 'KS11XTR',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS11XTR}',
                    lhablock = 'KS11XT',
                    lhacode = [ 2 ])

KS11XuL1 = Parameter(name = 'KS11XuL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuL1}',
                     lhablock = 'KS11XuL',
                     lhacode = [ 1 ])

KS11XuL2 = Parameter(name = 'KS11XuL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuL2}',
                     lhablock = 'KS11XuL',
                     lhacode = [ 2 ])

KS11XuL3 = Parameter(name = 'KS11XuL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuL3}',
                     lhablock = 'KS11XuL',
                     lhacode = [ 3 ])

KS11XuR1 = Parameter(name = 'KS11XuR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuR1}',
                     lhablock = 'KS11XuR',
                     lhacode = [ 1 ])

KS11XuR2 = Parameter(name = 'KS11XuR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuR2}',
                     lhablock = 'KS11XuR',
                     lhacode = [ 2 ])

KS11XuR3 = Parameter(name = 'KS11XuR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11XuR3}',
                     lhablock = 'KS11XuR',
                     lhacode = [ 3 ])

KS11YdL1 = Parameter(name = 'KS11YdL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdL1}',
                     lhablock = 'KS11YdL',
                     lhacode = [ 1 ])

KS11YdL2 = Parameter(name = 'KS11YdL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdL2}',
                     lhablock = 'KS11YdL',
                     lhacode = [ 2 ])

KS11YdL3 = Parameter(name = 'KS11YdL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdL3}',
                     lhablock = 'KS11YdL',
                     lhacode = [ 3 ])

KS11YdR1 = Parameter(name = 'KS11YdR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdR1}',
                     lhablock = 'KS11YdR',
                     lhacode = [ 1 ])

KS11YdR2 = Parameter(name = 'KS11YdR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdR2}',
                     lhablock = 'KS11YdR',
                     lhacode = [ 2 ])

KS11YdR3 = Parameter(name = 'KS11YdR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS11YdR3}',
                     lhablock = 'KS11YdR',
                     lhacode = [ 3 ])

KS12H1 = Parameter(name = 'KS12H1',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS12H1}',
                   lhablock = 'KS12H',
                   lhacode = [ 1 ])

KS12H2 = Parameter(name = 'KS12H2',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS12H2}',
                   lhablock = 'KS12H',
                   lhacode = [ 2 ])

KS12HH = Parameter(name = 'KS12HH',
                   nature = 'external',
                   type = 'real',
                   value = 0,
                   texname = '\\text{KS12HH}',
                   lhablock = 'KS12H',
                   lhacode = [ 3 ])

KS12XBL = Parameter(name = 'KS12XBL',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS12XBL}',
                    lhablock = 'KS12QQ',
                    lhacode = [ 1 ])

KS12XBR = Parameter(name = 'KS12XBR',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS12XBR}',
                    lhablock = 'KS12QQ',
                    lhacode = [ 2 ])

KS12YTL = Parameter(name = 'KS12YTL',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS12YTL}',
                    lhablock = 'KS12QQ',
                    lhacode = [ 3 ])

KS12YTR = Parameter(name = 'KS12YTR',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KS12YTR}',
                    lhablock = 'KS12QQ',
                    lhacode = [ 4 ])

KS12W = Parameter(name = 'KS12W',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KS12W}',
                  lhablock = 'KS12WW',
                  lhacode = [ 1 ])

KP12W = Parameter(name = 'KP12W',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KP12W}',
                  lhablock = 'KS12WW',
                  lhacode = [ 2 ])

KS12XDL1 = Parameter(name = 'KS12XDL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDL1}',
                     lhablock = 'KS12XDL',
                     lhacode = [ 1 ])

KS12XDL2 = Parameter(name = 'KS12XDL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDL2}',
                     lhablock = 'KS12XDL',
                     lhacode = [ 2 ])

KS12XDL3 = Parameter(name = 'KS12XDL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDL3}',
                     lhablock = 'KS12XDL',
                     lhacode = [ 3 ])

KS12XDR1 = Parameter(name = 'KS12XDR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDR1}',
                     lhablock = 'KS12XDR',
                     lhacode = [ 1 ])

KS12XDR2 = Parameter(name = 'KS12XDR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDR2}',
                     lhablock = 'KS12XDR',
                     lhacode = [ 2 ])

KS12XDR3 = Parameter(name = 'KS12XDR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12XDR3}',
                     lhablock = 'KS12XDR',
                     lhacode = [ 3 ])

KS12YUL1 = Parameter(name = 'KS12YUL1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUL1}',
                     lhablock = 'KS12YUL',
                     lhacode = [ 1 ])

KS12YUL2 = Parameter(name = 'KS12YUL2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUL2}',
                     lhablock = 'KS12YUL',
                     lhacode = [ 2 ])

KS12YUL3 = Parameter(name = 'KS12YUL3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUL3}',
                     lhablock = 'KS12YUL',
                     lhacode = [ 3 ])

KS12YUR1 = Parameter(name = 'KS12YUR1',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUR1}',
                     lhablock = 'KS12YUR',
                     lhacode = [ 1 ])

KS12YUR2 = Parameter(name = 'KS12YUR2',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUR2}',
                     lhablock = 'KS12YUR',
                     lhacode = [ 2 ])

KS12YUR3 = Parameter(name = 'KS12YUR3',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KS12YUR3}',
                     lhablock = 'KS12YUR',
                     lhacode = [ 3 ])

KTLh1 = Parameter(name = 'KTLh1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLh1}',
                  lhablock = 'KTLH',
                  lhacode = [ 1 ])

KTLh2 = Parameter(name = 'KTLh2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLh2}',
                  lhablock = 'KTLH',
                  lhacode = [ 2 ])

KTLh3 = Parameter(name = 'KTLh3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLh3}',
                  lhablock = 'KTLH',
                  lhacode = [ 3 ])

KTLw1 = Parameter(name = 'KTLw1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLw1}',
                  lhablock = 'KTLW',
                  lhacode = [ 1 ])

KTLw2 = Parameter(name = 'KTLw2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLw2}',
                  lhablock = 'KTLW',
                  lhacode = [ 2 ])

KTLw3 = Parameter(name = 'KTLw3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLw3}',
                  lhablock = 'KTLW',
                  lhacode = [ 3 ])

KTLz1 = Parameter(name = 'KTLz1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLz1}',
                  lhablock = 'KTLZ',
                  lhacode = [ 1 ])

KTLz2 = Parameter(name = 'KTLz2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLz2}',
                  lhablock = 'KTLZ',
                  lhacode = [ 2 ])

KTLz3 = Parameter(name = 'KTLz3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTLz3}',
                  lhablock = 'KTLZ',
                  lhacode = [ 3 ])

KTRh1 = Parameter(name = 'KTRh1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRh1}',
                  lhablock = 'KTRH',
                  lhacode = [ 1 ])

KTRh2 = Parameter(name = 'KTRh2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRh2}',
                  lhablock = 'KTRH',
                  lhacode = [ 2 ])

KTRh3 = Parameter(name = 'KTRh3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRh3}',
                  lhablock = 'KTRH',
                  lhacode = [ 3 ])

KTRw1 = Parameter(name = 'KTRw1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRw1}',
                  lhablock = 'KTRW',
                  lhacode = [ 1 ])

KTRw2 = Parameter(name = 'KTRw2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRw2}',
                  lhablock = 'KTRW',
                  lhacode = [ 2 ])

KTRw3 = Parameter(name = 'KTRw3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRw3}',
                  lhablock = 'KTRW',
                  lhacode = [ 3 ])

KTRz1 = Parameter(name = 'KTRz1',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRz1}',
                  lhablock = 'KTRZ',
                  lhacode = [ 1 ])

KTRz2 = Parameter(name = 'KTRz2',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRz2}',
                  lhablock = 'KTRZ',
                  lhacode = [ 2 ])

KTRz3 = Parameter(name = 'KTRz3',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KTRz3}',
                  lhablock = 'KTRZ',
                  lhacode = [ 3 ])

KWPWMSS10 = Parameter(name = 'KWPWMSS10',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWPWMSS10}',
                      lhablock = 'KWPWMSS',
                      lhacode = [ 1 ])

KWPWMSS11 = Parameter(name = 'KWPWMSS11',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWPWMSS11}',
                      lhablock = 'KWPWMSS',
                      lhacode = [ 2 ])

KWPWMSS12 = Parameter(name = 'KWPWMSS12',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWPWMSS12}',
                      lhablock = 'KWPWMSS',
                      lhacode = [ 3 ])

KWPWPSS11 = Parameter(name = 'KWPWPSS11',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWPWPSS11}',
                      lhablock = 'KWPWPSS',
                      lhacode = [ 1 ])

KWPWPSS12 = Parameter(name = 'KWPWPSS12',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWPWPSS12}',
                      lhablock = 'KWPWPSS',
                      lhacode = [ 2 ])

KWXTL = Parameter(name = 'KWXTL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWXTL}',
                  lhablock = 'KWQQ',
                  lhacode = [ 1 ])

KWXTR = Parameter(name = 'KWXTR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWXTR}',
                  lhablock = 'KWQQ',
                  lhacode = [ 2 ])

KWTBL = Parameter(name = 'KWTBL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWTBL}',
                  lhablock = 'KWQQ',
                  lhacode = [ 3 ])

KWTBR = Parameter(name = 'KWTBR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWTBR}',
                  lhablock = 'KWQQ',
                  lhacode = [ 4 ])

KWBYL = Parameter(name = 'KWBYL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWBYL}',
                  lhablock = 'KWQQ',
                  lhacode = [ 5 ])

KWBYR = Parameter(name = 'KWBYR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KWBYR}',
                  lhablock = 'KWQQ',
                  lhacode = [ 6 ])

KWS10S11 = Parameter(name = 'KWS10S11',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KWS10S11}',
                     lhablock = 'KWSS',
                     lhacode = [ 1 ])

KWS11S12 = Parameter(name = 'KWS11S12',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KWS11S12}',
                     lhablock = 'KWSS',
                     lhacode = [ 2 ])

KWZS10S11 = Parameter(name = 'KWZS10S11',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWZS10S11}',
                      lhablock = 'KWZSS',
                      lhacode = [ 1 ])

KWZS11S12 = Parameter(name = 'KWZS11S12',
                      nature = 'external',
                      type = 'real',
                      value = 0,
                      texname = '\\text{KWZS11S12}',
                      lhablock = 'KWZSS',
                      lhacode = [ 2 ])

KXL1 = Parameter(name = 'KXL1',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXL1}',
                 lhablock = 'KXLW',
                 lhacode = [ 1 ])

KXL2 = Parameter(name = 'KXL2',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXL2}',
                 lhablock = 'KXLW',
                 lhacode = [ 2 ])

KXL3 = Parameter(name = 'KXL3',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXL3}',
                 lhablock = 'KXLW',
                 lhacode = [ 3 ])

KXR1 = Parameter(name = 'KXR1',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXR1}',
                 lhablock = 'KXRW',
                 lhacode = [ 1 ])

KXR2 = Parameter(name = 'KXR2',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXR2}',
                 lhablock = 'KXRW',
                 lhacode = [ 2 ])

KXR3 = Parameter(name = 'KXR3',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KXR3}',
                 lhablock = 'KXRW',
                 lhacode = [ 3 ])

KYL1 = Parameter(name = 'KYL1',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYL1}',
                 lhablock = 'KYLW',
                 lhacode = [ 1 ])

KYL2 = Parameter(name = 'KYL2',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYL2}',
                 lhablock = 'KYLW',
                 lhacode = [ 2 ])

KYL3 = Parameter(name = 'KYL3',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYL3}',
                 lhablock = 'KYLW',
                 lhacode = [ 3 ])

KYR1 = Parameter(name = 'KYR1',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYR1}',
                 lhablock = 'KYRW',
                 lhacode = [ 1 ])

KYR2 = Parameter(name = 'KYR2',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYR2}',
                 lhablock = 'KYRW',
                 lhacode = [ 2 ])

KYR3 = Parameter(name = 'KYR3',
                 nature = 'external',
                 type = 'real',
                 value = 0,
                 texname = '\\text{KYR3}',
                 lhablock = 'KYRW',
                 lhacode = [ 3 ])

KZXXL = Parameter(name = 'KZXXL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZXXL}',
                  lhablock = 'KZQQ',
                  lhacode = [ 1 ])

KZXXR = Parameter(name = 'KZXXR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZXXR}',
                  lhablock = 'KZQQ',
                  lhacode = [ 2 ])

KZTTL = Parameter(name = 'KZTTL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZTTL}',
                  lhablock = 'KZQQ',
                  lhacode = [ 3 ])

KZTTR = Parameter(name = 'KZTTR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZTTR}',
                  lhablock = 'KZQQ',
                  lhacode = [ 4 ])

KZBBL = Parameter(name = 'KZBBL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZBBL}',
                  lhablock = 'KZQQ',
                  lhacode = [ 5 ])

KZBBR = Parameter(name = 'KZBBR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZBBR}',
                  lhablock = 'KZQQ',
                  lhacode = [ 6 ])

KZYYL = Parameter(name = 'KZYYL',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZYYL}',
                  lhablock = 'KZQQ',
                  lhacode = [ 7 ])

KZYYR = Parameter(name = 'KZYYR',
                  nature = 'external',
                  type = 'real',
                  value = 0,
                  texname = '\\text{KZYYR}',
                  lhablock = 'KZQQ',
                  lhacode = [ 8 ])

KZS11S11 = Parameter(name = 'KZS11S11',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KZS11S11}',
                     lhablock = 'KZSS',
                     lhacode = [ 1 ])

KZS12S12 = Parameter(name = 'KZS12S12',
                     nature = 'external',
                     type = 'real',
                     value = 0,
                     texname = '\\text{KZS12S12}',
                     lhablock = 'KZSS',
                     lhacode = [ 2 ])

KZZSS10 = Parameter(name = 'KZZSS10',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KZZSS10}',
                    lhablock = 'KZZSS',
                    lhacode = [ 1 ])

KZZSS11 = Parameter(name = 'KZZSS11',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KZZSS11}',
                    lhablock = 'KZZSS',
                    lhacode = [ 2 ])

KZZSS12 = Parameter(name = 'KZZSS12',
                    nature = 'external',
                    type = 'real',
                    value = 0,
                    texname = '\\text{KZZSS12}',
                    lhablock = 'KZZSS',
                    lhacode = [ 3 ])

aEWM1 = Parameter(name = 'aEWM1',
                  nature = 'external',
                  type = 'real',
                  value = 127.9,
                  texname = '\\text{aEWM1}',
                  lhablock = 'SMINPUTS',
                  lhacode = [ 1 ])

Gf = Parameter(name = 'Gf',
               nature = 'external',
               type = 'real',
               value = 0.0000116637,
               texname = 'G_f',
               lhablock = 'SMINPUTS',
               lhacode = [ 2 ])

aS = Parameter(name = 'aS',
               nature = 'external',
               type = 'real',
               value = 0.1184,
               texname = '\\alpha _s',
               lhablock = 'SMINPUTS',
               lhacode = [ 3 ])

ymb = Parameter(name = 'ymb',
                nature = 'external',
                type = 'real',
                value = 4.7,
                texname = '\\text{ymb}',
                lhablock = 'YUKAWA',
                lhacode = [ 5 ])

ymt = Parameter(name = 'ymt',
                nature = 'external',
                type = 'real',
                value = 172,
                texname = '\\text{ymt}',
                lhablock = 'YUKAWA',
                lhacode = [ 6 ])

ymtau = Parameter(name = 'ymtau',
                  nature = 'external',
                  type = 'real',
                  value = 1.777,
                  texname = '\\text{ymtau}',
                  lhablock = 'YUKAWA',
                  lhacode = [ 15 ])

MZ = Parameter(name = 'MZ',
               nature = 'external',
               type = 'real',
               value = 91.1876,
               texname = '\\text{MZ}',
               lhablock = 'MASS',
               lhacode = [ 23 ])

MTA = Parameter(name = 'MTA',
                nature = 'external',
                type = 'real',
                value = 1.777,
                texname = '\\text{MTA}',
                lhablock = 'MASS',
                lhacode = [ 15 ])

MT = Parameter(name = 'MT',
               nature = 'external',
               type = 'real',
               value = 172,
               texname = '\\text{MT}',
               lhablock = 'MASS',
               lhacode = [ 6 ])

MB = Parameter(name = 'MB',
               nature = 'external',
               type = 'real',
               value = 4.7,
               texname = '\\text{MB}',
               lhablock = 'MASS',
               lhacode = [ 5 ])

MH = Parameter(name = 'MH',
               nature = 'external',
               type = 'real',
               value = 125,
               texname = '\\text{MH}',
               lhablock = 'MASS',
               lhacode = [ 25 ])

MX = Parameter(name = 'MX',
               nature = 'external',
               type = 'real',
               value = 1000,
               texname = '\\text{MX}',
               lhablock = 'MASS',
               lhacode = [ 6000005 ])

MTP = Parameter(name = 'MTP',
                nature = 'external',
                type = 'real',
                value = 1100,
                texname = '\\text{MTP}',
                lhablock = 'MASS',
                lhacode = [ 6000006 ])

MBP = Parameter(name = 'MBP',
                nature = 'external',
                type = 'real',
                value = 1200,
                texname = '\\text{MBP}',
                lhablock = 'MASS',
                lhacode = [ 6000007 ])

MY = Parameter(name = 'MY',
               nature = 'external',
               type = 'real',
               value = 1300,
               texname = '\\text{MY}',
               lhablock = 'MASS',
               lhacode = [ 6000008 ])

MS10 = Parameter(name = 'MS10',
                 nature = 'external',
                 type = 'real',
                 value = 200,
                 texname = '\\text{MS10}',
                 lhablock = 'MASS',
                 lhacode = [ 6100001 ])

MS11 = Parameter(name = 'MS11',
                 nature = 'external',
                 type = 'real',
                 value = 500,
                 texname = '\\text{MS11}',
                 lhablock = 'MASS',
                 lhacode = [ 6100002 ])

MS12 = Parameter(name = 'MS12',
                 nature = 'external',
                 type = 'real',
                 value = 500.5,
                 texname = '\\text{MS12}',
                 lhablock = 'MASS',
                 lhacode = [ 6100003 ])

WZ = Parameter(name = 'WZ',
               nature = 'external',
               type = 'real',
               value = 2.4952,
               texname = '\\text{WZ}',
               lhablock = 'DECAY',
               lhacode = [ 23 ])

WW = Parameter(name = 'WW',
               nature = 'external',
               type = 'real',
               value = 2.085,
               texname = '\\text{WW}',
               lhablock = 'DECAY',
               lhacode = [ 24 ])

WT = Parameter(name = 'WT',
               nature = 'external',
               type = 'real',
               value = 1.50833649,
               texname = '\\text{WT}',
               lhablock = 'DECAY',
               lhacode = [ 6 ])

WH = Parameter(name = 'WH',
               nature = 'external',
               type = 'real',
               value = 0.00407,
               texname = '\\text{WH}',
               lhablock = 'DECAY',
               lhacode = [ 25 ])

WX = Parameter(name = 'WX',
               nature = 'external',
               type = 'real',
               value = 10.,
               texname = '\\text{WX}',
               lhablock = 'DECAY',
               lhacode = [ 6000005 ])

WTP = Parameter(name = 'WTP',
                nature = 'external',
                type = 'real',
                value = 11.,
                texname = '\\text{WTP}',
                lhablock = 'DECAY',
                lhacode = [ 6000006 ])

WBP = Parameter(name = 'WBP',
                nature = 'external',
                type = 'real',
                value = 12.,
                texname = '\\text{WBP}',
                lhablock = 'DECAY',
                lhacode = [ 6000007 ])

WY = Parameter(name = 'WY',
               nature = 'external',
               type = 'real',
               value = 13.,
               texname = '\\text{WY}',
               lhablock = 'DECAY',
               lhacode = [ 6000008 ])

WS10 = Parameter(name = 'WS10',
                 nature = 'external',
                 type = 'real',
                 value = 1.1,
                 texname = '\\text{WS10}',
                 lhablock = 'DECAY',
                 lhacode = [ 6100001 ])

WS11 = Parameter(name = 'WS11',
                 nature = 'external',
                 type = 'real',
                 value = 15,
                 texname = '\\text{WS11}',
                 lhablock = 'DECAY',
                 lhacode = [ 6100002 ])

WS12 = Parameter(name = 'WS12',
                 nature = 'external',
                 type = 'real',
                 value = 1.1,
                 texname = '\\text{WS12}',
                 lhablock = 'DECAY',
                 lhacode = [ 6100003 ])

aEW = Parameter(name = 'aEW',
                nature = 'internal',
                type = 'real',
                value = '1/aEWM1',
                texname = '\\alpha _{\\text{EW}}')

G = Parameter(name = 'G',
              nature = 'internal',
              type = 'real',
              value = '2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)',
              texname = 'G')

MW = Parameter(name = 'MW',
               nature = 'internal',
               type = 'real',
               value = 'cmath.sqrt(MZ**2/2. + cmath.sqrt(MZ**4/4. - (aEW*cmath.pi*MZ**2)/(Gf*cmath.sqrt(2))))',
               texname = 'M_W')

ee = Parameter(name = 'ee',
               nature = 'internal',
               type = 'real',
               value = '2*cmath.sqrt(aEW)*cmath.sqrt(cmath.pi)',
               texname = 'e')

sw2 = Parameter(name = 'sw2',
                nature = 'internal',
                type = 'real',
                value = '1 - MW**2/MZ**2',
                texname = '\\text{sw2}')

cw = Parameter(name = 'cw',
               nature = 'internal',
               type = 'real',
               value = 'cmath.sqrt(1 - sw2)',
               texname = 'c_w')

sw = Parameter(name = 'sw',
               nature = 'internal',
               type = 'real',
               value = 'cmath.sqrt(sw2)',
               texname = 's_w')

g1 = Parameter(name = 'g1',
               nature = 'internal',
               type = 'real',
               value = 'ee/cw',
               texname = 'g_1')

gw = Parameter(name = 'gw',
               nature = 'internal',
               type = 'real',
               value = 'ee/sw',
               texname = 'g_w')

vev = Parameter(name = 'vev',
                nature = 'internal',
                type = 'real',
                value = '(2*MW*sw)/ee',
                texname = '\\text{vev}')

lam = Parameter(name = 'lam',
                nature = 'internal',
                type = 'real',
                value = 'MH**2/(2.*vev**2)',
                texname = '\\text{lam}')

yb = Parameter(name = 'yb',
               nature = 'internal',
               type = 'real',
               value = '(ymb*cmath.sqrt(2))/vev',
               texname = '\\text{yb}')

yt = Parameter(name = 'yt',
               nature = 'internal',
               type = 'real',
               value = '(ymt*cmath.sqrt(2))/vev',
               texname = '\\text{yt}')

ytau = Parameter(name = 'ytau',
                 nature = 'internal',
                 type = 'real',
                 value = '(ymtau*cmath.sqrt(2))/vev',
                 texname = '\\text{ytau}')

muH = Parameter(name = 'muH',
                nature = 'internal',
                type = 'real',
                value = 'cmath.sqrt(lam*vev**2)',
                texname = '\\mu')

KHBB1 = Parameter(name = 'KHBB1',
                  nature = 'internal',
                  type = 'real',
                  value = '((-1 + KHBB)*yb)/cmath.sqrt(2)',
                  texname = '\\text{KHBB}_1')

KHTT1 = Parameter(name = 'KHTT1',
                  nature = 'internal',
                  type = 'real',
                  value = '((-1 + KHTT)*yt)/cmath.sqrt(2)',
                  texname = '\\text{KHTT}_1')

