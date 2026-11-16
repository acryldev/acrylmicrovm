// Modified by ACRYL, 2026
//! Host-side portal for communicating with guests via tonic/gRPC.

pub mod connection;
pub mod interfaces;
pub mod session;

pub use session::GuestSession;
