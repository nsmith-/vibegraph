//! MadGraph proc-card grammar: every command, and every process definition in full.
//!
//! The parser mirrors MadGraph5_aMC@NLO's own command interface
//! (`madgraph/interface/madgraph_interface.py`): `extract_process` for a process
//! line, `extract_decay_chain_process` for one with decay chains, `do_define` for
//! multiparticle labels, and `precmd`'s line handling (`#` comments, `;` command
//! separators, `\` continuations). Where MadGraph states its grammar as regular
//! expressions, the same expressions are applied here, in the same order, so the
//! two agree on every edge the expressions define — including the ones that look
//! accidental (`p p>e+ e-` is not a process to MadGraph, and it is not one here).
//!
//! Nothing is dropped: the result is a [`ProcCardAst`] that records every
//! feature a card names, supported or not. Deciding what this generator can
//! honour is [`super::check::check_supported`]'s job, and resolving names
//! against a model is [`super::resolve`]'s; this module needs neither a model
//! nor a policy.

use std::fmt::Display;
use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;

use super::alias::AliasTable;

// ── Error ─────────────────────────────────────────────────────────────────────

/// A card MadGraph itself would refuse to read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("wrong use of '>' in '{0}': a process needs one or two '>' separators")]
    Separators(String),
    #[error("no initial-state particle before '>' in '{0}'")]
    NoInitialState(String),
    #[error("no final-state particle in '{0}'")]
    NoFinalState(String),
    #[error(
        "'{token}' in '{line}' is not a particle: legs are separated by whitespace, and \
         '>', '/', '$' and '|' must stand apart from particle names"
    )]
    BadLeg { token: String, line: String },
    #[error("particle repeat count 0 in '{0}' would remove the leg")]
    ZeroRepeat(String),
    #[error("only an initial-state photon ('a' or '22') can be tagged, not '{0}'")]
    TaggedInitial(String),
    #[error("multiparticle label '{0}' cannot be tagged")]
    TaggedMultiparticle(String),
    #[error(
        "'{0}' is an or-multiparticle (defined with '|'), which can only name a required \
         s-channel"
    )]
    OrMultiparticle(String),
    #[error("'|' separates alternatives only in a required s-channel list: '{0}'")]
    OrInRestriction(String),
    #[error("malformed polarization in '{0}': expected 'name{{...}}' with nothing after '}}'")]
    Polarization(String),
    #[error(
        "coupling-order constraint '{name}{op}{value}' uses '{op}': MadGraph accepts only \
         '=', '<=', '==' and '>'"
    )]
    OrderOperator {
        name: String,
        op: String,
        value: i64,
    },
    #[error("coupling-order value in '{0}' does not fit an integer")]
    OrderValue(String),
    #[error(
        "NLO mode '{0}' in '[...]' is not one of all, real, virt, sqrvirt, tree, noborn, \
         LOonly, only"
    )]
    LoopMode(String),
    #[error("process number in '{0}' does not fit an integer")]
    BadTag(String),
    #[error("parentheses do not balance in '{0}'")]
    Parentheses(String),
    #[error("missing ')' closing a decay chain in '{0}'")]
    MissingParenthesis(String),
    #[error("'[...]' cannot be combined with decay chains: '{0}'")]
    LoopSpecWithDecayChain(String),
    #[error("'{0}' needs a process after it")]
    MissingProcess(String),
    #[error("'add' takes 'process' or 'model', not '{0}'")]
    BadAdd(String),
    #[error("malformed 'define': {0}")]
    Define(String),
    #[error("'import' needs a kind and an argument: '{0}'")]
    Import(String),
    #[error("the card ends inside a '\\' line continuation")]
    DanglingContinuation,
}

// ── The syntax tree ───────────────────────────────────────────────────────────

/// A whole `proc_card.dat`, command by command, in the order MadGraph runs them.
///
/// The effective process list is not stored: `generate` and `import model`
/// discard every earlier process, and a label means whatever `define` said last
/// before the line that uses it, so both are properties of the sequence. See
/// [`ProcCardAst::processes`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcCardAst {
    pub commands: Vec<Command>,
}

/// One command of a card.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// `import model NAME[-RESTRICT] [options]`. Discards every earlier process.
    ImportModel {
        import: ModelImport,
        /// Everything after the model name (`-modelname`, `--noprefix`, ...).
        options: Vec<String>,
    },
    /// Any other `import` (`import command FILE`, `import banner FILE`, ...).
    Import { kind: String, args: Vec<String> },
    /// `define LABEL [=] members... [/ excluded...]`.
    Define(MultiparticleDef),
    /// `generate PROCESS`: discards every earlier process, then adds this one.
    Generate(ProcessLine),
    /// `add process PROCESS`: adds a process to the current list.
    AddProcess(ProcessLine),
    /// `add model PATH ...`: merges a second model into the current one.
    AddModel(Vec<String>),
    /// `set OPTION VALUE...`, an interface option.
    Set { option: String, args: Vec<String> },
    /// `launch [args]`, with the lines a script feeds to its questions up to
    /// `done`: those are run-card and param-card edits, not interface commands.
    Launch {
        args: Vec<String>,
        dialogue: Vec<String>,
    },
    /// Every other command, verbatim (`output`, `display`, `help`, ...).
    Other { verb: String, args: String },
}

/// Information extracted from an `import model` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelImport {
    /// Model name (e.g. "sm", "loop_sm").
    pub name: String,
    /// Optional restrict variant (e.g. "no_b_mass" from "sm-no_b_mass").
    pub restrict_variant: Option<String>,
}

/// A `define` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiparticleDef {
    pub alias: String,
    /// The members as written. One group is an ordinary multiparticle; several
    /// (`define v = z | a`) make an or-multiparticle, whose groups are
    /// alternatives.
    pub groups: Vec<Vec<String>>,
    /// Particles subtracted via the optional `/ except` clause.
    pub except: Vec<String>,
}

impl MultiparticleDef {
    /// Whether the label was defined with `|`.
    pub fn is_or(&self) -> bool {
        self.groups.len() > 1
    }
}

/// The argument of a `generate` or `add process` command.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessLine {
    /// The process text as MadGraph reads it: whitespace-normalized, flags removed.
    pub text: String,
    /// `--` options on the line (`--no_warning=duplicate`, `--optimize`, ...).
    pub flags: Vec<String>,
    pub definition: ProcessDefinition,
}

/// MadGraph's `ProcessDefinition`, with names as written.
///
/// Names are resolved to PDG codes against a model by [`super::resolve`];
/// label membership is already decided here, since `define` is part of the card.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProcessDefinition {
    /// Initial legs first, then final legs, each repeat count already expanded.
    pub legs: Vec<Leg>,
    /// Required s-channels (`> A B >`): alternatives (split at `|`), each a list of
    /// names that must all appear.
    pub required_s_channels: Vec<Vec<String>>,
    /// `/ A B`: particles that may not appear as a propagator.
    pub forbidden_particles: Vec<String>,
    /// `$$ A B`: particles that may not appear as an s-channel propagator.
    pub forbidden_s_channels: Vec<String>,
    /// `$ A B`: particles that may not go on shell in an s-channel.
    pub forbidden_onsh_s_channels: Vec<String>,
    /// Coupling-order constraints, left to right as written.
    pub orders: Vec<CouplingConstraint>,
    /// `[option = orders]`.
    pub loop_spec: Option<LoopSpec>,
    /// `@N`.
    pub tag: Option<u32>,
    /// `ORDER=n` after the `@N` of a decay-chain line: bounds on the whole chain.
    pub overall_orders: Vec<(String, i64)>,
    /// Whether this is the core of a decay chain, which MadGraph reads without
    /// turning `==` / `>` amplitude constraints into squared-order ones.
    pub chain_core: bool,
    /// `, (A > B C, ...)`: one entry per decay, each possibly with its own chains.
    pub decay_chains: Vec<ProcessDefinition>,
}

impl ProcessDefinition {
    pub fn initial(&self) -> impl Iterator<Item = &Leg> {
        self.legs.iter().filter(|l| l.state == LegState::Initial)
    }

    pub fn final_state(&self) -> impl Iterator<Item = &Leg> {
        self.legs.iter().filter(|l| l.state == LegState::Final)
    }
}

/// Initial or final state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegState {
    Initial,
    Final,
}

/// One external leg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub state: LegState,
    pub particle: LegParticle,
    /// The token as written, without polarization braces or tag marks: `2e+`
    /// for a leg that came from a repeat count, which is what lets resolution
    /// notice a model particle whose name starts with a digit.
    pub token: String,
    /// The text between `{` and `}`, as written.
    pub polarization: Option<String>,
    /// `!a!`: a tagged photon.
    pub tagged: bool,
}

/// What a leg names, in the order MadGraph tries the readings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LegParticle {
    /// A multiparticle label (`p`, `j`, or one from `define`).
    Label(String),
    /// An integer: a PDG code (`11`, `-11`, `21`).
    Pdg(i64),
    /// A model particle name.
    Name(String),
}

impl Display for LegParticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LegParticle::Label(s) | LegParticle::Name(s) => f.write_str(s),
            LegParticle::Pdg(c) => write!(f, "{c}"),
        }
    }
}

/// Coupling order comparison operator — MadGraph's accepted set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CouplingOp {
    /// `=` — an upper bound, as `<=`.
    Eq,
    /// `<=`
    Le,
    /// `==` — exact equality.
    ExactEq,
    /// `>` — a strict lower bound.
    Gt,
}

impl Display for CouplingOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CouplingOp::Eq => "=",
            CouplingOp::Le => "<=",
            CouplingOp::ExactEq => "==",
            CouplingOp::Gt => ">",
        })
    }
}

/// One coupling-order constraint as written.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CouplingConstraint {
    /// The order name as written (`QCD`, `WEIGHTED`, `aEW`), without `^2`.
    pub name: String,
    /// `NAME^2`: a bound on the squared amplitude's order.
    pub squared: bool,
    pub op: CouplingOp,
    pub value: i64,
}

impl Display for CouplingConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if self.squared {
            f.write_str("^2")?;
        }
        write!(f, "{}{}", self.op, self.value)
    }
}

/// `[option = orders]`, the loop / perturbation specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopSpec {
    /// `real`, `virt`, `tree`, ...; `None` for a bare `[QCD]`.
    pub option: Option<String>,
    /// The perturbed orders as written (`QCD`, `all`, ...).
    pub orders: Vec<String>,
}

// ── MadGraph's expressions ───────────────────────────────────────────────────

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a MadGraph grammar expression compiles")
}

/// `check_process_format`: one or two `>` not followed by a digit (`QCD^2>2`).
static SEPARATOR: LazyLock<Regex> = LazyLock::new(|| re(r">\D"));
/// `extract_process`'s spacing fix-up. MadGraph writes a character class meant to
/// cover `[ ] / , $ > |`, but as Python parses it the class closes early and the
/// expression only ever matches a `]` between two non-blanks.
static BRACKET_SPACING: LazyLock<Regex> = LazyLock::new(|| re(r"(\S)(\])(\S)"));
static PROC_NUMBER: LazyLock<Regex> = LazyLock::new(|| re(r"^(.+)@\s*(\d+)\s*(.*)$"));
static CHAIN_PROC_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(.+)@\s*(\d+)\s*((\w+\s*<?=\s*\d+\s*)*)$"));
static CHAIN_ORDER: LazyLock<Regex> = LazyLock::new(|| re(r"^(.*?)\s*(\w+)\s*<?=\s*(\d+)\s*$"));
static PERTURBATION: LazyLock<Regex> = LazyLock::new(|| {
    re(r"^(?P<proc>.+>.+)\s*\[\s*((?P<option>\w+)\s*=)?\s*(?P<pert>(\w+\s*)*)\s*\]\s*(?P<rest>.*)$")
});
static ORDER: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"^(?P<before>.+>.+)\s+(?P<name>(\w|(\^2))+)\s*(?P<type>(=|(<=)|(==)|(===)|(!=)|(>=)|<|>))\s*(?P<value>-?\d+)\s*?(?P<after>.*)",
    )
});
static FORBIDDEN_BEFORE_DOLLAR: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(.+)\s*/\s*(.+\s*)(\$.*)$"));
static FORBIDDEN: LazyLock<Regex> = LazyLock::new(|| re(r"^(.+)\s*/\s*(.+\s*)$"));
static FORBIDDEN_S: LazyLock<Regex> = LazyLock::new(|| re(r"^(.+)\s*\$\s*\$\s*(.+)\s*$"));
static FORBIDDEN_ONSH: LazyLock<Regex> = LazyLock::new(|| re(r"^(.+)\s*\$\s*(.+)\s*$"));
static REQUIRED: LazyLock<Regex> = LazyLock::new(|| re(r"^(.+?)>(.+?)>(.+)$"));

const NLO_MODES: [&str; 8] = [
    "all", "real", "virt", "sqrvirt", "tree", "noborn", "LOonly", "only",
];

// ── Card ──────────────────────────────────────────────────────────────────────

/// Parse a `proc_card.dat` (or a MadGraph batch script) into its commands.
///
/// Commands are case-sensitive, as in MadGraph: `Generate` is not a command.
pub fn parse_proc_card_ast(content: &str) -> Result<ProcCardAst, ParseError> {
    let mut commands = Vec::new();
    let mut aliases = AliasTable::default_sm();
    let mut launch: Option<(Vec<String>, Vec<String>)> = None;
    let mut continued = String::new();

    for raw in content.lines() {
        let mut line = std::mem::take(&mut continued);
        line.push_str(raw.trim());
        if let Some(head) = line.strip_suffix('\\') {
            continued = head.to_owned();
            continue;
        }
        let line = line.split('#').next().unwrap_or("");
        for sub in line.split(';') {
            let sub = sub.trim();
            if sub.is_empty() {
                continue;
            }
            let (verb, rest) = match sub.split_once(char::is_whitespace) {
                Some((v, r)) => (v, r.trim()),
                None => (sub, ""),
            };
            if let Some((args, dialogue)) = launch.as_mut() {
                if verb == "done" {
                    commands.push(Command::Launch {
                        args: std::mem::take(args),
                        dialogue: std::mem::take(dialogue),
                    });
                    launch = None;
                } else {
                    dialogue.push(sub.to_owned());
                }
                continue;
            }
            if verb == "launch" {
                launch = Some((split_arg(rest), Vec::new()));
                continue;
            }
            commands.push(parse_command(verb, rest, &mut aliases)?);
        }
    }
    if !continued.is_empty() {
        return Err(ParseError::DanglingContinuation);
    }
    if let Some((args, dialogue)) = launch {
        commands.push(Command::Launch { args, dialogue });
    }
    Ok(ProcCardAst { commands })
}

/// One command line other than `launch`.
fn parse_command(verb: &str, rest: &str, aliases: &mut AliasTable) -> Result<Command, ParseError> {
    let args: Vec<String> = split_arg(rest);
    let line = || format!("{verb} {rest}");
    Ok(match verb {
        "import" => match args.split_first() {
            Some((kind, rest)) if kind.starts_with("model") => {
                let Some((name, options)) = rest.split_first() else {
                    return Err(ParseError::Import(line()));
                };
                Command::ImportModel {
                    import: parse_model_import(name),
                    options: options.to_vec(),
                }
            }
            Some((kind, rest)) if !rest.is_empty() => Command::Import {
                kind: kind.clone(),
                args: rest.to_vec(),
            },
            _ => return Err(ParseError::Import(line())),
        },
        "define" => {
            let def = parse_define_line(rest)?;
            aliases.apply(&def);
            Command::Define(def)
        }
        "generate" => Command::Generate(parse_process_line(verb, &args, aliases)?),
        "add" => match args.split_first() {
            Some((kind, rest)) if kind == "process" => {
                Command::AddProcess(parse_process_line("add process", rest, aliases)?)
            }
            Some((kind, rest)) if kind == "model" && !rest.is_empty() => {
                Command::AddModel(rest.to_vec())
            }
            Some((kind, _)) if kind != "process" && kind != "model" => {
                return Err(ParseError::BadAdd(kind.clone()))
            }
            _ => return Err(ParseError::MissingProcess(line())),
        },
        "set" if !args.is_empty() => Command::Set {
            option: args[0].clone(),
            args: args[1..].to_vec(),
        },
        _ => Command::Other {
            verb: verb.to_owned(),
            args: rest.to_owned(),
        },
    })
}

/// MadGraph's `split_arg`: whitespace-separated words, quotes kept together.
fn split_arg(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in line.chars() {
        match quote {
            Some(q) => {
                cur.push(c);
                if c == q {
                    quote = None;
                }
            }
            None if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            None => {
                if c == '\'' || c == '"' {
                    quote = Some(c);
                }
                cur.push(c);
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Parse an `import model` argument: `sm`, `sm-no_b_mass`, a path.
fn parse_model_import(spec: &str) -> ModelImport {
    let spec = spec.trim();
    match spec.find('-') {
        Some(dash) => ModelImport {
            name: spec[..dash].to_owned(),
            restrict_variant: Some(spec[dash + 1..].to_owned()),
        },
        None => ModelImport {
            name: spec.to_owned(),
            restrict_variant: None,
        },
    }
}

/// Parse a `define` line body: `alias [=] members... [| members...] [/ except...]`.
pub fn parse_define_line(s: &str) -> Result<MultiparticleDef, ParseError> {
    let spaced = s
        .replace('=', " = ")
        .replace('|', " | ")
        .replace('/', " / ");
    let mut args = split_arg(&spaced);
    if args.len() < 2 {
        return Err(ParseError::Define(format!(
            "'{s}' needs a label and at least one particle"
        )));
    }
    if args[1] == "=" {
        args.remove(1);
        if args.len() < 2 {
            return Err(ParseError::Define(format!(
                "'{s}' needs at least one particle after '='"
            )));
        }
    }
    if args.iter().any(|a| a == "=") {
        return Err(ParseError::Define(format!(
            "'{s}': '=' may only follow the label"
        )));
    }
    let alias = args.remove(0);
    let (members, except) = match args.iter().position(|a| a == "/") {
        Some(i) => (args[..i].to_vec(), args[i + 1..].to_vec()),
        None => (args, Vec::new()),
    };
    let groups = split_alternatives(members);
    if groups.is_empty() {
        return Err(ParseError::Define(format!("'{s}' has no members")));
    }
    Ok(MultiparticleDef {
        alias,
        groups,
        except,
    })
}

/// Split a name list into its `|`-separated alternatives, dropping empty ones.
fn split_alternatives(names: Vec<String>) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for name in names {
        if name == "|" {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else {
            current.push(name);
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

// ── Process line ──────────────────────────────────────────────────────────────

/// Parse the arguments of a `generate` / `add process` command, as `do_add`
/// does: flags come out first, then the text goes to the decay-chain reader
/// when it has a `,` and to the plain one otherwise.
fn parse_process_line(
    verb: &str,
    args: &[String],
    aliases: &AliasTable,
) -> Result<ProcessLine, ParseError> {
    let (flags, words): (Vec<String>, Vec<String>) =
        args.iter().cloned().partition(|a| a.starts_with("--"));
    if words.is_empty() {
        return Err(ParseError::MissingProcess(verb.to_owned()));
    }
    let text = words.join(" ");
    let definition = parse_definition(&text, aliases)?;
    Ok(ProcessLine {
        text,
        flags,
        definition,
    })
}

fn parse_definition(text: &str, aliases: &AliasTable) -> Result<ProcessDefinition, ParseError> {
    check_process_format(text)?;
    if text.contains(',') {
        if text.contains('[') || text.contains(']') {
            return Err(ParseError::LoopSpecWithDecayChain(text.to_owned()));
        }
        return Ok(extract_decay_chain_process(text, false, None, aliases)?.0);
    }
    extract_process(text, None, Vec::new(), false, aliases)
}

/// `check_process_format`: balanced parentheses, and one or two `>` per part.
fn check_process_format(text: &str) -> Result<(), ParseError> {
    if text.matches('(').count() != text.matches(')').count() {
        return Err(ParseError::Parentheses(text.to_owned()));
    }
    let flat = text.replace(['(', ')'], " ");
    for part in flat.split(',') {
        let n = SEPARATOR.find_iter(part).count();
        if n != 1 && n != 2 {
            return Err(ParseError::Separators(part.trim().to_owned()));
        }
    }
    Ok(())
}

/// Parse a single process string against the default labels, as the argument
/// of a `generate` line.
pub fn parse_process_string(s: &str) -> Result<ProcessDefinition, ParseError> {
    parse_definition(&split_arg(s).join(" "), &AliasTable::default_sm())
}

/// `extract_decay_chain_process`: the core process up to the first `,` or `)`,
/// then each decay, recursing into a parenthesised group. Returns the definition
/// and the unread remainder.
fn extract_decay_chain_process(
    line: &str,
    mut level_down: bool,
    mut tag: Option<u32>,
    aliases: &AliasTable,
) -> Result<(ProcessDefinition, String), ParseError> {
    let mut line = line.to_owned();
    let mut overall_orders = Vec::new();
    if let Some(c) = CHAIN_PROC_NUMBER.captures(&line) {
        tag = Some(parse_tag(&c[2], &line)?);
        let mut order_line = c[3].to_owned();
        line = c[1].to_owned();
        while let Some(o) = CHAIN_ORDER.captures(&order_line) {
            let value = o[3]
                .parse()
                .map_err(|_| ParseError::OrderValue(order_line.clone()))?;
            overall_orders.push((o[2].to_owned(), value));
            order_line = o[1].to_owned();
        }
        overall_orders.reverse();
    }

    let mut index_comma = line.find(',');
    let core_text = match first_of(index_comma, line.find(')')) {
        Some(i) => &line[..i],
        None => &line[..],
    };
    let mut core = extract_process(core_text, tag, overall_orders, true, aliases)?;

    while let Some(comma) = index_comma {
        line = line[comma + 1..].to_owned();
        if line.trim().is_empty() {
            break;
        }
        let mut index_par = line.find(')');
        if line.trim_start().starts_with('(') {
            if let Some(par) = index_par.filter(|&par| !line[..par].contains(',')) {
                let start = line.find('(').expect("the line starts with '('");
                line = format!("{} {}", &line[start + 1..par], &line[par + 1..]);
                index_par = line.find(')');
            }
        }
        let decay = if line.trim_start().starts_with('(') {
            let inner = line.trim_start()[1..].to_owned();
            let (decay, rest) = extract_decay_chain_process(&inner, true, None, aliases)?;
            line = rest;
            index_comma = line.find(',');
            index_par = line.find(')');
            decay
        } else {
            index_comma = line.find(',');
            let text = match first_of(index_comma, index_par) {
                Some(i) => &line[..i],
                None => &line[..],
            };
            extract_process(text, None, Vec::new(), false, aliases)?
        };
        core.decay_chains.push(decay);

        if level_down {
            let Some(par) = index_par else {
                return Err(ParseError::MissingParenthesis(line));
            };
            if index_comma.is_some_and(|comma| par < comma) {
                line = line[par + 1..].to_owned();
                level_down = false;
                break;
            }
        }
    }
    if level_down {
        let Some(par) = line.find(')') else {
            return Err(ParseError::MissingParenthesis(line));
        };
        line = line[par + 1..].to_owned();
    }
    Ok((core, line))
}

/// The earlier of two optional positions.
fn first_of(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

fn parse_tag(digits: &str, line: &str) -> Result<u32, ParseError> {
    digits
        .parse()
        .map_err(|_| ParseError::BadTag(line.to_owned()))
}

/// `extract_process`: strip the modifiers from the back of the line in
/// MadGraph's order — `@N`, `[...]`, coupling orders, `/`, `$$`, `$`, then the
/// required s-channels between two `>` — and read the legs that remain.
fn extract_process(
    line: &str,
    mut tag: Option<u32>,
    overall_orders: Vec<(String, i64)>,
    chain_core: bool,
    aliases: &AliasTable,
) -> Result<ProcessDefinition, ParseError> {
    let orig = line.trim().to_owned();
    let n_sep = SEPARATOR.find_iter(line).count();
    if n_sep != 1 && n_sep != 2 {
        return Err(ParseError::Separators(orig));
    }
    let mut line = BRACKET_SPACING.replace_all(line, "$1 $2 $3").into_owned();

    if let Some(c) = PROC_NUMBER.captures(&line) {
        tag = Some(parse_tag(&c[2], &orig)?);
        line = format!("{}{}", &c[1], &c[3]);
    }

    let mut loop_spec = None;
    if let Some(c) = PERTURBATION.captures(&line) {
        let option = c.name("option").map(|m| m.as_str().to_owned());
        if let Some(o) = &option {
            if !NLO_MODES.contains(&o.as_str()) {
                return Err(ParseError::LoopMode(o.clone()));
            }
        }
        let orders = c
            .name("pert")
            .map(|m| m.as_str().split_whitespace().map(String::from).collect())
            .unwrap_or_default();
        loop_spec = Some(LoopSpec { option, orders });
        line = format!("{}{}", &c["proc"], &c["rest"]);
    }

    let mut orders = Vec::new();
    while let Some(c) = ORDER.captures(&line) {
        let raw_name = &c["name"];
        let value: i64 = c["value"]
            .parse()
            .map_err(|_| ParseError::OrderValue(orig.clone()))?;
        let (name, squared) = match raw_name.strip_suffix("^2") {
            Some(base) => (base.to_owned(), true),
            None => (raw_name.to_owned(), false),
        };
        let op = match &c["type"] {
            "=" => CouplingOp::Eq,
            "<=" => CouplingOp::Le,
            "==" => CouplingOp::ExactEq,
            ">" => CouplingOp::Gt,
            other => {
                return Err(ParseError::OrderOperator {
                    name: raw_name.to_owned(),
                    op: other.to_owned(),
                    value,
                })
            }
        };
        orders.push(CouplingConstraint {
            name,
            squared,
            op,
            value,
        });
        line = format!("{} {}", &c["before"], &c["after"]);
    }
    orders.reverse();

    let mut forbidden_particles = String::new();
    let slash = line.find('/');
    let dollar = line.find('$');
    if slash.is_some_and(|s| s > 0) {
        if dollar > slash {
            if let Some(c) = FORBIDDEN_BEFORE_DOLLAR.captures(&line) {
                forbidden_particles = c[2].to_owned();
                line = format!("{}{}", &c[1], &c[3]);
            }
        } else if let Some(c) = FORBIDDEN.captures(&line) {
            forbidden_particles = c[2].to_owned();
            line = c[1].to_owned();
        }
    }
    let mut forbidden_s = String::new();
    if let Some(c) = FORBIDDEN_S.captures(&line) {
        forbidden_s = c[2].to_owned();
        line = c[1].to_owned();
    }
    let mut forbidden_onsh = String::new();
    if let Some(c) = FORBIDDEN_ONSH.captures(&line) {
        forbidden_onsh = c[2].to_owned();
        line = c[1].to_owned();
    }
    let mut required = String::new();
    if let Some(c) = REQUIRED.captures(&line) {
        required = c[2].to_owned();
        line = format!("{}>{}", &c[1], &c[3]);
    }

    let legs = extract_legs(&line, &orig, aliases)?;

    Ok(ProcessDefinition {
        legs,
        required_s_channels: split_alternatives(names(&required, &orig)?),
        forbidden_particles: restriction(&forbidden_particles, &orig, aliases)?,
        forbidden_s_channels: restriction(&forbidden_s, &orig, aliases)?,
        forbidden_onsh_s_channels: restriction(&forbidden_onsh, &orig, aliases)?,
        orders,
        loop_spec,
        tag,
        overall_orders,
        chain_core,
        decay_chains: Vec::new(),
    })
}

/// The names of a restriction list. A modifier symbol left among them is a
/// modifier MadGraph's expressions did not take off the line (`/ h $$ w+ $ z`
/// leaves `$$` in the `/` list), which MadGraph then fails to read as a particle.
fn names(text: &str, orig: &str) -> Result<Vec<String>, ParseError> {
    let names = split_arg(text);
    if let Some(bad) = names
        .iter()
        .find(|n| n.contains(['>', '$', '/', '[', ']', '(', ')', ',']))
    {
        return Err(ParseError::BadLeg {
            token: bad.clone(),
            line: orig.to_owned(),
        });
    }
    Ok(names)
}

/// A restriction list: names only, no alternatives.
fn restriction(text: &str, orig: &str, aliases: &AliasTable) -> Result<Vec<String>, ParseError> {
    let names = names(text, orig)?;
    if names.iter().any(|n| n == "|") {
        return Err(ParseError::OrInRestriction(text.trim().to_owned()));
    }
    if let Some(or) = names.iter().find(|n| aliases.is_or_label(n)) {
        return Err(ParseError::OrMultiparticle(or.clone()));
    }
    Ok(names)
}

/// The leg loop of `extract_process`.
fn extract_legs(line: &str, orig: &str, aliases: &AliasTable) -> Result<Vec<Leg>, ParseError> {
    let mut legs = Vec::new();
    let mut state = LegState::Initial;
    for word in split_arg(line) {
        if word == ">" {
            if legs.is_empty() {
                return Err(ParseError::NoInitialState(orig.to_owned()));
            }
            state = LegState::Final;
            continue;
        }
        let (part, tagged) = strip_tag(&word);
        if tagged && state == LegState::Initial && part != "a" && part != "22" {
            return Err(ParseError::TaggedInitial(part));
        }

        let (name, polarization) = match part.split_once('{') {
            Some((name, pol)) => match pol.split_once('}') {
                Some((pol, "")) => (name.to_owned(), Some(pol.to_owned())),
                _ => return Err(ParseError::Polarization(word.clone())),
            },
            None => (part.clone(), None),
        };
        if name.is_empty() || name.contains(['>', '$', '/', '|', '[', ']', '(', ')', ',']) {
            return Err(ParseError::BadLeg {
                token: word.clone(),
                line: orig.to_owned(),
            });
        }

        let (particle, repeat) = classify_leg(&name, aliases)?;
        if let LegParticle::Label(label) = &particle {
            if tagged && state == LegState::Final {
                return Err(ParseError::TaggedMultiparticle(label.clone()));
            }
        }
        if repeat == 0 {
            return Err(ParseError::ZeroRepeat(orig.to_owned()));
        }
        for _ in 0..repeat {
            legs.push(Leg {
                state,
                particle: particle.clone(),
                token: name.clone(),
                polarization: polarization.clone(),
                tagged,
            });
        }
    }
    if !legs.iter().any(|l| l.state == LegState::Final) {
        return Err(ParseError::NoFinalState(orig.to_owned()));
    }
    Ok(legs)
}

/// `!a!` (or `2!a!`): the tag marks off, and whether there were any.
fn strip_tag(word: &str) -> (String, bool) {
    if word.len() > 1 && word.starts_with('!') && word.ends_with('!') {
        return (word[1..word.len() - 1].to_owned(), true);
    }
    if let Some(bang) = word.find('!') {
        if word.ends_with('!')
            && word.matches('!').count() == 2
            && word[..bang].chars().all(|c| c.is_ascii_digit())
        {
            return (word.replace('!', ""), true);
        }
    }
    (word.to_owned(), false)
}

/// MadGraph's reading order for a leg: a label, then a PDG code, then a
/// particle name; a leading digit is a repeat count only when none of those
/// match. Whether a model particle carries the whole token as its name is left
/// to resolution, which sees the model.
fn classify_leg(name: &str, aliases: &AliasTable) -> Result<(LegParticle, u32), ParseError> {
    let label = |name: &str| -> Result<Option<LegParticle>, ParseError> {
        if !aliases.is_label(name) {
            return Ok(None);
        }
        if aliases.is_or_label(name) {
            return Err(ParseError::OrMultiparticle(name.to_owned()));
        }
        Ok(Some(LegParticle::Label(name.to_owned())))
    };
    if let Some(l) = label(name)? {
        return Ok((l, 1));
    }
    let digits = name.strip_prefix('-').unwrap_or(name);
    if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        let code = name.parse().map_err(|_| ParseError::BadLeg {
            token: name.to_owned(),
            line: name.to_owned(),
        })?;
        return Ok((LegParticle::Pdg(code), 1));
    }
    let mut chars = name.chars();
    if let Some(first) = chars.next().filter(char::is_ascii_digit) {
        let rest = chars.as_str();
        if !rest.is_empty() {
            let count = first.to_digit(10).expect("an ASCII digit");
            let particle = label(rest)?.unwrap_or_else(|| LegParticle::Name(rest.to_owned()));
            return Ok((particle, count));
        }
    }
    Ok((LegParticle::Name(name.to_owned()), 1))
}

// ── Command sequence semantics ───────────────────────────────────────────────

/// One process of the card's effective list.
#[derive(Debug, Clone)]
pub struct CardProcess<'a> {
    pub line: &'a ProcessLine,
    /// MadGraph's process number: the `@N` if given, otherwise the line's
    /// position among the `generate` / `add process` lines since the last reset.
    pub id: u32,
    /// The labels as defined when the line was read.
    pub aliases: AliasTable,
}

impl ProcCardAst {
    /// The processes MadGraph would hold after running the card: `generate` and
    /// `import model` discard what came before, `add process` appends.
    pub fn processes(&self) -> Vec<CardProcess<'_>> {
        let mut out = Vec::new();
        let mut aliases = AliasTable::default_sm();
        let mut ordinal = 0u32;
        for command in &self.commands {
            match command {
                Command::ImportModel { .. } => {
                    out.clear();
                    ordinal = 0;
                }
                Command::Define(def) => aliases.apply(def),
                Command::Generate(line) | Command::AddProcess(line) => {
                    if matches!(command, Command::Generate(_)) {
                        out.clear();
                        ordinal = 0;
                    }
                    ordinal += 1;
                    out.push(CardProcess {
                        line,
                        id: line.definition.tag.unwrap_or(ordinal),
                        aliases: aliases.clone(),
                    });
                }
                _ => {}
            }
        }
        out
    }

    /// The last `import model`, if any.
    pub fn model(&self) -> Option<&ModelImport> {
        self.commands.iter().rev().find_map(|c| match c {
            Command::ImportModel { import, .. } => Some(import),
            _ => None,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> ProcessDefinition {
        parse_process_string(s).expect("parse failed")
    }

    fn names<'a>(legs: impl Iterator<Item = &'a Leg>) -> Vec<String> {
        legs.map(|l| l.particle.to_string()).collect()
    }

    #[test]
    fn simple_process() {
        let p = parse("e+ e- > mu+ mu-");
        assert_eq!(names(p.initial()), ["e+", "e-"]);
        assert_eq!(names(p.final_state()), ["mu+", "mu-"]);
        assert!(p.orders.is_empty());
        assert!(p.tag.is_none());
    }

    #[test]
    fn tag_then_orders_parses_both() {
        let p = parse("p p > j j @1 QED=0");
        assert_eq!(p.tag, Some(1));
        assert_eq!(names(p.final_state()), ["j", "j"]);
        assert_eq!(p.orders.len(), 1);
        assert_eq!(p.orders[0].to_string(), "QED=0");
    }

    #[test]
    fn coupling_order_gt_after_final_state_is_an_order() {
        let p = parse("e+ e- > mu+ mu- QCD > 2");
        assert_eq!(p.orders[0].op, CouplingOp::Gt);
        assert_eq!(names(p.final_state()), ["mu+", "mu-"]);
        assert!(p.required_s_channels.is_empty());
    }

    #[test]
    fn orders_read_left_to_right() {
        let p = parse("p p > t t~ QCD<=2 QED==0 QCD^2>4");
        let written: Vec<String> = p.orders.iter().map(|o| o.to_string()).collect();
        assert_eq!(written, ["QCD<=2", "QED==0", "QCD^2>4"]);
    }

    #[test]
    fn operators_madgraph_refuses_are_errors() {
        for op in ["<", ">=", "!=", "==="] {
            let r = parse_process_string(&format!("e+ e- > mu+ mu- QED{op}2"));
            assert!(
                matches!(r, Err(ParseError::OrderOperator { .. })),
                "{op}: {r:?}"
            );
        }
    }

    #[test]
    fn required_s_channels_are_or_of_and() {
        let p = parse("e+ e- > z a | h > mu+ mu-");
        assert_eq!(
            p.required_s_channels,
            vec![vec!["z".to_owned(), "a".to_owned()], vec!["h".to_owned()]]
        );
    }

    #[test]
    fn restrictions_are_parsed_and_kept() {
        let p = parse("p p > e+ e- $ z $$ w+ / h");
        assert_eq!(p.forbidden_particles, ["h"]);
        assert_eq!(p.forbidden_s_channels, ["w+"]);
        assert_eq!(p.forbidden_onsh_s_channels, ["z"]);
    }

    /// MadGraph strips `/`, then `$$`, then `$`, each with one greedy expression,
    /// so only some orders of the three read; the others leave a modifier among
    /// the names, and MadGraph fails on it as a particle.
    #[test]
    fn restriction_order_follows_madgraphs_expressions() {
        for line in [
            "p p > e+ e- / h $$ w+ $ z",
            "p p > e+ e- $$ w+ $ z",
            "p p > e+ e- / h $$ w+",
        ] {
            assert!(
                matches!(parse_process_string(line), Err(ParseError::BadLeg { .. })),
                "{line}"
            );
        }
        let p = parse("p p > e+ e- $$ w+ / h");
        assert_eq!(p.forbidden_particles, ["h"]);
        assert_eq!(p.forbidden_s_channels, ["w+"]);
    }

    #[test]
    fn pdg_codes_are_codes_not_repeat_counts() {
        let p = parse("11 -11 > 21 21");
        assert_eq!(
            p.legs
                .iter()
                .map(|l| l.particle.clone())
                .collect::<Vec<_>>(),
            [11, -11, 21, 21].map(LegParticle::Pdg)
        );
    }

    #[test]
    fn a_leading_digit_is_a_repeat_count() {
        let p = parse("e+ e- > 2e+ 2j");
        assert_eq!(names(p.final_state()), ["e+", "e+", "j", "j"]);
        assert_eq!(p.legs[2].token, "2e+");
        assert!(matches!(p.legs[4].particle, LegParticle::Label(_)));
    }

    #[test]
    fn unspaced_separators_are_not_legs() {
        assert!(matches!(
            parse_process_string("p p>e+ e-"),
            Err(ParseError::BadLeg { .. })
        ));
    }

    #[test]
    fn polarization_and_tags_are_kept() {
        let p = parse("e+ e- > w+{0} w-{T}");
        assert_eq!(p.legs[2].polarization.as_deref(), Some("0"));
        assert_eq!(p.legs[3].polarization.as_deref(), Some("T"));
        let p = parse("!a! e- > e- a");
        assert!(p.legs[0].tagged);
    }

    #[test]
    fn loop_spec_is_kept() {
        let p = parse("p p > e+ e- [real=QCD]");
        let spec = p.loop_spec.as_ref().unwrap();
        assert_eq!(spec.option.as_deref(), Some("real"));
        assert_eq!(spec.orders, ["QCD"]);
        assert_eq!(names(p.final_state()), ["e+", "e-"]);
    }

    #[test]
    fn decay_chains_nest() {
        let p = parse("p p > t t~, (t > w+ b, w+ > j j), (t~ > w- b~, w- > l- vl~)");
        assert_eq!(p.decay_chains.len(), 2);
        assert_eq!(p.decay_chains[0].decay_chains.len(), 1);
        assert_eq!(names(p.decay_chains[1].initial()), ["t~"]);
        assert_eq!(
            names(p.decay_chains[1].decay_chains[0].final_state()),
            ["l-", "vl~"]
        );
    }

    #[test]
    fn chain_tag_takes_overall_orders() {
        let p = parse("p p > t t~, t > w+ b @2 QED=2");
        assert_eq!(p.tag, Some(2));
        assert_eq!(p.overall_orders, [("QED".to_owned(), 2)]);
        assert!(p.orders.is_empty());
    }

    #[test]
    fn generate_resets_and_add_appends() {
        let ast = parse_proc_card_ast(
            "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta- @5\n\
             add process e+ e- > e+ e-\ngenerate e+ e- > a a\nadd process e+ e- > z z\n",
        )
        .unwrap();
        let ids: Vec<(u32, String)> = ast
            .processes()
            .iter()
            .map(|p| (p.id, p.line.text.clone()))
            .collect();
        assert_eq!(
            ids,
            [(1, "e+ e- > a a".to_owned()), (2, "e+ e- > z z".to_owned())]
        );
    }

    #[test]
    fn malformed_add_process_is_an_error() {
        assert!(parse_proc_card_ast("add process").is_err());
        assert!(parse_proc_card_ast("add process e+ e-").is_err());
        assert!(parse_proc_card_ast("add proces e+ e- > a a").is_err());
    }

    #[test]
    fn define_with_or_and_except() {
        let d = parse_define_line("v = z | a").unwrap();
        assert!(d.is_or());
        let d = parse_define_line("q p / g").unwrap();
        assert_eq!(d.alias, "q");
        assert_eq!(d.groups, [vec!["p".to_owned()]]);
        assert_eq!(d.except, ["g"]);
    }

    #[test]
    fn launch_dialogue_is_one_command() {
        let ast = parse_proc_card_ast(
            "generate e+ e- > mu+ mu-\noutput x\nlaunch\n  set ebeam1 45.6\n  done\n",
        )
        .unwrap();
        assert_eq!(ast.commands.len(), 3);
        assert_eq!(
            ast.commands[2],
            Command::Launch {
                args: vec![],
                dialogue: vec!["set ebeam1 45.6".to_owned()]
            }
        );
    }

    #[test]
    fn comments_semicolons_and_continuations() {
        let ast = parse_proc_card_ast(
            "import model sm # the SM\ndefine x = u ; generate x x > \\\n e+ e-\n",
        )
        .unwrap();
        assert_eq!(ast.commands.len(), 3);
        assert_eq!(ast.model().unwrap().name, "sm");
        assert_eq!(ast.processes()[0].line.text, "x x > e+ e-");
    }

    #[test]
    fn model_import_with_variant() {
        let ast = parse_proc_card_ast("import model sm-no_b_mass\n").unwrap();
        let m = ast.model().unwrap();
        assert_eq!(m.name, "sm");
        assert_eq!(m.restrict_variant.as_deref(), Some("no_b_mass"));
    }
}
