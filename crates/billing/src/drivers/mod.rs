//! The drivers that ship with this crate.
//!
//! One so far. A provider that is not here is not a gap to fill by editing
//! this module — implement [`Gateway`](crate::Gateway) wherever it suits and
//! register it, which is what the registry taking trait objects buys.

pub mod stripe;
