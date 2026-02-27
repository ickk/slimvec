// Copyright © ickk, 2026

use {
  crate::SlimVec,
  ::core::{
    convert::AsRef,
    fmt,
    iter::{FusedIterator, IntoIterator, Iterator},
    mem,
    mem::ManuallyDrop,
    ops::Range,
    slice,
  },
};

/// An iterator that moves out of a [`SlimVec`]
///
/// This may be created using [`SlimVec::into_iter`].
pub struct IntoIter<T> {
  /// The borrowed `SlimVec`, with temporarily truncated length
  slimvec: ManuallyDrop<SlimVec<T>>,
  /// The range of elements to be yielded
  yield_range: Range<usize>,
}

impl<T> Default for IntoIter<T> {
  #[inline]
  fn default() -> Self {
    IntoIter::new(SlimVec::new())
  }
}

impl<T> IntoIter<T> {
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    if self.slimvec.raw.is_capacity_gt_zero() {
      let range = &self.yield_range;
      unsafe {
        let ptr = self.slimvec.raw.element_ptr(range.start).as_ptr();
        slice::from_raw_parts(ptr, range.len())
      }
    } else {
      &[]
    }
  }

  #[inline]
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    if self.slimvec.raw.is_capacity_gt_zero() {
      let range = &self.yield_range;
      unsafe {
        let ptr = self.slimvec.raw.element_ptr(range.start).as_ptr();
        slice::from_raw_parts_mut(ptr, range.len())
      }
    } else {
      &mut []
    }
  }
}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  #[inline]
  fn next(&mut self) -> Option<T> {
    if self.len() == 0 {
      return None;
    }
    self.yield_range.start += 1;
    // safety: `capacity >= len > 0`.
    let element = unsafe { self.slimvec.raw.read(self.yield_range.start - 1) };
    Some(element)
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.yield_range.len();
    (len, Some(len))
  }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
  #[inline]
  fn next_back(&mut self) -> Option<T> {
    if self.len() == 0 {
      return None;
    }
    self.yield_range.end -= 1;
    // safety: `capacity >= len > 0`.
    let element = unsafe { self.slimvec.raw.read(self.yield_range.end) };
    Some(element)
  }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> AsRef<[T]> for IntoIter<T> {
  #[inline]
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T> fmt::Debug for IntoIter<T>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
  }
}

impl<T> Clone for IntoIter<T>
where
  T: Clone,
{
  #[inline]
  fn clone(&self) -> IntoIter<T> {
    SlimVec::from(self.as_slice()).into_iter()
  }
}

/// Private methods
impl<T> IntoIter<T> {
  #[inline]
  pub(crate) fn new(mut slimvec: SlimVec<T>) -> IntoIter<T> {
    let yield_range = 0..slimvec.len();
    if slimvec.raw.is_capacity_gt_zero() {
      // Unwind-safety of `SlimVec` is maintained by setting its length to zero
      // before creation of the `IntoIter`. This prevents the state of the
      // `SlimVec`, which would otherwise be logically broken, from being
      // observed.
      //
      // This shifts the responsibility onto the `IntoIter` to run destructors
      // for remaining elements.

      // safety: `capacity > 0`.
      unsafe { slimvec.raw.set_length(0) };
    }
    IntoIter {
      slimvec: ManuallyDrop::new(slimvec),
      yield_range,
    }
  }
}

impl<T> Drop for IntoIter<T> {
  #[inline]
  fn drop(&mut self) {
    // Unwind safety; prevent running element destructors more than once.
    let yield_range = mem::take(&mut self.yield_range);
    // Prevent double free of slimvec allocation.
    let mut slimvec = ::core::mem::take(&mut self.slimvec);
    // safety: elements in `range` are still valid.
    unsafe { slimvec.raw.drop_in_place(yield_range) };
    // Length of slimvec is always zero, so this only deallocates its memory.
    _ = ManuallyDrop::into_inner(slimvec);
  }
}
