//! The gates a request passes before a handler sees it.
//!
//! ```text
//!   signature.rs  the luxctl HMAC boundary          404
//!   reader.rs     cookie -> Session in extensions   401
//!   csrf.rs       header vs the session's token     403
//!   throttle.rs   writes per minute, per reader     429
//! ```
//!
//! One layer per gate, so a route says what it needs rather than a single layer
//! deciding for it: a `GET` wants a reader and nothing else, a `POST` wants all
//! three. Bundling them meant the layer inspected the method to work out which
//! checks applied, which is the router's job.
//!
//! **Order matters, and getting it wrong is loud.** `csrf` and `throttle` both
//! read the `Session` that `reader` inserts. Applied in the wrong order they
//! find none — and both refuse rather than skip, so a misordered mount answers
//! 403 or 429 to everything instead of quietly letting writes through.

pub(crate) mod csrf;
pub(crate) mod reader;
pub(crate) mod signature;
pub(crate) mod throttle;
