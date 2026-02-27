// Copyright © ickk, 2026

use {
  crate::{SlimVec, utils::conform_range},
  ::core::{
    convert::AsRef,
    fmt,
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    mem,
    ops::{Range, RangeBounds},
    panic::{RefUnwindSafe, UnwindSafe},
    ptr, slice,
  },
};

/// A draining iterator for [`SlimVec`]
///
/// This may be created using [`SlimVec::drain`].
pub struct Drain<'v, T>
where
  T: 'v,
{
  /// The borrowed `SlimVec`, with temporarily truncated length
  pub(crate) slimvec: &'v mut SlimVec<T>,
  /// The tailing elements, to be fixed up in the destructor
  pub(crate) tail: Range<usize>,
  /// The range of elements to be yielded
  pub(crate) yield_range: Range<usize>,
}

impl<T> Drain<'_, T> {
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    if self.slimvec.raw.is_capacity_gt_zero() {
      // safety: capacity > 0.
      unsafe {
        let ptr = self
          .slimvec
          .raw
          .element_ptr(self.yield_range.start)
          .as_ptr();
        slice::from_raw_parts(ptr, self.len())
      }
    } else {
      &[]
    }
  }
}

impl<T> Iterator for Drain<'_, T> {
  type Item = T;

  #[inline]
  fn next(&mut self) -> Option<T> {
    if self.len() == 0 {
      return None;
    }
    self.yield_range.start += 1;
    // safety: `self.len != 0` implies `slimvec.capacity >= slimvec.len > 0`.
    let element = unsafe { self.slimvec.raw.read(self.yield_range.start - 1) };
    Some(element)
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.yield_range.len();
    (len, Some(len))
  }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.len() == 0 {
      return None;
    }
    self.yield_range.end -= 1;
    // safety: `self.len != 0` implies `slimvec.capacity >= slimvec.len > 0`.
    let element = unsafe { self.slimvec.raw.read(self.yield_range.end) };
    Some(element)
  }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

impl<T> FusedIterator for Drain<'_, T> {}

impl<T> fmt::Debug for Drain<'_, T>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Drain").field(&self.as_slice()).finish()
  }
}

impl<T> AsRef<[T]> for Drain<'_, T> {
  #[inline]
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

/// Internal methods
impl<T> Drain<'_, T> {
  #[inline]
  pub(crate) fn new(
    slimvec: &mut SlimVec<T>,
    range: impl RangeBounds<usize>,
  ) -> Drain<'_, T> {
    let yield_range = conform_range(range, slimvec.len());
    let tail = yield_range.end..slimvec.len();
    // See notes `UnwindSafe` impl below.
    if !slimvec.is_empty() {
      // safety: `len != 0` implies `capacity > 0`.
      unsafe { slimvec.raw.set_length(yield_range.start) };
    }

    Drain {
      slimvec,
      tail,
      yield_range,
    }
  }

  /// The range containing the current void, spanning between the head and tail
  #[inline]
  pub(crate) fn void(&self) -> Range<usize> {
    self.slimvec.len()..self.tail.start
  }

  /// Fill the void in the slimvec with the iterator
  ///
  /// If the iterator yields fewer elements than the length of the void, then a
  /// (smaller) void will remain.
  ///
  /// If the void is completely filled by the iterator, then iterator will
  /// still contain the remaining elements after this call.
  #[inline]
  pub(crate) fn fill_void(&mut self, with: &mut impl Iterator<Item = T>) {
    for element in with.by_ref().take(self.void().len()) {
      // safety: `capacity >= void.len > 0`
      unsafe { self.slimvec.raw.push_unchecked(element) };
    }
  }

  /// Shift the tail so that it starts at `to_index`
  ///
  /// This does not change the length.
  ///
  /// # Safety
  ///
  /// - `self.slimvec`'s capacity must be greater than zero.
  /// - `self.slimvec` must already have sufficient capacity for the tail.
  #[inline]
  pub(crate) unsafe fn shift_tail(&mut self, to_index: usize) {
    debug_assert!(
      (self.slimvec.raw.is_capacity_gt_zero())
        && (self.slimvec.capacity() >= (to_index + self.tail.len())),
      "required safety criteria"
    );
    unsafe {
      ptr::NonNull::copy_from(
        self.slimvec.raw.element_ptr(to_index),
        self.slimvec.raw.element_ptr(self.tail.start),
        self.tail.len(),
      );
    }
    self.tail = to_index..(to_index + self.tail.len());
  }
}

// See notes on `UnwindSafe` impl below.
impl<T> Drop for Drain<'_, T> {
  fn drop(&mut self) {
    let yield_range = mem::take(&mut self.yield_range);
    unsafe { self.slimvec.raw.drop_in_place(yield_range) };
    if !self.tail.is_empty() {
      let new_len = self.slimvec.len() + self.tail.len();
      // safety:
      // - `tail.len != 0` implies `slimvec.capacity > 0`.
      unsafe {
        self.shift_tail(self.slimvec.len());
        self.slimvec.raw.set_length(new_len);
      }
    }
  }
}

// While iterating over the `range` and yielding elements, an uninitialised
// hole will form. When the destructor runs, it will copy the tailing elements
// back over to patch this hole and it will set the length appropriately.
//
// The length is initially truncated to the start of the range; this protects
// against invalid state being observed in case `Drain` leaks, and also ensures
// `Drain` is `UnwindSafe`.
impl<T> UnwindSafe for Drain<'_, T> where T: RefUnwindSafe {}
