C     Per-diagram amplitude probe for e+ e- > mu+ mu- ta+ ta- (QCD=0).
C
C     Exposes MadGraph's individual AMP(1:NGRAPHS) for one explicit helicity
C     assignment, so vibegraph's per-diagram amplitudes can be matched against
C     MadGraph diagram-by-diagram (by magnitude) to localize the continuum
C     relative-phase bug.  Relies on the COMMON/DBG_AMP/ block patched into
C     matrix1_orig.f's MATRIX1 (which copies AMP into AMP_DBG each call).
C
C       P           REAL*8 P(0:3, NEXTERNAL): external 4-momenta [E,px,py,pz]
C       NHEL        INTEGER NHEL(NEXTERNAL): helicities (+/-1) per leg
C       PARAM_PATH  path to param_card.dat (read once on first call)
C       AMP_OUT     COMPLEX*16 AMP_OUT(NGRAPHS): per-diagram amplitudes
      SUBROUTINE MG_EVAL_AMP(P, NHEL, PARAM_PATH, AMP_OUT)
Cf2py intent(in)  P, NHEL, PARAM_PATH
Cf2py intent(out) AMP_OUT
      IMPLICIT NONE
      INTEGER    NGRAPHS
      PARAMETER (NGRAPHS=25)
      INCLUDE 'nexternal.inc'
      CHARACTER*(*) PARAM_PATH
      REAL*8 P(0:3, NEXTERNAL)
      INTEGER NHEL(NEXTERNAL)
      COMPLEX*16 AMP_OUT(NGRAPHS)

      COMPLEX*16 AMP_DBG(NGRAPHS)
      COMMON/DBG_AMP/AMP_DBG

      REAL*8 MATRIX1, T
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

      T = MATRIX1(P, NHEL, IC, 1)

      DO I = 1, NGRAPHS
        AMP_OUT(I) = AMP_DBG(I)
      END DO
      END

C     Return intermediate and external wavefunctions saved during MATRIX1 evaluation.
C     WF_OUT: COMPLEX*16 (6, 8)
C       col 1 = W7:  gamma current from e-spine (FFV1P0_3)
C       col 2 = W8:  gamma current from mu-spine (FFV1P0_3)
C       col 3 = W10: Z current from e-spine (FFV2_4_3)
C       col 4 = W9:  off-shell ta- absorbing e-Z (FFV2_4_2)
C       col 5 = W11: Z current from mu-spine (FFV2_4_3)
C       col 6 = off-shell e+ after absorbing gamma[mu] (FFV1_1)  -> AMP(18)
C       col 7 = off-shell e+ after absorbing Z[mu]     (FFV2_4_1) -> AMP(22)
C       col 8 = gamma[ta] current (shared e-spine sink boson)
C     EXT_OUT: COMPLEX*16 (6, 6) - all 6 external wavefunctions W1..W6
C       col 1 = W1: e+  (OXXXXX, nsf=-1)
C       col 2 = W2: e-  (IXXXXX, nsf=+1)
C       col 3 = W3: mu- (IXXXXX, nsf=-1, outgoing particle)
C       col 4 = W4: mu+ (OXXXXX, nsf=+1, outgoing antiparticle)
C       col 5 = W5: ta- (IXXXXX, nsf=-1, outgoing particle, massive)
C       col 6 = W6: ta+ (OXXXXX, nsf=+1, outgoing antiparticle, massive)
      SUBROUTINE MG_EVAL_WFUNCS(P, NHEL, PARAM_PATH, WF_OUT, EXT_OUT,
     $   AMP_OUT)
Cf2py intent(in)  P, NHEL, PARAM_PATH
Cf2py intent(out) WF_OUT, EXT_OUT, AMP_OUT
      IMPLICIT NONE
      INTEGER    NGRAPHS
      PARAMETER (NGRAPHS=25)
      INCLUDE 'nexternal.inc'
      CHARACTER*(*) PARAM_PATH
      REAL*8 P(0:3, NEXTERNAL)
      INTEGER NHEL(NEXTERNAL)
      COMPLEX*16 WF_OUT(6, 8)
      COMPLEX*16 EXT_OUT(6, 6)
      COMPLEX*16 AMP_OUT(NGRAPHS)

      COMPLEX*16 WF_DBG(6, 8)
      COMMON/DBG_WFUNCS/WF_DBG

      COMPLEX*16 EXT_WF(6, 6)
      COMMON/DBG_EXT/EXT_WF

      COMPLEX*16 AMP_DBG(NGRAPHS)
      COMMON/DBG_AMP/AMP_DBG

      REAL*8 MATRIX1, T
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

      T = MATRIX1(P, NHEL, IC, 1)

      DO J = 1, 8
        DO I = 1, 6
          WF_OUT(I, J) = WF_DBG(I, J)
        END DO
      END DO
      DO J = 1, 6
        DO I = 1, 6
          EXT_OUT(I, J) = EXT_WF(I, J)
        END DO
      END DO
      DO I = 1, NGRAPHS
        AMP_OUT(I) = AMP_DBG(I)
      END DO
      END
