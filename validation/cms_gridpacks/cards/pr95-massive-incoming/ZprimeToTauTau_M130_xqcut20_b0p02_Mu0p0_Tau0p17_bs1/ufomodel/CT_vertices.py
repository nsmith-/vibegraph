# This file was automatically created by FeynRules 2.3.29
# Mathematica version: 10.3.1 for Mac OS X x86 (64-bit) (December 9, 2015)
# Date: Thu 14 Dec 2017 12:51:28


from object_library import all_vertices, all_CTvertices, Vertex, CTVertex
import particles as P
import CT_couplings as C
import lorentz as L


V_1 = CTVertex(name = 'V_1',
               type = 'R2',
               particles = [ P.g, P.g, P.g ],
               color = [ 'f(1,2,3)' ],
               lorentz = [ L.VVV1, L.VVV2, L.VVV4, L.VVV6, L.VVV7, L.VVV8 ],
               loop_particles = [ [ [P.b], [P.c], [P.d], [P.s], [P.t], [P.u] ], [ [P.g] ] ],
               couplings = {(0,0,0):C.R2GC_206_93,(0,0,1):C.R2GC_206_94,(0,1,0):C.R2GC_209_95,(0,1,1):C.R2GC_209_96,(0,2,0):C.R2GC_209_95,(0,2,1):C.R2GC_209_96,(0,3,0):C.R2GC_206_93,(0,3,1):C.R2GC_206_94,(0,4,0):C.R2GC_206_93,(0,4,1):C.R2GC_206_94,(0,5,0):C.R2GC_209_95,(0,5,1):C.R2GC_209_96})

V_2 = CTVertex(name = 'V_2',
               type = 'R2',
               particles = [ P.g, P.g, P.g, P.g ],
               color = [ 'd(-1,1,3)*d(-1,2,4)', 'd(-1,1,3)*f(-1,2,4)', 'd(-1,1,4)*d(-1,2,3)', 'd(-1,1,4)*f(-1,2,3)', 'd(-1,2,3)*f(-1,1,4)', 'd(-1,2,4)*f(-1,1,3)', 'f(-1,1,2)*f(-1,3,4)', 'f(-1,1,3)*f(-1,2,4)', 'f(-1,1,4)*f(-1,2,3)', 'Identity(1,2)*Identity(3,4)', 'Identity(1,3)*Identity(2,4)', 'Identity(1,4)*Identity(2,3)' ],
               lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
               loop_particles = [ [ [P.b], [P.c], [P.d], [P.s], [P.t], [P.u] ], [ [P.g] ] ],
               couplings = {(2,0,0):C.R2GC_134_47,(2,0,1):C.R2GC_134_48,(0,0,0):C.R2GC_134_47,(0,0,1):C.R2GC_134_48,(4,0,0):C.R2GC_132_43,(4,0,1):C.R2GC_132_44,(3,0,0):C.R2GC_132_43,(3,0,1):C.R2GC_132_44,(8,0,0):C.R2GC_133_45,(8,0,1):C.R2GC_133_46,(7,0,0):C.R2GC_138_54,(7,0,1):C.R2GC_216_102,(6,0,0):C.R2GC_137_52,(6,0,1):C.R2GC_217_103,(5,0,0):C.R2GC_132_43,(5,0,1):C.R2GC_132_44,(1,0,0):C.R2GC_132_43,(1,0,1):C.R2GC_132_44,(11,0,0):C.R2GC_136_50,(11,0,1):C.R2GC_136_51,(10,0,0):C.R2GC_136_50,(10,0,1):C.R2GC_136_51,(9,0,1):C.R2GC_135_49,(2,1,0):C.R2GC_134_47,(2,1,1):C.R2GC_134_48,(0,1,0):C.R2GC_134_47,(0,1,1):C.R2GC_134_48,(6,1,0):C.R2GC_213_98,(6,1,1):C.R2GC_213_99,(4,1,0):C.R2GC_132_43,(4,1,1):C.R2GC_132_44,(3,1,0):C.R2GC_132_43,(3,1,1):C.R2GC_132_44,(8,1,0):C.R2GC_133_45,(8,1,1):C.R2GC_215_101,(7,1,0):C.R2GC_138_54,(7,1,1):C.R2GC_138_55,(5,1,0):C.R2GC_132_43,(5,1,1):C.R2GC_132_44,(1,1,0):C.R2GC_132_43,(1,1,1):C.R2GC_132_44,(11,1,0):C.R2GC_136_50,(11,1,1):C.R2GC_136_51,(10,1,0):C.R2GC_136_50,(10,1,1):C.R2GC_136_51,(9,1,1):C.R2GC_135_49,(2,2,0):C.R2GC_134_47,(2,2,1):C.R2GC_134_48,(0,2,0):C.R2GC_134_47,(0,2,1):C.R2GC_134_48,(4,2,0):C.R2GC_132_43,(4,2,1):C.R2GC_132_44,(3,2,0):C.R2GC_132_43,(3,2,1):C.R2GC_132_44,(8,2,0):C.R2GC_133_45,(8,2,1):C.R2GC_212_97,(6,2,0):C.R2GC_137_52,(6,2,1):C.R2GC_137_53,(7,2,0):C.R2GC_214_100,(7,2,1):C.R2GC_134_48,(5,2,0):C.R2GC_132_43,(5,2,1):C.R2GC_132_44,(1,2,0):C.R2GC_132_43,(1,2,1):C.R2GC_132_44,(11,2,0):C.R2GC_136_50,(11,2,1):C.R2GC_136_51,(10,2,0):C.R2GC_136_50,(10,2,1):C.R2GC_136_51,(9,2,1):C.R2GC_135_49})

V_3 = CTVertex(name = 'V_3',
               type = 'R2',
               particles = [ P.u__tilde__, P.d, P.G__plus__ ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS3, L.FFS4 ],
               loop_particles = [ [ [P.d, P.g, P.u] ] ],
               couplings = {(0,0,0):C.R2GC_226_106,(0,1,0):C.R2GC_227_107})

V_4 = CTVertex(name = 'V_4',
               type = 'R2',
               particles = [ P.c__tilde__, P.s, P.G__plus__ ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS3, L.FFS4 ],
               loop_particles = [ [ [P.c, P.g, P.s] ] ],
               couplings = {(0,0,0):C.R2GC_187_80,(0,1,0):C.R2GC_184_77})

V_5 = CTVertex(name = 'V_5',
               type = 'R2',
               particles = [ P.t__tilde__, P.b, P.G__plus__ ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS3, L.FFS4 ],
               loop_particles = [ [ [P.b, P.g, P.t] ] ],
               couplings = {(0,0,0):C.R2GC_199_85,(0,1,0):C.R2GC_200_86})

V_6 = CTVertex(name = 'V_6',
               type = 'R2',
               particles = [ P.d__tilde__, P.d, P.G0 ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS1 ],
               loop_particles = [ [ [P.d, P.g] ] ],
               couplings = {(0,0,0):C.R2GC_175_73})

V_7 = CTVertex(name = 'V_7',
               type = 'R2',
               particles = [ P.s__tilde__, P.s, P.G0 ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS1 ],
               loop_particles = [ [ [P.g, P.s] ] ],
               couplings = {(0,0,0):C.R2GC_189_82})

V_8 = CTVertex(name = 'V_8',
               type = 'R2',
               particles = [ P.b__tilde__, P.b, P.G0 ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS1 ],
               loop_particles = [ [ [P.b, P.g] ] ],
               couplings = {(0,0,0):C.R2GC_159_65})

V_9 = CTVertex(name = 'V_9',
               type = 'R2',
               particles = [ P.d__tilde__, P.d, P.H ],
               color = [ 'Identity(1,2)' ],
               lorentz = [ L.FFS2 ],
               loop_particles = [ [ [P.d, P.g] ] ],
               couplings = {(0,0,0):C.R2GC_174_72})

V_10 = CTVertex(name = 'V_10',
                type = 'R2',
                particles = [ P.s__tilde__, P.s, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_188_81})

V_11 = CTVertex(name = 'V_11',
                type = 'R2',
                particles = [ P.b__tilde__, P.b, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_158_64})

V_12 = CTVertex(name = 'V_12',
                type = 'R2',
                particles = [ P.d__tilde__, P.u, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.d, P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_228_108,(0,1,0):C.R2GC_225_105})

V_13 = CTVertex(name = 'V_13',
                type = 'R2',
                particles = [ P.s__tilde__, P.c, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.c, P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_185_78,(0,1,0):C.R2GC_186_79})

V_14 = CTVertex(name = 'V_14',
                type = 'R2',
                particles = [ P.b__tilde__, P.t, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.b, P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_201_87,(0,1,0):C.R2GC_198_84})

V_15 = CTVertex(name = 'V_15',
                type = 'R2',
                particles = [ P.u__tilde__, P.u, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_229_109})

V_16 = CTVertex(name = 'V_16',
                type = 'R2',
                particles = [ P.c__tilde__, P.c, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_166_69})

V_17 = CTVertex(name = 'V_17',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_202_88})

V_18 = CTVertex(name = 'V_18',
                type = 'R2',
                particles = [ P.u__tilde__, P.u, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_230_110})

V_19 = CTVertex(name = 'V_19',
                type = 'R2',
                particles = [ P.c__tilde__, P.c, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_167_70})

V_20 = CTVertex(name = 'V_20',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_203_89})

V_21 = CTVertex(name = 'V_21',
                type = 'R2',
                particles = [ P.u__tilde__, P.u, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_141_58})

V_22 = CTVertex(name = 'V_22',
                type = 'R2',
                particles = [ P.c__tilde__, P.c, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_141_58})

V_23 = CTVertex(name = 'V_23',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_141_58})

V_24 = CTVertex(name = 'V_24',
                type = 'R2',
                particles = [ P.d__tilde__, P.d, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_139_56})

V_25 = CTVertex(name = 'V_25',
                type = 'R2',
                particles = [ P.s__tilde__, P.s, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_139_56})

V_26 = CTVertex(name = 'V_26',
                type = 'R2',
                particles = [ P.b__tilde__, P.b, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_139_56})

V_27 = CTVertex(name = 'V_27',
                type = 'R2',
                particles = [ P.u__tilde__, P.u, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_28 = CTVertex(name = 'V_28',
                type = 'R2',
                particles = [ P.c__tilde__, P.c, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_29 = CTVertex(name = 'V_29',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_30 = CTVertex(name = 'V_30',
                type = 'R2',
                particles = [ P.d__tilde__, P.d, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_31 = CTVertex(name = 'V_31',
                type = 'R2',
                particles = [ P.s__tilde__, P.s, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_32 = CTVertex(name = 'V_32',
                type = 'R2',
                particles = [ P.b__tilde__, P.b, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_140_57})

V_33 = CTVertex(name = 'V_33',
                type = 'R2',
                particles = [ P.d__tilde__, P.u, P.W__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.d, P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_34 = CTVertex(name = 'V_34',
                type = 'R2',
                particles = [ P.s__tilde__, P.c, P.W__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.c, P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_35 = CTVertex(name = 'V_35',
                type = 'R2',
                particles = [ P.b__tilde__, P.t, P.W__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.b, P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_36 = CTVertex(name = 'V_36',
                type = 'R2',
                particles = [ P.u__tilde__, P.d, P.W__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.d, P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_37 = CTVertex(name = 'V_37',
                type = 'R2',
                particles = [ P.c__tilde__, P.s, P.W__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.c, P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_38 = CTVertex(name = 'V_38',
                type = 'R2',
                particles = [ P.t__tilde__, P.b, P.W__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2 ],
                loop_particles = [ [ [P.b, P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_181_76})

V_39 = CTVertex(name = 'V_39',
                type = 'R2',
                particles = [ P.u__tilde__, P.u, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_164_67,(0,1,0):C.R2GC_165_68})

V_40 = CTVertex(name = 'V_40',
                type = 'R2',
                particles = [ P.c__tilde__, P.c, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_164_67,(0,1,0):C.R2GC_165_68})

V_41 = CTVertex(name = 'V_41',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_164_67,(0,1,0):C.R2GC_165_68})

V_42 = CTVertex(name = 'V_42',
                type = 'R2',
                particles = [ P.d__tilde__, P.d, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_156_62,(0,1,0):C.R2GC_157_63})

V_43 = CTVertex(name = 'V_43',
                type = 'R2',
                particles = [ P.s__tilde__, P.s, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_156_62,(0,1,0):C.R2GC_157_63})

V_44 = CTVertex(name = 'V_44',
                type = 'R2',
                particles = [ P.b__tilde__, P.b, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_156_62,(0,1,0):C.R2GC_157_63})

V_45 = CTVertex(name = 'V_45',
                type = 'R2',
                particles = [ P.b__tilde__, P.b, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV3 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_154_60})

V_46 = CTVertex(name = 'V_46',
                type = 'R2',
                particles = [ P.s__tilde__, P.b, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV3 ],
                loop_particles = [ [ [P.b, P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_179_74})

V_47 = CTVertex(name = 'V_47',
                type = 'R2',
                particles = [ P.b__tilde__, P.s, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV3 ],
                loop_particles = [ [ [P.b, P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_179_74})

V_48 = CTVertex(name = 'V_48',
                type = 'R2',
                particles = [ P.t__tilde__, P.t, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV3 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_154_60})

V_49 = CTVertex(name = 'V_49',
                type = 'R2',
                particles = [ P.u__tilde__, P.u ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_221_104,(0,2,0):C.R2GC_221_104,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_50 = CTVertex(name = 'V_50',
                type = 'R2',
                particles = [ P.c__tilde__, P.c ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_163_66,(0,2,0):C.R2GC_163_66,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_51 = CTVertex(name = 'V_51',
                type = 'R2',
                particles = [ P.t__tilde__, P.t ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.R2GC_194_83,(0,2,0):C.R2GC_194_83,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_52 = CTVertex(name = 'V_52',
                type = 'R2',
                particles = [ P.d__tilde__, P.d ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_171_71,(0,2,0):C.R2GC_171_71,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_53 = CTVertex(name = 'V_53',
                type = 'R2',
                particles = [ P.s__tilde__, P.s ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.R2GC_180_75,(0,2,0):C.R2GC_180_75,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_54 = CTVertex(name = 'V_54',
                type = 'R2',
                particles = [ P.b__tilde__, P.b ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.R2GC_155_61,(0,2,0):C.R2GC_155_61,(0,1,0):C.R2GC_151_59,(0,3,0):C.R2GC_151_59})

V_55 = CTVertex(name = 'V_55',
                type = 'R2',
                particles = [ P.g, P.g ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VV1, L.VV2, L.VV3 ],
                loop_particles = [ [ [P.b] ], [ [P.b], [P.c], [P.d], [P.s], [P.t], [P.u] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,4):C.R2GC_205_92,(0,1,0):C.R2GC_120_11,(0,1,2):C.R2GC_120_12,(0,1,3):C.R2GC_120_13,(0,1,5):C.R2GC_120_14,(0,1,6):C.R2GC_120_15,(0,1,7):C.R2GC_120_16,(0,2,1):C.R2GC_204_90,(0,2,4):C.R2GC_204_91})

V_56 = CTVertex(name = 'V_56',
                type = 'R2',
                particles = [ P.g, P.g, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVS1 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_125_25,(0,0,1):C.R2GC_125_26,(0,0,2):C.R2GC_125_27,(0,0,3):C.R2GC_125_28,(0,0,4):C.R2GC_125_29,(0,0,5):C.R2GC_125_30})

V_57 = CTVertex(name = 'V_57',
                type = 'R2',
                particles = [ P.g, P.g, P.Zp, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b, P.s] ], [ [P.b], [P.t] ] ],
                couplings = {(0,0,1):C.R2GC_127_37,(0,0,0):C.R2GC_127_38,(0,1,1):C.R2GC_127_37,(0,1,0):C.R2GC_127_38,(0,2,1):C.R2GC_127_37,(0,2,0):C.R2GC_127_38})

V_58 = CTVertex(name = 'V_58',
                type = 'R2',
                particles = [ P.a, P.g, P.g, P.Zp ],
                color = [ 'Identity(2,3)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b] ], [ [P.t] ] ],
                couplings = {(0,0,0):C.R2GC_114_1,(0,0,1):C.R2GC_114_2,(0,1,0):C.R2GC_114_1,(0,1,1):C.R2GC_114_2,(0,2,0):C.R2GC_114_1,(0,2,1):C.R2GC_114_2})

V_59 = CTVertex(name = 'V_59',
                type = 'R2',
                particles = [ P.g, P.g, P.Z, P.Zp ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b] ], [ [P.t] ] ],
                couplings = {(0,0,0):C.R2GC_117_5,(0,0,1):C.R2GC_117_6,(0,1,0):C.R2GC_117_5,(0,1,1):C.R2GC_117_6,(0,2,0):C.R2GC_117_5,(0,2,1):C.R2GC_117_6})

V_60 = CTVertex(name = 'V_60',
                type = 'R2',
                particles = [ P.g, P.g, P.g, P.Zp ],
                color = [ 'd(1,2,3)', 'f(1,2,3)' ],
                lorentz = [ L.VVVV1, L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.t] ] ],
                couplings = {(1,0,0):C.R2GC_116_4,(0,1,0):C.R2GC_115_3,(0,2,0):C.R2GC_115_3,(0,3,0):C.R2GC_115_3})

V_61 = CTVertex(name = 'V_61',
                type = 'R2',
                particles = [ P.g, P.g, P.W__minus__, P.W__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b, P.t], [P.c, P.s], [P.d, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_128_39,(0,1,0):C.R2GC_128_39,(0,2,0):C.R2GC_128_39})

V_62 = CTVertex(name = 'V_62',
                type = 'R2',
                particles = [ P.a, P.g, P.g, P.Z ],
                color = [ 'Identity(2,3)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.d], [P.s] ], [ [P.c], [P.t], [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_121_17,(0,0,1):C.R2GC_121_18,(0,1,0):C.R2GC_121_17,(0,1,1):C.R2GC_121_18,(0,2,0):C.R2GC_121_17,(0,2,1):C.R2GC_121_18})

V_63 = CTVertex(name = 'V_63',
                type = 'R2',
                particles = [ P.g, P.g, P.Z, P.Z ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.d], [P.s] ], [ [P.c], [P.t], [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_124_23,(0,0,1):C.R2GC_124_24,(0,1,0):C.R2GC_124_23,(0,1,1):C.R2GC_124_24,(0,2,0):C.R2GC_124_23,(0,2,1):C.R2GC_124_24})

V_64 = CTVertex(name = 'V_64',
                type = 'R2',
                particles = [ P.a, P.a, P.g, P.g ],
                color = [ 'Identity(3,4)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.d], [P.s] ], [ [P.c], [P.t], [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_118_7,(0,0,1):C.R2GC_118_8,(0,1,0):C.R2GC_118_7,(0,1,1):C.R2GC_118_8,(0,2,0):C.R2GC_118_7,(0,2,1):C.R2GC_118_8})

V_65 = CTVertex(name = 'V_65',
                type = 'R2',
                particles = [ P.g, P.g, P.g, P.Z ],
                color = [ 'd(1,2,3)', 'f(1,2,3)' ],
                lorentz = [ L.VVVV1, L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.d], [P.s] ], [ [P.c], [P.t], [P.u] ] ],
                couplings = {(1,0,0):C.R2GC_123_21,(1,0,1):C.R2GC_123_22,(0,1,0):C.R2GC_122_19,(0,1,1):C.R2GC_122_20,(0,2,0):C.R2GC_122_19,(0,2,1):C.R2GC_122_20,(0,3,0):C.R2GC_122_19,(0,3,1):C.R2GC_122_20})

V_66 = CTVertex(name = 'V_66',
                type = 'R2',
                particles = [ P.a, P.g, P.g, P.g ],
                color = [ 'd(2,3,4)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b], [P.d], [P.s] ], [ [P.c], [P.t], [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_119_9,(0,0,1):C.R2GC_119_10,(0,1,0):C.R2GC_119_9,(0,1,1):C.R2GC_119_10,(0,2,0):C.R2GC_119_9,(0,2,1):C.R2GC_119_10})

V_67 = CTVertex(name = 'V_67',
                type = 'R2',
                particles = [ P.g, P.g, P.H, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVSS1 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_126_31,(0,0,1):C.R2GC_126_32,(0,0,2):C.R2GC_126_33,(0,0,3):C.R2GC_126_34,(0,0,4):C.R2GC_126_35,(0,0,5):C.R2GC_126_36})

V_68 = CTVertex(name = 'V_68',
                type = 'R2',
                particles = [ P.g, P.g, P.G0, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVSS1 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,0):C.R2GC_126_31,(0,0,1):C.R2GC_126_32,(0,0,2):C.R2GC_126_33,(0,0,3):C.R2GC_126_34,(0,0,4):C.R2GC_126_35,(0,0,5):C.R2GC_126_36})

V_69 = CTVertex(name = 'V_69',
                type = 'R2',
                particles = [ P.g, P.g, P.G__minus__, P.G__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.VVSS1 ],
                loop_particles = [ [ [P.b, P.t] ], [ [P.c, P.s] ], [ [P.d, P.u] ] ],
                couplings = {(0,0,0):C.R2GC_129_40,(0,0,1):C.R2GC_129_41,(0,0,2):C.R2GC_129_42})

V_70 = CTVertex(name = 'V_70',
                type = 'UV',
                particles = [ P.g, P.g, P.g ],
                color = [ 'f(1,2,3)' ],
                lorentz = [ L.VVV1, L.VVV2, L.VVV3, L.VVV4, L.VVV5, L.VVV6, L.VVV7, L.VVV8 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,0):C.UVGC_206_116,(0,0,1):C.UVGC_206_117,(0,0,2):C.UVGC_206_118,(0,0,3):C.UVGC_206_119,(0,0,4):C.UVGC_131_4,(0,0,5):C.UVGC_206_120,(0,0,6):C.UVGC_206_121,(0,0,7):C.UVGC_206_122,(0,1,0):C.UVGC_209_127,(0,1,1):C.UVGC_209_128,(0,1,2):C.UVGC_209_129,(0,1,3):C.UVGC_209_130,(0,1,4):C.UVGC_209_131,(0,1,5):C.UVGC_209_132,(0,1,6):C.UVGC_209_133,(0,1,7):C.UVGC_209_134,(0,3,0):C.UVGC_209_127,(0,3,1):C.UVGC_209_128,(0,3,2):C.UVGC_209_129,(0,3,3):C.UVGC_211_137,(0,3,4):C.UVGC_130_2,(0,3,5):C.UVGC_209_132,(0,3,6):C.UVGC_209_133,(0,3,7):C.UVGC_209_134,(0,5,0):C.UVGC_206_116,(0,5,1):C.UVGC_206_117,(0,5,2):C.UVGC_206_118,(0,5,3):C.UVGC_208_125,(0,5,4):C.UVGC_208_126,(0,5,5):C.UVGC_206_120,(0,5,6):C.UVGC_206_121,(0,5,7):C.UVGC_206_122,(0,6,0):C.UVGC_206_116,(0,6,1):C.UVGC_206_117,(0,6,2):C.UVGC_206_118,(0,6,3):C.UVGC_207_123,(0,6,4):C.UVGC_207_124,(0,6,5):C.UVGC_206_120,(0,6,6):C.UVGC_206_121,(0,6,7):C.UVGC_206_122,(0,7,0):C.UVGC_209_127,(0,7,1):C.UVGC_209_128,(0,7,2):C.UVGC_209_129,(0,7,3):C.UVGC_210_135,(0,7,4):C.UVGC_210_136,(0,7,5):C.UVGC_209_132,(0,7,6):C.UVGC_209_133,(0,7,7):C.UVGC_209_134,(0,2,3):C.UVGC_130_1,(0,2,4):C.UVGC_130_2,(0,4,3):C.UVGC_131_3,(0,4,4):C.UVGC_131_4})

V_71 = CTVertex(name = 'V_71',
                type = 'UV',
                particles = [ P.g, P.g, P.g, P.g ],
                color = [ 'd(-1,1,3)*d(-1,2,4)', 'd(-1,1,3)*f(-1,2,4)', 'd(-1,1,4)*d(-1,2,3)', 'd(-1,1,4)*f(-1,2,3)', 'd(-1,2,3)*f(-1,1,4)', 'd(-1,2,4)*f(-1,1,3)', 'f(-1,1,2)*f(-1,3,4)', 'f(-1,1,3)*f(-1,2,4)', 'f(-1,1,4)*f(-1,2,3)', 'Identity(1,2)*Identity(3,4)', 'Identity(1,3)*Identity(2,4)', 'Identity(1,4)*Identity(2,3)' ],
                lorentz = [ L.VVVV2, L.VVVV3, L.VVVV4 ],
                loop_particles = [ [ [P.b] ], [ [P.b], [P.c], [P.d], [P.s], [P.t], [P.u] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(2,0,4):C.UVGC_133_8,(2,0,5):C.UVGC_133_7,(0,0,4):C.UVGC_133_8,(0,0,5):C.UVGC_133_7,(4,0,4):C.UVGC_132_5,(4,0,5):C.UVGC_132_6,(3,0,4):C.UVGC_132_5,(3,0,5):C.UVGC_132_6,(8,0,4):C.UVGC_133_7,(8,0,5):C.UVGC_133_8,(7,0,0):C.UVGC_216_164,(7,0,2):C.UVGC_216_165,(7,0,3):C.UVGC_216_166,(7,0,4):C.UVGC_216_167,(7,0,5):C.UVGC_216_168,(7,0,6):C.UVGC_216_169,(7,0,7):C.UVGC_216_170,(7,0,8):C.UVGC_216_171,(6,0,0):C.UVGC_216_164,(6,0,2):C.UVGC_216_165,(6,0,3):C.UVGC_216_166,(6,0,4):C.UVGC_217_172,(6,0,5):C.UVGC_217_173,(6,0,6):C.UVGC_216_169,(6,0,7):C.UVGC_216_170,(6,0,8):C.UVGC_216_171,(5,0,4):C.UVGC_132_5,(5,0,5):C.UVGC_132_6,(1,0,4):C.UVGC_132_5,(1,0,5):C.UVGC_132_6,(11,0,4):C.UVGC_136_11,(11,0,5):C.UVGC_136_12,(10,0,4):C.UVGC_136_11,(10,0,5):C.UVGC_136_12,(9,0,4):C.UVGC_135_9,(9,0,5):C.UVGC_135_10,(2,1,4):C.UVGC_133_8,(2,1,5):C.UVGC_133_7,(0,1,4):C.UVGC_133_8,(0,1,5):C.UVGC_133_7,(6,1,0):C.UVGC_213_146,(6,1,2):C.UVGC_213_147,(6,1,3):C.UVGC_213_148,(6,1,4):C.UVGC_213_149,(6,1,5):C.UVGC_213_150,(6,1,6):C.UVGC_213_151,(6,1,7):C.UVGC_213_152,(6,1,8):C.UVGC_213_153,(4,1,4):C.UVGC_132_5,(4,1,5):C.UVGC_132_6,(3,1,4):C.UVGC_132_5,(3,1,5):C.UVGC_132_6,(8,1,0):C.UVGC_215_156,(8,1,2):C.UVGC_215_157,(8,1,3):C.UVGC_215_158,(8,1,4):C.UVGC_215_159,(8,1,5):C.UVGC_215_160,(8,1,6):C.UVGC_215_161,(8,1,7):C.UVGC_215_162,(8,1,8):C.UVGC_215_163,(7,1,1):C.UVGC_137_13,(7,1,4):C.UVGC_138_15,(7,1,5):C.UVGC_138_16,(5,1,4):C.UVGC_132_5,(5,1,5):C.UVGC_132_6,(1,1,4):C.UVGC_132_5,(1,1,5):C.UVGC_132_6,(11,1,4):C.UVGC_136_11,(11,1,5):C.UVGC_136_12,(10,1,4):C.UVGC_136_11,(10,1,5):C.UVGC_136_12,(9,1,4):C.UVGC_135_9,(9,1,5):C.UVGC_135_10,(2,2,4):C.UVGC_133_8,(2,2,5):C.UVGC_133_7,(0,2,4):C.UVGC_133_8,(0,2,5):C.UVGC_133_7,(4,2,4):C.UVGC_132_5,(4,2,5):C.UVGC_132_6,(3,2,4):C.UVGC_132_5,(3,2,5):C.UVGC_132_6,(8,2,0):C.UVGC_212_138,(8,2,2):C.UVGC_212_139,(8,2,3):C.UVGC_212_140,(8,2,4):C.UVGC_212_141,(8,2,5):C.UVGC_212_142,(8,2,6):C.UVGC_212_143,(8,2,7):C.UVGC_212_144,(8,2,8):C.UVGC_212_145,(6,2,1):C.UVGC_137_13,(6,2,4):C.UVGC_137_14,(6,2,5):C.UVGC_135_9,(7,2,0):C.UVGC_213_146,(7,2,2):C.UVGC_213_147,(7,2,3):C.UVGC_213_148,(7,2,4):C.UVGC_214_154,(7,2,5):C.UVGC_214_155,(7,2,6):C.UVGC_213_151,(7,2,7):C.UVGC_213_152,(7,2,8):C.UVGC_213_153,(5,2,4):C.UVGC_132_5,(5,2,5):C.UVGC_132_6,(1,2,4):C.UVGC_132_5,(1,2,5):C.UVGC_132_6,(11,2,4):C.UVGC_136_11,(11,2,5):C.UVGC_136_12,(10,2,4):C.UVGC_136_11,(10,2,5):C.UVGC_136_12,(9,2,4):C.UVGC_135_9,(9,2,5):C.UVGC_135_10})

V_72 = CTVertex(name = 'V_72',
                type = 'UV',
                particles = [ P.u__tilde__, P.d, P.G__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.d, P.g] ], [ [P.d, P.g, P.u] ], [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.UVGC_226_185,(0,0,2):C.UVGC_226_186,(0,0,1):C.UVGC_226_187,(0,1,0):C.UVGC_227_188,(0,1,2):C.UVGC_227_189,(0,1,1):C.UVGC_227_190})

V_73 = CTVertex(name = 'V_73',
                type = 'UV',
                particles = [ P.c__tilde__, P.s, P.G__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.c, P.g] ], [ [P.c, P.g, P.s] ], [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.UVGC_187_74,(0,0,2):C.UVGC_187_75,(0,0,1):C.UVGC_187_76,(0,1,0):C.UVGC_184_65,(0,1,2):C.UVGC_184_66,(0,1,1):C.UVGC_184_67})

V_74 = CTVertex(name = 'V_74',
                type = 'UV',
                particles = [ P.t__tilde__, P.b, P.G__plus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.t] ], [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.UVGC_199_91,(0,0,2):C.UVGC_199_92,(0,0,1):C.UVGC_199_93,(0,1,0):C.UVGC_200_94,(0,1,2):C.UVGC_200_95,(0,1,1):C.UVGC_200_96})

V_75 = CTVertex(name = 'V_75',
                type = 'UV',
                particles = [ P.d__tilde__, P.d, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_175_52})

V_76 = CTVertex(name = 'V_76',
                type = 'UV',
                particles = [ P.s__tilde__, P.s, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.UVGC_189_78})

V_77 = CTVertex(name = 'V_77',
                type = 'UV',
                particles = [ P.b__tilde__, P.b, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_159_36})

V_78 = CTVertex(name = 'V_78',
                type = 'UV',
                particles = [ P.d__tilde__, P.d, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_174_51})

V_79 = CTVertex(name = 'V_79',
                type = 'UV',
                particles = [ P.s__tilde__, P.s, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.UVGC_188_77})

V_80 = CTVertex(name = 'V_80',
                type = 'UV',
                particles = [ P.b__tilde__, P.b, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_158_35})

V_81 = CTVertex(name = 'V_81',
                type = 'UV',
                particles = [ P.d__tilde__, P.u, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.d, P.g] ], [ [P.d, P.g, P.u] ], [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.UVGC_228_191,(0,0,2):C.UVGC_228_192,(0,0,1):C.UVGC_228_193,(0,1,0):C.UVGC_225_182,(0,1,2):C.UVGC_225_183,(0,1,1):C.UVGC_225_184})

V_82 = CTVertex(name = 'V_82',
                type = 'UV',
                particles = [ P.s__tilde__, P.c, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.c, P.g] ], [ [P.c, P.g, P.s] ], [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.UVGC_185_68,(0,0,2):C.UVGC_185_69,(0,0,1):C.UVGC_185_70,(0,1,0):C.UVGC_186_71,(0,1,2):C.UVGC_186_72,(0,1,1):C.UVGC_186_73})

V_83 = CTVertex(name = 'V_83',
                type = 'UV',
                particles = [ P.b__tilde__, P.t, P.G__minus__ ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS3, L.FFS4 ],
                loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.t] ], [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.UVGC_201_97,(0,0,2):C.UVGC_201_98,(0,0,1):C.UVGC_201_99,(0,1,0):C.UVGC_198_88,(0,1,2):C.UVGC_198_89,(0,1,1):C.UVGC_198_90})

V_84 = CTVertex(name = 'V_84',
                type = 'UV',
                particles = [ P.u__tilde__, P.u, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.UVGC_229_194})

V_85 = CTVertex(name = 'V_85',
                type = 'UV',
                particles = [ P.c__tilde__, P.c, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_166_43})

V_86 = CTVertex(name = 'V_86',
                type = 'UV',
                particles = [ P.t__tilde__, P.t, P.G0 ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS1 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.UVGC_202_100})

V_87 = CTVertex(name = 'V_87',
                type = 'UV',
                particles = [ P.u__tilde__, P.u, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.UVGC_230_195})

V_88 = CTVertex(name = 'V_88',
                type = 'UV',
                particles = [ P.c__tilde__, P.c, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_167_44})

V_89 = CTVertex(name = 'V_89',
                type = 'UV',
                particles = [ P.t__tilde__, P.t, P.H ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFS2 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.UVGC_203_101})

V_90 = CTVertex(name = 'V_90',
                type = 'UV',
                particles = [ P.u__tilde__, P.u, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.u] ] ],
                couplings = {(0,0,0):C.UVGC_141_19,(0,1,0):C.UVGC_219_175,(0,2,0):C.UVGC_219_175})

V_91 = CTVertex(name = 'V_91',
                type = 'UV',
                particles = [ P.c__tilde__, P.c, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.c, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_141_19,(0,1,0):C.UVGC_161_38,(0,2,0):C.UVGC_161_38})

V_92 = CTVertex(name = 'V_92',
                type = 'UV',
                particles = [ P.t__tilde__, P.t, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.t] ] ],
                couplings = {(0,0,0):C.UVGC_141_19,(0,1,0):C.UVGC_191_80,(0,2,0):C.UVGC_191_80})

V_93 = CTVertex(name = 'V_93',
                type = 'UV',
                particles = [ P.d__tilde__, P.d, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.d, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_139_17,(0,1,0):C.UVGC_169_46,(0,2,0):C.UVGC_169_46})

V_94 = CTVertex(name = 'V_94',
                type = 'UV',
                particles = [ P.s__tilde__, P.s, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.g, P.s] ] ],
                couplings = {(0,0,0):C.UVGC_139_17,(0,1,0):C.UVGC_177_54,(0,2,0):C.UVGC_177_54})

V_95 = CTVertex(name = 'V_95',
                type = 'UV',
                particles = [ P.b__tilde__, P.b, P.a ],
                color = [ 'Identity(1,2)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b, P.g] ] ],
                couplings = {(0,0,0):C.UVGC_139_17,(0,1,0):C.UVGC_152_21,(0,2,0):C.UVGC_152_21})

V_96 = CTVertex(name = 'V_96',
                type = 'UV',
                particles = [ P.u__tilde__, P.u, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.g, P.u] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,5):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,1):C.UVGC_153_23,(0,1,2):C.UVGC_153_24,(0,1,3):C.UVGC_153_25,(0,1,4):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,5):C.UVGC_220_176,(0,2,0):C.UVGC_153_22,(0,2,1):C.UVGC_153_23,(0,2,2):C.UVGC_153_24,(0,2,3):C.UVGC_153_25,(0,2,4):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,5):C.UVGC_220_176})

V_97 = CTVertex(name = 'V_97',
                type = 'UV',
                particles = [ P.c__tilde__, P.c, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.c, P.g] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,2):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,1):C.UVGC_153_23,(0,1,3):C.UVGC_153_24,(0,1,4):C.UVGC_153_25,(0,1,5):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,2):C.UVGC_162_39,(0,2,0):C.UVGC_153_22,(0,2,1):C.UVGC_153_23,(0,2,3):C.UVGC_153_24,(0,2,4):C.UVGC_153_25,(0,2,5):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,2):C.UVGC_162_39})

V_98 = CTVertex(name = 'V_98',
                type = 'UV',
                particles = [ P.t__tilde__, P.t, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.g, P.t] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,5):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,1):C.UVGC_153_23,(0,1,2):C.UVGC_153_24,(0,1,3):C.UVGC_153_25,(0,1,4):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,5):C.UVGC_192_81,(0,2,0):C.UVGC_153_22,(0,2,1):C.UVGC_153_23,(0,2,2):C.UVGC_153_24,(0,2,3):C.UVGC_153_25,(0,2,4):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,5):C.UVGC_192_81})

V_99 = CTVertex(name = 'V_99',
                type = 'UV',
                particles = [ P.d__tilde__, P.d, P.g ],
                color = [ 'T(3,2,1)' ],
                lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.d, P.g] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                couplings = {(0,0,3):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,1):C.UVGC_153_23,(0,1,2):C.UVGC_153_24,(0,1,4):C.UVGC_153_25,(0,1,5):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,3):C.UVGC_170_47,(0,2,0):C.UVGC_153_22,(0,2,1):C.UVGC_153_23,(0,2,2):C.UVGC_153_24,(0,2,4):C.UVGC_153_25,(0,2,5):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,3):C.UVGC_170_47})

V_100 = CTVertex(name = 'V_100',
                 type = 'UV',
                 particles = [ P.s__tilde__, P.s, P.g ],
                 color = [ 'T(3,2,1)' ],
                 lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.g, P.s] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                 couplings = {(0,0,5):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,1):C.UVGC_153_23,(0,1,2):C.UVGC_153_24,(0,1,3):C.UVGC_153_25,(0,1,4):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,5):C.UVGC_178_55,(0,2,0):C.UVGC_153_22,(0,2,1):C.UVGC_153_23,(0,2,2):C.UVGC_153_24,(0,2,3):C.UVGC_153_25,(0,2,4):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,5):C.UVGC_178_55})

V_101 = CTVertex(name = 'V_101',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.b, P.g ],
                 color = [ 'T(3,2,1)' ],
                 lorentz = [ L.FFV1, L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.b] ], [ [P.b, P.g] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                 couplings = {(0,0,1):C.UVGC_140_18,(0,1,0):C.UVGC_153_22,(0,1,2):C.UVGC_153_23,(0,1,3):C.UVGC_153_24,(0,1,4):C.UVGC_153_25,(0,1,5):C.UVGC_153_26,(0,1,6):C.UVGC_153_27,(0,1,7):C.UVGC_153_28,(0,1,8):C.UVGC_153_29,(0,1,1):C.UVGC_153_30,(0,2,0):C.UVGC_153_22,(0,2,2):C.UVGC_153_23,(0,2,3):C.UVGC_153_24,(0,2,4):C.UVGC_153_25,(0,2,5):C.UVGC_153_26,(0,2,6):C.UVGC_153_27,(0,2,7):C.UVGC_153_28,(0,2,8):C.UVGC_153_29,(0,2,1):C.UVGC_153_30})

V_102 = CTVertex(name = 'V_102',
                 type = 'UV',
                 particles = [ P.d__tilde__, P.u, P.W__minus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.d, P.g] ], [ [P.d, P.g, P.u] ], [ [P.g, P.u] ] ],
                 couplings = {(0,0,0):C.UVGC_222_178,(0,0,2):C.UVGC_222_179,(0,0,1):C.UVGC_181_62})

V_103 = CTVertex(name = 'V_103',
                 type = 'UV',
                 particles = [ P.s__tilde__, P.c, P.W__minus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.c, P.g] ], [ [P.c, P.g, P.s] ], [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_181_60,(0,0,2):C.UVGC_181_61,(0,0,1):C.UVGC_181_62})

V_104 = CTVertex(name = 'V_104',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.t, P.W__minus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.t] ], [ [P.g, P.t] ] ],
                 couplings = {(0,0,0):C.UVGC_195_84,(0,0,2):C.UVGC_195_85,(0,0,1):C.UVGC_181_62})

V_105 = CTVertex(name = 'V_105',
                 type = 'UV',
                 particles = [ P.u__tilde__, P.d, P.W__plus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.d, P.g] ], [ [P.d, P.g, P.u] ], [ [P.g, P.u] ] ],
                 couplings = {(0,0,0):C.UVGC_222_178,(0,0,2):C.UVGC_222_179,(0,0,1):C.UVGC_181_62})

V_106 = CTVertex(name = 'V_106',
                 type = 'UV',
                 particles = [ P.c__tilde__, P.s, P.W__plus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.c, P.g] ], [ [P.c, P.g, P.s] ], [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_181_60,(0,0,2):C.UVGC_181_61,(0,0,1):C.UVGC_181_62})

V_107 = CTVertex(name = 'V_107',
                 type = 'UV',
                 particles = [ P.t__tilde__, P.b, P.W__plus__ ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2 ],
                 loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.t] ], [ [P.g, P.t] ] ],
                 couplings = {(0,0,0):C.UVGC_195_84,(0,0,2):C.UVGC_195_85,(0,0,1):C.UVGC_181_62})

V_108 = CTVertex(name = 'V_108',
                 type = 'UV',
                 particles = [ P.u__tilde__, P.u, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.g, P.u] ] ],
                 couplings = {(0,0,0):C.UVGC_223_180,(0,1,0):C.UVGC_224_181})

V_109 = CTVertex(name = 'V_109',
                 type = 'UV',
                 particles = [ P.c__tilde__, P.c, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.c, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_164_41,(0,1,0):C.UVGC_165_42})

V_110 = CTVertex(name = 'V_110',
                 type = 'UV',
                 particles = [ P.t__tilde__, P.t, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.g, P.t] ] ],
                 couplings = {(0,0,0):C.UVGC_196_86,(0,1,0):C.UVGC_197_87})

V_111 = CTVertex(name = 'V_111',
                 type = 'UV',
                 particles = [ P.d__tilde__, P.d, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.d, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_172_49,(0,1,0):C.UVGC_173_50})

V_112 = CTVertex(name = 'V_112',
                 type = 'UV',
                 particles = [ P.s__tilde__, P.s, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_182_63,(0,1,0):C.UVGC_183_64})

V_113 = CTVertex(name = 'V_113',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.b, P.Z ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV2, L.FFV3 ],
                 loop_particles = [ [ [P.b, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_156_33,(0,1,0):C.UVGC_157_34})

V_114 = CTVertex(name = 'V_114',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.b, P.Zp ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV3 ],
                 loop_particles = [ [ [P.b, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_154_31})

V_115 = CTVertex(name = 'V_115',
                 type = 'UV',
                 particles = [ P.s__tilde__, P.b, P.Zp ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV3 ],
                 loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.s] ], [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_179_56,(0,0,2):C.UVGC_179_57,(0,0,1):C.UVGC_179_58})

V_116 = CTVertex(name = 'V_116',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.s, P.Zp ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV3 ],
                 loop_particles = [ [ [P.b, P.g] ], [ [P.b, P.g, P.s] ], [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_179_56,(0,0,2):C.UVGC_179_57,(0,0,1):C.UVGC_179_58})

V_117 = CTVertex(name = 'V_117',
                 type = 'UV',
                 particles = [ P.t__tilde__, P.t, P.Zp ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FFV3 ],
                 loop_particles = [ [ [P.g, P.t] ] ],
                 couplings = {(0,0,0):C.UVGC_193_82})

V_118 = CTVertex(name = 'V_118',
                 type = 'UV',
                 particles = [ P.u__tilde__, P.u ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.g, P.u] ] ],
                 couplings = {(0,0,0):C.UVGC_221_177,(0,2,0):C.UVGC_221_177,(0,1,0):C.UVGC_218_174,(0,3,0):C.UVGC_218_174})

V_119 = CTVertex(name = 'V_119',
                 type = 'UV',
                 particles = [ P.c__tilde__, P.c ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.c, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_163_40,(0,2,0):C.UVGC_163_40,(0,1,0):C.UVGC_160_37,(0,3,0):C.UVGC_160_37})

V_120 = CTVertex(name = 'V_120',
                 type = 'UV',
                 particles = [ P.t__tilde__, P.t ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.g, P.t] ] ],
                 couplings = {(0,0,0):C.UVGC_194_83,(0,2,0):C.UVGC_194_83,(0,1,0):C.UVGC_190_79,(0,3,0):C.UVGC_190_79})

V_121 = CTVertex(name = 'V_121',
                 type = 'UV',
                 particles = [ P.d__tilde__, P.d ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.d, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_171_48,(0,2,0):C.UVGC_171_48,(0,1,0):C.UVGC_168_45,(0,3,0):C.UVGC_168_45})

V_122 = CTVertex(name = 'V_122',
                 type = 'UV',
                 particles = [ P.s__tilde__, P.s ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.g, P.s] ] ],
                 couplings = {(0,0,0):C.UVGC_180_59,(0,2,0):C.UVGC_180_59,(0,1,0):C.UVGC_176_53,(0,3,0):C.UVGC_176_53})

V_123 = CTVertex(name = 'V_123',
                 type = 'UV',
                 particles = [ P.b__tilde__, P.b ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.FF1, L.FF2, L.FF3, L.FF4 ],
                 loop_particles = [ [ [P.b, P.g] ] ],
                 couplings = {(0,0,0):C.UVGC_155_32,(0,2,0):C.UVGC_155_32,(0,1,0):C.UVGC_151_20,(0,3,0):C.UVGC_151_20})

V_124 = CTVertex(name = 'V_124',
                 type = 'UV',
                 particles = [ P.g, P.g ],
                 color = [ 'Identity(1,2)' ],
                 lorentz = [ L.VV1, L.VV3 ],
                 loop_particles = [ [ [P.b] ], [ [P.c] ], [ [P.d] ], [ [P.g] ], [ [P.ghG] ], [ [P.s] ], [ [P.t] ], [ [P.u] ] ],
                 couplings = {(0,0,0):C.UVGC_205_108,(0,0,1):C.UVGC_205_109,(0,0,2):C.UVGC_205_110,(0,0,3):C.UVGC_205_111,(0,0,4):C.UVGC_205_112,(0,0,5):C.UVGC_205_113,(0,0,6):C.UVGC_205_114,(0,0,7):C.UVGC_205_115,(0,1,0):C.UVGC_204_102,(0,1,1):C.UVGC_204_103,(0,1,2):C.UVGC_204_104,(0,1,5):C.UVGC_204_105,(0,1,6):C.UVGC_204_106,(0,1,7):C.UVGC_204_107})

