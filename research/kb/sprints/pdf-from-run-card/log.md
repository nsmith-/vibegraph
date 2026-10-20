# pdf-from-run-card log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20 (CVMFS layout, measured on the CERN host and relayed by the planner)
* `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current` is a real directory (3202 entries). Each set appears twice, unpacked as `<Set>/` and as `<Set>.tar.gz`. It also holds `pdfsets.index` (47618 B, lines `<lhaid> <name> <version>`), `@README`, `safe-resync.sh`, and the directories `@DEPRECATED` and `@UNVALIDATED`, which are not sets and are skipped.
* `lhapdf.conf` is not in this directory; per-set `.info` keys override it, so the tier is a pure data path.
* `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` (325300): 103 members of about 1.25 MB, tarball 44.9 MB, under `HttpFetch`'s 256 MiB cap. Its `.info` has `SetIndex: 325300 ` **with a trailing space**: the parser trims, and a test pins it. sha256 `.info` 63efe51f3207bffdfcdde3703dca712f760ff2611623c4796c3918ba1354706b, `_0000.dat` ccbf6996fbaff05dfcf5e740e3649dc0873abaffb586ab5991037957248bc86f.
* `NNPDF23_lo_as_0130_qed` (247000): 101 members, tarball 27.6 MB; `.info` f961a10504b4cf070bc0d588006a599954b1b856f3c104ba6e9085f5f81ee4be, `_0000.dat` 5ac9499dcf7f6fe0a363f89f9320782f412eb91732725f7c435f0e2c119e2bf0, identical to the pinned copy, so a CVMFS hit for the default set reproduces today's digests.
* Index entries: 247000 NNPDF23_lo_as_0130_qed, 306000 NNPDF31_nnlo_hessian_pdfas, 325300 NNPDF31_nnlo_as_0118_mc_hessian_pdfas.
