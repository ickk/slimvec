// Copyright © ickk, 2026

use {crate::SlimVec, ::alloc::boxed::Box, ::core::ptr};

/// Methods extending `[T]` that return a `SlimVec<T>`
pub trait SliceExt {
  type Item;

  fn into_slimvec(self: Box<Self>) -> SlimVec<Self::Item>;

  fn to_slimvec(&self) -> SlimVec<Self::Item>
  where
    Self::Item: Clone;

  fn repeat_to_slimvec(&self, n: usize) -> SlimVec<Self::Item>
  where
    Self::Item: Copy;
}

impl<T> SliceExt for [T] {
  type Item = T;

  #[inline]
  fn into_slimvec(self: Box<[T]>) -> SlimVec<T> {
    SlimVec::from(self)
  }

  #[inline]
  fn to_slimvec(&self) -> SlimVec<T>
  where
    T: Clone,
  {
    SlimVec::from(self)
  }

  #[inline]
  fn repeat_to_slimvec(&self, n: usize) -> SlimVec<T>
  where
    T: Copy,
  {
    let mut slimvec = SlimVec::new();
    let capacity = self.len().checked_mul(n).expect("overflow");
    if capacity > 0 {
      // After this reserve, `capacity > 0`.
      slimvec.reserve_exact(capacity);
      for i in 0..n {
        unsafe {
          ptr::copy_nonoverlapping(
            self.as_ptr(),
            slimvec.raw.element_ptr(i * self.len()).as_ptr(),
            self.len(),
          );
        }
      }
      // safety: `capacity > 0`.
      unsafe { slimvec.raw.set_length(self.len() * n) };
    }
    slimvec
  }
}
