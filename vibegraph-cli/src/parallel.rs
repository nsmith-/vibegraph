//! Worker-thread count for the commands that enumerate or integrate.
//!
//! The pool is built once, before any work starts, because rayon's global pool is
//! immutable after its first use — which is why the flag is applied at the top of
//! each command rather than where the work happens.
//!
//! What it currently buys differs by command: `integrate` spends nearly all of its
//! time in the per-channel VEGAS adaptations, which run on the pool. `generate`
//! compiles its subprocesses on the pool and replays frozen grids through a
//! serial accept/reject pass; it takes the flag so a driver can set one thread
//! count for a whole pipeline. It enumerates nothing (the diagram sets are the
//! artifact's), so it takes `-j` alone ([`ThreadArgs`]) and refuses
//! `--parallel-diagrams` as an unknown argument.
//!
//! Without `-j` the pool is sized to what the process may use, which is the
//! smallest of the CPUs it is allowed to run on (the affinity mask), the CPU
//! quota of its cgroup (v1 and v2), and `OMP_NUM_THREADS` when that is set to
//! a positive integer. The first two are what
//! [`std::thread::available_parallelism`] reads from the kernel, so a batch
//! slot that confines a job by `taskset`, `cpuset` or a quota is honoured
//! without parsing `/proc` or `/sys` here; `OMP_NUM_THREADS` is what HTCondor
//! and most schedulers export as the job's `request_cpus`. `RAYON_NUM_THREADS`,
//! when set, is an explicit choice and wins over all three, as it does for any
//! rayon program. The run logs the count and what set it.
//!
//! The thread count is a scheduling knob and nothing more: the parallel VEGAS
//! adaptation ([`vibegraph::vegas`]) reproduces the sequential draw and
//! accumulation order whatever the pool size,
//! so `-j 1` and `-j 16` write byte-identical artifacts. That is what makes a
//! single-threaded validation run a measurement of the numbers the parallel
//! command produces.
//!
//! Diagram enumeration decides per process-line walk: a walk large enough to
//! gain from the pool ([`vibegraph::diagrams::AUTO_PARALLEL_COST`]) assigns its
//! subprocesses across it, and a smaller one runs on one thread, since
//! feyngraph's internal fan-out is short and contended: a 2→3 process
//! enumerates *slower* on sixteen threads than on one. `--parallel-diagrams`
//! hands every walk the pool. See [`EnumerationPool`] for the measurements.

use clap::Args;
use vibegraph::diagrams::EnumerationPool;

/// What set the worker count when `-j` did not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadSource {
    /// `RAYON_NUM_THREADS`.
    RayonEnv,
    /// `OMP_NUM_THREADS`, below what the process could otherwise use.
    OmpNumThreads,
    /// The affinity mask and cgroup CPU quota, as the standard library reads
    /// them (the CPUs of the machine when neither confines the process).
    Available,
}

impl ThreadSource {
    fn describe(self) -> &'static str {
        match self {
            ThreadSource::RayonEnv => "RAYON_NUM_THREADS",
            ThreadSource::OmpNumThreads => "OMP_NUM_THREADS",
            ThreadSource::Available => "the CPU affinity mask and cgroup quota",
        }
    }
}

/// The first positive integer of an environment value: `OMP_NUM_THREADS` may
/// be a comma-separated list, one entry per nesting level, of which the
/// outermost is the one that sizes this pool. Anything else is not a count.
fn first_count(value: &str) -> Option<usize> {
    value
        .split(',')
        .next()
        .and_then(|first| first.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
}

/// The default worker count and what decided it, from the `available`
/// parallelism the standard library reports and the two environment values.
fn default_threads(
    available: usize,
    omp: Option<&str>,
    rayon: Option<&str>,
) -> (usize, ThreadSource) {
    if let Some(n) = rayon.and_then(first_count) {
        return (n, ThreadSource::RayonEnv);
    }
    match omp.and_then(first_count) {
        Some(n) if n < available => (n, ThreadSource::OmpNumThreads),
        _ => (available.max(1), ThreadSource::Available),
    }
}

/// The `-j/--parallel` flag alone, for a command that enumerates nothing.
#[derive(Args, Debug, Clone, Copy)]
pub(crate) struct ThreadArgs {
    /// Worker threads (default: the smallest of the CPUs the process may run
    /// on, its cgroup CPU quota and `$OMP_NUM_THREADS`). Results do not depend on it: any
    /// thread count produces byte-identical output.
    #[arg(long = "parallel", short = 'j', value_name = "N")]
    pub(crate) parallel: Option<usize>,
}

/// The `-j/--parallel` flag and the enumeration's own, shared by every command
/// that enumerates.
#[derive(Args, Debug, Clone, Copy)]
pub(crate) struct ParallelArgs {
    #[command(flatten)]
    pub(crate) threads: ThreadArgs,

    /// Enumerate every process line's diagrams across the worker pool. Without
    /// it, a line large enough to gain is enumerated across the pool and a
    /// smaller one on one thread, where the fan-out is a slowdown. Timing only:
    /// the artifact is identical either way.
    #[arg(long = "parallel-diagrams")]
    pub(crate) parallel_diagrams: bool,
}

impl ParallelArgs {
    /// The pool diagram enumeration runs on.
    pub(crate) fn enumeration(&self) -> EnumerationPool {
        if self.parallel_diagrams {
            EnumerationPool::Ambient
        } else {
            EnumerationPool::Auto
        }
    }

    /// Size the global rayon pool ([`ThreadArgs::install`]).
    pub(crate) fn install(&self) -> Result<(), String> {
        self.threads.install()
    }
}

impl ThreadArgs {
    /// Size the global rayon pool, before anything touches it.
    ///
    /// A number the platform cannot honour is a refusal rather than a silent
    /// fallback to the default: a run asked for a thread count is usually being
    /// timed, and quietly giving it another one would misattribute the result.
    pub(crate) fn install(&self) -> Result<(), String> {
        let Some(n) = self.parallel else {
            let available = std::thread::available_parallelism().map_or(1, |n| n.get());
            let (n, source) = default_threads(
                available,
                std::env::var("OMP_NUM_THREADS").ok().as_deref(),
                std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
            );
            tracing::info!("threads: {n} (from {})", source.describe());
            return rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()
                .map_err(|e| format!("cannot size the worker pool to {n} threads: {e}"));
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
    fn the_default_is_the_smallest_limit_and_names_it() {
        // Neither environment value: what the platform reports, which already
        // folds the affinity mask and the cgroup quota into one number.
        assert_eq!(default_threads(8, None, None), (8, ThreadSource::Available));
        assert_eq!(default_threads(0, None, None), (1, ThreadSource::Available));
        // OMP_NUM_THREADS below the platform's count wins; at or above it, the
        // platform's limit is the binding one.
        assert_eq!(
            default_threads(8, Some("3"), None),
            (3, ThreadSource::OmpNumThreads)
        );
        assert_eq!(
            default_threads(8, Some("8"), None),
            (8, ThreadSource::Available)
        );
        assert_eq!(
            default_threads(2, Some("16"), None),
            (2, ThreadSource::Available)
        );
        // The outermost entry of a list sizes the pool; a value that is not a
        // positive integer is not a limit.
        assert_eq!(
            default_threads(8, Some("4,2"), None),
            (4, ThreadSource::OmpNumThreads)
        );
        for junk in ["", "0", "-1", "many", " "] {
            assert_eq!(
                default_threads(8, Some(junk), None),
                (8, ThreadSource::Available),
                "{junk:?}"
            );
        }
        // An explicit rayon count outranks both.
        assert_eq!(
            default_threads(8, Some("3"), Some("6")),
            (6, ThreadSource::RayonEnv)
        );
        assert_eq!(
            default_threads(8, Some("3"), Some("0")),
            (3, ThreadSource::OmpNumThreads)
        );
    }

    #[test]
    fn enumeration_decides_per_walk_unless_the_flag_asks_otherwise() {
        let off = ParallelArgs {
            threads: ThreadArgs { parallel: Some(16) },
            parallel_diagrams: false,
        };
        assert_eq!(off.enumeration(), EnumerationPool::Auto);
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
