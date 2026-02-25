// © ickk 2026.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(rustc::unsafe_op_in_unsafe_fn)]

//! # `slimvec`
//!
//! [github](https://github.com/ickk/slimvec)
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
//!
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
mod into_iter;
mod raw_slimvec;
mod slice_ext;
mod slimvec;
mod splice;

pub use crate::{
  drain::Drain, extract_if::ExtractIf, into_iter::IntoIter,
  slice_ext::SliceExt, slimvec::SlimVec, splice::Splice,
};

mod utils {
  use ::core::{
    mem,
    ops::{Bound, Range, RangeBounds},
  };

  /// Internal trait used for specialisation
  pub(crate) trait TypeMeta {
    /// This is `true` when `T` is a zero-sized-type. It is used to specialise
    /// the implementation so that [`RawSlimVec`]s containing ZST elements never
    /// allocate.
    const IS_ZST: bool;
  }

  impl<T> TypeMeta for T {
    const IS_ZST: bool = mem::size_of::<T>() == 0;
  }

  /// Conform `RangeBounds` to a `Range<usize>` bounded between `0..max`
  ///
  /// # Panics
  ///
  /// - Panics if `range.start_bound()` is greater than `range.end_bound()`.
  /// - Panics if `range.end_bound()` is greater than `max`.
  #[inline]
  pub(crate) fn conform_range(
    range: impl RangeBounds<usize>,
    max: usize,
  ) -> Range<usize> {
    let end = match range.end_bound() {
      Bound::Excluded(&b) if b <= max => Some(b),
      Bound::Included(&b) if b < max => Some(b + 1),
      Bound::Unbounded => Some(max),
      _ => None,
    };
    let start = match range.start_bound() {
      Bound::Included(&b) if Some(b) <= end => Some(b),
      Bound::Excluded(&b) if Some(b) < end => Some(b + 1),
      Bound::Unbounded => Some(0),
      _ => None,
    };
    let (Some(start), Some(end)) = (start, end) else {
      panic!("invalid range")
    };
    start..end
  }
}
