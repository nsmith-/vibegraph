//! Regenerate the interned SM assets under `vibegraph-lib/src/ufo/sm_assets/`.
//!
//! Reads the SM UFO model from the `research/refs/mg5amcnlo` submodule, parses it
//! with the crate's own parser, and writes the compressed pre-restriction blob plus
//! the nine restrict cards into the committed source tree. Normal builds only
//! `include_bytes!`/`include_str!` those files; this binary is the only thing that
//! reads the submodule.
//!
//! Run after the submodule SM model changes:
//!
//! ```text
//! cargo run -p vibegraph-lib --bin gen_sm_blob
//! # or with explicit paths:
//! cargo run -p vibegraph-lib --bin gen_sm_blob -- <sm_dir> <out_dir>
//! ```

use std::path::PathBuf;

use vibegraph::ufo::sm;

fn main() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let args: Vec<String> = std::env::args().skip(1).collect();

    let sm_dir = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(manifest).join("../research/refs/mg5amcnlo/models/sm"));
    let out_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(manifest).join("src/ufo/sm_assets"));

    if !sm_dir.exists() {
        eprintln!(
            "SM UFO model not found at {}\n\
             Run: git submodule update --init --recursive",
            sm_dir.display()
        );
        std::process::exit(1);
    }

    match sm::regenerate(&sm_dir, &out_dir) {
        Ok(()) => {
            let blob = out_dir.join("sm_parsed.bin.zst");
            let size = std::fs::metadata(&blob).map(|m| m.len()).unwrap_or(0);
            println!(
                "Wrote {} ({} bytes) + 9 restrict cards to {}",
                blob.display(),
                size,
                out_dir.display()
            );
        }
        Err(e) => {
            eprintln!("gen_sm_blob failed: {e}");
            std::process::exit(1);
        }
    }
}
