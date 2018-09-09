//! Support for various versions of `futures`.
//!
//! The executors in this module will never fail to spawn a future, barring
//! extreme circumstances such as OOM errors.
//!
//! ## A note on poll order
//!
//! Futures can spawn additional futures while they are themselves being
//! polled. The implementations here do not use a queue to handle this
//! situation, rather they immediately poll the freshly spawned future. This
//! should not affect usage of futures.

pub mod v01;
pub mod v02;
