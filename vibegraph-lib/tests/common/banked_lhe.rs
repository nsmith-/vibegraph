//! The banked MadGraph event files: reading one, and the run card its banner
//! records.

use std::io::Read;
use std::path::{Path, PathBuf};

/// `validation/madgraph/output/<run>/Events/<events>`.
pub fn banked_events_dir(run: &str, events: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/output")
        .join(run)
        .join("Events")
        .join(events)
}

/// An event file, plain or gzipped.
pub fn read_lhe(path: &Path) -> String {
    let file = std::fs::File::open(path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; fetch the banked runs with `pixi run fetch-refdata`",
            path.display()
        )
    });
    let mut text = String::new();
    if path.extension().is_some_and(|e| e == "gz") {
        flate2::read::GzDecoder::new(file).read_to_string(&mut text)
    } else {
        std::io::BufReader::new(file).read_to_string(&mut text)
    }
    .expect("the file reads");
    text
}

/// The banked run's own event file, `Events/run_01/unweighted_events.lhe.gz`.
pub fn banked_text(run: &str) -> String {
    read_lhe(&banked_events_dir(run, "run_01").join("unweighted_events.lhe.gz"))
}

/// The `<MGRunCard>` text the banner records.
pub fn banner_card(text: &str) -> String {
    let start = text.find("<MGRunCard>").expect("an <MGRunCard>") + "<MGRunCard>".len();
    let end = text.find("</MGRunCard>").expect("</MGRunCard>");
    text[start..end].replace("<![CDATA[", "").replace("]]>", "")
}
