//! MadGraph-style process string parser.
//!
//! Implements the same sequential modifier-stripping algorithm used by MadGraph5_aMC@NLO's
//! `extract_process` (madgraph/interface/madgraph_interface.py, line 4822). Each modifier
//! is stripped from the process string in a fixed order using plain string operations;
//! the residual `initial > final` tokens are whitespace-split.

use std::fmt::Display;

use thiserror::Error;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("process string has no '>' separator: '{0}'")]
    NoSeparator(String),
    #[error("too many '>' separators (found {found}, expected 1 or 2): '{input}'")]
    TooManySeparators { found: usize, input: String },
    #[error("empty particle list on {side} side of '>'")]
    EmptyLeg { side: &'static str },
    #[error("malformed process tag (expected '@N'): '{0}'")]
    BadTag(String),
    #[error("forbidden hard s-channel (`$$`) not allowed — removing s-channels can violate gauge invariance; set ParsingOptions::allow_forbidden_s_channels = true to override")]
    ForbiddenSChannelDisabled,
    #[error("forbidden on-shell s-channel (`$`) not allowed — set ParsingOptions::allow_forbidden_onsh_s_channels = true to override")]
    ForbiddenOnshSChannelDisabled,
    #[error("loop spec (`[...]`) not allowed in this context — set ParsingOptions::allow_loop_spec = true to override")]
    LoopSpecDisabled,
    #[error("malformed loop spec — unclosed '[': '{0}'")]
    UnclosedLoopSpec(String),
    #[error("malformed particle token '{0}'")]
    BadParticleTok(String),
}

// ── Options ───────────────────────────────────────────────────────────────────

/// Controls which syntax features are accepted during parsing.
#[derive(Debug, Clone)]
pub struct ParsingOptions {
    /// If false, `$$` (forbidden hard s-channels) is rejected with an error.
    ///
    /// Removing s-channels with `$$` can break gauge invariance — e.g. excluding
    /// the Z propagator from a process that requires it for amplitude cancellations.
    /// Set to `true` only when the model supports this restriction.
    pub allow_forbidden_s_channels: bool,
    /// If false, `$` (forbidden on-shell s-channels) is rejected with an error.
    pub allow_forbidden_onsh_s_channels: bool,
    /// If false, loop specs `[QCD]` / `[all=QCD]` cause an error instead of being silently dropped.
    pub allow_loop_spec: bool,
}

impl Default for ParsingOptions {
    fn default() -> Self {
        Self {
            allow_forbidden_s_channels: false, // reject $$ by default for gauge safety
            allow_forbidden_onsh_s_channels: true,
            allow_loop_spec: true,
        }
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single external leg in a process specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleLeg {
    /// Particle name or alias (may not be a concrete model particle yet).
    pub name: String,
    /// Duplication count (`2e+` → count=2, name="e+"). Always ≥ 1.
    pub count: usize,
}

/// Coupling order comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouplingOp {
    /// `=` — treated as `<=` for amplitude orders (MadGraph semantics).
    Eq,
    /// `==` — exact equality on amplitude order.
    ExactEq,
    /// `===` — alias for `==` in MadGraph.
    StrictEq,
    /// `<=`
    Le,
    /// `<`
    Lt,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `!=`
    Ne,
}

/// One coupling order constraint extracted from the process string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouplingConstraint {
    /// Coupling name, e.g. `"QCD"`.
    pub name: String,
    /// True if the token was `NAME^2` (squared-order constraint).
    pub squared: bool,
    pub op: CouplingOp,
    pub value: i64,
}

/// A fully parsed simple process specification.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub initial: Vec<ParticleLeg>,
    /// Required s-channel particles, from `A > X Y > B` (middle segment).
    pub required_s_channels: Vec<String>,
    pub final_state: Vec<ParticleLeg>,
    /// Forbidden propagators, from `/ X Y`.
    pub forbidden_particles: Vec<String>,
    /// Forbidden hard s-channels, from `$$ X Y`.
    pub forbidden_s_channels: Vec<String>,
    /// Forbidden on-shell s-channels, from `$ X Y`.
    pub forbidden_onsh_s_channels: Vec<String>,
    pub coupling_constraints: Vec<CouplingConstraint>,
    /// Process tag from `@N`; `None` if absent.
    pub tag: Option<u32>,
}

impl Display for ProcessSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reconstruct a process string from the parsed spec (without modifiers).
        // TODO: refine this to include modifiers if we want to round-trip test the parser.
        let initial = self
            .initial
            .iter()
            .map(|leg| {
                if leg.count > 1 {
                    format!("{}{}", leg.count, leg.name)
                } else {
                    leg.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let final_state = self
            .final_state
            .iter()
            .map(|leg| {
                if leg.count > 1 {
                    format!("{}{}", leg.count, leg.name)
                } else {
                    leg.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{} > {}", initial, final_state)
    }
}

/// A `define alias = particles... [/ except...]` command.
#[derive(Debug, Clone)]
pub struct MultiparticleDef {
    pub alias: String,
    /// Particle names on the RHS (before any `/`).
    pub particles: Vec<String>,
    /// Particles subtracted via the optional `/ except` clause.
    pub except: Vec<String>,
}

/// Information extracted from an `import model` directive.
#[derive(Debug, Clone)]
pub struct ModelImport {
    /// Model name (e.g. "sm", "loop_sm").
    pub name: String,
    /// Optional restrict variant (e.g. "no_b_mass" from "sm-no_b_mass").
    pub restrict_variant: Option<String>,
}

/// The result of parsing an entire `proc_card.dat` file.
#[derive(Debug, Clone)]
pub struct ParsedProcCard {
    /// Model import directive if present (e.g. `import model sm-no_b_mass`).
    pub model: Option<ModelImport>,
    /// All `define` commands, in order.
    pub defines: Vec<MultiparticleDef>,
    /// All processes from `generate` and `add process` commands, in order.
    pub processes: Vec<ProcessSpec>,
}

// ── Top-level parse functions ─────────────────────────────────────────────────

/// Parse a `proc_card.dat` string into a `ParsedProcCard`.
pub fn parse_proc_card(content: &str, opts: &ParsingOptions) -> Result<ParsedProcCard, ParseError> {
    let mut card = ParsedProcCard {
        model: None,
        defines: Vec::new(),
        processes: Vec::new(),
    };

    for raw in content.lines() {
        let line = strip_inline_comment(raw).trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();

        if let Some(rest) = lower.strip_prefix("import model ") {
            let rest_original = &line[line.len() - rest.len()..];
            card.model = Some(parse_model_import(rest_original)?);
        } else if let Some(rest) = lower.strip_prefix("generate ") {
            let rest_original = &line[line.len() - rest.len()..];
            card.processes
                .push(parse_process_string(rest_original, opts)?);
        } else if lower.starts_with("add ") {
            // "add process ..."
            let tokens: Vec<&str> = line.splitn(3, char::is_whitespace).collect();
            if tokens.len() >= 3 && tokens[1].to_lowercase() == "process" {
                card.processes.push(parse_process_string(tokens[2], opts)?);
            }
        } else if let Some(rest) = lower.strip_prefix("define ") {
            let rest_original = &line[line.len() - rest.len()..];
            card.defines.push(parse_define_line(rest_original)?);
        }
        // silently skip: output, launch, set, etc.
    }

    Ok(card)
}

/// Parse a single MadGraph process string (e.g. `"p p > e+ e- j QCD<=2 @1"`).
pub fn parse_process_string(s: &str, opts: &ParsingOptions) -> Result<ProcessSpec, ParseError> {
    let mut line = s.trim().to_owned();

    // Step 1: strip process tag @N
    let tag = strip_proc_tag(&mut line)?;

    // Step 2: strip loop spec [...]
    strip_loop_spec(&mut line, opts)?;

    // Step 3: strip coupling order constraints (repeated)
    let coupling_constraints = strip_coupling_orders(&mut line);

    // Step 4: strip / forbidden particles (before $$ and $)
    let forbidden_particles = strip_forbidden_particles(&mut line);

    // Step 5: strip $$ forbidden s-channels (before $ to avoid prefix collision)
    let forbidden_s_channels = strip_forbidden_s_channels(&mut line, opts)?;

    // Step 6: strip $ forbidden on-shell s-channels
    let forbidden_onsh_s_channels = strip_forbidden_onsh_s_channels(&mut line, opts)?;

    // Step 7 & 8: parse the remaining "initial > [required >] final"
    let (initial, required_s_channels, final_state) = parse_process_body(&line)?;

    Ok(ProcessSpec {
        initial,
        required_s_channels,
        final_state,
        forbidden_particles,
        forbidden_s_channels,
        forbidden_onsh_s_channels,
        coupling_constraints,
        tag,
    })
}

/// Parse a `define` line body: `alias = particles... [/ except...]`.
pub fn parse_define_line(s: &str) -> Result<MultiparticleDef, ParseError> {
    let Some(eq_pos) = s.find('=') else {
        return Err(ParseError::NoSeparator(s.to_owned()));
    };
    let alias = s[..eq_pos].trim().to_owned();
    let rhs = s[eq_pos + 1..].trim();

    // Split on `/` to separate particles from except list.
    let (particles_part, except_part) = if let Some(slash) = rhs.find('/') {
        (&rhs[..slash], &rhs[slash + 1..])
    } else {
        (rhs, "")
    };

    let particles = tokenize_names(particles_part);
    let except = tokenize_names(except_part);

    Ok(MultiparticleDef {
        alias,
        particles,
        except,
    })
}

/// Parse an `import model` directive.
/// Examples: "sm", "loop_sm", "sm-no_b_mass", "loop_sm-no_top"
fn parse_model_import(s: &str) -> Result<ModelImport, ParseError> {
    let model_spec = s.trim();

    // Split on the first '-' after the model name to extract restrict variant.
    // Models are typically: "sm", "loop_sm", etc.
    // Variants: "sm-no_b_mass", "loop_sm-no_b_mass"
    if let Some(dash_pos) = model_spec.find('-') {
        let name = model_spec[..dash_pos].to_owned();
        let restrict_variant = model_spec[dash_pos + 1..].to_owned();
        Ok(ModelImport {
            name,
            restrict_variant: Some(restrict_variant),
        })
    } else {
        Ok(ModelImport {
            name: model_spec.to_owned(),
            restrict_variant: None,
        })
    }
}

// ── Stripping helpers ─────────────────────────────────────────────────────────

fn strip_inline_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("")
}

/// Step 1: extract and remove `@N` process tag from the end of `line`.
fn strip_proc_tag(line: &mut String) -> Result<Option<u32>, ParseError> {
    let Some(at) = line.rfind('@') else {
        return Ok(None);
    };
    let tail = line[at + 1..].trim();
    if tail.chars().all(|c| c.is_ascii_digit()) && !tail.is_empty() {
        let tag: u32 = tail.parse().map_err(|_| ParseError::BadTag(line.clone()))?;
        line.truncate(at);
        return Ok(Some(tag));
    }
    // '@' present but not followed by digits — not a process tag.
    Ok(None)
}

/// Step 2: remove `[...]` loop spec from `line`; contents are discarded for LO.
fn strip_loop_spec(line: &mut String, opts: &ParsingOptions) -> Result<(), ParseError> {
    let Some(open) = line.find('[') else {
        return Ok(());
    };
    let Some(close) = line[open..].find(']').map(|i| open + i) else {
        return Err(ParseError::UnclosedLoopSpec(line.clone()));
    };
    if !opts.allow_loop_spec {
        return Err(ParseError::LoopSpecDisabled);
    }
    line.replace_range(open..=close, "");
    Ok(())
}

/// Step 3: extract all `NAME OP VALUE` coupling order constraints from `line`.
///
/// Handles both spaced (`QCD <= 2`) and compact (`QCD<=2`) forms.
/// Only the region after the first `>` is searched, so the process-body `>`
/// separator is never confused with a `>` coupling-order operator.
fn strip_coupling_orders(line: &mut String) -> Vec<CouplingConstraint> {
    let mut constraints = Vec::new();

    loop {
        let Some(gt) = line.find('>') else { break };

        // Own the post-separator region to avoid borrow conflicts.
        let region: String = line[gt + 1..].to_owned();

        if let Some((start_in_region, end_in_region, c)) = find_rightmost_coupling_order(&region) {
            constraints.push(c);
            // Convert region-relative offsets to full-string offsets.
            let abs_start = gt + 1 + start_in_region;
            let abs_end = gt + 1 + end_in_region;
            line.replace_range(abs_start..abs_end, "");
        } else {
            break;
        }
    }

    constraints
}

/// Scan `s` for the rightmost `NAME OP VALUE` pattern (with optional whitespace
/// around OP). Returns `(start, end, constraint)` byte offsets into `s`.
///
/// NAME = `[A-Za-z][A-Za-z0-9_]*(^2)?`
/// OP   = one of `===`, `==`, `<=`, `>=`, `!=`, `<`, `>`, `=`
/// VALUE = `-?\d+`
///
/// The match must start after whitespace or at the start of the string, so that
/// particle names like `e+` or `mu-` in the final-state list are not confused
/// with coupling names.
fn find_rightmost_coupling_order(s: &str) -> Option<(usize, usize, CouplingConstraint)> {
    let bytes = s.as_bytes();
    let mut best: Option<(usize, usize, CouplingConstraint)> = None;
    let mut i = 0;

    while i < s.len() {
        // The coupling name must start at the beginning of the region or after whitespace.
        let preceded_by_ws = i == 0 || bytes[i - 1].is_ascii_whitespace();
        if !preceded_by_ws || !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }

        let name_start = i;

        // Scan the base name: letters, digits, underscores.
        while i < s.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        let base_end = i;

        // Check for optional `^2` squared-order suffix.
        let squared = i + 1 < s.len() && bytes[i] == b'^' && bytes[i + 1] == b'2';
        if squared {
            i += 2;
        }

        let name = s[name_start..base_end].to_owned();

        // Skip optional whitespace before operator.
        let ws_before_op = i;
        while i < s.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        // Try to match an operator.
        let (op, op_len) = if s[i..].starts_with("===") {
            (CouplingOp::StrictEq, 3)
        } else if s[i..].starts_with("==") {
            (CouplingOp::ExactEq, 2)
        } else if s[i..].starts_with("<=") {
            (CouplingOp::Le, 2)
        } else if s[i..].starts_with(">=") {
            (CouplingOp::Ge, 2)
        } else if s[i..].starts_with("!=") {
            (CouplingOp::Ne, 2)
        } else if i < s.len() && bytes[i] == b'<' {
            (CouplingOp::Lt, 1)
        } else if i < s.len() && bytes[i] == b'>' {
            (CouplingOp::Gt, 1)
        } else if i < s.len() && bytes[i] == b'=' {
            (CouplingOp::Eq, 1)
        } else {
            // No operator — reset cursor to just after the name.
            i = ws_before_op;
            continue;
        };
        i += op_len;

        // Skip optional whitespace after operator.
        while i < s.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        // Match optional minus sign and digits.
        let val_start = i;
        if i < s.len() && bytes[i] == b'-' {
            i += 1;
        }
        let digits_start = i;
        while i < s.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }

        if i == digits_start {
            // No digits found — not a valid value. Reset.
            i = ws_before_op;
            continue;
        }

        let value: i64 = s[val_start..i].parse().unwrap_or(0);
        best = Some((
            name_start,
            i,
            CouplingConstraint {
                name,
                squared,
                op,
                value,
            },
        ));
        // Continue scanning — we want the rightmost occurrence.
    }

    best
}

/// Step 4: extract `/` forbidden-particle list (appearing after the last `>`).
fn strip_forbidden_particles(line: &mut String) -> Vec<String> {
    // Find the last `>` first; the `/` must come after it to be a restriction.
    let after_last_gt = match line.rfind('>') {
        Some(pos) => pos + 1,
        None => 0,
    };
    let search_region = &line[after_last_gt..];

    let Some(slash_rel) = search_region.find('/') else {
        return Vec::new();
    };
    let slash = after_last_gt + slash_rel;

    // Stop before `$` or `$$` if present.
    let tail = &line[slash + 1..];
    let stop = tail.find('$').map(|i| slash + 1 + i).unwrap_or(line.len());
    let particles = tokenize_names(&line[slash + 1..stop]);

    line.replace_range(slash..stop, "");
    particles
}

/// Step 5: extract `$$` forbidden s-channel list.
fn strip_forbidden_s_channels(
    line: &mut String,
    opts: &ParsingOptions,
) -> Result<Vec<String>, ParseError> {
    let Some(pos) = line.find("$$") else {
        return Ok(Vec::new());
    };
    if !opts.allow_forbidden_s_channels {
        return Err(ParseError::ForbiddenSChannelDisabled);
    }
    let particles = tokenize_names(&line[pos + 2..]);
    line.truncate(pos);
    Ok(particles)
}

/// Step 6: extract `$` forbidden on-shell s-channel list (after `$$` is gone).
fn strip_forbidden_onsh_s_channels(
    line: &mut String,
    opts: &ParsingOptions,
) -> Result<Vec<String>, ParseError> {
    let Some(pos) = line.find('$') else {
        return Ok(Vec::new());
    };
    if !opts.allow_forbidden_onsh_s_channels {
        return Err(ParseError::ForbiddenOnshSChannelDisabled);
    }
    let particles = tokenize_names(&line[pos + 1..]);
    line.truncate(pos);
    Ok(particles)
}

/// Steps 7–8: split the residual `"initial [> required] > final"` on `>`.
fn parse_process_body(
    line: &str,
) -> Result<(Vec<ParticleLeg>, Vec<String>, Vec<ParticleLeg>), ParseError> {
    // Count `>` in the residual.
    let parts: Vec<&str> = line.splitn(4, '>').collect();
    match parts.len() {
        1 => Err(ParseError::NoSeparator(line.to_owned())),
        2 => {
            let initial = parse_leg_list(parts[0], "initial")?;
            let final_state = parse_leg_list(parts[1], "final")?;
            Ok((initial, Vec::new(), final_state))
        }
        3 => {
            let initial = parse_leg_list(parts[0], "initial")?;
            let required: Vec<String> = tokenize_names(parts[1]);
            let final_state = parse_leg_list(parts[2], "final")?;
            Ok((initial, required, final_state))
        }
        _ => Err(ParseError::TooManySeparators {
            found: parts.len() - 1,
            input: line.to_owned(),
        }),
    }
}

fn parse_leg_list(s: &str, side: &'static str) -> Result<Vec<ParticleLeg>, ParseError> {
    let legs: Result<Vec<_>, _> = s.split_whitespace().map(parse_particle_leg).collect();
    let legs = legs?;
    if legs.is_empty() {
        return Err(ParseError::EmptyLeg { side });
    }
    // Expand duplication: `ParticleLeg { count: 2, name: "e+" }` → `[e+, e+]`
    Ok(legs
        .into_iter()
        .flat_map(|leg| {
            (0..leg.count).map(move |_| ParticleLeg {
                name: leg.name.clone(),
                count: 1,
            })
        })
        .collect())
}

fn parse_particle_leg(tok: &str) -> Result<ParticleLeg, ParseError> {
    if tok.is_empty() {
        return Err(ParseError::BadParticleTok(tok.to_owned()));
    }
    let mut chars = tok.chars();
    let first = chars.next().unwrap();

    // Leading non-zero digit → duplication count.
    if first.is_ascii_digit() && first != '0' {
        let count = first.to_digit(10).unwrap() as usize;
        let name = &tok[1..];
        if name.is_empty() {
            return Err(ParseError::BadParticleTok(tok.to_owned()));
        }
        return Ok(ParticleLeg {
            name: name.to_owned(),
            count,
        });
    }

    // Otherwise: entire token is the particle name (including PDG codes like "11").
    Ok(ParticleLeg {
        name: tok.to_owned(),
        count: 1,
    })
}

/// Split whitespace and return non-empty name tokens.
fn tokenize_names(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> ParsingOptions {
        ParsingOptions {
            allow_forbidden_s_channels: true,
            ..Default::default()
        }
    }

    fn parse(s: &str) -> ProcessSpec {
        parse_process_string(s, &opts()).expect("parse failed")
    }

    #[test]
    fn test_simple_process() {
        let p = parse("e+ e- > mu+ mu-");
        assert_eq!(p.initial.len(), 2);
        assert_eq!(p.final_state.len(), 2);
        assert_eq!(p.initial[0].name, "e+");
        assert_eq!(p.final_state[1].name, "mu-");
        assert!(p.coupling_constraints.is_empty());
        assert!(p.tag.is_none());
    }

    #[test]
    fn test_proc_tag() {
        let p = parse("e+ e- > mu+ mu- @1");
        assert_eq!(p.tag, Some(1));
        assert_eq!(p.initial.len(), 2);
    }

    #[test]
    fn test_coupling_order_le() {
        let p = parse("p p > e+ e- j QCD<=2");
        assert_eq!(p.coupling_constraints.len(), 1);
        let c = &p.coupling_constraints[0];
        assert_eq!(c.name, "QCD");
        assert_eq!(c.op, CouplingOp::Le);
        assert_eq!(c.value, 2);
        assert!(!c.squared);
        // Process body should be parsed correctly despite the order strip.
        assert_eq!(p.initial.len(), 2);
        assert_eq!(p.final_state.len(), 3);
    }

    #[test]
    fn test_coupling_order_gt_disambiguation() {
        // This tests the key ambiguity: `>` after final state is a coupling op, not a separator.
        let p = parse("e+ e- > mu+ mu- QCD > 2");
        assert_eq!(p.coupling_constraints.len(), 1);
        assert_eq!(p.coupling_constraints[0].op, CouplingOp::Gt);
        assert_eq!(p.final_state.len(), 2);
        assert!(p.required_s_channels.is_empty());
    }

    #[test]
    fn test_coupling_order_exact() {
        let p = parse("e+ e- > mu+ mu- QED == 4");
        assert_eq!(p.coupling_constraints[0].op, CouplingOp::ExactEq);
        assert_eq!(p.coupling_constraints[0].value, 4);
    }

    #[test]
    fn test_required_s_channel() {
        let p = parse("e+ e- > Z > mu+ mu-");
        assert_eq!(p.required_s_channels, vec!["Z"]);
        assert_eq!(p.initial.len(), 2);
        assert_eq!(p.final_state.len(), 2);
    }

    #[test]
    fn test_forbidden_particles() {
        let p = parse("p p > e+ e- / t");
        assert_eq!(p.forbidden_particles, vec!["t"]);
        assert_eq!(p.initial.len(), 2);
    }

    #[test]
    fn test_forbidden_hard_s_channel() {
        let p = parse("p p > e+ e- $$ Z");
        assert_eq!(p.forbidden_s_channels, vec!["Z"]);
    }

    #[test]
    fn test_forbidden_hard_s_channel_disabled() {
        let strict = ParsingOptions::default(); // allow_forbidden_s_channels = false
        let result = parse_process_string("p p > e+ e- $$ Z", &strict);
        assert!(matches!(result, Err(ParseError::ForbiddenSChannelDisabled)));
    }

    #[test]
    fn test_forbidden_onsh_s_channel() {
        let p = parse("p p > e+ e- $ Z");
        assert_eq!(p.forbidden_onsh_s_channels, vec!["Z"]);
    }

    #[test]
    fn test_duplication() {
        let p = parse("2e+ > mu+ mu-");
        assert_eq!(p.initial.len(), 2);
        assert!(p.initial.iter().all(|l| l.name == "e+"));
    }

    #[test]
    fn test_multiple_coupling_constraints() {
        let p = parse("p p > t t~ QCD<=2 QED==0");
        assert_eq!(p.coupling_constraints.len(), 2);
    }

    #[test]
    fn test_squared_order() {
        let p = parse("e+ e- > mu+ mu- QCD^2 <= 4");
        assert_eq!(p.coupling_constraints[0].squared, true);
        assert_eq!(p.coupling_constraints[0].name, "QCD");
    }

    #[test]
    fn test_proc_card_basic() {
        let card = r#"
# A simple proc card
generate e+ e- > mu+ mu-
add process e+ e- > ta+ ta-
define myp = u d
"#;
        let parsed = parse_proc_card(card, &opts()).expect("proc_card parse failed");
        assert_eq!(parsed.processes.len(), 2);
        assert_eq!(parsed.defines.len(), 1);
        assert_eq!(parsed.defines[0].alias, "myp");
        assert_eq!(parsed.defines[0].particles, vec!["u", "d"]);
    }

    #[test]
    fn test_define_with_except() {
        let d = parse_define_line("q = p / g").unwrap();
        assert_eq!(d.alias, "q");
        assert_eq!(d.except, vec!["g"]);
    }

    #[test]
    fn test_loop_spec_silently_ignored() {
        let p = parse("p p > e+ e- [QCD]");
        // Loop spec stripped; process body still parsed.
        assert_eq!(p.initial.len(), 2);
    }

    #[test]
    fn test_loop_spec_disabled() {
        let no_loop = ParsingOptions {
            allow_loop_spec: false,
            ..opts()
        };
        let result = parse_process_string("p p > e+ e- [QCD]", &no_loop);
        assert!(matches!(result, Err(ParseError::LoopSpecDisabled)));
    }

    #[test]
    fn test_model_import_basic() {
        let card = r#"
import model sm
generate e+ e- > mu+ mu-
"#;
        let parsed = parse_proc_card(card, &opts()).expect("proc_card parse failed");
        assert!(parsed.model.is_some());
        let model = parsed.model.unwrap();
        assert_eq!(model.name, "sm");
        assert_eq!(model.restrict_variant, None);
    }

    #[test]
    fn test_model_import_with_variant() {
        let card = r#"
import model sm-no_b_mass
generate e+ e- > mu+ mu-
"#;
        let parsed = parse_proc_card(card, &opts()).expect("proc_card parse failed");
        assert!(parsed.model.is_some());
        let model = parsed.model.unwrap();
        assert_eq!(model.name, "sm");
        assert_eq!(model.restrict_variant, Some("no_b_mass".to_string()));
    }
}
