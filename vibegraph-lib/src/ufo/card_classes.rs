//! What happens to every construct MadGraph's SLHA reader accepts, in a run's
//! param card ([`super::bind::bind_card`]) and in a restriction card
//! ([`super::ParsedModel::apply_restriction`]).
//!
//! Both cards go through MadGraph's one reader, `check_param_card.ParamCard`
//! (`models/check_param_card.py`), and this crate's one,
//! [`ParamCard`](super::slha::ParamCard); they differ in what reads the result.
//! A run's card is MadGraph's `write_inc_file` (the values the Fortran reads,
//! by block and code from the model's `ident_card`), its `update_dependent`, and
//! the `param_card_rule` check of the restriction's fixed values. A restriction
//! card is `model_reader.set_parameters_and_couplings` and
//! `RestrictModel.restrict_model` (`models/import_ufo.py`).
//!
//! The rule is the run card's ([`crate::runcard`]'s field classes): a construct
//! MadGraph acts on and this crate silently drops is a wrong answer, so each
//! construct is consumed, inert with an argument why, or refused with an error
//! naming it. The tables are specifications: the tests below run each row's
//! example through the reader and its consumer and hold the outcome to the
//! class.

/// What a card construct does here.
pub(crate) enum ConstructClass {
    /// Read, and it moves the run. The string names the consumer.
    Consumed(&'static str),
    /// Accepted and unable to reach the cross section, a weight or the event
    /// record. The string argues that.
    IgnoredBenign(&'static str),
    /// Refused with an error naming the construct. The string says why it
    /// could bite.
    Refused(&'static str),
}

/// A card text: a literal, or edits to a base card (each `(from, to)` replacing
/// the one occurrence of `from`; an empty `from` appends `to`).
pub(crate) enum Card {
    Text(&'static str),
    Edits(&'static [(&'static str, &'static str)]),
}

/// What a row's example must do, which the test holds to its class.
pub(crate) enum Expect {
    /// Accepted, and the result is the baseline's (an `IgnoredBenign` row).
    Same(Card),
    /// Accepted, and the result differs from the baseline's (`Consumed`).
    Differs(Card),
    /// Refused, with this text in the error (`Refused`).
    Error(&'static str),
    /// Not a card text: the named test pins the behaviour.
    PinnedBy(&'static str),
}

/// One construct of a card.
pub(crate) struct Construct {
    pub(crate) name: &'static str,
    /// What MadGraph's reader does with it, citing the line at the pin.
    pub(crate) madgraph: &'static str,
    pub(crate) class: ConstructClass,
    pub(crate) example: Card,
    pub(crate) expect: Expect,
}

use Card::{Edits, Text};
use ConstructClass::{Consumed, IgnoredBenign, Refused};
use Expect::{Differs, Error, PinnedBy, Same};

/// Every construct of a run's param card. The examples bind to the test model
/// of [`tests::param_model`]: the default-restricted SM with `MT` and `ymt`
/// tied as MadGraph's restriction ties equal values, and `WT` held at one.
pub(crate) const PARAM_CARD: &[Construct] = &[
    Construct {
        name: "comment line or blank line",
        madgraph: "skipped (ParamCard.read, L353)",
        class: IgnoredBenign(
            "no value: a comment names a parameter only for analyze_param_card, which \
             reads the comments of MadGraph's own default card for the edit-card `set \
             <name>` shortcut, never a run's values",
        ),
        example: Text("# a card\n\nBlock MASS\n# the Z\n 23 9.2e1\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "trailing comment",
        madgraph: "split off the entry (Parameter.load_str, L65)",
        class: IgnoredBenign(
            "the entry is read by block and code (write_inc_file, L663), so a comment \
             naming another parameter moves nothing",
        ),
        example: Text("Block MASS\n 23 9.2e1 # MT\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "block and keyword case",
        madgraph: "every line lower-cased before it is read (L355)",
        class: IgnoredBenign("block names and DECAY compare without case here as there"),
        example: Text("bLoCk mass\n 23 9.2e1\ndecay 23 2.5\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\nDECAY 23 2.5\n")),
    },
    Construct {
        name: "entry: `<code…> <value>` at an external parameter",
        madgraph: "a Parameter of the block (L397-L405), written to the Fortran \
                   (write_inc_file, L663)",
        class: Consumed("EvaluatedModel::from_model_card, through ParameterSet::bound_value"),
        example: Text("Block MASS\n 23 9.2e1\nBlock SMINPUTS\n 1 1.3e2\n"),
        expect: Differs(Text("")),
    },
    Construct {
        name: "`DECAY <pdg> <width>` at an external width",
        madgraph: "a Parameter of block decay (L362-L372)",
        class: Consumed("EvaluatedModel::from_model_card, as block DECAY code [pdg]"),
        example: Text("DECAY 23 2.5\n"),
        expect: Differs(Text("")),
    },
    Construct {
        name: "`BLOCK <name> Q= <scale>`",
        madgraph: "the scale is stored on the block (Block.load_str, L272-L279) and read \
                   only as mdl__<block>__scale for a model with running elements \
                   (export_v4.py L2812-L2823, write_inc_file L669-L707)",
        class: IgnoredBenign(
            "the loader reads no running.py, so no parameter here depends on a block's \
             scale; every value of the block is read as without it",
        ),
        example: Text("Block MASS Q= 9.1e1\n 23 9.2e1\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "`BLOCK QNUMBERS <pdg>` and its entries",
        madgraph: "kept as block `qnumbers <pdg>` (L275), left out of the parameters \
                   (model_reader.py L102)",
        class: IgnoredBenign(
            "quantum numbers are not parameters; the event file's <slha> header writes \
             the model's own QNUMBERS (lhef::write::slha_qnumbers), as MadGraph's \
             write_param_card derives them from the model",
        ),
        example: Text(
            "Block QNUMBERS 9000005 # x\n 1 0\n 2 1\n 3 1\n 4 0\nBlock MASS\n 23 9.2e1\n",
        ),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "a `(block, code)` repeated at the same value",
        madgraph: "the repeat is dropped (Block.append, L240-L244)",
        class: IgnoredBenign("the value is the one already read"),
        example: Text("Block MASS\n 23 9.2e1\n 23 92.0\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "an entry for a mass or width the model computes",
        madgraph: "overwritten with the computed value (update_dependent, L463)",
        class: IgnoredBenign(
            "the Fortran computes it from the external parameters, and so does the \
             evaluation here; a value off the computed one is reported \
             (BoundCard::ignored)",
        ),
        example: Text("Block MASS\n 24 8.0e1\n 1 5.0e-3\nDECAY 22 1.0\n"),
        expect: Same(Text("")),
    },
    Construct {
        name: "a line before the first block",
        madgraph: "skipped (cur_block is None, L380-L381)",
        class: IgnoredBenign("MadGraph's run never reads it either"),
        example: Text(" 23 9.5e1\nBlock MASS\n 23 9.2e1\n"),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "a `(block, code)` repeated at another value",
        madgraph: "InvalidParamCard \"is already define\" (Block.append, L241-L243)",
        class: Refused("MadGraph refuses it"),
        example: Text("Block MASS\n 23 9.2e1\n 23 9.5e1\n"),
        expect: Error("on line 2"),
    },
    Construct {
        name: "a block opened twice",
        madgraph: "the second replaces the first, dropping its entries (ParamCard.append, \
                   L770)",
        class: Refused("the entries of the first would be dropped silently"),
        example: Text("Block MASS\n 23 9.2e1\nBlock MASS\n 25 1.3e2\n"),
        expect: Error("opened again"),
    },
    Construct {
        name: "an entry the model has no parameter for (lone values and unknown blocks included)",
        madgraph: "never read: write_inc_file reads only the model's ident_card (L655)",
        class: Refused("a mistyped block or code would leave the parameter meant at its default"),
        example: Text("Block ALPHA\n -1.1e-1\n"),
        expect: Error("model has no parameter"),
    },
    Construct {
        name: "an entry whose code is not integers",
        madgraph: "the line is dropped with a warning (L398-L403)",
        class: Refused("a dropped line is a value the card meant and the run lacks"),
        example: Text("Block MASS\n mz 9.2e1\n"),
        expect: Error("integer code"),
    },
    Construct {
        name: "a value that is not a number (Fortran `d` exponents included)",
        madgraph: "kept as a string (Parameter.load_str, L87-L91)",
        class: Refused("no value can be read from it"),
        example: Text("Block MASS\n 23 9.2d1\n"),
        expect: Error("expected float"),
    },
    Construct {
        name: "a non-finite value",
        madgraph: "read by Python's float (L88)",
        class: Refused("no amplitude is finite at it"),
        example: Text("Block MASS\n 23 inf\n"),
        expect: Error("not a finite number"),
    },
    Construct {
        name: "a `scan:` value",
        madgraph: "a parameter scan, one run per value (ParamCardIterator, L931)",
        class: Refused("one run here is one point"),
        example: Text("Block MASS\n 23 scan:[91, 92]\n"),
        expect: Error("scan"),
    },
    Construct {
        name: "`DECAY <pdg> Auto`",
        madgraph: "MadWidth computes the width at launch",
        class: Refused("no width is computed here"),
        example: Text("DECAY 23 Auto\n"),
        expect: Error("Auto"),
    },
    Construct {
        name: "a `DECAY` line with fields after the width",
        madgraph: "the width is read as the string of everything after the code \
                   (Parameter.load_str, L76-L86)",
        class: Refused("MadGraph's width is then no number"),
        example: Text("DECAY 23 2.5 1.0\n"),
        expect: Error("after the width"),
    },
    Construct {
        name: "a decay table (branching-ratio rows after `DECAY`)",
        madgraph: "kept as decay_table_<pdg> (L383-L395) and written with the card into \
                   the event file's <slha> header (banner.py add, L457), where a shower \
                   reads the decays of the states it treats",
        class: Consumed(
            "EvaluatedModel::decay_tables, recorded in the artifact and written below the \
             particle's DECAY line of the <slha> header (lhef::write::slha_block)",
        ),
        example: Text("DECAY 23 2.5\n 1.0 2 11 -11\n"),
        expect: Differs(Text("DECAY 23 2.5\n")),
    },
    Construct {
        name: "a decay table that does not sum to one or names a particle the model lacks",
        madgraph: "read as a number and a list of codes (Parameter.load_decay, L100-L120); \
                   nothing checks the sum or the codes, and the header carries the rows as \
                   written",
        class: Consumed("carried as written, as any decay table"),
        example: Text("DECAY 23 2.5\n 0.7 2 11 -11\n 0.1 2 9999999 -11\n"),
        expect: Differs(Text("DECAY 23 2.5\n")),
    },
    Construct {
        name: "a decay channel repeated at the same branching ratio",
        madgraph: "the repeat is dropped (Block.append, L240-L244; the daughters are \
                   sorted, L116, so their order does not matter)",
        class: IgnoredBenign("the channel is carried once, as MadGraph keeps it"),
        example: Text("DECAY 23 2.5\n 1.0 2 11 -11\n 1.0 2 -11 11\n"),
        expect: Same(Text("DECAY 23 2.5\n 1.0 2 11 -11\n")),
    },
    Construct {
        name: "a decay channel given two branching ratios",
        madgraph: "the first is kept: the InvalidParamCard of Block.append is caught \
                   (L391-L395)",
        class: Refused("the header MadGraph writes carries both rows, and a shower reads both"),
        example: Text("DECAY 23 2.5\n 0.9 2 11 -11\n 0.8 2 -11 11\n"),
        expect: Error("on line 2"),
    },
    Construct {
        name: "a decay-table row whose NDA is not its number of daughters",
        madgraph: "NDA is dropped and the daughters counted (Parameter.load_decay, L115-L117)",
        class: Refused("a shower reading the header reads NDA"),
        example: Text("DECAY 23 2.5\n 1.0 3 11 -11\n"),
        expect: Error("NDA = 3"),
    },
    Construct {
        name: "a decay-table row with no daughter or a field that is not a number",
        madgraph: "a row with no daughter is a channel of none; a field that is not a \
                   number raises (int or float in Parameter.load_decay, L115-L119)",
        class: Refused("no shower decays a particle into nothing"),
        example: Text("DECAY 23 2.5\n 1.0 0\n"),
        expect: Error("no daughter"),
    },
    Construct {
        name: "a decay table below the DECAY line of a negative code",
        madgraph: "kept as decay_table_<code> under the antiparticle's code",
        class: Refused(
            "the header lists each particle's DECAY line once, under its positive code, \
             so the rows have no line to sit under",
        ),
        example: Text("DECAY -24 2.085\n 1.0 2 11 -12\n"),
        expect: Error("positive PDG code"),
    },
    Construct {
        name: "an `XSECTION` block",
        madgraph: "its rows are kept unparsed up to the next BLOCK or DECAY \
                   (not_parsed_entry, L374-L377) and written back with the card",
        class: IgnoredBenign(
            "SLHA cross-section information for other tools: no parameter is read from \
             it, and the reader resumes at the next block as MadGraph's does",
        ),
        example: Text(
            "Block MASS\n 23 9.2e1\nXSECTION 13000 2212 2212 2 6 -6\n 0 0 0 0 0 0 9.0e-1 0 0\n",
        ),
        expect: Same(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "a negative mass",
        madgraph: "the width at the same code is negated with it in the Fortran \
                   (write_inc_file, L692-L695); the negated width is pinned by \
                   bind.rs's a_negative_mass_negates_its_width_and_rebinds",
        class: Consumed(
            "EvaluatedModel's width sign: the external width at the same code is \
             evaluated negated, as MadGraph's Fortran reads it",
        ),
        example: Text("Block MASS\n 23 -9.2e1\n"),
        expect: Differs(Text("Block MASS\n 23 9.2e1\n")),
    },
    Construct {
        name: "a value for a parameter the restriction fixed to zero",
        madgraph: "the parameter and its vertices are gone (fix_parameter_values)",
        class: Refused("the pruned vertices cannot be restored"),
        example: Text("Block MASS\n 4 1.27\n"),
        expect: Error("fixes it to zero"),
    },
    Construct {
        name: "a value other than one for a parameter the restriction fixed to one",
        madgraph: "refused by the process's param_card_rule (check_param_card, L1308)",
        class: Refused("MadGraph refuses it"),
        example: Text("DECAY 6 2.0\n"),
        expect: Error("fixes it to one"),
    },
    Construct {
        name: "one parameter of a merged group moved alone",
        madgraph: "the others are evaluated as the group's first (merge_iden_parameters)",
        class: Refused("the card's value for the member would not be MadGraph's"),
        example: Text("Block YUKAWA\n 6 1.5e2\n"),
        expect: Error("merged them"),
    },
];

/// Every construct of a restriction card. The examples are edits to the SM's
/// `restrict_default.dat`, restricting the interned SM.
pub(crate) const RESTRICT_CARD: &[Construct] = &[
    Construct {
        name: "comment line, blank line or trailing comment",
        madgraph: "skipped or split off (ParamCard.read L353, Parameter.load_str L65)",
        class: IgnoredBenign(
            "the restriction reads values by block and code (model_reader.py L177-L181)",
        ),
        example: Edits(&[("", "\n# a comment\n\n"), ("# MT \n", "# MZ \n")]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "entry at an external parameter",
        madgraph: "the parameter's value for the restriction and the model's default \
                   (model_reader.py L177-L188)",
        class: Consumed("ParameterSet::apply_restrict, as the restricted default"),
        example: Edits(&[("    6 1.730000e+02 # MT", "    6 1.750000e+02 # MT")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "an entry at zero",
        madgraph: "the parameter is fixed to zero (detect_special_parameters, \
                   import_ufo.py L2615) and the couplings it zeroes are pruned \
                   (detect_identical_couplings, L2525)",
        class: Consumed("ParameterSet::zeros and ParsedModel::apply_restriction's pruning"),
        example: Edits(&[("    5 4.700000e+00 # MB", "    5 0.000000e+00 # MB")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "an entry at exactly one",
        madgraph: "the parameter is fixed to one (detect_special_parameters, L2615)",
        class: Consumed("ParameterSet::ones, which bind_card and reweighting hold"),
        example: Edits(&[("   15 1.777000e+00 # MTA", "   15 1.000000e+00 # MTA")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "equal values (up to sign) in one block",
        madgraph: "the parameters are merged (detect_identical_parameters, L2709)",
        class: Consumed("ParameterSet::merged, which bind_card and reweighting hold"),
        example: Edits(&[("    5 4.700000e+00 # ymb", "    5 1.730000e+02 # ymb")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "`DECAY <pdg> <width>` at an external width",
        madgraph: "the width's value (model_reader.py L177-L188); widths are never \
                   merged (L2727)",
        class: Consumed("ParameterSet::apply_restrict, as the restricted default"),
        example: Edits(&[("DECAY   6 1.491500E+00", "DECAY   6 1.5E+00")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "a negative mass",
        madgraph: "read as the restricted default like any value; the Fortran negates \
                   the width at the same code (write_inc_file, L692-L695)",
        class: Consumed(
            "ParameterSet::apply_restrict, as the restricted default; the restriction \
             reads it as MadGraph's does, and the evaluation negates the width beside it",
        ),
        example: Edits(&[("   25 1.250000e+02 # MH", "   25 -1.250000e+02 # MH")]),
        expect: Differs(Edits(&[])),
    },
    Construct {
        name: "`BLOCK <name> Q= <scale>`",
        madgraph: "the scale is stored on the block (Block.load_str, L272-L279)",
        class: IgnoredBenign("the restriction reads values only, never a block's scale"),
        example: Edits(&[("Block MASS ", "Block MASS Q= 9.1e1")]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "`BLOCK QNUMBERS <pdg>` and its entries",
        madgraph: "left out of the blocks the restriction reads (model_reader.py L102)",
        class: IgnoredBenign("quantum numbers are not parameters"),
        example: Edits(&[("", "\nBlock QNUMBERS 9000005\n 1 0\n 2 1\n")]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "an entry or block the model has no parameter for",
        madgraph: "never read (model_reader.py L177-L181 reads the model's parameters); \
                   an unknown block only warns (L127)",
        class: IgnoredBenign(
            "MadGraph's restriction never reads it either; a model's own cards carry \
             such entries for its computed masses and widths",
        ),
        example: Edits(&[("", "\nBlock NOSUCH\n 1 2.0\nBlock FRALPHA\n 1 0.5\n")]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "a decay table",
        madgraph: "kept as decay_table_<pdg> and left out of the blocks read \
                   (model_reader.py L102)",
        class: IgnoredBenign(
            "the restriction reads no branching ratio, and MadGraph's process param \
             card is written from the model, not copied from the restriction card",
        ),
        example: Edits(&[(
            "DECAY  25 6.38233934e-03",
            "DECAY  25 6.38233934e-03\n 1.0 2 5 -5",
        )]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "an `XSECTION` block",
        madgraph: "its rows are kept unparsed up to the next BLOCK or DECAY (L374-L377)",
        class: IgnoredBenign("no parameter is read from it"),
        example: Edits(&[(
            "",
            "\nXSECTION 13000 2212 2212 2 6 -6\n 0 0 0 0 0 0 9.0e-1 0 0\n",
        )]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "a line before the first block",
        madgraph: "skipped (L380-L381)",
        class: IgnoredBenign("MadGraph's restriction never reads it either"),
        example: Edits(&[("## PARAM_CARD", " 23 9.5e1\n## PARAM_CARD")]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "a `(block, code)` repeated at the same value",
        madgraph: "the repeat is dropped (Block.append, L240-L244)",
        class: IgnoredBenign("the value is the one already read"),
        example: Edits(&[(
            "   25 1.250000e+02 # MH",
            "   25 1.250000e+02 # MH\n   25 125.0",
        )]),
        expect: Same(Edits(&[])),
    },
    Construct {
        name: "a `(block, code)` repeated at another value",
        madgraph: "InvalidParamCard \"is already define\" (Block.append, L241-L243)",
        class: Refused("MadGraph refuses it"),
        example: Edits(&[(
            "   25 1.250000e+02 # MH",
            "   25 1.250000e+02 # MH\n   25 126.0",
        )]),
        expect: Error("is set to"),
    },
    Construct {
        name: "a block opened twice",
        madgraph: "the second replaces the first (ParamCard.append, L770)",
        class: Refused("the entries of the first would be dropped silently"),
        example: Edits(&[("", "\nBlock MASS\n 6 1.73e2\n")]),
        expect: Error("opened again"),
    },
    Construct {
        name: "an external parameter the card does not set",
        madgraph: "MadGraph5Error \"not define\" (model_reader.py L184), and a missing \
                   block is an \"Invalid restriction card\" (L115); the exceptions are \
                   conversions of other conventions (`loop`, SLHA1, `alpha`)",
        class: Refused("the loader would keep parameters.py's value in its place"),
        example: Edits(&[("    3 1.180000e-01 # aS \n", "")]),
        expect: Error("aS"),
    },
    Construct {
        name: "`9.999999e-1` or `0.000001e-99`",
        madgraph: "neither fixed nor merged, then replaced by 1 or 0 as the default \
                   (restrict_model, import_ufo.py L2455-L2461, L2724)",
        class: Refused("the loader would merge or keep it as written"),
        example: Edits(&[("   15 1.777000e+00 # MTA", "   15 9.999999e-01 # MTA")]),
        expect: Error("leaves free"),
    },
    Construct {
        name: "a non-finite value",
        madgraph: "read by Python's float (L88)",
        class: Refused("no coupling can be evaluated at it"),
        example: Edits(&[("   25 1.250000e+02 # MH", "   25 nan # MH")]),
        expect: Error("finite"),
    },
    Construct {
        name: "`DECAY <pdg> Auto`",
        madgraph: "kept free and written as Auto for MadWidth (modify_autowidth, L2386)",
        class: Refused("no width is computed here"),
        example: Edits(&[("DECAY   6 1.491500E+00", "DECAY   6 Auto")]),
        expect: Error("Auto"),
    },
    Construct {
        name: "a `DECAY` line with fields after the width",
        madgraph: "the width is read as a string (Parameter.load_str, L76-L86)",
        class: Refused("MadGraph's width is then no number"),
        example: Edits(&[("DECAY   6 1.491500E+00", "DECAY   6 1.491500E+00 2")]),
        expect: Error("after the width"),
    },
    Construct {
        name: "an entry whose code is not integers, or whose value is not a number",
        madgraph: "the line is dropped with a warning (L398-L403), or the value kept as \
                   a string",
        class: Refused("a dropped line is a value the card meant and the restriction lacks"),
        example: Edits(&[("   25 1.250000e+02 # MH", "   25 1.25d2 # MH")]),
        expect: Error("expected float"),
    },
    Construct {
        name: "a `scan:` value",
        madgraph: "a string to the restriction",
        class: Refused("no value can be read from it"),
        example: Edits(&[("   25 1.250000e+02 # MH", "   25 scan:[120, 130] # MH")]),
        expect: Error("scan"),
    },
    Construct {
        name: "a `param_<x>.dat` beside `restrict_<x>.dat`",
        madgraph: "its values become the model's defaults after the restriction \
                   (restrict_model, import_ufo.py L2491-L2496)",
        class: Refused("the loader reads no such file"),
        example: Text(""),
        expect: PinnedBy("a_param_card_beside_the_restrict_card_is_refused"),
    },
];

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    use super::*;
    use crate::ufo::bind::bind_card;
    use crate::ufo::slha::ParamCard;
    use crate::ufo::sm::{sm_model, sm_parsed_model, SMRestrict};
    use crate::ufo::UFOModel;

    /// The default-restricted SM, with `MT` and `ymt` (equal defaults in two
    /// blocks) tied as a merged pair and `WT` held at one, so the merge and
    /// fixed-one rows have something to refuse; no SM restriction makes either.
    pub(crate) fn param_model() -> Arc<UFOModel> {
        let mut model = (*sm_model(SMRestrict::Default)).clone();
        model.params.merged = vec![vec![("MT".to_string(), 1.0), ("ymt".to_string(), 1.0)]];
        model.params.ones.insert("WT".to_string());
        Arc::new(model)
    }

    fn text(card: &Card, base: &str) -> String {
        match card {
            Text(t) => (*t).to_string(),
            Edits(edits) => {
                let mut out = base.to_string();
                for (from, to) in *edits {
                    if from.is_empty() {
                        out.push_str(to);
                    } else {
                        assert_eq!(out.matches(from).count(), 1, "`{from}`");
                        out = out.replacen(from, to, 1);
                    }
                }
                out
            }
        }
    }

    /// The table's own invariants: names unique, every string filled, and each
    /// expectation the one its class asks for.
    fn assert_well_formed(table: &[Construct]) {
        let mut names = BTreeSet::new();
        for row in table {
            assert!(names.insert(row.name), "{}: named twice", row.name);
            let why = match row.class {
                Consumed(s) | IgnoredBenign(s) | Refused(s) => s,
            };
            assert!(
                !why.trim().is_empty() && !row.madgraph.trim().is_empty(),
                "{}",
                row.name
            );
            let matches = matches!(
                (&row.class, &row.expect),
                (Consumed(_), Differs(_))
                    | (IgnoredBenign(_), Same(_))
                    | (Refused(_), Error(_) | PinnedBy(_))
            );
            assert!(
                matches,
                "{}: the expectation does not fit the class",
                row.name
            );
        }
    }

    /// What binding a param card gives: the refusal's text, or the moved
    /// parameters, every parameter's value as bits, and the decay tables the
    /// evaluation carries to the header.
    type Bound = (Vec<(String, u64)>, BTreeMap<String, (u64, u64)>, String);

    fn bind(text: &str) -> Result<Bound, String> {
        let card: ParamCard = text.parse().map_err(|e| format!("{e}"))?;
        let bound = bind_card(param_model(), &card).map_err(|e| e.to_string())?;
        let moved = bound
            .moved
            .iter()
            .map(|m| (m.name.clone(), m.value.to_bits()))
            .collect();
        let values = bound
            .evaluated
            .param_values
            .iter()
            .map(|(k, v)| (k.clone(), (v.re.to_bits(), v.im.to_bits())))
            .collect();
        Ok((
            moved,
            values,
            format!("{:?}", bound.evaluated.decay_tables()),
        ))
    }

    /// Restricting the SM: the refusal's text, or what the restriction decides
    /// (each vertex's couplings, each external's default, and the zero, one and
    /// merged sets).
    fn restrict(text: &str) -> Result<String, String> {
        let card: ParamCard = text.parse().map_err(|e| format!("{e}"))?;
        let model = sm_parsed_model()
            .into_model(Some(&card))
            .map_err(|e| e.to_string())?;
        let vertices: BTreeMap<&String, Vec<String>> = model
            .vertices
            .iter()
            .map(|(name, v)| (name, v.couplings.keys().map(|k| format!("{k:?}")).collect()))
            .collect();
        let defaults: Vec<(String, u64)> = model
            .params
            .externals
            .iter()
            .filter_map(|p| match &p.nature {
                crate::ufo::parameters::ParamNature::External { default_value, .. } => {
                    Some((p.name.clone(), default_value.to_bits()))
                }
                _ => None,
            })
            .collect();
        Ok(format!(
            "{vertices:?}\n{defaults:?}\n{:?}\n{:?}\n{:?}",
            model.params.zeros, model.params.ones, model.params.merged
        ))
    }

    /// Hold each row's example to its class through `run`, the reader and its
    /// consumer.
    pub(crate) fn assert_rows<T: PartialEq + std::fmt::Debug>(
        table: &[Construct],
        base: &str,
        run: impl Fn(&str) -> Result<T, String>,
    ) {
        assert_well_formed(table);
        for row in table {
            let example = || run(&text(&row.example, base));
            match &row.expect {
                Same(baseline) | Differs(baseline) => {
                    let got = example().unwrap_or_else(|e| panic!("{}: refused: {e}", row.name));
                    let want = run(&text(baseline, base))
                        .unwrap_or_else(|e| panic!("{}: baseline refused: {e}", row.name));
                    if matches!(row.expect, Same(_)) {
                        assert_eq!(got, want, "{}: moved the result", row.name);
                    } else {
                        assert_ne!(got, want, "{}: moved nothing", row.name);
                    }
                }
                Error(needle) => match example() {
                    Ok(_) => panic!("{}: accepted", row.name),
                    Err(e) => assert!(e.contains(needle), "{}: `{needle}` not in: {e}", row.name),
                },
                PinnedBy(test) => assert!(!test.is_empty(), "{}", row.name),
            }
        }
    }

    #[test]
    fn every_param_card_construct_behaves_as_classified() {
        assert_rows(PARAM_CARD, "", bind);
    }

    #[test]
    fn every_restrict_card_construct_behaves_as_classified() {
        assert_rows(
            RESTRICT_CARD,
            SMRestrict::Default.restrict_card_text(),
            restrict,
        );
    }

    /// The baseline every restrict row edits is accepted.
    #[test]
    fn the_restrict_rows_base_card_restricts() {
        restrict(SMRestrict::Default.restrict_card_text()).unwrap();
    }
}
