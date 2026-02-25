// Copyright © ickk, 2026

use {
  crate::{SlimVec, utils::conform_range},
  ::core::{
    fmt,
    ops::{Range, RangeBounds},
    ptr,
  },
};

/// An iterator for [`SlimVec`] that uses a closure to determine whether to
/// remove an element
///
/// This may be created using [`SlimVec::extract_if`].
pub struct ExtractIf<'v, T, F> {
  /// The borrowed `SlimVec`, with temporarily truncated length
  slimvec: &'v mut SlimVec<T>,
  /// The range of elements to be considered for extraction
  filter_range: Range<usize>,
  original_len: usize,
  filter: F,
}

impl<T, F> Iterator for ExtractIf<'_, T, F>
where
  F: FnMut(&mut T) -> bool,
{
  type Item = T;

  fn next(&mut self) -> Option<T> {
    while self.filter_range.start < self.filter_range.end {
      let index = self.filter_range.start;
      let mut element_ptr = unsafe { self.slimvec.raw.element_ptr(index) };
      let should_yield = (self.filter)(unsafe { element_ptr.as_mut() });
      self.filter_range.start += 1;

      if should_yield {
        let owned: T = unsafe { element_ptr.read() };
        return Some(owned);
      }

      let len = self.slimvec.len();
      if index > len {
        unsafe {
          ptr::NonNull::copy_from_nonoverlapping(
            self.slimvec.raw.element_ptr(len),
            element_ptr,
            1,
          )
        };
      }
      unsafe { self.slimvec.raw.set_length(len + 1) };
    }
    None
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, Some(self.filter_range.len()))
  }
}

/// Internal methods
impl<T, F> ExtractIf<'_, T, F> {
  pub(crate) fn new(
    slimvec: &mut SlimVec<T>,
    range: impl RangeBounds<usize>,
    filter: F,
  ) -> ExtractIf<'_, T, F>
  where
    F: FnMut(&mut T) -> bool,
  {
    let filter_range = conform_range(range, slimvec.len());
    let original_len = slimvec.len();
    if original_len > 0 {
      unsafe { slimvec.raw.set_length(filter_range.start) };
    }
    ExtractIf {
      slimvec,
      filter_range,
      original_len,
      filter,
    }
  }
}

impl<T, F> Drop for ExtractIf<'_, T, F> {
  fn drop(&mut self) {
    let tail = self.filter_range.start..self.original_len;
    if !tail.is_empty() {
      let len = self.slimvec.len();
      unsafe {
        ptr::NonNull::copy_from(
          self.slimvec.raw.element_ptr(len),
          self.slimvec.raw.element_ptr(tail.start),
          tail.len(),
        );
        self.slimvec.raw.set_length(len + tail.len());
      }
    }
  }
}

impl<T, F> fmt::Debug for ExtractIf<'_, T, F>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let index = self.filter_range.start;
    let peek = match !self.filter_range.is_empty() {
      true => Some(unsafe { self.slimvec.raw.element_ptr(index).as_ref() }),
      _ => None,
    };
    f.debug_struct("ExtractIf")
      .field("peek", &peek)
      .finish_non_exhaustive()
  }
}
