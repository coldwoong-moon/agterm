//! Terminal module for AgTerm
//!
//! Provides PTY (Pseudo-Terminal) management for terminal sessions.

pub mod pty;

pub use pty::{PtyManager, PtyId, PtyError};
