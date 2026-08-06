//! Worker-thread count for the commands that integrate.
//!
//! The pool is built once, before any work starts, because rayon's global pool is
//! immutable after its first use — which is why the flag is applied at the top of
//! each command rather than where the work happens.
//!
//! What it currently buys differs by command: `integrate` spends nearly all of its
//! time in the per-channel VEGAS adaptations, which run on the pool. `generate`
//! replays frozen grids through a serial accept/reject pass, so the flag sizes its
//! pool and changes nothing else; it is accepted there so a driver can set one
//! thread count for a whole pipeline.
//!
//! The thread count is a scheduling knob and nothing more:
//! [`VegasGrid::adapt_parallel_seeded`](vibegraph::vegas::VegasGrid::adapt_parallel_seeded)
//! reproduces the sequential draw and accumulation order whatever the pool size,
//! so `-j 1` and `-j 16` write byte-identical artifacts. That is what makes a
//! single-threaded validation run a measurement of the numbers the parallel
//! command produces.
//!
//! Diagram enumeration is the exception, and runs on one thread whatever `-j`
//! says. feyngraph parallelises the topology search and the per-assignment
//! diagram construction internally, but those sections are short and contended:
//! a 2→3 process enumerates *slower* on sixteen threads than on one, and only a
//! process large enough for enumeration to cost seconds gains from the fan-out.
//! `--parallel-diagrams` hands it the pool for that case. See
//! [`EnumerationPool`] for the measurements.

use clap::Args;
use vibegraph::diagrams::EnumerationPool;

/// The `-j/--parallel` flag, shared by every command that integrates.
#[derive(Args, Debug, Clone, Copy)]
pub struct ParallelArgs {
    /// Worker threads (default: one per core). Results do not depend on it: any
    /// thread count produces byte-identical output.
    #[arg(long = "parallel", short = 'j', value_name = "N")]
    pub parallel: Option<usize>,

    /// Enumerate diagrams on the worker pool too, instead of on one thread. Pays
    /// off only for processes whose enumeration costs seconds; below that the
    /// fan-out is a slowdown. Timing only: the artifact is identical either way.
    #[arg(long = "parallel-diagrams")]
    pub parallel_diagrams: bool,
}

impl ParallelArgs {
    /// The pool diagram enumeration runs on.
    pub fn enumeration(&self) -> EnumerationPool {
        if self.parallel_diagrams {
            EnumerationPool::Ambient
        } else {
            EnumerationPool::Serial
        }
    }

    /// Size the global rayon pool, before anything touches it.
    ///
    /// A number the platform cannot honour is a refusal rather than a silent
    /// fallback to the default: a run asked for a thread count is usually being
    /// timed, and quietly giving it another one would misattribute the result.
    pub fn install(&self) -> Result<(), String> {
        let Some(n) = self.parallel else {
            return Ok(());
        };
        if n == 0 {
            return Err("--parallel needs at least one thread".to_string());
        }
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| format!("cannot size the worker pool to {n} threads: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumeration_is_serial_unless_the_flag_asks_otherwise() {
        let off = ParallelArgs {
            parallel: Some(16),
            parallel_diagrams: false,
        };
        assert_eq!(off.enumeration(), EnumerationPool::Serial);
        assert_eq!(
            ParallelArgs {
                parallel_diagrams: true,
                ..off
            }
            .enumeration(),
            EnumerationPool::Ambient
        );
    }
}
