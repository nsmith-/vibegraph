//! Whether a missing asset may be downloaded, and what a refusal says.
//!
//! Every download this binary performs is an asset the user did not ask for by
//! path — a PDF set named only by its set name, resolved on their behalf. That
//! makes consent, not reachability, the thing to decide, so the decision is a
//! pure function of three inputs: what the flags and environment say, whether
//! there is a terminal to ask on, and (when there is) what the user answered.
//!
//! # Defaults
//!
//! Absent an explicit answer the policy is [`NetworkPolicy::Ask`], and asking
//! **without a terminal is a refusal**. A test harness, a CI job, a cron entry
//! and a container build all run without one, so none of them can start a
//! download by accident: reaching the network from a non-interactive context
//! takes an explicit `-y` on the command line. The refusal always names the
//! switch that would have allowed it, since "it refused and I cannot see why"
//! is the failure mode that costs a user the most time.
//!
//! [`decide`] takes its terminal-ness and its input stream as parameters rather
//! than reading `stdin` itself, so the whole matrix is exercised offline by the
//! tests below; [`confirm`] is the thin wrapper that supplies the real ones.
//!
//! With the live status display up the streams are not askable at all — the
//! terminal is in raw mode and the display's thread is reading its keys — so
//! [`confirm`] puts the question through the display instead, with the same
//! terms in the scrollback and the same decline text when the answer is no.

use std::io::{BufRead, IsTerminal, Write};

/// Set to any value to forbid downloads. Anything that must stay offline — a
/// test run, a sandboxed build — sets this and cannot then reach the network,
/// whatever the command line says.
pub const NO_NETWORK_VAR: &str = "VIBEGRAPH_NO_NETWORK";
/// Command-line spelling of the same refusal.
pub const NO_NETWORK_FLAG: &str = "--no-network";
/// Command-line consent, standing in for a "yes" at the prompt. Named by its
/// short spelling in every message (`--yes` is the long form of the same flag).
pub const CONSENT_FLAG: &str = "-y";

/// What forbade a download, so a refusal can name the one thing the user has to
/// change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    Flag,
    Env,
}

impl Denial {
    fn describe(self) -> &'static str {
        match self {
            Denial::Flag => NO_NETWORK_FLAG,
            Denial::Env => NO_NETWORK_VAR,
        }
    }

    fn remedy(self) -> String {
        match self {
            Denial::Flag => format!("drop {NO_NETWORK_FLAG}"),
            Denial::Env => format!("unset ${NO_NETWORK_VAR}"),
        }
    }
}

/// How this run answers a "may I download this?" question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// Never; `Denial` says what forbade it.
    Deny(Denial),
    /// Ask, if there is a terminal to ask on.
    Ask,
    /// Consent was given up front on the command line.
    Allow,
}

impl NetworkPolicy {
    /// Resolve the policy from the two flags and the environment.
    ///
    /// Refusal wins over consent whenever both are expressed, and the flag is
    /// reported over the environment variable when both refuse: the remedy the
    /// message names has to be one that actually unblocks the run, and dropping
    /// the flag while the variable is still set would not.
    pub fn resolve(no_network_flag: bool, consent_flag: bool, env_denies: bool) -> Self {
        if no_network_flag {
            NetworkPolicy::Deny(Denial::Flag)
        } else if env_denies {
            NetworkPolicy::Deny(Denial::Env)
        } else if consent_flag {
            NetworkPolicy::Allow
        } else {
            NetworkPolicy::Ask
        }
    }

    /// Resolve against the real environment.
    pub fn from_env(no_network_flag: bool, consent_flag: bool) -> Self {
        Self::resolve(
            no_network_flag,
            consent_flag,
            std::env::var_os(NO_NETWORK_VAR).is_some(),
        )
    }
}

/// A download the user is being asked to authorise, in the terms the pin
/// records it: where it comes from, how big it is, and what it must hash to.
#[derive(Clone, Copy, Debug)]
pub struct Download<'a> {
    /// What the bytes are, phrased for a sentence: `"PDF set NNPDF23_…"`.
    pub what: &'a str,
    pub url: &'a str,
    pub bytes: u64,
    pub sha256: &'a str,
    /// Where the unpacked archive would land, which is also where a user who
    /// downloads it themselves should put it.
    pub destination: &'a str,
}

impl Download<'_> {
    fn megabytes(&self) -> f64 {
        self.bytes as f64 / (1024.0 * 1024.0)
    }

    /// The URL, size, checksum and destination, one per line. Shown before the
    /// question and repeated in every refusal, so a user who cannot or will not
    /// let the binary fetch it has everything needed to do it themselves.
    pub fn terms(&self) -> String {
        format!(
            "  source:  {}\n  size:    {:.1} MB\n  sha256:  {}\n  unpacks to: {}",
            self.url,
            self.megabytes(),
            self.sha256,
            self.destination
        )
    }

    /// The question in one line, for a display whose answer row has no room for
    /// the terms — those go into the scrollback as [`Download::notice`].
    pub fn question(&self) -> String {
        format!("download {} ({:.1} MB)?", self.what, self.megabytes())
    }

    /// What precedes the question, line by line: that the asset is missing, and
    /// the terms of fetching it.
    pub fn notice(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "{} is not available locally. It can be downloaded now:",
            self.what
        )];
        lines.extend(self.terms().lines().map(str::to_string));
        lines
    }
}

/// The refusal for a download the user was asked about and said no to,
/// identical whichever surface put the question.
fn declined(download: &Download<'_>) -> String {
    format!(
        "{} is not available locally and the download was declined;\n{}\nTo allow it, \
         answer `y` or pass {CONSENT_FLAG}.",
        download.what,
        download.terms()
    )
}

/// The answer to one download question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Consent {
    Granted,
    /// Refused, carrying the message explaining why and what would change it.
    Refused(String),
}

impl Consent {
    #[cfg(test)]
    fn granted(&self) -> bool {
        matches!(self, Consent::Granted)
    }
}

/// Answers `y`/`yes`, case-insensitively; anything else — including an empty
/// line and a closed stream — is a no.
fn is_yes(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Decide whether `download` may proceed.
///
/// `interactive` is whether a prompt can be both shown and answered; `input`
/// and `out` are the streams it would use.
pub fn decide(
    policy: NetworkPolicy,
    download: &Download<'_>,
    interactive: bool,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Consent {
    match policy {
        NetworkPolicy::Allow => Consent::Granted,
        NetworkPolicy::Deny(by) => Consent::Refused(format!(
            "{} is not available locally and downloads are disabled by {};\n{}\nTo let vibegraph \
             fetch it, {}; to install it yourself, unpack that archive at the path above.",
            download.what,
            by.describe(),
            download.terms(),
            by.remedy(),
        )),
        NetworkPolicy::Ask if !interactive => Consent::Refused(format!(
            "{} is not available locally, and there is no terminal to ask on;\n{}\nTo let \
             vibegraph fetch it, pass {CONSENT_FLAG}; to install it yourself, unpack that archive \
             at the path above.",
            download.what,
            download.terms(),
        )),
        NetworkPolicy::Ask => {
            let _ = writeln!(
                out,
                "{} is not available locally. It can be downloaded now:\n{}",
                download.what,
                download.terms()
            );
            let _ = write!(out, "Download it? [y/N] ");
            let _ = out.flush();

            let mut answer = String::new();
            match input.read_line(&mut answer) {
                Ok(0) | Err(_) => Consent::Refused(format!(
                    "{} is not available locally and the prompt could not be read; \
                     pass {CONSENT_FLAG} to allow the download without asking",
                    download.what
                )),
                Ok(_) if is_yes(&answer) => Consent::Granted,
                Ok(_) => Consent::Refused(declined(download)),
            }
        }
    }
}

/// [`decide`], against the process's own streams — unless the live status
/// display holds the terminal, in which case an [`NetworkPolicy::Ask`] goes
/// through the display's own question row instead: raw mode leaves `stdin`
/// nothing line-shaped to read, and the display's thread is the one reading
/// keys.
///
/// On the stream path, both `stdin` and `stderr` have to be terminals for this
/// to count as interactive: a redirected `stdin` cannot answer, and a
/// redirected `stderr` hides the question. The prompt goes to `stderr` so that
/// piping the command's `stdout` never swallows it.
pub fn confirm(policy: NetworkPolicy, download: &Download<'_>) -> Consent {
    if policy == NetworkPolicy::Ask {
        if let Some(granted) = crate::tui::ask_to_download(&download.question(), download.notice())
        {
            return if granted {
                Consent::Granted
            } else {
                Consent::Refused(declined(download))
            };
        }
    }
    let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut err = std::io::stderr();
    decide(policy, download, interactive, &mut input, &mut err)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIN: Download<'static> = Download {
        what: "PDF set TestSet",
        url: "https://example.invalid/TestSet.tar.gz",
        bytes: 27_625_668,
        sha256: "abc123",
        destination: "/tmp/vibegraph-home/pdf/TestSet",
    };

    fn run(policy: NetworkPolicy, interactive: bool, typed: &str) -> (Consent, String) {
        let mut input = typed.as_bytes();
        let mut out: Vec<u8> = Vec::new();
        let consent = decide(policy, &PIN, interactive, &mut input, &mut out);
        (consent, String::from_utf8(out).unwrap())
    }

    #[test]
    fn flags_and_env_resolve_to_a_policy() {
        assert_eq!(
            NetworkPolicy::resolve(false, false, false),
            NetworkPolicy::Ask
        );
        assert_eq!(
            NetworkPolicy::resolve(false, true, false),
            NetworkPolicy::Allow
        );
        assert_eq!(
            NetworkPolicy::resolve(true, false, false),
            NetworkPolicy::Deny(Denial::Flag)
        );
        assert_eq!(
            NetworkPolicy::resolve(false, false, true),
            NetworkPolicy::Deny(Denial::Env)
        );
    }

    /// Consent and refusal are not symmetric: whichever way they are combined,
    /// the refusal stands. A `--yes` inherited from a wrapper script must not be
    /// able to undo an offline environment.
    #[test]
    fn refusal_outranks_consent_however_it_is_expressed() {
        assert_eq!(
            NetworkPolicy::resolve(true, true, false),
            NetworkPolicy::Deny(Denial::Flag)
        );
        assert_eq!(
            NetworkPolicy::resolve(false, true, true),
            NetworkPolicy::Deny(Denial::Env)
        );
        assert_eq!(
            NetworkPolicy::resolve(true, true, true),
            NetworkPolicy::Deny(Denial::Flag),
            "with both refusals in play, the message must name the flag, since \
             unsetting the variable alone would not unblock the run"
        );
    }

    /// The property the whole module exists for. A non-interactive run under the
    /// default policy refuses, so no unattended context can start a download.
    #[test]
    fn the_default_policy_refuses_without_a_terminal() {
        let (consent, out) = run(NetworkPolicy::Ask, false, "y\n");
        assert!(!consent.granted());
        assert_eq!(out, "", "a refusal without a terminal must ask nothing");
        let Consent::Refused(msg) = consent else {
            unreachable!()
        };
        assert!(msg.contains(CONSENT_FLAG), "refusal must name -y: {msg}");
    }

    /// A non-interactive run does not become interactive because something is
    /// piped into it: input that says yes is not consent when the question was
    /// never put.
    #[test]
    fn piped_input_does_not_grant_consent() {
        assert!(!run(NetworkPolicy::Ask, false, "yes\n").0.granted());
    }

    #[test]
    fn an_interactive_yes_grants_and_anything_else_refuses() {
        for answer in ["y\n", "Y\n", "yes\n", "YES\n", " y \n"] {
            assert!(
                run(NetworkPolicy::Ask, true, answer).0.granted(),
                "{answer:?}"
            );
        }
        for answer in ["\n", "n\n", "no\n", "q\n", "yeah\n", ""] {
            assert!(
                !run(NetworkPolicy::Ask, true, answer).0.granted(),
                "{answer:?}"
            );
        }
    }

    /// The prompt has to carry the three things that let a user judge it, and
    /// they have to survive into the refusal too — a user who says no still
    /// needs the URL to fetch it themselves.
    #[test]
    fn the_prompt_and_its_refusal_both_state_url_size_and_checksum() {
        let (consent, shown) = run(NetworkPolicy::Ask, true, "n\n");
        let Consent::Refused(msg) = consent else {
            unreachable!()
        };
        for text in [PIN.url, PIN.sha256, PIN.destination, "26.3 MB"] {
            assert!(shown.contains(text), "prompt should state {text}: {shown}");
            assert!(msg.contains(text), "refusal should state {text}: {msg}");
        }
        assert!(
            shown.contains("[y/N]"),
            "the default must be visible: {shown}"
        );
    }

    /// The display splits the prompt into a one-line question and the notice
    /// lines above it; between them they must carry the same facts the stream
    /// prompt shows in one piece.
    #[test]
    fn the_display_question_and_notice_carry_the_same_facts_as_the_prompt() {
        assert_eq!(PIN.question(), "download PDF set TestSet (26.3 MB)?");
        let notice = PIN.notice().join("\n");
        assert!(
            notice.starts_with("PDF set TestSet is not available locally"),
            "{notice}"
        );
        for text in [PIN.url, PIN.sha256, PIN.destination] {
            assert!(notice.contains(text), "notice should state {text}: {notice}");
        }
    }

    /// Each refusal names the one switch that produced it, not a menu of every
    /// switch that could have.
    #[test]
    fn each_refusal_names_what_would_allow_it() {
        let flag = run(NetworkPolicy::Deny(Denial::Flag), true, "y\n").0;
        let Consent::Refused(flag_msg) = flag else {
            unreachable!()
        };
        assert!(flag_msg.contains(NO_NETWORK_FLAG));
        assert!(flag_msg.contains("drop"));

        let env = run(NetworkPolicy::Deny(Denial::Env), true, "y\n").0;
        let Consent::Refused(env_msg) = env else {
            unreachable!()
        };
        assert!(env_msg.contains(NO_NETWORK_VAR));
        assert!(env_msg.contains("unset"));
    }

    /// An explicit refusal never asks, even at a terminal, and never consumes
    /// the answer to a question it did not put.
    #[test]
    fn a_denied_policy_asks_nothing() {
        let (_, out) = run(NetworkPolicy::Deny(Denial::Env), true, "y\n");
        assert_eq!(out, "");
    }

    #[test]
    fn up_front_consent_asks_nothing_either() {
        let (consent, out) = run(NetworkPolicy::Allow, true, "");
        assert!(consent.granted());
        assert_eq!(out, "");
    }

    /// A terminal whose input stream ends immediately (a closed pipe on a
    /// controlling tty) is a refusal, not a hang and not a default yes.
    #[test]
    fn a_closed_input_stream_refuses() {
        assert!(!run(NetworkPolicy::Ask, true, "").0.granted());
    }
}
