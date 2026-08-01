C     f2py probe for the Drell-Yan pointwise integrand oracle: evaluates both
C     matrix elements of the grouped p p > e+ e- subprocess (MATRIX1 = u u~,
C     MATRIX2 = d d~) at one phase-space point, returning the un-averaged
C     helicity+color-summed |M|^2 for each (MadGraph's MATRIX1 convention,
C     matching vibegraph's eval_m2).  Compiled together with matrix1_optim.f +
C     matrix2_optim.f + libmodel/libdhelas.

C     Stubs for the genps/DiscreteSampler symbols referenced by SMATRIX* but not
C     by the per-helicity MATRIX* we call; satisfy the linker without the
C     madevent dependency chain (mirrors wrappers/generic.f).
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

      SUBROUTINE DY_M2(P, PARAM_PATH, M2_UP, M2_DOWN)
Cf2py intent(in)  P, PARAM_PATH
Cf2py intent(out) M2_UP, M2_DOWN
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      INCLUDE 'maxamps.inc'
      INTEGER MAXHEL
      PARAMETER (MAXHEL = 3**NEXTERNAL)
      CHARACTER*(*) PARAM_PATH
      DOUBLE PRECISION P(0:3, NEXTERNAL), M2_UP, M2_DOWN
      DOUBLE PRECISION TS(MAXHEL)
C     Scratch the callee fills; see wrappers/generic.f for the 3.7.x signature.
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
      M2_UP = 0.0D0
      DO I = 1, MAXHEL
        M2_UP = M2_UP + TS(I)
      END DO

      DO I = 1, MAXHEL
        TS(I) = 0.0D0
      END DO
      CALL MATRIX2(P, IC, TS, AMP2, JAMP2, 1)
      M2_DOWN = 0.0D0
      DO I = 1, MAXHEL
        M2_DOWN = M2_DOWN + TS(I)
      END DO
      END
