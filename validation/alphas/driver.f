C-----------------------------------------------------------------------------
C     Reference generator for MadGraph's running alpha_s.
C
C     Drives ALPHAS from the unmodified MG5 LO template source
C     (Source/alfas_functions.f) over a grid of (asmz, nloop, Q) and writes one
C     CSV row per point with 17 significant digits, which round-trips exactly
C     through an IEEE double.
C
C     The Q grid straddles both flavour thresholds (CMASS = 1.42, BMASS = 4.7)
C     and both sides of ZMASS = 91.188, sampling each threshold from immediately
C     below and immediately above so a mis-placed branch cannot hide between
C     grid points.
C-----------------------------------------------------------------------------
      PROGRAM ALPHAS_REFERENCE
      IMPLICIT NONE

      INCLUDE 'alfas.inc'

      DOUBLE PRECISION ALPHAS
      EXTERNAL ALPHAS

      INTEGER NASMZ, NQFIX, NQLOG, NQ
      PARAMETER (NASMZ=4, NQFIX=26, NQLOG=40)
      PARAMETER (NQ=NQFIX+NQLOG)

      DOUBLE PRECISION ASGRID(NASMZ), QGRID(NQ)
      INTEGER IA, IL, IQ
      DOUBLE PRECISION QLO, QHI, VAL

C     alpha_s(M_Z) values: the two parameter-card settings the banked runs use,
C     the value the G -> asmz round trip of setrun.f actually produces for a card
C     holding 0.130, and the nn23lo/nn23nlo table entry.
      DATA ASGRID/0.118D0, 0.130D0, 0.13000000000000003D0, 0.119D0/

C     Fixed points: both sides of each threshold and of M_Z, plus the scales the
C     banked runs actually report.
      DATA QGRID(1) /1.0D0/
      DATA QGRID(2) /1.1D0/
      DATA QGRID(3) /1.2D0/
      DATA QGRID(4) /1.4D0/
      DATA QGRID(5) /1.4199999D0/
      DATA QGRID(6) /1.42D0/
      DATA QGRID(7) /1.4200001D0/
      DATA QGRID(8) /1.5D0/
      DATA QGRID(9) /2.0D0/
      DATA QGRID(10)/3.0D0/
      DATA QGRID(11)/4.6D0/
      DATA QGRID(12)/4.6999999D0/
      DATA QGRID(13)/4.7D0/
      DATA QGRID(14)/4.7000001D0/
      DATA QGRID(15)/5.0D0/
      DATA QGRID(16)/5.1491350D0/
      DATA QGRID(17)/10.0D0/
      DATA QGRID(18)/50.0D0/
      DATA QGRID(19)/91.187D0/
      DATA QGRID(20)/91.188D0/
      DATA QGRID(21)/91.189D0/
      DATA QGRID(22)/91.2D0/
      DATA QGRID(23)/250.0D0/
      DATA QGRID(24)/500.0D0/
      DATA QGRID(25)/1000.0D0/
      DATA QGRID(26)/13000.0D0/

C     Logarithmic sweep filling the gaps between the fixed points.
      QLO = 1.0D0
      QHI = 14000.0D0
      DO IQ = 1, NQLOG
         QGRID(NQFIX+IQ) = QLO*(QHI/QLO)**(DBLE(IQ-1)/DBLE(NQLOG-1))
      ENDDO

      OPEN(UNIT=10, FILE='reference.csv', STATUS='REPLACE')
      WRITE(10,'(A)') 'asmz,nloop,q,alphas'
      DO IA = 1, NASMZ
         DO IL = 1, 3
            asmz = ASGRID(IA)
            nloop = IL
            DO IQ = 1, NQ
               VAL = ALPHAS(QGRID(IQ))
               WRITE(10,'(ES26.17E3,A,I1,A,ES26.17E3,A,ES26.17E3)')
     &              asmz, ',', nloop, ',', QGRID(IQ), ',', VAL
            ENDDO
         ENDDO
      ENDDO
      CLOSE(10)

      WRITE(6,*) 'wrote reference.csv:', NASMZ*3*NQ, ' rows'

      END
