// Copyright © ickk, 2026

#![cfg_attr(not(feature = "std"), no_std)]

//! # `slimvec`
//!
//! [crates.io] | [docs.rs] | [github]
//!
//! [crates.io]: https://crates.io/crates/slimvec
//! [docs.rs]: https://docs.rs/slimvec
//! [github]: https://github.com/ickk/slimvec
//!
//! <details open>
//! <summary>
//!
//! ## Overview
//! </summary>
//!
#![doc = include_str!("../docs/Overview.md")]
//!
//! </details>
//! <details>
//! <summary>
//!
//! ## Features
//! </summary>
//!
#![doc = include_str!("../docs/Features.md")]
//!
//! </details>
//! <details>
//! <summary>
//!
//! ## Architecture
//! </summary>
//!
#![doc = include_str!("../docs/Architecture.md")]
//!
//! </details>
//!
#![doc = include_str!("../LICENSE.md")]

extern crate alloc;

mod drain;
mod extract_if;
mod features;
mod into_iter;
mod raw_slimvec;
mod slice_ext;
mod slimvec;
mod splice;
mod utils;

pub use crate::{
  drain::Drain, extract_if::ExtractIf, into_iter::IntoIter,
  slice_ext::SliceExt, slimvec::SlimVec, splice::Splice,
};
