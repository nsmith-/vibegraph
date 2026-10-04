// Shower an MLM-matched Les Houches file through Pythia 8 with kT-MLM jet
// matching and record, per Les Houches event, what the matching decided.
//
// The settings come from a Pythia command file, as MadGraph's own driver
// (Pythia's main164, which MadEvent's `pythia8` command runs) takes them. The
// matching hook is main164's: CombineMatchingInput hands a Madgraph-format
// LHEF with `JetMatching:scheme = 1` to JetMatchingMadgraph. It is subclassed
// here only to observe it: every decision is the base class's.
//
// Output, one tab-separated line per Les Houches event in file order:
//   index  idprup  proc_veto  match_veto  accepted  n_djr  djr0  djr1  djr2
// `proc_veto` is doVetoProcessLevel's decision (more hard partons than
// nJetMax), `match_veto` doVetoPartonLevelEarly's (the MLM veto; -1 where it
// was not reached), `accepted` whether Pythia::next() returned the event.
// `djr<i>` is JetMatchingMadgraph::getDJR()[i], the kT clustering scale (GeV)
// at which the showered hard system goes from i+1 to i jets (d_01, d_12,
// d_23), -1 where there are fewer steps.
//
// A summary block follows on stdout, prefixed `MLM `.
//
// Usage: mlm_match <settings.cmnd> <out.tsv>
#include "Pythia8/Pythia.h"
#include "Pythia8Plugins/JetMatching.h"

#include <cstdio>
#include <fstream>
#include <vector>

using namespace Pythia8;

namespace {

struct Record {
  int idprup = -1;
  int procVeto = 0;
  int matchVeto = -1;
  int accepted = 0;
  vector<double> djr;
};

class RecordingMatching : public JetMatchingMadgraph {
 public:
  vector<Record> records;

  bool doVetoProcessLevel(Event& process) override {
    Record r;
    r.idprup = infoPtr->codeSub();
    bool veto = JetMatchingMadgraph::doVetoProcessLevel(process);
    r.procVeto = veto ? 1 : 0;
    records.push_back(r);
    return veto;
  }

  bool doVetoPartonLevelEarly(const Event& event) override {
    bool veto = JetMatchingMadgraph::doVetoPartonLevelEarly(event);
    if (!records.empty()) {
      records.back().matchVeto = veto ? 1 : 0;
      records.back().djr = getDJR();
    }
    return veto;
  }
};

}  // namespace

int main(int argc, char** argv) {
  if (argc != 3) {
    cerr << "usage: " << argv[0] << " <settings.cmnd> <out.tsv>" << endl;
    return 2;
  }
  Pythia pythia;
  if (!pythia.readFile(argv[1])) {
    cerr << "MLM error: cannot read " << argv[1] << endl;
    return 2;
  }
  auto hook = make_shared<RecordingMatching>();
  if (pythia.flag("JetMatching:merge") || pythia.flag("JetMatching:setMad"))
    pythia.setUserHooksPtr(hook);
  if (!pythia.init()) {
    cout << "MLM init failed" << endl;
    return 1;
  }

  cout << "MLM version " << pythia.parm("Pythia:versionNumber") << endl;
  cout << "MLM header MGRunCard bytes " << pythia.info.header("MGRunCard").size()
       << endl;
  cout << "MLM settings merge " << pythia.flag("JetMatching:merge") << " setMad "
       << pythia.flag("JetMatching:setMad") << " qCut "
       << pythia.parm("JetMatching:qCut") << " nQmatch "
       << pythia.mode("JetMatching:nQmatch") << " clFact "
       << pythia.parm("JetMatching:clFact") << " nJetMax "
       << pythia.mode("JetMatching:nJetMax") << " etaJetMax "
       << pythia.parm("JetMatching:etaJetMax") << " coneRadius "
       << pythia.parm("JetMatching:coneRadius") << " doShowerKt "
       << pythia.flag("JetMatching:doShowerKt") << " doVeto "
       << pythia.flag("JetMatching:doVeto") << " scheme "
       << pythia.mode("JetMatching:scheme") << " exclusive "
       << pythia.mode("JetMatching:exclusive") << " productionScalesFromLHEF "
       << pythia.flag("Beams:setProductionScalesFromLHEF") << " seed "
       << pythia.mode("Random:seed") << endl;
  for (int i = 0; i < pythia.info.nProcessesLHEF(); ++i)
    cout << "MLM sigmaLHEF " << i << " " << pythia.info.sigmaLHEF(i) << endl;

  int nFailed = 0;
  while (true) {
    if (!pythia.next()) {
      if (pythia.info.atEndOfFile()) break;
      ++nFailed;
      continue;
    }
    if (!hook->records.empty()) hook->records.back().accepted = 1;
  }

  std::ofstream out(argv[2]);
  out.precision(10);
  for (size_t i = 0; i < hook->records.size(); ++i) {
    const Record& r = hook->records[i];
    out << i << '\t' << r.idprup << '\t' << r.procVeto << '\t' << r.matchVeto
        << '\t' << r.accepted << '\t' << r.djr.size();
    for (int k = 0; k < 3; ++k)
      out << '\t' << (k < int(r.djr.size()) ? r.djr[k] : -1.0);
    out << '\n';
  }
  out.close();

  cout << "MLM records " << hook->records.size() << " failed_next " << nFailed
       << " tried " << pythia.info.nTried() << " selected "
       << pythia.info.nSelected() << " accepted " << pythia.info.nAccepted()
       << endl;
  cout << "MLM sigmaGen_pb " << pythia.info.sigmaGen() * 1e9 << " +- "
       << pythia.info.sigmaErr() * 1e9 << endl;
  pythia.stat();
  return 0;
}
