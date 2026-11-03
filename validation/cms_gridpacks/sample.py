"""The stratified gridpack sampling rule.

Strata:
  A_run3_sm   Run 3 SM backgrounds: {PdmV,CommonBkg}/{RunIII2024Summer24,Run3Summer22}
              MadGraph5_aMCatNLO family folders (not Tests/NewTests). Within a
              family, tarballs are grouped by process stem (the name up to the
              first binning token) and stems are taken most-binned first:
              `lo_stems[family]` LO stems and `nlo_stems` NLO stems.
  A2_run3_sm  curated Run 3 SM processes absent from the background folders
              (`extras`), one regex each.
  B_run3_bsm  cms-griddata pp/13p6TeV/{madgraph,madgraphMLM}/<folder>: the
              `n_bsm_folders` largest folders.
  C_run2      Run 2 2017/UL MadGraph folders matching `run2` regexes.

Every group then contributes its first `per_group` members in sha256(basename)
order. That order does not depend on the rest of the inventory, so raising any
count only adds rows: a grown sample contains the smaller one.
"""

import hashlib
import re
from collections import defaultdict

RULE = "stratified-v2"
REPACKED = re.compile(r"original|_TEST|noiter|trimmed|before_|_sys_patch|wo_runcmsgrid")


def default_params():
    return {
        "lo_stems": {"DY": 3, "W": 3, "Z": 2, "QCD": 3, "GJ": 2, "TT": 3, "Others": 3,
                     "VBF": 2, "WZ": 1, "ZZ": 1, "WW": 1, "SingleT": 1},
        "lo_stems_default": 1,
        "nlo_stems": 1,
        "n_bsm_folders": 30,
        "per_group": 1,
        "run2": [
            ["DY", "^DYJets_HT_mll50$"], ["DY", "^DY2Jets_madgraph_5f_LO$"], ["W", "^WJetsToLNu_HT"],
            ["GJ", "^GJets$"], ["QCD", "^QCD_HT700to1000$"], ["QCD", "^QCD_bEnriched_HT500to700$"],
            ["Z", "^ZJetsToNuNu_HT"], ["TT", "^TTJets"], ["VBF", "^ewkzlljj_5f_LO$"], ["TTV", "^TTWJetsToLNu"],
            ["DY", "^dyellell012j_5f_NLO_FXFX$"], ["Higgs", "^ggh012j_5f_NLO_FXFX_125_HZZ4l$"],
            ["VBS", "^VVjj$"], ["DY", "^DYToLL_MassBinned_JetBinned$"],
        ],
        "extras": [
            ["TTV", "^TTZ2b_5f_LO_"], ["TTV", "^TTWW_5f_LO_"], ["TTV", "^TTHH_5f_LO_madspinON_"],
            ["TTV-EFT", "^ttZ_3l_4l_EFT_CPV_point_0_13p6"], ["SingleT-EFT", "^tZq_EFT_CPV_point_1_13p6"],
            ["Higgs", "^tth_4f_ckm_LO_ctcvcp_"], ["Higgs", "^thq_4f_ckm_LO_ctcvcp_MH125_"],
            ["Higgs", "^VBF_HH_kl_1p00_cv_1p00_c2v_1p00_"], ["Higgs", "^VBF_H_"], ["Higgs", "^ggHZ_"],
            ["VBS", "^VBS_SSWW_EWK_PolarizationLL_"], ["VBS", "^WWjj_SS_ewk_"], ["Diboson", "^WW_LL_"],
            ["Triboson", "^WZZJets_5f_"], ["Higgs", "^vbfh_5f_NLO_125_"], ["TTV", "^tth01j_5f_ckm_NLO_FXFX_MH125_"],
            ["Triboson", "^WWWJets_4f_NLO_"], ["Higgs", "^ggh012j_5f_NLO_FXFX_125_"],
        ],
    }


def order_guess(name):
    return "NLO" if re.search(r"amcatnlo|NLO|FXFX|FxFx", name) else "LO"


def stem(name):
    return re.split(r"_(Bin|HT|MLL|MLNu|PTG|PTLL|PTLNu|PTQQ)[-_]", name)[0]


def _key(path):
    return hashlib.sha256(path.rsplit("/", 1)[1].encode()).hexdigest()


def stratified(paths, params):
    """-> [{'path', 'stratum', 'family', 'order_guess', 'rank'}] for the given inventory paths."""
    seen, keep = set(), []
    for p in sorted(paths):
        b = p.rsplit("/", 1)[1]
        if REPACKED.search(b) or b in seen:
            continue
        seen.add(b)
        keep.append(p)
    per = params["per_group"]
    rows = []

    def take(stratum, family, members, og=None):
        for rank, p in enumerate(sorted(members, key=_key)[:per]):
            rows.append({"path": p, "stratum": stratum, "family": family, "rank": rank,
                         "order_guess": og or order_guess(p.rsplit("/", 1)[1])})

    famre = re.compile(r"/(?:PdmV|CommonBkg)/(RunIII2024Summer24|Run3Summer22)/MadGraph5_aMCatNLO/([^/]+)/")
    fam = defaultdict(list)
    for p in keep:
        m = famre.search(p)
        if m and m.group(2) not in ("NewTests", "Tests"):
            fam[m.group(2)].append(p)
    for f in sorted(fam):
        stems = defaultdict(list)
        for p in fam[f]:
            stems[stem(p.rsplit("/", 1)[1])].append(p)
        ranked = sorted(stems, key=lambda s: (-len(stems[s]), s))
        og = {s: order_guess(stems[s][0].rsplit("/", 1)[1]) for s in ranked}
        lo = [s for s in ranked if og[s] == "LO"][:params["lo_stems"].get(f, params["lo_stems_default"])]
        nlo = [s for s in ranked if og[s] == "NLO"][:params["nlo_stems"]]
        for s in lo + nlo:
            take("A_run3_sm", "%s/%s" % (f, s), stems[s], og[s])

    for f, pat in params["extras"]:
        hits = [p for p in keep if "/gridpacks/RunIII/13p6TeV/" in p
                and not re.search(r"/(Powheg|powheg|JHUGen|MCFM|Sherpa)/", p) and p.endswith(".tar.xz")
                and re.search(pat, p.rsplit("/", 1)[1])]
        if hits:
            take("A2_run3_sm", "%s:%s" % (f, pat), hits)

    bsm = defaultdict(list)
    bre = re.compile(r"/gridpacks_tarball/pp/13p6TeV/(madgraph|madgraphMLM)/([^/]+)/")
    for p in keep:
        m = bre.search(p)
        if m:
            bsm[m.group(1) + "/" + m.group(2)].append(p)
    for k in sorted(bsm, key=lambda k: (-len(bsm[k]), k))[:params["n_bsm_folders"]]:
        take("B_run3_bsm", k, bsm[k])

    r2 = defaultdict(list)
    cre = re.compile(r"/gridpacks/(2017/13TeV/madgraph/V5_2\.4\.2|UL/13TeV/madgraph/V5_2\.6\.5)/([^/]+)/")
    for p in keep:
        m = cre.search(p)
        if m:
            r2[m.group(2)].append(p)
    for f, pat in params["run2"]:
        hits = sorted(k for k in r2 if re.search(pat, k))
        if hits:
            take("C_run2", "%s:%s" % (f, hits[0]), r2[hits[0]])
    return rows
