//! What happens to every construct MadGraph's reweight module accepts in a
//! `reweight_card.dat` (`madgraph/interface/reweight_interface.py`, with the
//! parameter edits of a `launch` block read by the edit-card interface,
//! `AskforEditCard` in `madgraph/interface/common_run_interface.py`).
//!
//! The classes and the test are the param card's
//! ([`crate::ufo::card_classes`]): each row's example is parsed and resolved on
//! the same test model, and the outcome held to the class. The launches compare
//! as resolved: their ids, descriptions and parameter values.

use crate::ufo::card_classes::Card::Text;
use crate::ufo::card_classes::Construct;
use crate::ufo::card_classes::ConstructClass::{Consumed, IgnoredBenign, Refused};
use crate::ufo::card_classes::Expect::{Differs, Error, PinnedBy, Same};

/// Every construct of a reweight card.
pub(crate) const REWEIGHT_CARD: &[Construct] = &[
    Construct {
        name: "comment line, blank line or trailing comment",
        madgraph: "skipped by the command reader (extended_cmd.py import_command_file)",
        class: IgnoredBenign("no command"),
        example: Text("# a card\n\nlaunch --rwgt_name=a # the first\n set ymb 5.0 # ymb\n"),
        expect: Same(Text("launch --rwgt_name=a\n set ymb 5.0\n")),
    },
    Construct {
        name: "`launch`",
        madgraph: "one hypothesis, whose weight every event receives (do_launch, L497)",
        class: Consumed("reweight::resolve, one Launch per block"),
        example: Text("launch\n set ymb 5.0\nlaunch\n set ymb 6.0\n"),
        expect: Differs(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`launch` with no parameter change",
        madgraph: "a hypothesis at the card's own parameters, weight equal to the event's",
        class: Consumed("reweight::resolve, a Launch with no values"),
        example: Text("launch\n set ymb 5.0\nlaunch\n"),
        expect: Differs(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`launch --rwgt_name=<id>`",
        madgraph: "the weight's id (check_launch L462-L468, do_launch L502)",
        class: Consumed("Launch::id, the <wgt id> in the event file"),
        example: Text("launch --rwgt_name=heavy\n set ymb 5.0\n"),
        expect: Differs(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`launch --rwgt_info=<text>`",
        madgraph: "the weight's description in <initrwgt> (do_launch L504, L921)",
        class: Consumed("Launch::info"),
        example: Text("launch --rwgt_info=a heavier b\n set ymb 5.0\n"),
        expect: Differs(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "any other `launch` option",
        madgraph: "stored and ignored (check_launch L462-L468), or `--keep_card`, which \
                   reuses the previous hypothesis's card (handle_param_card L815)",
        class: Refused("`--keep_card` changes the hypothesis; a typo'd option would be inert"),
        example: Text("launch --keep_card\n set ymb 5.0\n"),
        expect: Error("unknown launch option"),
    },
    Construct {
        name: "a `--rwgt_name` used twice",
        madgraph: "the later weight replaces the earlier, with a warning (L889-L896)",
        class: Refused("one of the two hypotheses would be lost"),
        example: Text("launch --rwgt_name=a\n set ymb 5.0\nlaunch --rwgt_name=a\n set ymb 6.0\n"),
        expect: Error("both named"),
    },
    Construct {
        name: "a `--rwgt_name` outside letters, digits, `_`, `-` and `.`",
        madgraph: "written into the event file as given",
        class: Refused("it cannot be written as an XML attribute safely"),
        example: Text("launch --rwgt_name=a<b\n set ymb 5.0\n"),
        expect: Error("cannot name a weight"),
    },
    Construct {
        name: "`set [param_card] <block> <code…> <value>`",
        madgraph: "the param card's entry (AskforEditCard.do_set, common_run_interface.py \
                   L5868)",
        class: Consumed("reweight::resolve, by LHA address"),
        example: Text("launch\n set param_card yukawa 5 5.0\n"),
        expect: Differs(Text("launch\n")),
    },
    Construct {
        name: "`set <name> <value>`",
        madgraph: "the parameter named in the default card's comments (do_set, \
                   pname2block from analyze_param_card)",
        class: Consumed("reweight::resolve, by external parameter name, case folded"),
        example: Text("launch\n set YMB 5.0\n"),
        expect: Differs(Text("launch\n")),
    },
    Construct {
        name: "`set` with or without the `param_card` keyword, a name for an address",
        madgraph: "the same entry of the param card",
        class: IgnoredBenign("the spelling of one change"),
        example: Text("launch --rwgt_info=x\n set param_card yukawa 5 5.0\n"),
        expect: Same(Text("launch --rwgt_info=x\n set ymb 5.0\n")),
    },
    Construct {
        name: "one parameter set twice in a launch",
        madgraph: "the later edit wins",
        class: IgnoredBenign("the later value wins here too"),
        example: Text("launch --rwgt_info=x\n set ymb 4.0\n set ymb 5.0\n"),
        expect: Same(Text("launch --rwgt_info=x\n set ymb 5.0\n")),
    },
    Construct {
        name: "a value with a Fortran `d` exponent",
        madgraph: "read as a number",
        class: IgnoredBenign("the same number"),
        example: Text("launch --rwgt_info=x\n set ymb 5.0d0\n"),
        expect: Same(Text("launch --rwgt_info=x\n set ymb 5.0\n")),
    },
    Construct {
        name: "a value that is not a number (`Auto`, `default`)",
        madgraph: "MadWidth's width, or the default card's value",
        class: Refused("neither is computed or read here"),
        example: Text("launch\n set decay 25 Auto\n"),
        expect: Error("is not a number"),
    },
    Construct {
        name: "a `scan:` value",
        madgraph: "one hypothesis per value (handle_param_card, ParamCardIterator, \
                   L825-L845)",
        class: Refused("one launch here is one hypothesis"),
        example: Text("launch\n set ymb scan:[4, 5]\n"),
        expect: Error("scan"),
    },
    Construct {
        name: "`set <name> = <value>` or `set <name>=<value>`",
        madgraph: "the `=` is dropped (do_set L5878-L5882)",
        class: Refused("not read here"),
        example: Text("launch\n set ymb = 5.0\n"),
        expect: Error("integer LHA code"),
    },
    Construct {
        name: "a parameter the model does not have",
        madgraph: "a warning, and the line is skipped",
        class: Refused("the hypothesis would not be the one written"),
        example: Text("launch\n set nosuch 1.0\n"),
        expect: Error("no external parameter"),
    },
    Construct {
        name: "an internal parameter",
        madgraph: "a computed entry, overwritten (update_dependent)",
        class: Refused("the hypothesis would not be the one written"),
        example: Text("launch\n set MW 8.0e1\n"),
        expect: Error("internal parameter"),
    },
    Construct {
        name: "a parameter the restriction fixed to zero",
        madgraph: "removed from the model the reweighting library is built from",
        class: Refused("its vertices are pruned"),
        example: Text("launch\n set MC 1.27\n"),
        expect: Error("fixed to zero"),
    },
    Construct {
        name: "a parameter the restriction fixed to one, set elsewhere",
        madgraph: "removed from the model the reweighting library is built from",
        class: Refused("MadGraph's weight would not move with it"),
        example: Text("launch\n set WT 2.0\n"),
        expect: Error("fixed to one"),
    },
    Construct {
        name: "one parameter of a merged group moved alone",
        madgraph: "the others are evaluated as the group's first",
        class: Refused("MadGraph's weight would not be this hypothesis's"),
        example: Text("launch\n set ymt 1.5e2\n"),
        expect: Error("merged"),
    },
    Construct {
        name: "a parameter that moves the strong coupling",
        madgraph: "reweighted, with the event's own aS",
        class: Refused("each event's aS comes from the run's scale choice"),
        example: Text("launch\n set aS 0.12\n"),
        expect: Error("strong coupling"),
    },
    Construct {
        name: "the mass of an external particle",
        madgraph: "the event's momenta are moved to the new mass shell \
                   (change_kinematics, L1105)",
        class: Refused("the events here keep their momenta"),
        example: Text(""),
        expect: PinnedBy("engine::tests::a_hypothesis_moving_an_external_mass_is_refused"),
    },
    Construct {
        name: "`set` before the first `launch`",
        madgraph: "a `launch` is inserted before it, with a warning (do_set, L1050)",
        class: Refused("a card that relies on it is written wrongly"),
        example: Text("set ymb 5.0\nlaunch\n"),
        expect: Error("before the first"),
    },
    Construct {
        name: "a param card path in a launch block",
        madgraph: "the hypothesis is that card (AskforEditCard.default, \
                   common_run_interface.py L7217)",
        class: Refused("not read here"),
        example: Text("launch\n ./param_card.dat\n"),
        expect: Error("param card in place"),
    },
    Construct {
        name: "`change helicity False`",
        madgraph: "the helicity-summed ratio (do_change L402, calculate_matrix_element \
                   L1597-L1601)",
        class: IgnoredBenign("the helicity-summed ratio is the one generate writes"),
        example: Text("change helicity False\nlaunch\n set ymb 5.0\n"),
        expect: Same(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`change helicity True`",
        madgraph: "the ratio in the event's own helicity, MadGraph's default (L112)",
        class: Refused("only the helicity-summed ratio is written"),
        example: Text("change helicity True\nlaunch\n"),
        expect: Error("helicity"),
    },
    Construct {
        name: "`change output default`",
        madgraph: "weights added to the events (L399-L401), MadGraph's default",
        class: IgnoredBenign("the only output generate writes"),
        example: Text("change output default\nlaunch\n set ymb 5.0\n"),
        expect: Same(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`change output 2.0` or `unweight`",
        madgraph: "one event file per hypothesis, weighted or unweighted (L399-L401, \
                   L722-L738)",
        class: Refused("not written here"),
        example: Text("change output unweight\nlaunch\n"),
        expect: Error("output"),
    },
    Construct {
        name: "`change rwgt_dir <path>`",
        madgraph: "where MadGraph builds its reweighting library (L419-L423)",
        class: IgnoredBenign("a working directory of MadGraph's; nothing is built there here"),
        example: Text("change rwgt_dir ./rw\nlaunch\n set ymb 5.0\n"),
        expect: Same(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`change multicore <n>`",
        madgraph: "read by the launching interface; \"no direct impact on this class\" \
                   (L437-L440)",
        class: IgnoredBenign("how MadGraph parallelises, not what it computes"),
        example: Text("change multicore 4\nlaunch\n set ymb 5.0\n"),
        expect: Same(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "`change model` or `change process`",
        madgraph: "the hypotheses are evaluated in another model or process (L376-L394)",
        class: Refused("reweighting evaluates the generated process in the generated model"),
        example: Text("change model sm-full\nlaunch\n"),
        expect: Error("model"),
    },
    Construct {
        name: "`change mode`",
        madgraph: "LO, NLO or NLO_tree reweighting (L404-L418)",
        class: Refused("only the leading-order ratio is computed"),
        example: Text("change mode LO\nlaunch\n"),
        expect: Error("mode"),
    },
    Construct {
        name: "`change systematics`",
        madgraph: "systematics on each hypothesis, switching the output to 2.0 \
                   (L424-L434)",
        class: Refused("not written here"),
        example: Text("change systematics True\nlaunch\n"),
        expect: Error("systematics"),
    },
    Construct {
        name: "`change keep_ordering`, `use_eventid` or `allow_missing_finalstate`",
        madgraph: "how an event is matched to a matrix element (L382-L385)",
        class: Refused("the match here is the generated subprocess's"),
        example: Text("change use_eventid True\nlaunch\n"),
        expect: Error("use_eventid"),
    },
    Construct {
        name: "`change boost`",
        madgraph: "events boosted before evaluation (L395)",
        class: Refused("the events are evaluated as written"),
        example: Text("change boost True\nlaunch\n"),
        expect: Error("boost"),
    },
    Construct {
        name: "`change tree_path` or `virtual_path`",
        madgraph: "a matrix-element library of the user's (L397-L398)",
        class: Refused("the matrix element is this crate's"),
        example: Text("change tree_path ./me\nlaunch\n"),
        expect: Error("tree_path"),
    },
    Construct {
        name: "`change soft_threshold`",
        madgraph: "a threshold on the reweighting of soft events (L435-L436)",
        class: Refused("no threshold is applied here"),
        example: Text("change soft_threshold 1e-3\nlaunch\n"),
        expect: Error("soft_threshold"),
    },
    Construct {
        name: "`change identical_particle_in_prod_and_decay`",
        madgraph: "how a decayed event with identical particles is matched (L441-L444)",
        class: Refused("not implemented here"),
        example: Text("change identical_particle_in_prod_and_decay max\nlaunch\n"),
        expect: Error("identical_particle_in_prod_and_decay"),
    },
    Construct {
        name: "`change include_sudakov`",
        madgraph: "electroweak Sudakov factors in the weight (L445-L448)",
        class: Refused("not implemented here"),
        example: Text("change include_sudakov True\nlaunch\n"),
        expect: Error("include_sudakov"),
    },
    Construct {
        name: "`change` with another option",
        madgraph: "\"unknown option! Discard line\" (L449-L450)",
        class: Refused("a typo'd option would be inert in both"),
        example: Text("change helicty False\nlaunch\n"),
        expect: Error("helicty"),
    },
    Construct {
        name: "`done`, `exit` or `quit` with nothing after it",
        madgraph: "ends the parameter edit, or the module",
        class: IgnoredBenign("nothing follows it"),
        example: Text("launch\n set ymb 5.0\ndone\n# end\n"),
        expect: Same(Text("launch\n set ymb 5.0\n")),
    },
    Construct {
        name: "a command after `done`, `exit` or `quit`",
        madgraph: "still run: the card is read to its end (extended_cmd.py \
                   import_command_file)",
        class: Refused("its hypotheses would be dropped"),
        example: Text("launch\n set ymb 5.0\ndone\nlaunch\n set ymb 6.0\n"),
        expect: Error("after `done`"),
    },
    Construct {
        name: "any other command (`compute_widths`, `import`, `update`, `decay`, …)",
        madgraph: "the reweight module's or the edit card's own commands",
        class: Refused("none is implemented here"),
        example: Text("launch\n compute_widths 25\n"),
        expect: Error("unknown directive"),
    },
    Construct {
        name: "a card with no `launch`",
        madgraph: "no hypothesis, no weight",
        class: Refused("a card that asks for nothing is a mistake"),
        example: Text("# nothing\n"),
        expect: Error("no `launch`"),
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reweight::card::ReweightCard;
    use crate::reweight::{resolve, Launch};
    use crate::ufo::card_classes::tests::{assert_rows, param_model};

    fn launches(text: &str) -> Result<Vec<Launch>, String> {
        let card: ReweightCard = text.parse().map_err(|e| format!("{e}"))?;
        resolve(&card, &param_model()).map_err(|e| e.to_string())
    }

    #[test]
    fn every_reweight_card_construct_behaves_as_classified() {
        assert_rows(REWEIGHT_CARD, "", launches);
    }
}
