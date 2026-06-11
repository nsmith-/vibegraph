C     Stubs for symbols referenced by SMATRIX1 but not by MATRIX1.
C     SMATRIX1 is never called from Python; these satisfy the linker only.
C     Satisfies the linker without pulling in the full genps.f dependency chain.
      DOUBLE PRECISION FUNCTION GET_CHANNEL_CUT(P, CONFIG)
      IMPLICIT NONE
      DOUBLE PRECISION P(0:3,4)
      INTEGER CONFIG
      GET_CHANNEL_CUT = 1.0D0
      END

      SUBROUTINE RANMAR(R)
      IMPLICIT NONE
      DOUBLE PRECISION R
      R = 0.5D0
      END

C     f2py wrapper: evaluate MadGraph e+ e- > mu+ mu- amplitude.
C     Computes sum_hel |M|^2 (not divided by IDEN=4).
C     Single-event entry point.
C
C     Common blocks are populated from scalar SM inputs before MATRIX1
C     is called.  The MATRIX1 subroutine caches fk_MDL_WZ on its first
C     invocation; ensure SM parameters are stable across all calls.
      SUBROUTINE MG_EVAL_M2(P, AEWM1_IN, GF_IN, MZ_IN, WZ_IN, M2_OUT)
Cf2py intent(in)  P, AEWM1_IN, GF_IN, MZ_IN, WZ_IN
Cf2py intent(out) M2_OUT
      IMPLICIT NONE
      INCLUDE 'maxamps.inc'
      INCLUDE 'coupl.inc'
      INTEGER NCOMB
      PARAMETER (NCOMB=4)
      DOUBLE PRECISION P(0:3,4)
      DOUBLE PRECISION AEWM1_IN, GF_IN, MZ_IN, WZ_IN, M2_OUT
      DOUBLE PRECISION TS(NCOMB)
      INTEGER IC(4), I
      DOUBLE PRECISION PI, AEW, EE, SW2, SW, CW, SUMTS
      DOUBLE PRECISION AMP2_LOC(MAXAMPS), JAMP2_LOC(0:MAXFLOW)
      COMMON/TO_AMPS/ AMP2_LOC, JAMP2_LOC
      DOUBLE PRECISION SMALL_WIDTH_TREATMENT
      COMMON/NARROW_WIDTH/ SMALL_WIDTH_TREATMENT
      DOUBLE PRECISION TMIN_FOR_CHANNEL
      INTEGER SDE_STRAT
      COMMON/TO_CHANNEL_STRAT/ TMIN_FOR_CHANNEL, SDE_STRAT
      PARAMETER (PI=3.141592653589793D0)
C     Set masses and widths in coupl.inc common blocks
      MDL_MZ = MZ_IN
      MDL_WZ = WZ_IN
      MDL_MB = 4.7D0
      MDL_MT = 173.0D0
      MDL_MH = 125.0D0
      MDL_MTA = 1.777D0
      MDL_WH = 6.382339D-3
      MDL_WT = 1.4915D0
      MDL_WW = 2.0476D0
C     Set MC/channel steering common blocks
      SMALL_WIDTH_TREATMENT = 1.0D-6
      SDE_STRAT = 1
      TMIN_FOR_CHANNEL = 0.0D0
      DO I = 1, MAXAMPS
        AMP2_LOC(I) = 0.0D0
      END DO
      DO I = 0, MAXFLOW
        JAMP2_LOC(I) = 0.0D0
      END DO
C     Derive EW couplings matching UFO SM default param_card values.
C     Formulas mirror validation/helas/gen_reference.py and
C     vibegraph-lib/tests/helas_validation.rs (derive_gammaz_couplings).
      AEW = 1.0D0 / AEWM1_IN
      EE = SQRT(4.0D0 * PI * AEW)
      SW2 = 0.5D0 - SQRT(0.25D0
     &    - PI * AEW / (GF_IN * SQRT(2.0D0) * MZ_IN**2))
      SW = SQRT(SW2)
      CW = SQRT(1.0D0 - SW2)
      MDL_MW = MZ_IN * CW
C     GC_3  = i*(-e)           (photon coupling)
C     GC_59 = i*(e*sw/(2*cw))  (Z right component; gr_z = 2*GC_59.im)
C     GC_50 = i*(gl_z) - GC_59 (Z left component; gl_z + gr_z/2 = GC_59.im + GC_50.im)
      GC_3 = DCMPLX(0D0, -EE)
      GC_59 = DCMPLX(0D0, EE * SW / (2.0D0 * CW))
      GC_50 = DCMPLX(0D0, EE * (-0.5D0 + SW2) / (SW * CW)) - GC_59
C     Set all color charges to +1 (color-singlet process)
      DO I = 1, 4
        IC(I) = 1
      END DO
C     Evaluate and sum over optimized helicity combinations
      CALL MATRIX1(P, IC, TS)
      SUMTS = 0.0D0
      DO I = 1, NCOMB
        SUMTS = SUMTS + TS(I)
      END DO
      M2_OUT = SUMTS
      END

C     Batch wrapper: evaluate N events in a single call.
C     Common blocks and EW couplings are set up once before the loop.
C     N is hidden from Python and inferred from the third axis of P_BATCH.
      SUBROUTINE MG_EVAL_M2_BATCH(P_BATCH, N, AEWM1_IN, GF_IN,
     &                             MZ_IN, WZ_IN, M2_OUT)
Cf2py intent(in)  P_BATCH, AEWM1_IN, GF_IN, MZ_IN, WZ_IN
Cf2py intent(out) M2_OUT
Cf2py integer intent(hide), depend(P_BATCH) :: N = shape(P_BATCH, 2)
      IMPLICIT NONE
      INCLUDE 'maxamps.inc'
      INCLUDE 'coupl.inc'
      INTEGER NCOMB
      PARAMETER (NCOMB=4)
      INTEGER N, I
      DOUBLE PRECISION P_BATCH(0:3, 4, N)
      DOUBLE PRECISION AEWM1_IN, GF_IN, MZ_IN, WZ_IN
      DOUBLE PRECISION M2_OUT(N)
      DOUBLE PRECISION TS(NCOMB)
      INTEGER IC(4), J
      DOUBLE PRECISION PI, AEW, EE, SW2, SW, CW, SUMTS
      DOUBLE PRECISION AMP2_LOC(MAXAMPS), JAMP2_LOC(0:MAXFLOW)
      COMMON/TO_AMPS/ AMP2_LOC, JAMP2_LOC
      DOUBLE PRECISION SMALL_WIDTH_TREATMENT
      COMMON/NARROW_WIDTH/ SMALL_WIDTH_TREATMENT
      DOUBLE PRECISION TMIN_FOR_CHANNEL
      INTEGER SDE_STRAT
      COMMON/TO_CHANNEL_STRAT/ TMIN_FOR_CHANNEL, SDE_STRAT
      PARAMETER (PI=3.141592653589793D0)
C     Set up common blocks once for the entire batch
      MDL_MZ = MZ_IN
      MDL_WZ = WZ_IN
      MDL_MB = 4.7D0
      MDL_MT = 173.0D0
      MDL_MH = 125.0D0
      MDL_MTA = 1.777D0
      MDL_WH = 6.382339D-3
      MDL_WT = 1.4915D0
      MDL_WW = 2.0476D0
      SMALL_WIDTH_TREATMENT = 1.0D-6
      SDE_STRAT = 1
      TMIN_FOR_CHANNEL = 0.0D0
      DO J = 1, MAXAMPS
        AMP2_LOC(J) = 0.0D0
      END DO
      DO J = 0, MAXFLOW
        JAMP2_LOC(J) = 0.0D0
      END DO
      AEW = 1.0D0 / AEWM1_IN
      EE = SQRT(4.0D0 * PI * AEW)
      SW2 = 0.5D0 - SQRT(0.25D0
     &    - PI * AEW / (GF_IN * SQRT(2.0D0) * MZ_IN**2))
      SW = SQRT(SW2)
      CW = SQRT(1.0D0 - SW2)
      MDL_MW = MZ_IN * CW
      GC_3 = DCMPLX(0D0, -EE)
      GC_59 = DCMPLX(0D0, EE * SW / (2.0D0 * CW))
      GC_50 = DCMPLX(0D0, EE * (-0.5D0 + SW2) / (SW * CW)) - GC_59
      DO J = 1, 4
        IC(J) = 1
      END DO
C     Loop: P_BATCH(0,1,I) is the start of the I-th contiguous 4x4 momentum block
      DO I = 1, N
        CALL MATRIX1(P_BATCH(0, 1, I), IC, TS)
        SUMTS = 0.0D0
        DO J = 1, NCOMB
          SUMTS = SUMTS + TS(J)
        END DO
        M2_OUT(I) = SUMTS
      END DO
      END
