C     Generic f2py wrapper for any MadGraph MadEvent subprocess.
C
C     Replaces the per-process wrappers (ee_to_mumu.f, pp_to_ll_qcd0.f): instead
C     of hand-deriving the EW couplings (GC_*, whose numbering changes per
C     process), it calls MadGraph's own SETPARA to populate coupl.inc from the
C     standard param_card.dat.  Everything else is read from the subprocess's own
C     include files, so this single file compiles unchanged against any process.
C
C     Assumptions:
C       * All external particles have 2 helicity states (massless fermions /
C         massless vectors), so the helicity count is N_MAX_CL = 2^NEXTERNAL.
C         True for our EW lepton+quark processes; external massive vectors would
C         need a different count.
C       * MATRIX1(P, IC, TS) returns the per-helicity, color-summed |M|^2 in
C         TS(NCOMB) (the helicity-recycled matrix1_optim.f API).
C
C     Returns M2_OUT = sum_hel sum_color |M|^2  (NOT divided by IDEN / averaged),
C     matching vibegraph's AmplitudeEvaluator.eval_m2 convention.  The remaining
C     single-color-flow factor (CF(1,1)) is applied on the vibegraph side.

C     Stubs for symbols referenced by SMATRIX1 but not by MATRIX1; satisfy the
C     linker without pulling in the genps.f / DiscreteSampler dependency chain.
      DOUBLE PRECISION FUNCTION GET_CHANNEL_CUT(P, CONFIG)
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      DOUBLE PRECISION P(0:3, NEXTERNAL)
      INTEGER CONFIG
      GET_CHANNEL_CUT = 1.0D0
      END

      SUBROUTINE RANMAR(R)
      IMPLICIT NONE
      DOUBLE PRECISION R
      R = 0.5D0
      END

C     Single-event entry point.
C       P           REAL*8 P(0:3, NEXTERNAL): external 4-momenta [E,px,py,pz]
C       PARAM_PATH  path to param_card.dat (passed from Python to avoid CWD
C                    assumptions); read once on the first call.
C       M2_OUT      sum_hel sum_color |M|^2
      SUBROUTINE MG_EVAL_M2(P, PARAM_PATH, M2_OUT)
Cf2py intent(in)  P, PARAM_PATH
Cf2py intent(out) M2_OUT
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      INCLUDE 'ncombs.inc'
      CHARACTER*(*) PARAM_PATH
      DOUBLE PRECISION P(0:3, NEXTERNAL), M2_OUT
      DOUBLE PRECISION TS(N_MAX_CL)
      INTEGER IC(NEXTERNAL), I
      LOGICAL FIRST
      SAVE FIRST
      DATA FIRST /.TRUE./

      IF (FIRST) THEN
        CALL SETPARA(PARAM_PATH)
        FIRST = .FALSE.
      ENDIF

      DO I = 1, NEXTERNAL
        IC(I) = 1
      END DO
      DO I = 1, N_MAX_CL
        TS(I) = 0.0D0
      END DO

      CALL MATRIX1(P, IC, TS)

      M2_OUT = 0.0D0
      DO I = 1, N_MAX_CL
        M2_OUT = M2_OUT + TS(I)
      END DO
      END

C     Batch entry point: evaluate N events in one call (amortizes setup for the
C     timing benchmark).  P_BATCH is REAL*8 (0:3, NEXTERNAL, N).
      SUBROUTINE MG_EVAL_M2_BATCH(P_BATCH, N, PARAM_PATH, M2_OUT)
Cf2py intent(in)  P_BATCH, PARAM_PATH
Cf2py intent(out) M2_OUT
Cf2py integer intent(hide), depend(P_BATCH) :: N = shape(P_BATCH, 2)
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      INCLUDE 'ncombs.inc'
      CHARACTER*(*) PARAM_PATH
      INTEGER N
      DOUBLE PRECISION P_BATCH(0:3, NEXTERNAL, N)
      DOUBLE PRECISION M2_OUT(N)
      DOUBLE PRECISION TS(N_MAX_CL), SUMTS
      INTEGER IC(NEXTERNAL), I, J
      LOGICAL FIRST
      SAVE FIRST
      DATA FIRST /.TRUE./

      IF (FIRST) THEN
        CALL SETPARA(PARAM_PATH)
        FIRST = .FALSE.
      ENDIF

      DO I = 1, NEXTERNAL
        IC(I) = 1
      END DO

      DO J = 1, N
        DO I = 1, N_MAX_CL
          TS(I) = 0.0D0
        END DO
        CALL MATRIX1(P_BATCH(0, 1, J), IC, TS)
        SUMTS = 0.0D0
        DO I = 1, N_MAX_CL
          SUMTS = SUMTS + TS(I)
        END DO
        M2_OUT(J) = SUMTS
      END DO
      END
