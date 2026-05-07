// VBR Standard Library
// A collection of friendly wrappers for common Rust operations
// designed for VBA developers learning Rust via VBR.
//
// Each module wraps a standard Rust library or crate.
// Reading the source of each module is encouraged —
// it is real idiomatic Rust and a great learning resource.

pub mod filesystem;
pub mod json;
pub mod http;
pub mod datetime;
pub mod regex;

// V2 — requires async support
// pub mod database;

pub use filesystem::FileSystem;
pub use json::Json;
pub use http::Http;
pub use datetime::DateTime;
pub use regex::Regex;
