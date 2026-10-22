"""Parser checks on excerpts of real gridpack files and vibegraph output."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import parse as P  # noqa: E402

# gridpack_generation.log of TTZ2b_5f_LO (MG 2.9.18): the pilot survey did not
# converge, so its only σ has an error larger than itself, and the gridpack's
# own test run prints a second, zero result.
LO_LOG = """\
System release  Red Hat Enterprise Linux release 8.10 (Ootpa)
6 processes with 484 diagrams generated in 2.773 s
Total: 6 processes with 484 diagrams
Generated helas calls for 4 subprocesses (388 diagrams) in 0.682 s
Wrote files for 637 helas calls in 1.152 s
INFO: Running Survey
INFO:  Idle: 51,  Running: 16,  Completed: 0 [ current time: 19h43 ]
INFO:  Idle: 50,  Running: 16,  Completed: 1 [  4.4s  ]
INFO:  Idle: 0,  Running: 0,  Completed: 67 [  1h 2m 5s  ]
sum of cpu time of last step: 3h39m48s
  === Results Summary for run: pilotrun tag: tag_1 ===

     Cross-section :   1.013e+07 +- 1.9e+11 pb
     Nb of events :  0
INFO: End survey
Now generating 1000 events with random seed 234567 and granularity 1
sum of cpu time of last step: 1 day,16h32m55s
     Cross-section :   0 +- 0 pb
     Nb of events :  1000
"""

# DYto2L-2Jets FxFx (aMC@NLO): intermediate results, then the final one after collection.
NLO_LOG = """\
INFO: Generated 525 subprocesses with 82500 real emission diagrams, 5520 born diagrams and 92970 virtual diagrams
INFO: Starting run
      Intermediate results:
      Total cross section:      7.464e-01 +- 4.6e-03 pb
INFO: Collecting events
      Total cross section: 7.442e-01 +- 2.0e-03 pb
"""

RUN_CARD = """\
#*********************************************************************
  1	= lpp1 ! beam 1 type
  6800.0	= ebeam1 ! beam 1 total energy in GeV
  sum	= event_norm ! average/sum. Normalization of the weight in the LHEF
  False = pdfwgt # hidden_parameter
  325300 = lhaid ! if pdlabel=lhapdf, this is the lhapdf number
"""

VG_OK = """\
 INFO enumerate: enumerated 8 diagrams in 3 subprocess (0.004 s)
 INFO 4 outgoing legs (@0): σ = 0.000543 ± 0.000039 pb over 5 channels
 INFO 5 outgoing legs (@1): σ = 0.001410 ± 0.000192 pb over 48 channels
 INFO model:    DMsimp_s_spin0-default (1c5615f093eb3e4e2a2d7db83df7b57ca79f8c596c9f3342517572ef4b549ef4)
 INFO VEGAS:    183677 evals over 3 iters, seed 1 (χ²/dof = 0.000)
 INFO channels: 53 grids, 71429 evals in the last iteration, allocated by α
σ = 0.001952 ± 0.000196 pb
"""
VG_TRACE = """\
2026-10-19T19:32:42.985233Z TRACE ufo_load: vibegraph::progress: stage="ufo_load" done=0 total=1
2026-10-19T19:32:43.985233Z TRACE ufo_load: vibegraph::progress: stage="ufo_load" done=1 total=1
2026-10-19T19:32:44.000000Z  INFO compile{process=b b~ > Xd Xd~ t t~ g}: vibegraph::helas::eval::compile: x
2026-10-19T19:32:46.500000Z  INFO compile{process=b b~ > Xd Xd~ t t~ g}: vibegraph::helas::eval::compile: y
"""
VG_REFUSED = """\
 WARN enumerate: 'p p > wp+ > ta+ vt' selects diagrams by their s-channel propagators
ERROR failed to decompose into flavour groups: subprocesses u u > e+ e- u u and u u > ta+ ta- u u have different outgoing masses, so no single phase-space map covers the sum
"""
VG_PANIC = """\
thread 'main' (42501) panicked at vibegraph-lib/src/helas/eval/run.rs:1354:21:
index out of bounds: the len is 0 but the index is 0
"""


class GenLog(unittest.TestCase):
    def test_lo(self):
        lg = P.parse_gen_log(LO_LOG)
        self.assertEqual(lg["os"], "Red Hat Enterprise Linux release 8.10 (Ootpa)")
        self.assertEqual(lg["generation"]["n_diagrams"], 484)
        self.assertEqual(lg["generation"]["n_helas_calls"], 637)
        self.assertEqual(lg["diagram_gen"], [{"n_processes": 6, "n_diagrams": 484, "seconds": 2.773}])
        survey, test = lg["steps"]
        self.assertEqual((survey["step"], survey["wall_s"], survey["cpu_s"], survey["n_jobs"]),
                         ("survey", 3725.0, 13188.0, 67))
        self.assertEqual((test["step"], test["cpu_s"]), ("gridpack_test", 145975.0))
        self.assertEqual([(x["step"], x["sigma_pb"], x["nevents"]) for x in lg["xsecs"]],
                         [("survey", 1.013e7, 0), ("gridpack_test", 0.0, 1000)])

    def test_unconverged_survey_is_never_the_reference(self):
        self.assertIsNone(P.choose_reference(P.parse_gen_log(LO_LOG)["xsecs"]))

    def test_nlo_final_is_the_reference(self):
        lg = P.parse_gen_log(NLO_LOG)
        self.assertEqual(lg["generation"]["nlo_virtual_diagrams"], 92970)
        ref = P.choose_reference(lg["xsecs"])
        self.assertEqual((lg["xsecs"][ref]["step"], lg["xsecs"][ref]["sigma_pb"]), ("nlo_final", 0.7442))

    def test_durations(self):
        self.assertEqual(P.parse_duration("96 days,18h19m24s"), 96 * 86400 + 18 * 3600 + 19 * 60 + 24)
        self.assertEqual(P.parse_duration("4.4s"), 4.4)
        self.assertIsNone(P.parse_duration("n/a"))


class Cards(unittest.TestCase):
    def test_run_card_keeps_hidden_parameters(self):
        rc = P.parse_run_card(RUN_CARD)
        self.assertIn(("pdfwgt", "False", True), rc)
        self.assertIn(("event_norm", "sum", False), rc)
        self.assertEqual(P.default_for({"event_norm": "average"}, "event_norm", "sum"), ("average", 0))
        self.assertEqual(P.default_for({"pdfwgt": True}, "pdfwgt", "False"), ("True", 0))
        self.assertEqual(P.default_for({"lpp1": 1}, "lpp1", "1"), ("1", 1))

    def test_proc_card(self):
        pc = P.parse_proc_card("import model sm-ckm_no_b_mass\ndefine l+ = e+ mu+ ta+\n"
                               "generate p p > w+ w+ j j QED=4 QCD=0, w+ > l+ vl @ 1\n"
                               "add process p p > l+ l- j j / t t~ h $ z @2\n"
                               "generate p p > e+ e- [QCD]\nset run_card nevents 10\noutput x -nojpeg\n")
        self.assertEqual(pc["import"], ("sm", "ckm_no_b_mass", ""))
        self.assertEqual(pc["defines"], {"l+": "e+ mu+ ta+"})
        (k1, _, a), (k2, _, b), (_, _, c) = pc["processes"]
        self.assertEqual((k1, a["final_state"], a["n_final"], a["tag"], a["has_decay_chain"]),
                         ("generate", "w+ w+ j j", 4, 1, True))
        self.assertEqual(a["coupling_orders"], {"QED": "=4", "QCD": "=0"})
        self.assertEqual((k2, b["final_state"], b["excluded"], b["tag"]), ("add", "l+ l- j j", ["/", "t", "t~", "h", "$", "z"], 2))
        self.assertEqual((c["final_state"], c["nlo_bracket"]), ("e+ e-", "QCD"))
        self.assertEqual(pc["commands"], [("set", "set run_card nevents 10")])

    def test_param_diff(self):
        default = "Block MASS\n 6 1.730000e+02 # MT\nDECAY 6 1.5\n"
        card = "Block mass\n 6 1.725000e+02 # MT\nDECAY 6 1.50\n"
        self.assertEqual(P.param_diff(card, default), [("MASS", "6", "1.725000e+02", "1.730000e+02")])


class Ufo(unittest.TestCase):
    FILES = {
        "particles.py": "from object_library import all_particles, Particle\n"
                        "u = Particle(pdg_code=2, name='u', antiname='u~', spin=2, color=3)\n"
                        "u__tilde__ = u.anti()\n"
                        "n1 = Particle(pdg_code=1000022, name='n1', antiname='n1', spin=2, color=1)\n"
                        "S = Particle(pdg_code=9, name='S', antiname='S', spin=1, color=6)\n",
        "vertices.py": "V_1 = Vertex(name='V_1', particles=[P.u, P.u, P.S], color=['1'], lorentz=[L.FFS1], couplings={})\n"
                       "V_2 = Vertex(name='V_2', particles=[P.u__tilde__, P.u, P.S], color=['1'], lorentz=[], couplings={})\n",
        "couplings.py": "GC_1 = Coupling(name='GC_1', value='complex(0,1)*(a if b else c)', order={'QED':1})\n",
        "parameters.py": "a = Parameter(name='a', nature='external', type='real', value=1.0, lhablock='X', lhacode=[1])\n",
        "lorentz.py": "print 'a python 2 file'\n",
    }

    def test_stats(self):
        st = P.ufo_stats(self.FILES)
        self.assertEqual((st["n_particles"], st["n_vertices"], st["n_couplings"]), (3, 2, 1))
        self.assertEqual((st["spins"], st["colour_reps"]), ("1,2", "1,3,6"))
        self.assertEqual((st["has_majorana"], st["has_fnv_vertex"], st["has_conditional_expr"]), (1, 1, 1))
        self.assertEqual((st["n_external_parameters"], st["python_dialect"]), (1, "py2"))
        self.assertNotIn("n_lorentz", st)

    def test_identity_ignores_order(self):
        self.assertEqual(P.model_identity({"a.py": "1", "b.py": "2"}), P.model_identity({"b.py": "2", "a.py": "1"}))
        self.assertNotEqual(P.model_identity({"a.py": "1"}), P.model_identity({"a.py": "2"}))


class VgOutput(unittest.TestCase):
    def test_ok(self):
        r = P.parse_vg_output(VG_OK, VG_TRACE, 0, False)
        self.assertEqual((r["outcome"], r["sigma_pb"], r["err_pb"]), ("ok", 0.001952, 0.000196))
        self.assertEqual((r["n_evals"], r["chi2_dof"], r["n_channels"]), (183677, 0.0, 53))
        self.assertEqual((r["n_diagrams"], r["n_subprocesses"]), (8, 3))
        self.assertEqual(sorted((m["tag"], m["n_out"]) for m in r["multiplicities"]), [(0, 4), (1, 5)])
        self.assertEqual(r["vg_model_hash"][:8], "1c5615f0")
        self.assertAlmostEqual(r["stages"]["ufo_load"], 1.0)
        self.assertAlmostEqual(r["stages"]["compile"], 2.5)

    def test_refused(self):
        r = P.parse_vg_output(VG_REFUSED, "", 1, False)
        self.assertEqual((r["outcome"], r["refusal_class"]), ("refused", "flavour_mixed_mass"))
        self.assertTrue(r["error_text"].startswith("ERROR failed to decompose"))

    def test_panic_and_timeout(self):
        r = P.parse_vg_output(VG_PANIC, "", 101, False)
        self.assertEqual((r["outcome"], r["refusal_class"]), ("panic", None))
        self.assertIn("run.rs:1354", r["error_text"])
        self.assertEqual(P.parse_vg_output("", "", None, True)["outcome"], "timeout")


class Schema(unittest.TestCase):
    def test_fresh_database_matches_schema_version(self):
        import tempfile

        import gpdb
        with tempfile.TemporaryDirectory() as d:
            db = gpdb.connect(os.path.join(d, "t.sqlite"))
            self.assertEqual(db.execute("PRAGMA user_version").fetchone()[0], gpdb.SCHEMA_VERSION)
            db.close()
            gpdb.connect(os.path.join(d, "t.sqlite")).close()

    def test_one_reference_per_gridpack(self):
        import sqlite3
        import tempfile

        import gpdb
        with tempfile.TemporaryDirectory() as d:
            db = gpdb.connect(os.path.join(d, "t.sqlite"))
            gid = db.execute("INSERT INTO gridpack (path, label) VALUES ('p', 'l')").lastrowid
            db.execute("INSERT INTO mg_xsec VALUES (?, 'gen_log', 1, 'survey', 1.0, 0.1, NULL, 1)", (gid,))
            with self.assertRaises(sqlite3.IntegrityError):
                db.execute("INSERT INTO mg_xsec VALUES (?, 'gen_log', 2, 'survey', 2.0, 0.1, NULL, 1)", (gid,))
            with self.assertRaises(sqlite3.IntegrityError):  # an unconverged result cannot be the reference
                db.execute("INSERT INTO mg_xsec VALUES (?, 'gen_log', 3, 'survey', 1.0, 5.0, NULL, 1)", (gid,))


if __name__ == "__main__":
    unittest.main()
