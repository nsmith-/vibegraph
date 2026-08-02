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
      integer ln, pid, ierr
      integer getpid
      intrinsic getpid
      if (vg_state.ne.0) return
      vg_state = -1
      call get_environment_variable('VG_KTDUMP', dest, ln)
      if (ln.le.0 .or. ln.gt.480) return
      pid = getpid()
      write(dest(ln+1:), '(a,i10.10)') '.', pid
      open(newunit=vg_unit, file=dest, status='unknown',
     &     position='append', form='formatted')
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
         vg_trunc = vg_trunc+1
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
