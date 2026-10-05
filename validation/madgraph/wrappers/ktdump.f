c*************************************************************************
c     The kT-clustering dump writer.
c
c     Records are pipe-separated and tagged: a caller opens one with VG_BEG,
c     appends fields with VG_I / VG_D / VG_L / VG_S, and closes it with VG_REC
c     (into the per-event buffer) or VG_NOW (straight to the shard, for the
c     tables that belong to a process directory rather than to an event).
c     Reals carry 18 significant digits because the measures the clustering
c     compares are differences of nearly equal numbers — the event file's 11
c     digits cannot replay a (E-pz)(E+pz) cancellation.
c
c     Each operating-system process gets its own shard, named after its pid,
c     because MadGraph runs its channels concurrently. A shard opens with the
c     directory it is running in, so the tables in it can be told apart from
c     another subprocess directory's.
c*************************************************************************

      subroutine vg_arm()
c     Consult the environment once. VG_KTDUMP names the shard prefix; without
c     it every entry point below returns immediately.
      implicit none
      include 'ktdump.inc'
      character*512 dest
      character*512 cwd
      integer ln, pid, ierr, lgz
      character*8 gz
      integer getpid
      intrinsic getpid
      if (vg_state.ne.0) return
      vg_state = -1
      call get_environment_variable('VG_KTDUMP', dest, ln)
      if (ln.le.0 .or. ln.gt.480) return
      pid = getpid()
      write(dest(ln+1:), '(a,i10.10)') '.', pid
c     VG_KTDUMP_GZIP=1 makes the shard a named pipe that a gzip started here
c     drains into <shard>.gz: a matched 2 -> 3 flushes gigabytes of record
c     sets per run, nearly all of them for points the unweighting later drops.
      call get_environment_variable('VG_KTDUMP_GZIP', gz, lgz)
      if (lgz.gt.0 .and. gz(1:1).eq.'1') then
c        The pipe exists before the command returns; the drain runs detached
c        from this process's standard streams, which MadGraph reads to EOF.
         call execute_command_line('mkfifo '//trim(dest)//'; (gzip -1'
     &        //' < '//trim(dest)//' > '//trim(dest)//'.gz; rm -f '
     &        //trim(dest)//') </dev/null >/dev/null 2>&1 &',
     &        wait=.true.)
         open(newunit=vg_unit, file=dest, status='old',
     &        action='write', form='formatted')
      else
         open(newunit=vg_unit, file=dest, status='unknown',
     &        position='append', form='formatted')
      endif
      vg_state = 1
      vg_nline = 0
      vg_attempt = 0
      vg_pass = 0
      vg_trunc = 0
      call getcwd(cwd, ierr)
      call vg_beg('SHARD')
      call vg_s(cwd(1:len_trim(cwd)))
      call vg_i(pid)
      call vg_now()
      return
      end

      logical function vg_active()
      implicit none
      include 'ktdump.inc'
      if (vg_state.eq.0) call vg_arm()
      vg_active = vg_state.eq.1
      return
      end

      subroutine vg_reset()
c     Start a fresh per-event record set.
      implicit none
      include 'ktdump.inc'
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      vg_nline = 0
      vg_settrunc = 0
      vg_attempt = 0
      vg_pass = 0
      return
      end

      subroutine vg_beg(tag)
      implicit none
      include 'ktdump.inc'
      character*(*) tag
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
c     Assign only the prefix: a whole-variable assignment would blank-pad the
c     scratch record's full length on every record opened.
      vg_curlen = len_trim(tag)
      vg_cur(1:vg_curlen) = tag(1:vg_curlen)
      return
      end

      subroutine vg_app(field)
c     Append one already-formatted field.
      implicit none
      include 'ktdump.inc'
      character*(*) field
      integer n
      n = len_trim(field)
      if (vg_curlen+n+1 .gt. vg_curmax) then
         vg_trunc = vg_trunc+1
         return
      endif
      vg_cur(vg_curlen+1:vg_curlen+1) = '|'
      vg_cur(vg_curlen+2:vg_curlen+1+n) = field(1:n)
      vg_curlen = vg_curlen+1+n
      return
      end

      subroutine vg_i(k)
      implicit none
      include 'ktdump.inc'
      integer k
      character*13 t
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      write(t, '(i13)') k
      call vg_app(adjustl(t))
      return
      end

      subroutine vg_d(x)
      implicit none
      include 'ktdump.inc'
      double precision x
      character*26 t
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
c     A three-digit exponent field: the default drops the E on a denormal, and
c     a record that cannot be parsed is worse than one that is wide.
      write(t, '(1pe26.17e3)') x
      call vg_app(adjustl(t))
      return
      end

      subroutine vg_l(b)
      implicit none
      include 'ktdump.inc'
      logical b
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      if (b) then
         call vg_app('T')
      else
         call vg_app('F')
      endif
      return
      end

      subroutine vg_s(str)
      implicit none
      include 'ktdump.inc'
      character*(*) str
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_app(str)
      return
      end

      subroutine vg_rec()
c     Close the open record into the per-event buffer.
      implicit none
      include 'ktdump.inc'
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      if (vg_nline.ge.vg_maxlines .or. vg_curlen.gt.vg_linelen) then
         vg_settrunc = vg_settrunc+1
         return
      endif
      vg_nline = vg_nline+1
      vg_line(vg_nline) = vg_cur(1:vg_curlen)
      return
      end

      subroutine vg_now()
c     Close the open record straight to the shard.
      implicit none
      include 'ktdump.inc'
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      write(vg_unit, '(a)') vg_cur(1:vg_curlen)
      return
      end

      subroutine vg_flush()
c     Emit the buffered event. BEG opens the record set and END closes it, so a
c     shard a later process appended to is still unambiguous.
      implicit none
      include 'ktdump.inc'
      integer i
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      if (vg_nline.le.0) return
      vg_trunc = vg_trunc+vg_settrunc
      vg_settrunc = 0
      write(vg_unit, '(a)') 'BEG'
      do i = 1, vg_nline
         write(vg_unit, '(a)') vg_line(i)(1:len_trim(vg_line(i)))
      enddo
      write(vg_unit, '(a,i0)') 'END|', vg_trunc
      flush(vg_unit)
      vg_nline = 0
      return
      end

      subroutine vg_mom(tag, i, p, m2)
c     One momentum row: TAG|i|E|px|py|pz|m2.
      implicit none
      include 'ktdump.inc'
      character*(*) tag
      integer i, j
      double precision p(0:3), m2
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg(tag)
      call vg_i(i)
      do j = 0, 3
         call vg_d(p(j))
      enddo
      call vg_d(m2)
      call vg_rec()
      return
      end

      subroutine vg_djname(name)
c     Name the arm of DJ that produced the last final-state measure.
      implicit none
      include 'ktdump.inc'
      character*(*) name
      if (vg_dj_branch.eq.1) then
         name = 'FS_DJ_DURHAM'
      else if (vg_dj_branch.eq.2) then
         name = 'FS_DJ_MLESS_MASSIVE_1'
      else if (vg_dj_branch.eq.3) then
         name = 'FS_DJ_MLESS_MASSIVE_2'
      else if (vg_dj_branch.eq.4) then
         name = 'FS_DJ_HAD'
      else
         name = 'FS_DJ_DEGENERATE'
      endif
      return
      end

      subroutine vg_cand(iatt, ipass, i, j, legi, legj, idi, idj, idij,
     &     adm, branch, raw, infl, pt2, z, ngraph)
c     One candidate pair, admissible or not: a pair the clustering declined to
c     measure is as much of the record as one it won on.
      implicit none
      include 'ktdump.inc'
      integer iatt, ipass, i, j, legi, legj, idi, idj, idij, ngraph
      logical adm, infl
      character*(*) branch
      double precision raw, pt2, z
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('CAND')
      call vg_i(iatt)
      call vg_i(ipass)
      call vg_i(i)
      call vg_i(j)
      call vg_i(legi)
      call vg_i(legj)
      call vg_i(idi)
      call vg_i(idj)
      call vg_i(idij)
      call vg_l(adm)
      call vg_s(branch)
      call vg_d(raw)
      call vg_l(infl)
      call vg_d(pt2)
      call vg_d(z)
      call vg_i(ngraph)
      call vg_rec()
      return
      end

      subroutine vg_graphs(when)
c     The surviving graph list, recorded either side of the point where the
c     integration channel is allowed to claim it.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      character*(*) when
      integer i
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('GRPH')
      call vg_i(vg_attempt)
      call vg_s(when)
      call vg_i(igraphs(0))
      do i = 1, igraphs(0)
         call vg_i(igraphs(i))
      enddo
      call vg_rec()
      return
      end

      subroutine vg_jidx(when, jfirst, jlast, jcentral)
c     The three beam-side vertex indices, before and after the jfirst fixup.
      implicit none
      include 'ktdump.inc'
      character*(*) when
      integer jfirst(2), jlast(2), jcentral(2)
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('JIDX')
      call vg_s(when)
      call vg_i(jfirst(1))
      call vg_i(jfirst(2))
      call vg_i(jlast(1))
      call vg_i(jlast(2))
      call vg_i(jcentral(1))
      call vg_i(jcentral(2))
      call vg_rec()
      return
      end

      subroutine vg_lines(ipart, goodjet, igraph, iproc)
c     Every line the beam-side walk could have asked about: the externals and
c     the mothers the merge sequence produced, with the provenance and jet
c     flags the walk reads them through.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      integer ipart(2,n_max_cl), igraph, iproc
      logical goodjet(n_max_cl)
      integer i, n, mask
      logical isqcd, isjet, vg_active
      external isqcd, isjet, vg_active
      if (.not.vg_active()) return
      do i = 1, nexternal
         call vg_line1(ishft(1,i-1), ipart, goodjet, igraph, iproc)
      enddo
      do n = 1, nexternal-2
         mask = imocl(n)
         call vg_line1(mask, ipart, goodjet, igraph, iproc)
      enddo
      return
      end

      subroutine vg_line1(mask, ipart, goodjet, igraph, iproc)
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      integer mask, ipart(2,n_max_cl), igraph, iproc
      logical goodjet(n_max_cl)
      logical isqcd, isjet
      external isqcd, isjet
      if (mask.le.0 .or. mask.gt.n_max_cl) return
      call vg_beg('LINE')
      call vg_i(mask)
      call vg_i(ipdgcl(mask,igraph,iproc))
      call vg_i(ipart(1,mask))
      call vg_i(ipart(2,mask))
      call vg_l(isqcd(ipdgcl(mask,igraph,iproc)))
      call vg_l(isjet(ipdgcl(mask,igraph,iproc)))
      call vg_l(goodjet(mask))
      call vg_rec()
      return
      end

      subroutine vg_pt2(stage)
c     Every vertex scale at one point in the rewrite chain, so which rewrite
c     moved which vertex is a direct read.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      character*(*) stage
      integer n
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      do n = 1, nexternal-2
         call vg_beg('PT2')
         call vg_s(stage)
         call vg_i(n)
         call vg_d(pt2ijcl(n))
         call vg_d(mt2ij(n))
         call vg_rec()
      enddo
      return
      end

      subroutine vg_rej(which)
      implicit none
      include 'ktdump.inc'
      character*(*) which
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('REJ')
      call vg_s(which)
      call vg_rec()
      return
      end

      subroutine vg_muf(branch, q2f1, q2f2)
      implicit none
      include 'ktdump.inc'
      character*(*) branch
      double precision q2f1, q2f2
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('MUF')
      call vg_s(branch)
      call vg_d(q2f1)
      call vg_d(q2f2)
      call vg_rec()
      return
      end

      subroutine vg_mur(branch, jlast, jcentral, mur)
c     The mu_R branch with the pt2ijcl values that fed it, so the geometric
c     mean can be recomputed from the record alone.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      character*(*) branch
      integer jlast(2), jcentral(2)
      double precision mur
      integer i
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('MUR')
      call vg_s(branch)
      do i = 1, 2
         if (jlast(i).gt.0) then
            call vg_d(pt2ijcl(jlast(i)))
         else
            call vg_d(0d0)
         endif
         if (jcentral(i).gt.0) then
            call vg_d(pt2ijcl(jcentral(i)))
         else
            call vg_d(0d0)
         endif
      enddo
      call vg_d(pt2ijcl(nexternal-2))
      call vg_d(mur)
      call vg_rec()
      return
      end

c*************************************************************************
c     Records for matched generation (ickkw > 0): the second setclscales call,
c     the jet memo's re-cluster branches, rewgt's vertex and PDF-ratio
c     decisions, and ptclus.
c*************************************************************************

      subroutine vg_newcall(keep)
c     setclscales opens a point's record set on its first call (keepq2bck
c     false, from the scale update); rewgt's second call (keepq2bck true)
c     appends to it. The clustering attempt counters restart on either.
      implicit none
      include 'ktdump.inc'
      logical keep
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      if (.not.vg_c2done) then
         vg_c2done = .true.
         call vg_const2()
      endif
      if (keep) then
         vg_ncall(2) = vg_ncall(2)+1
         vg_attempt = 0
         vg_pass = 0
      else
         vg_ncall(1) = vg_ncall(1)+1
         call vg_reset()
      endif
      return
      end

      subroutine vg_const2()
c     The run-card constants matching reads, as setcuts left them: ptj and
c     mmjj after the xqcut rewrite, drjj / drjl after it zeroed them.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'run.inc'
      include 'cuts.inc'
      include 'ktdump.inc'
      call vg_beg('CONST2')
      call vg_i(ickkw)
      call vg_d(xqcut)
      call vg_d(alpsfact)
      call vg_i(asrwgtflavor)
      call vg_i(maxjetflavor)
      call vg_l(pdfwgt)
      call vg_l(hmult)
      call vg_i(ktscheme)
      call vg_l(chcluster)
      call vg_l(auto_ptj_mjj)
      call vg_d(ptj)
      call vg_d(mmjj)
      call vg_d(drjj)
      call vg_d(drjl)
      call vg_d(xptj)
      call vg_l(use_syst)
      call vg_d(scalefact)
      call vg_now()
      return
      end

      subroutine vg_memox(kind, iconfig, iproc, njets, nstore, igraph1,
     &     keep)
c     A jet-memo re-cluster branch, written straight to the shard whether or
c     not the point is ever kept, so the census covers every point.
      implicit none
      include 'ktdump.inc'
      character*(*) kind
      integer iconfig, iproc, njets, nstore, igraph1
      logical keep
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      if (kind.eq.'STORE_AND_RECLUSTER') then
         vg_nmemo(1) = vg_nmemo(1)+1
      else
         vg_nmemo(2) = vg_nmemo(2)+1
      endif
      call vg_beg('MEMOX')
      call vg_s(kind)
      call vg_i(iconfig)
      call vg_i(iproc)
      call vg_i(njets)
      call vg_i(nstore)
      call vg_i(igraph1)
      call vg_l(keep)
      call vg_i(vg_ncall(1))
      call vg_i(vg_ncall(2))
      call vg_now()
      return
      end

      subroutine vg_counts()
c     The per-process running counts, carried by every flushed event.
      implicit none
      include 'ktdump.inc'
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('CNT')
      call vg_i(vg_ncall(1))
      call vg_i(vg_ncall(2))
      call vg_i(vg_nmemo(1))
      call vg_i(vg_nmemo(2))
      call vg_rec()
      return
      end

      subroutine vg_q2ovr(when, jcentral)
c     The central-vertex scales either side of the ickkw > 0 overwrite by
c     the q2fact the call was entered with.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'run.inc'
      include 'ktdump.inc'
      character*(*) when
      integer jcentral(2), i
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('Q2OVR')
      call vg_s(when)
      call vg_i(jcentral(1))
      call vg_i(jcentral(2))
      do i = 1, 2
         if (jcentral(i).gt.0) then
            call vg_d(pt2ijcl(jcentral(i)))
         else
            call vg_d(0d0)
         endif
      enddo
      call vg_d(q2fact(1))
      call vg_d(q2fact(2))
      call vg_rec()
      return
      end

      subroutine vg_ptcl(when)
c     ptclus per external leg: in setclscales' leg order (SETCL) or after
c     write_leshouche's symmetry reordering (OUT).
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'ktdump.inc'
      character*(*) when
      integer i
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('PTCL')
      call vg_s(when)
      call vg_i(nexternal)
      do i = 1, nexternal
         call vg_d(ptclus(i))
      enddo
      call vg_rec()
      return
      end

      subroutine vg_cfg(ivec)
c     The configuration the event record takes its colour and mothers from:
c     the integration channel, the clustering's graph, and the per-event
c     graph rewgt saved for addmothers.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'run.inc'
      include 'ktdump.inc'
      integer ivec
      integer mapconfig(0:lmaxconfigs), iconfig
      common/to_mconfigs/mapconfig, iconfig
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('CFG')
      call vg_i(iconfig)
      call vg_i(igraphs(1))
      call vg_i(vec_igraph(ivec))
      call vg_i(ickkw)
      call vg_rec()
      return
      end

      subroutine vg_rwkill(reason, n)
      implicit none
      include 'ktdump.inc'
      character*(*) reason
      integer n
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      call vg_beg('RWKILL')
      call vg_s(reason)
      call vg_i(n)
      call vg_rec()
      return
      end

      subroutine vg_rwvx(n, cls, isr, fsr, gmo, ipart, goodjet, igraph,
     &     iproc, q2now, asnum, asref, rwrun)
c     One clustering vertex as rewgt's alpha_s loop judged it. cls is CORE
c     (the 2 -> 2 vertex, never reweighted), ISR / FSR (reweighted), NONE
c     (neither condition), FAKE_ID (qualified, mother is fake_id) or KILL_Q2
c     (qualified with q2now <= 4 GeV^2, the event is dropped). isr / fsr are
c     the two arms of the qualifying condition, gmo the mother's goodjet.
      implicit none
      include 'genps.inc'
      include 'nexternal.inc'
      include 'maxamps.inc'
      include 'cluster.inc'
      include 'run.inc'
      include 'ktdump.inc'
      integer n, ipart(2,n_max_cl), igraph, iproc
      character*(*) cls
      logical isr, fsr, gmo, goodjet(n_max_cl)
      double precision q2now, asnum, asref, rwrun, mu, ratio
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      mu = 0d0
      if (q2now.gt.0d0) mu = alpsfact*sqrt(q2now)
      ratio = 1d0
      if (asnum.gt.0d0 .and. asref.gt.0d0) ratio = asnum/asref
      call vg_beg('RWVX')
      call vg_i(n)
      call vg_s(cls)
      call vg_i(imocl(n))
      call vg_i(idacl(n,1))
      call vg_i(idacl(n,2))
      call vg_i(ipdgcl(imocl(n),igraph,iproc))
      call vg_i(ipdgcl(idacl(n,1),igraph,iproc))
      call vg_i(ipdgcl(idacl(n,2),igraph,iproc))
      call vg_i(ipart(1,imocl(n)))
      call vg_i(ipart(2,imocl(n)))
      call vg_l(isr)
      call vg_l(fsr)
      call vg_l(gmo)
      call vg_l(goodjet(idacl(n,1)))
      call vg_l(goodjet(idacl(n,2)))
      call vg_d(q2now)
      call vg_d(mu)
      call vg_d(asnum)
      call vg_d(asref)
      call vg_d(ratio)
      call vg_d(rwrun)
      call vg_rec()
      return
      end

      subroutine vg_rwpdf(n, j, i, ibeam, pdg, z, x, q2now, q2prev,
     &     act, pnum, pden, rwrun)
c     One step of the PDF-ratio walk up beam j's line at vertex n, through
c     daughter slot i. x is the momentum fraction after this vertex's z,
c     q2prev the scale the line entered with (pt2pdf of the daughter), act
c     FIRST (no ratio: the matrix-element PDF), RATIO, NOT_RISING (scale did
c     not rise), NONE (rising scale past jlast), PS_START or KILL_PDF.
      implicit none
      include 'ktdump.inc'
      integer n, j, i, ibeam, pdg
      double precision z, x, q2now, q2prev, pnum, pden, rwrun, ratio
      character*(*) act
      logical vg_active
      external vg_active
      if (.not.vg_active()) return
      ratio = 1d0
      if (act.eq.'RATIO' .and. pden.ne.0d0) ratio = pnum/pden
      call vg_beg('RWPDF')
      call vg_i(n)
      call vg_i(j)
      call vg_i(i)
      call vg_i(ibeam)
      call vg_i(pdg)
      call vg_d(z)
      call vg_d(x)
      call vg_d(q2now)
      call vg_d(q2prev)
      call vg_s(act)
      call vg_d(pnum)
      call vg_d(pden)
      call vg_d(ratio)
      call vg_d(rwrun)
      call vg_rec()
      return
      end
