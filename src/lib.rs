#![warn(rust_2018_idioms)]
#![cfg(windows)]

// We don't need this in Rust 1.80 (Ref: https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html)
// #[cfg(all(feature = "ergo", feature = "raw"))]
// compile_error!("ergo support and raw support are exclusive. only one of them can be enabled at the same time.");


#[cfg(not(feature = "raw"))]
pub mod ergo;
#[cfg(not(feature = "raw"))]
pub use ergo::*;

#[cfg(not(feature = "raw"))]
mod raw;
#[cfg(feature = "raw")]
pub mod raw;
