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

use clap::Args;

/// The `-j/--parallel` flag, shared by every command that integrates.
#[derive(Args, Debug, Clone, Copy)]
pub struct ParallelArgs {
    /// Worker threads (default: one per core). Results do not depend on it: any
    /// thread count produces byte-identical output.
    #[arg(long = "parallel", short = 'j', value_name = "N")]
    pub parallel: Option<usize>,
}

impl ParallelArgs {
    /// Size the global rayon pool, before anything touches it.
    ///
    /// A number the platform cannot honour is a refusal rather than a silent
    /// fallback to the default: a run asked for a thread count is usually being
    /// timed, and quietly giving it another one would misattribute the result.
    pub fn install(&self) -> Result<(), String> {
        let Some(n) = self.parallel else { return Ok(()) };
        if n == 0 {
            return Err("--parallel needs at least one thread".to_string());
        }
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| format!("cannot size the worker pool to {n} threads: {e}"))
    }
}
