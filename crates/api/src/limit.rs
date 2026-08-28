//! A rate limit for writes, kept in this process's memory.
//!
//! Ten writes a minute per reader, which is what `RateLimiter::for('notes')`
//! allows in the laravel app. The number is not about abuse alone — a note is
//! typed, and nobody types ten in a minute — so hitting it means a script, a
//! stuck retry loop, or a bug.
//!
//! **Fixed window, not sliding.** Twenty writes can land in two seconds either
//! side of a window boundary. A sliding window costs a timestamp per hit and
//! buys precision that does not matter here; the laravel limiter this replaces
//! is a fixed window too.
//!
//! ponytail: in-process, so the counters reset on deploy and would not be
//! shared if a second container ever ran. Both are fine for one droplet running
//! one container — the upgrade, if that changes, is the same counters in redis.

use std::{
    collections::HashMap,
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

/// When the map is larger than this, expired windows are dropped before adding
/// another. Without it, one entry per reader who has ever written accumulates
/// for the life of the process.
const PRUNE_ABOVE: usize = 1024;

#[derive(Debug)]
struct Window {
    started: Instant,
    hits: u32,
}

#[derive(Debug)]
pub(crate) struct RateLimit {
    limit: u32,
    window: Duration,
    /// Keyed by a string rather than a user id, because the caller is not
    /// always a reader. `RateLimiter::for('api')` in the laravel app keys on
    /// `$request->user()?->id ?: $request->ip()`, and this has to be able to
    /// say both.
    ///
    /// A `Mutex` rather than a lock-free structure: this is contended for the
    /// length of a hash lookup, on a path that is about to do a database write.
    ///
    /// ponytail: one lock for every reader. Per-shard locks if it ever shows up
    /// in a profile, which at this traffic it will not.
    windows: Mutex<HashMap<String, Window>>,
}

impl RateLimit {
    /// Ten writes a minute, which is what `RateLimiter::for('notes')` allows in
    /// the laravel app. Nobody types ten notes in a minute, so hitting it means
    /// a script, a stuck retry loop, or a bug.
    ///
    /// A function, not a `const` item: a `const` is substituted at each mention,
    /// so two references would be two limiters with separate counters.
    pub(crate) fn note_writes() -> Self {
        Self::new(10, Duration::from_secs(60))
    }

    /// Sixty a minute, which is `RateLimiter::for('api')` in the laravel app —
    /// the limit every route there inherits unless it names a stricter one.
    ///
    /// It sits *outside* the per-route limits rather than replacing them: a
    /// note write counts against this and against `note_writes`, exactly as a
    /// laravel route carrying both `throttle:api` and `throttle:notes` does.
    pub(crate) fn requests() -> Self {
        Self::new(60, Duration::from_secs(60))
    }

    /// The budget is the caller's, not this module's: a search box and a note
    /// editor are not the same kind of traffic, and one number for both is one
    /// that is wrong for at least one of them.
    pub(crate) fn new(per_window: u32, window: Duration) -> Self {
        Self {
            limit: per_window,
            window,
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Counts one write against `key`, and says whether it is allowed.
    ///
    /// `None` to proceed. `Some(seconds)` to refuse, carrying what belongs in
    /// `Retry-After` — a 429 without one tells a client to back off but not by
    /// how much, so it guesses, and the guess is usually "immediately".
    pub(crate) fn check(&self, key: &str) -> Option<u64> {
        let now = Instant::now();

        // A poisoned lock means a previous holder panicked while holding it.
        // Nothing here can panic — and `panic = "abort"` in release means the
        // process would be gone anyway — so taking the value back is strictly
        // better than refusing every write from here on.
        let mut windows =
            self.windows.lock().unwrap_or_else(PoisonError::into_inner);

        if windows.len() > PRUNE_ABOVE {
            windows.retain(|_, window| {
                now.duration_since(window.started) < self.window
            });
        }

        let window = windows.entry(key.to_owned()).or_insert(Window {
            started: now,
            hits: 0,
        });

        // Expired, so this hit starts a fresh one rather than inheriting a count
        // from a minute ago.
        if now.duration_since(window.started) >= self.window {
            *window = Window {
                started: now,
                hits: 0,
            };
        }

        if window.hits >= self.limit {
            let elapsed = now.duration_since(window.started);

            // At least a second: `Retry-After: 0` reads as "go ahead", which is
            // the opposite of what a 429 means.
            return Some(self.window.saturating_sub(elapsed).as_secs().max(1));
        }

        window.hits += 1;

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The note budget, so the numbers below read against a real setting.
    const BUDGET: u32 = 10;

    #[test]
    fn the_first_ten_writes_pass_and_the_eleventh_does_not() {
        let limit = RateLimit::note_writes();

        for attempt in 1..=BUDGET {
            assert_eq!(
                limit.check("user:1"),
                None,
                "write {attempt} was refused"
            );
        }

        assert!(limit.check("user:1").is_some());
    }

    #[test]
    fn one_reader_hitting_the_limit_does_not_stop_another() {
        let limit = RateLimit::note_writes();

        for _ in 0..=BUDGET {
            let _ = limit.check("user:1");
        }

        assert!(limit.check("user:1").is_some());
        assert_eq!(limit.check("user:2"), None);
    }

    #[test]
    fn a_refusal_says_how_long_to_wait() {
        let limit = RateLimit::note_writes();

        for _ in 0..BUDGET {
            let _ = limit.check("user:1");
        }

        let retry = limit.check("user:1").unwrap();

        assert!(retry >= 1);
        assert!(retry <= 60);
    }

    #[test]
    fn the_window_expires_and_the_count_starts_again() {
        let limit = RateLimit::new(2, Duration::from_millis(30));

        assert_eq!(limit.check("user:1"), None);
        assert_eq!(limit.check("user:1"), None);
        assert!(limit.check("user:1").is_some());

        std::thread::sleep(Duration::from_millis(40));

        assert_eq!(limit.check("user:1"), None, "the window did not expire");
    }

    #[test]
    fn the_map_does_not_grow_with_every_reader_who_ever_wrote() {
        // A zero window so every entry is expired the moment it is looked at,
        // which makes the prune's effect exact rather than a race with how fast
        // this loop runs. What is under test is that pruning happens at all —
        // the threshold is a ceiling on the map, not a promise it stays empty.
        let limit = RateLimit::new(1, Duration::ZERO);
        let readers = i64::try_from(PRUNE_ABOVE).unwrap() * 4;

        for key in 0..readers {
            let _ = limit.check(&format!("user:{key}"));
        }

        let held = limit.windows.lock().unwrap().len();

        assert!(
            held <= PRUNE_ABOVE + 1,
            "{held} windows held after {readers} readers"
        );
    }
}
