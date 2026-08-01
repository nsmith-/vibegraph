C     Generic f2py wrapper for any MadGraph MadEvent subprocess.
C
C     Instead of hand-deriving the EW couplings (GC_*, whose numbering changes
C     per process), it calls MadGraph's own SETPARA to populate coupl.inc from
C     the standard param_card.dat.  Everything else is read from the
C     subprocess's own include files, so this single file compiles unchanged
C     against any process.
C
C     Assumptions:
C       * MATRIX1(P, IC, TS, AMP2, JAMP2, IVEC) returns the per-helicity,
C         color-summed |M|^2 in TS(NCOMB) (the helicity-recycled
C         matrix1_optim.f API of MadGraph 3.7.x; 3.5.x took the first three
C         arguments only).  NCOMB is not exported by any include file, so TS is
C         oversized to 3^NEXTERNAL (>= any product of SM per-leg helicity
C         counts, incl. massive vectors' 3) and zero-initialized; the sum over
C         unused entries is a no-op.  AMP2 and JAMP2 are scratch the callee
C         fills and nothing here reads; IVEC indexes the vectorized-event
C         dimension, which a generated MadEvent directory sizes at 1.
C
C     Returns M2_OUT = sum_hel sum_color |M|^2  (NOT divided by IDEN / averaged),
C     matching vibegraph's AmplitudeEvaluator.eval_m2 convention.  The remaining
C     single-color-flow factor (CF(1,1)) is applied on the vibegraph side.

C     Stubs for symbols referenced by SMATRIX1 but not by MATRIX1; satisfy the
C     linker without pulling in the genps.f / DiscreteSampler dependency chain.
C     SELECT_COLOR is one of these from 3.7.x on, where SMATRIX1 draws the event's
C     colour flow itself; nothing here reads the drawn flow.
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

      SUBROUTINE SELECT_COLOR(RCOL, JAMP2, ICONFIG, IPROC, ICOL, IVEC)
      IMPLICIT NONE
      INCLUDE 'maxamps.inc'
      DOUBLE PRECISION RCOL, JAMP2(0:MAXFLOW)
      INTEGER ICONFIG, IPROC, ICOL, IVEC
      ICOL = 1
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
      INCLUDE 'maxamps.inc'
      INTEGER MAXHEL
      PARAMETER (MAXHEL = 3**NEXTERNAL)
      CHARACTER*(*) PARAM_PATH
      DOUBLE PRECISION P(0:3, NEXTERNAL), M2_OUT
      DOUBLE PRECISION TS(MAXHEL)
      DOUBLE PRECISION AMP2(MAXAMPS), JAMP2(0:MAXFLOW)
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
      DO I = 1, MAXHEL
        TS(I) = 0.0D0
      END DO

      CALL MATRIX1(P, IC, TS, AMP2, JAMP2, 1)

      M2_OUT = 0.0D0
      DO I = 1, MAXHEL
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
      INCLUDE 'maxamps.inc'
      INTEGER MAXHEL
      PARAMETER (MAXHEL = 3**NEXTERNAL)
      CHARACTER*(*) PARAM_PATH
      INTEGER N
      DOUBLE PRECISION P_BATCH(0:3, NEXTERNAL, N)
      DOUBLE PRECISION M2_OUT(N)
      DOUBLE PRECISION TS(MAXHEL), SUMTS
      DOUBLE PRECISION AMP2(MAXAMPS), JAMP2(0:MAXFLOW)
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
        DO I = 1, MAXHEL
          TS(I) = 0.0D0
        END DO
        CALL MATRIX1(P_BATCH(0, 1, J), IC, TS, AMP2, JAMP2, 1)
        SUMTS = 0.0D0
        DO I = 1, MAXHEL
          SUMTS = SUMTS + TS(I)
        END DO
        M2_OUT(J) = SUMTS
      END DO
      END
