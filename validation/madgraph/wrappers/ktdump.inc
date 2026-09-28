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
c     wide to hold is reported rather than silently short; vg_settrunc counts
c     records the buffer had no room for in the record set under construction,
c     and joins vg_trunc only if that set is flushed with an event.
c
c     Under ickkw > 0 one point's record set spans two setclscales calls, the
c     scale update's and rewgt's. vg_ncall counts the calls per process (1: the
c     first call, keepq2bck false; 2: rewgt's), vg_nmemo the jet-memo branches
c     taken in any call (1: store and re-cluster, 2: restricted re-cluster),
c     and vg_c2done whether the run-card constants record has been written.
c*************************************************************************
      integer vg_maxlines, vg_linelen, vg_curmax
      parameter (vg_maxlines=6000)
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
      integer vg_settrunc, vg_ncall(2), vg_nmemo(2)
      logical vg_c2done
      common/vg_ktd_n/vg_settrunc,vg_ncall,vg_nmemo,vg_c2done
