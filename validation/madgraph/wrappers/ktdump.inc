c*************************************************************************
c     Per-event buffer for the kT-clustering dump.
c
c     setclscales runs on every phase-space point, of which only a small
c     fraction is ever written out, so the clustering records go into this
c     buffer and are flushed when the event reaches write_leshouche. A record
c     set that no event claims is overwritten by the next point.
c
c     vg_state is 0 until the dump destination is read from the environment — the
c     value an untouched common block starts at — then -1 when there is none and
c     1 when the shard is open. vg_dj_branch carries which
c     arm of Source/kin_functions.f's DJ produced the last final-state measure.
c     vg_trunc counts fields dropped for want of line length, so a record too
c     wide to hold is reported rather than silently short.
c*************************************************************************
      integer vg_maxlines, vg_linelen, vg_curmax
      parameter (vg_maxlines=3000)
      parameter (vg_linelen=2048)
c     The merge-graph table lists every config a mask can be reached from, and a
c     2 -> 6 has hundreds of them, so the record under construction gets far more
c     room than the per-event records that are buffered.
      parameter (vg_curmax=65536)

      integer vg_nline, vg_curlen, vg_unit, vg_attempt, vg_pass
      integer vg_dj_branch, vg_state, vg_trunc
      character*(vg_linelen) vg_line(vg_maxlines)
      character*(vg_curmax) vg_cur
      common/vg_ktd_i/vg_nline,vg_curlen,vg_unit,vg_attempt,vg_pass,
     &     vg_dj_branch,vg_state,vg_trunc
      common/vg_ktd_c/vg_line,vg_cur
