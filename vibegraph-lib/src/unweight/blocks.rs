//! Accept/reject over fixed-size blocks of trials, run in parallel and taken in
//! block order.
//!
//! The trial sequence is cut into blocks of [`BLOCK_TRIALS`] trials. Block `b`
//! is a pure function of the seed, its index and the frozen maxima: its trials
//! draw from two counter-based substreams opened at offsets fixed by `b` alone,
//! the way an integration's chunk of points opens its grid stream at the chunk's
//! first point times the draws a point consumes ([`crate::phasespace::rng`]).
//!
//! - The trial stream, at draw `b · BLOCK_TRIALS · stride`. A trial in channel
//!   `j` consumes `2 + channel_grid_ndim(j)` draws (channel, grid coordinates,
//!   acceptance) and an accepted one a further `event_draws` for its event's
//!   discrete labels, so `stride = 2 + max_j channel_grid_ndim(j) + event_draws`
//!   bounds every trial and no block reaches into the next block's draws.
//! - The trailing-uniform stream, at draw `b · BLOCK_TRIALS · scale_draw_ndim`:
//!   every trial takes exactly `scale_draw_ndim` of them, the per-point layout an
//!   integration gives its scale-draw stream.
//!
//! Blocks are computed a batch at a time on the worker pool and handed out in
//! block order, each block's accepted points in trial order. The statistics of
//! the pass are those of the trials up to and including the one that produced
//! the last point handed out: every earlier block's in full, merged in block
//! order, and the current block's as they stood after that trial. Blocks computed
//! past that point are speculative and contribute nothing. Which thread ran a
//! block, how many threads there are, and how far ahead the batch reached are
//! therefore invisible in everything the pass returns.

use std::collections::VecDeque;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use super::{AcceptedPoint, ChannelIntegrand, UnweightStats, Unweighter};
use crate::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE, WORDS_PER_DRAW};

/// Trials per block.
///
/// Small enough that a block of the cheapest trials (a `2 → 2` matrix element at
/// well under a microsecond) costs far more than the dispatch, merge and stream
/// seek it carries, and that the speculative tail of the last batch is a small
/// fraction of any sample worth generating in parallel; large enough that a
/// block of `2 → 6` trials at a few hundred microseconds each is still a fraction
/// of a second, so the batch barrier balances.
pub const BLOCK_TRIALS: u64 = 1024;

/// Blocks per worker thread in one batch: enough that a thread finishing early
/// steals another block rather than idling at the barrier, few enough that the
/// speculative work past the last event stays small.
const BLOCKS_PER_THREAD: usize = 4;

/// The trial stream's index under the pass's seed.
const TRIAL_STREAM: u64 = 0;

/// One accepted point of a block, with what the caller's preparation made of it
/// and the block's statistics as they stood after its trial.
struct BlockEvent<T> {
    /// The trial's index within its block.
    trial: u64,
    point: AcceptedPoint,
    item: T,
    stats: UnweightStats,
}

struct Block<T> {
    events: VecDeque<BlockEvent<T>>,
    stats: UnweightStats,
}

/// Where a pass's blocks draw from.
#[derive(Clone, Copy, Debug)]
struct Layout {
    seed: u64,
    /// The most trial-stream draws one trial may consume.
    stride: u64,
    event_draws: usize,
}

/// An accept/reject pass over frozen maxima whose output is fixed by its seed
/// and independent of the worker pool's size.
///
/// Each accepted point is handed to the caller's `prepare` on the worker that
/// accepted it, together with `event_draws` uniforms from the trial stream for
/// whatever discrete labels the event needs, so the work an accepted point
/// costs beyond its trial runs in parallel too.
pub struct BlockedPass<T> {
    unweighter: Unweighter,
    layout: Layout,
    /// Trials an event may cost before the pass gives up.
    max_trials: u64,
    next_block: u64,
    ready: VecDeque<Block<T>>,
    /// The index within the front block of its first trial not yet accounted for.
    trial_cursor: u64,
    /// Trials since the last point handed out, through `trial_cursor`.
    since_accept: u64,
    /// Every fully consumed block's statistics, in block order.
    consumed: UnweightStats,
    /// `consumed` and the front block's statistics after the last point handed out.
    stats: UnweightStats,
    exhausted: bool,
}

impl<T: Send> BlockedPass<T> {
    /// A pass over `unweighter`'s frozen maxima on `seed`'s streams, handing each
    /// accepted point `event_draws` uniforms and giving up after `max_trials`
    /// trials without one.
    pub fn new(unweighter: Unweighter, seed: u64, event_draws: usize, max_trials: u64) -> Self {
        let stride = (2 + unweighter.widest_grid() + event_draws) as u64;
        BlockedPass {
            unweighter,
            layout: Layout {
                seed,
                stride,
                event_draws,
            },
            max_trials,
            next_block: 0,
            ready: VecDeque::new(),
            trial_cursor: 0,
            since_accept: 0,
            consumed: UnweightStats::default(),
            stats: UnweightStats::default(),
            exhausted: false,
        }
    }

    /// The frozen maxima the pass draws against.
    pub fn unweighter(&self) -> &Unweighter {
        &self.unweighter
    }

    /// The statistics of every trial up to the one that produced the last point
    /// handed out.
    pub fn stats(&self) -> &UnweightStats {
        &self.stats
    }

    /// [`Unweighter::sigma_from_events`] over [`stats`](Self::stats): the cross
    /// section the points handed out so far carry, in the integrand's units.
    pub fn sigma_from_events(&self) -> f64 {
        let trials = self.stats.trials as f64;
        if trials > 0.0 {
            self.unweighter.total_w_max() * self.stats.event_weight_sum / trials
        } else {
            0.0
        }
    }

    /// Return the pass to its first trial, so the identical sequence follows.
    pub fn restart(&mut self) {
        self.next_block = 0;
        self.ready.clear();
        self.trial_cursor = 0;
        self.since_accept = 0;
        self.consumed = UnweightStats::default();
        self.stats = UnweightStats::default();
        self.exhausted = false;
    }

    /// The next accepted point and what `prepare` made of it, or `None` once
    /// `max_trials` trials pass without one.
    ///
    /// `prepare` runs on the worker pool, on every point any computed block
    /// accepts, including those past the last one the caller takes; it must be a
    /// function of its arguments alone.
    pub fn next<I, F>(&mut self, integrand: &I, prepare: &F) -> Option<(AcceptedPoint, T)>
    where
        I: ChannelIntegrand + Sync,
        F: Fn(&AcceptedPoint, &[f64]) -> T + Sync,
    {
        if self.exhausted {
            return None;
        }
        loop {
            let Some(block) = self.ready.front_mut() else {
                self.run_batch(integrand, prepare);
                continue;
            };
            if let Some(event) = block.events.pop_front() {
                self.since_accept += event.trial + 1 - self.trial_cursor;
                self.trial_cursor = event.trial + 1;
                if self.since_accept > self.max_trials {
                    self.exhausted = true;
                    return None;
                }
                self.since_accept = 0;
                self.stats = self.consumed.clone();
                self.stats.absorb(&event.stats);
                return Some((event.point, event.item));
            }
            let block = self
                .ready
                .pop_front()
                .expect("the front block was just read");
            self.since_accept += BLOCK_TRIALS - self.trial_cursor;
            self.trial_cursor = 0;
            self.consumed.absorb(&block.stats);
            self.stats = self.consumed.clone();
            if self.since_accept >= self.max_trials {
                self.exhausted = true;
                return None;
            }
        }
    }

    fn run_batch<I, F>(&mut self, integrand: &I, prepare: &F)
    where
        I: ChannelIntegrand + Sync,
        F: Fn(&AcceptedPoint, &[f64]) -> T + Sync,
    {
        let width = (rayon::current_num_threads() * BLOCKS_PER_THREAD) as u64;
        let first = self.next_block;
        let unweighter = &self.unweighter;
        let layout = self.layout;
        let blocks: Vec<Block<T>> = (first..first + width)
            .into_par_iter()
            .map(|b| run_block(unweighter, integrand, layout, b, prepare))
            .collect();
        self.next_block += width;
        self.ready.extend(blocks);
    }
}

/// Block `b`'s trials, on the streams [`Layout`] places it at.
fn run_block<I, T, F>(
    unweighter: &Unweighter,
    integrand: &I,
    layout: Layout,
    b: u64,
    prepare: &F,
) -> Block<T>
where
    I: ChannelIntegrand,
    F: Fn(&AcceptedPoint, &[f64]) -> T,
{
    let first = b * BLOCK_TRIALS;
    let start = u128::from(first) * u128::from(layout.stride) * WORDS_PER_DRAW;
    let mut rng = ChaCha8Rng::seed_from_u64(layout.seed);
    rng.set_stream(TRIAL_STREAM);
    rng.set_word_pos(start);
    let mut trailing = SubStream::new(
        layout.seed,
        SCALE_DRAW_STREAM_BASE,
        first * unweighter.scale_ndim() as u64,
    );
    let mut u = vec![0.0; unweighter.point_len()];
    let mut draws = vec![0.0; layout.event_draws];
    let mut stats = UnweightStats::default();
    let mut events = VecDeque::new();
    for trial in 0..BLOCK_TRIALS {
        let accepted = unweighter.trial_with(
            integrand,
            &mut rng,
            |_, out| trailing.fill_uniforms(out),
            &mut u,
            &mut stats,
        );
        if let Some(point) = accepted {
            for d in draws.iter_mut() {
                *d = rng.random();
            }
            let item = prepare(&point, &draws);
            events.push_back(BlockEvent {
                trial,
                point,
                item,
                stats: stats.clone(),
            });
        }
    }
    debug_assert!(
        rng.get_word_pos() <= start + u128::from(BLOCK_TRIALS * layout.stride) * WORDS_PER_DRAW,
        "block {b} drew past its share of the trial stream"
    );
    Block { events, stats }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unweight::MaxRule;
    use crate::vegas::VegasGrid;

    /// Channel `j` is `(pⱼ+1)·x^pⱼ` scaled by `sⱼ` on the first coordinate, and
    /// reads a trailing uniform it ignores, so the trailing stream is exercised.
    struct PowerChannels {
        sigma: Vec<f64>,
        power: Vec<f64>,
    }

    impl ChannelIntegrand for PowerChannels {
        fn channel_count(&self) -> usize {
            self.sigma.len()
        }
        fn channel_grid_ndim(&self, _channel: usize) -> usize {
            1
        }
        fn scale_draw_ndim(&self) -> usize {
            1
        }
        fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
            assert!((0.0..1.0).contains(&u[1]));
            let p = self.power[channel];
            self.sigma[channel] * (p + 1.0) * u[0].powf(p)
        }
    }

    fn channels() -> PowerChannels {
        PowerChannels {
            sigma: vec![1.0, 0.25, 3.0],
            power: vec![1.0, 4.0, 0.5],
        }
    }

    fn scan(integ: &PowerChannels, rule: MaxRule) -> Unweighter {
        let grids: Vec<VegasGrid> = (0..integ.sigma.len())
            .map(|_| VegasGrid::new(1, 50, 0.0))
            .collect();
        Unweighter::scan_with(integ, grids.iter().map(|g| (g, 2_000)), 11, rule)
    }

    /// What a pass hands out: each point's channel, coordinates, weight and the
    /// uniforms it was prepared with, bit for bit, and the statistics after it.
    type Taken = Vec<(usize, Vec<u64>, u64, Vec<u64>, u64, u64, u64)>;

    fn take(pass: &mut BlockedPass<Vec<f64>>, integ: &PowerChannels, n: usize) -> Taken {
        let prepare = |_: &AcceptedPoint, draws: &[f64]| draws.to_vec();
        (0..n)
            .map(|_| {
                let (point, draws) = pass.next(integ, &prepare).expect("an event");
                let stats = pass.stats();
                (
                    point.channel,
                    point.u.iter().map(|x| x.to_bits()).collect(),
                    point.weight.to_bits(),
                    draws.iter().map(|x| x.to_bits()).collect(),
                    stats.trials,
                    stats.ratio_sum.to_bits(),
                    stats.event_weight_sum.to_bits(),
                )
            })
            .collect()
    }

    fn run_on(threads: usize, n: usize) -> (Taken, u64) {
        let integ = channels();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("a pool");
        pool.install(|| {
            let mut pass = BlockedPass::new(scan(&integ, MaxRule::default()), 0xB10C, 3, 1 << 20);
            let taken = take(&mut pass, &integ, n);
            (taken, pass.sigma_from_events().to_bits())
        })
    }

    /// The pool's size reaches neither the points, the uniforms they were
    /// prepared with, nor any statistic read after each of them. Several
    /// thousand events span many blocks and, at four threads, several batches.
    #[test]
    fn the_output_is_independent_of_the_thread_count() {
        let one = run_on(1, 3_000);
        for threads in [2, 3, 4] {
            assert!(
                run_on(threads, 3_000) == one,
                "{threads} threads departed from one"
            );
        }
    }

    /// An independent serial replay of the stream layout: one trial stream read
    /// from each block's offset in turn, one trailing stream read straight
    /// through, the trials accepted against the same maxima. The pass must hand
    /// out exactly these points, and its trial count after each must be the
    /// global index of the trial that accepted it, plus one.
    #[test]
    fn the_blocks_tile_one_serial_sequence() {
        let integ = channels();
        let uw = scan(&integ, MaxRule::default());
        let event_draws = 2;
        let seed = 0x5EED_1;
        let stride = 2 + 1 + event_draws as u64;
        let mut reference = Vec::new();
        let mut trailing = SubStream::new(seed, SCALE_DRAW_STREAM_BASE, 0);
        let mut u = vec![0.0; 2];
        let mut stats = UnweightStats::default();
        'blocks: for b in 0.. {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(TRIAL_STREAM);
            rng.set_word_pos(u128::from(b * BLOCK_TRIALS * stride) * WORDS_PER_DRAW);
            for t in 0..BLOCK_TRIALS {
                if let Some(p) = uw.trial_with(
                    &integ,
                    &mut rng,
                    |_, out| trailing.fill_uniforms(out),
                    &mut u,
                    &mut stats,
                ) {
                    let draws: Vec<f64> = (0..event_draws).map(|_| rng.random()).collect();
                    reference.push((p.channel, p.u, p.weight, draws, b * BLOCK_TRIALS + t + 1));
                    if reference.len() == 2_500 {
                        break 'blocks;
                    }
                }
            }
        }
        let mut pass = BlockedPass::new(uw, seed, event_draws, 1 << 20);
        let prepare = |_: &AcceptedPoint, draws: &[f64]| draws.to_vec();
        for (k, (channel, u, weight, draws, trials)) in reference.into_iter().enumerate() {
            let (point, got) = pass.next(&integ, &prepare).expect("an event");
            assert_eq!(point.channel, channel, "event {k}");
            assert_eq!(point.u, u, "event {k}");
            assert_eq!(point.weight, weight, "event {k}");
            assert_eq!(got, draws, "event {k}");
            assert_eq!(pass.stats().trials, trials, "event {k}");
            assert_eq!(pass.stats().accepted, k as u64 + 1, "event {k}");
        }
    }

    /// The statistics merged block by block are the same counts a single pass
    /// over the same trials keeps: integer counts exactly, sums to rounding, and
    /// the heaviest trials the same ones in the same order.
    #[test]
    fn the_merged_statistics_are_the_serial_ones() {
        let integ = channels();
        let uw = scan(&integ, MaxRule::default());
        let mut pass = BlockedPass::new(uw.clone(), 3, 0, 1 << 20);
        let prepare = |_: &AcceptedPoint, _: &[f64]| ();
        for _ in 0..4_000 {
            pass.next(&integ, &prepare).expect("an event");
        }
        let got = pass.stats().clone();
        let mut trailing = SubStream::new(3, SCALE_DRAW_STREAM_BASE, 0);
        let mut u = vec![0.0; 2];
        let mut serial = UnweightStats::default();
        let stride = 3;
        for b in 0..got.trials.div_ceil(BLOCK_TRIALS) {
            let mut rng = ChaCha8Rng::seed_from_u64(3);
            rng.set_word_pos(u128::from(b * BLOCK_TRIALS * stride) * WORDS_PER_DRAW);
            for t in 0..BLOCK_TRIALS {
                if b * BLOCK_TRIALS + t == got.trials {
                    break;
                }
                uw.trial_with(
                    &integ,
                    &mut rng,
                    |_, out| trailing.fill_uniforms(out),
                    &mut u,
                    &mut serial,
                );
            }
        }
        assert_eq!(got.trials, serial.trials);
        assert_eq!(got.accepted, serial.accepted);
        assert_eq!(got.vanishing, serial.vanishing);
        assert_eq!(got.overweight, serial.overweight);
        assert_eq!(got.ratio_max, serial.ratio_max);
        for (a, b) in [
            (got.ratio_sum, serial.ratio_sum),
            (got.event_weight_sum, serial.event_weight_sum),
            (got.overweight_ratio_sum, serial.overweight_ratio_sum),
            (got.excess_sum, serial.excess_sum),
        ] {
            assert!((a - b).abs() <= 1e-12 * b.abs(), "{a} vs {b}");
        }
        assert!(
            got.overweight > 0,
            "the truncating rule leaves overweights to merge"
        );
        let ratios = |s: &UnweightStats| s.heaviest.iter().map(|h| h.ratio).collect::<Vec<_>>();
        assert_eq!(ratios(&got), ratios(&serial));
    }

    /// A restart replays the identical sequence, statistics included.
    #[test]
    fn a_restart_replays_the_pass() {
        let integ = channels();
        let mut pass = BlockedPass::new(scan(&integ, MaxRule::default()), 77, 1, 1 << 20);
        let first = take(&mut pass, &integ, 1_500);
        pass.restart();
        assert_eq!(pass.stats().trials, 0);
        assert!(take(&mut pass, &integ, 1_500) == first);
    }

    /// The kept weight reproduces each channel's integral: the blocked pass is
    /// the same accept/reject estimator, not merely a reproducible one.
    #[test]
    fn the_sample_reproduces_the_integral() {
        let integ = channels();
        let mut pass = BlockedPass::new(scan(&integ, MaxRule::default()), 5, 0, 1 << 20);
        let prepare = |p: &AcceptedPoint, _: &[f64]| p.weight;
        let mut per_channel = [0.0; 3];
        for _ in 0..200_000 {
            let (point, weight) = pass.next(&integ, &prepare).expect("an event");
            per_channel[point.channel] += weight;
        }
        let total: f64 = per_channel.iter().sum();
        let sigma: f64 = integ.sigma.iter().sum();
        let rel = pass.sigma_from_events() / sigma - 1.0;
        assert!(rel.abs() < 0.01, "sample sigma off by {rel:+.4}");
        for (j, &w) in per_channel.iter().enumerate() {
            let share = w / total;
            let want = integ.sigma[j] / sigma;
            let err = (want * (1.0 - want) / 200_000.0).sqrt();
            assert!(
                (share - want).abs() < 5.0 * err + 2e-3,
                "channel {j}: share {share:.4} against {want:.4}"
            );
        }
    }

    /// An integrand that stops accepting ends the pass after the trial budget,
    /// on every later call too.
    #[test]
    fn a_pass_that_cannot_accept_gives_up() {
        struct Vanishing(PowerChannels, std::sync::atomic::AtomicBool);
        impl ChannelIntegrand for Vanishing {
            fn channel_count(&self) -> usize {
                self.0.channel_count()
            }
            fn channel_grid_ndim(&self, c: usize) -> usize {
                self.0.channel_grid_ndim(c)
            }
            fn scale_draw_ndim(&self) -> usize {
                1
            }
            fn value_in_channel(&self, c: usize, u: &[f64]) -> f64 {
                if self.1.load(std::sync::atomic::Ordering::Relaxed) {
                    0.0
                } else {
                    self.0.value_in_channel(c, u)
                }
            }
        }
        let integ = Vanishing(channels(), false.into());
        let uw = scan(&integ.0, MaxRule::default());
        integ.1.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut pass = BlockedPass::new(uw, 1, 0, 3 * BLOCK_TRIALS + 17);
        let prepare = |_: &AcceptedPoint, _: &[f64]| ();
        assert!(pass.next(&integ, &prepare).is_none());
        assert!(pass.next(&integ, &prepare).is_none());
        assert!(pass.stats().trials >= 3 * BLOCK_TRIALS + 17);
        assert_eq!(pass.stats().accepted, 0);
    }
}
