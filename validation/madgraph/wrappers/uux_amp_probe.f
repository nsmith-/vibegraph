C     Per-diagram amplitude probe for u u~ > c c~ e+ e- mu+ mu- (QCD=0).
C
C     Exposes MadGraph's individual AMP(1:NGRAPHS) for one explicit helicity
C     assignment, so vibegraph's per-diagram amplitudes can be matched against
C     MadGraph diagram-by-diagram to localize the uux 2->6 continuum residual.
C     Relies on the COMMON/DBG_AMP/ block patched into matrix1_orig.f's
C     MATRIX1 at build time (build_amplitude.sh build_uux_amp_probe), which
C     copies AMP into AMP_DBG each call.
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
      PARAMETER (NGRAPHS=579)
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
