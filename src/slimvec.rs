// © ickk 2026.

use {
  crate::{
    Drain, ExtractIf, Splice,
    raw_slimvec::RawSlimVec,
    utils::{TypeMeta, conform_range},
  },
  ::core::{mem, ops::RangeBounds, ptr, slice},
};

/// `SlimVec` is analogous to the Standard Library’s [`Vec`] collection type,
/// however it has a smaller inline-size
///
/// View the [crate documentation](crate) for more details.
///
/// [`Vec`]: https://doc.rust-lang.org/alloc/Vec.html
#[derive(Clone)]
pub struct SlimVec<T> {
  pub(crate) raw: RawSlimVec<T>,
}

impl<T> SlimVec<T> {
  /// The maximum possible capacity for a `SlimVec::<T>`, assuming unlimited
  /// memory
  ///
  /// In practice the capacity of any `SlimVec` will be limited by the memory
  /// of the host.
  pub const MAX_CAPACITY: usize = RawSlimVec::<T>::MAX_CAPACITY;

  #[inline]
  pub const fn new() -> Self {
    SlimVec {
      raw: RawSlimVec::EMPTY,
    }
  }

  /// Create a new `SlimVec<T>` with at least `capacity`
  ///
  /// See: [`MAX_CAPACITY`](Self::MAX_CAPACITY).
  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    let mut vec = Self::new();
    vec.reserve(capacity);
    vec
  }

  /// Reserve additional capacity
  ///
  /// The new capacity will be at least `self.len() + additional`.
  ///
  /// # Panics
  ///
  /// Panics if the new capacity exceeds [`MAX_CAPACITY`](Self::MAX_CAPACITY)
  /// or if allocation fails.
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    let required_capacity = self.len().checked_add(additional);
    if required_capacity <= Some(self.capacity()) {
      return;
    }
    // Reserve up to the next power-of-two
    let new_capacity = required_capacity
      .and_then(|required| required.checked_next_power_of_two())
      .expect("capacity exceeds MAX_CAPACITY");
    self.raw.grow_to(new_capacity);
  }

  /// Reserve additional capacity
  ///
  /// The new capacity will be at least `self.len() + additional`.
  ///
  /// # Panics
  ///
  /// Panics if the new capacity exceeds [`MAX_CAPACITY`](Self::MAX_CAPACITY)
  /// or if allocation fails.
  #[inline]
  pub fn reserve_exact(&mut self, additional: usize) {
    let new_capacity = self
      .len()
      .checked_add(additional)
      .expect("capacity exceeds MAX_CAPACITY");
    if new_capacity <= self.capacity() {
      return;
    }
    self.raw.grow_to(new_capacity);
  }

  #[inline]
  pub fn clear(&mut self) {
    self.truncate(0);
  }

  #[inline]
  pub fn truncate(&mut self, new_length: usize) {
    let length = self.len();
    if new_length < length {
      debug_assert!(
        self.capacity() > 0,
        "The vector is allocated or T is zero-sized, since the length is non-zero"
      );
      // safety:
      // - The vector has allocated or `T` is zero-sized;
      // - `new_length` is less than `length`, so is trivially a valid length
      //   for either non-zero-sized or zero-sized `T`.
      // - The previous observation also guarantees that all elements in the
      //   range `0..new_length` are initialised.
      unsafe {
        self.raw.set_length(new_length);
        self.raw.drop_in_place(new_length..length);
      }
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.raw.length()
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.raw.capacity()
  }

  #[inline]
  pub fn push(&mut self, v: T) {
    if self.len() == self.capacity() {
      self.reserve(1);
    }
    // safety: reserved capacity for the element.
    unsafe { self.raw.push_unchecked(v) };
  }

  #[inline]
  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      // safety: checked that the length is not zero.
      Some(unsafe { self.raw.pop_unchecked() })
    }
  }

  #[inline]
  pub fn pop_if(
    &mut self,
    predicate: impl FnOnce(&mut T) -> bool,
  ) -> Option<T> {
    if predicate(self.last_mut()?) {
      self.pop()
    } else {
      None
    }
  }

  #[inline]
  pub fn as_slice(&self) -> &[T] {
    if self.raw.is_allocated() || T::IS_ZST {
      // safety: the vector has allocated or `T` is zero-sized.
      unsafe {
        let buffer_ptr = self.raw.buffer_ptr().as_ptr();
        slice::from_raw_parts(buffer_ptr, self.len())
      }
    } else {
      &[]
    }
  }

  #[inline]
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    if self.raw.is_allocated() || T::IS_ZST {
      // safety: the vector has allocated or `T` is zero-sized.
      unsafe {
        let buffer_ptr = self.raw.buffer_ptr().as_ptr();
        slice::from_raw_parts_mut(buffer_ptr, self.len())
      }
    } else {
      &mut []
    }
  }

  #[inline]
  pub fn swap_remove(&mut self, index: usize) -> T {
    let count = self.len();
    if index >= count {
      panic!("index is out of bounds");
    }

    unsafe {
      self.raw.set_length(count - 1);
      ptr::NonNull::swap(
        self.raw.element_ptr(index),
        self.raw.element_ptr(count - 1),
      );
      self.raw.read(count - 1)
    }
  }

  #[inline]
  pub fn insert(&mut self, index: usize, element: T) {
    let count = self.len();
    if index > count {
      panic!("index is out of bounds");
    }

    self.reserve(1);
    if index == count {
      unsafe { self.raw.push_unchecked(element) };
      return;
    }

    unsafe {
      self.raw.set_length(0);
      ptr::NonNull::copy_from(
        self.raw.element_ptr(index + 1),
        self.raw.element_ptr(index),
        count - index,
      );
      self.raw.write(index, element);
      self.raw.set_length(count + 1);
    }
  }

  #[inline]
  pub fn remove(&mut self, index: usize) -> T {
    let count = self.len();
    if index >= count {
      panic!("index is out of bounds");
    }

    if index == count - 1 {
      return unsafe { self.raw.pop_unchecked() };
    }

    let out: T;
    unsafe {
      self.raw.set_length(0);
      out = self.raw.read(index);
      ptr::NonNull::copy_from(
        self.raw.element_ptr(index),
        self.raw.element_ptr(index + 1),
        count - index - 1,
      );
      self.raw.set_length(count - 1);
    }
    out
  }

  /// Moves all the elements of `other` into `self`, leaving `other` empty
  #[inline]
  pub fn append(&mut self, other: &mut SlimVec<T>) {
    let other_length = other.len();
    let self_length = self.len();
    if other_length > 0 {
      self.reserve(other_length);
      unsafe {
        other.raw.set_length(0);
        ptr::NonNull::copy_from(
          self.raw.element_ptr(self_length),
          other.raw.buffer_ptr(),
          other_length,
        );
        self.raw.set_length(self_length + other_length)
      }
    }
  }

  #[inline]
  pub fn extend_from_slice(&mut self, other: &[T])
  where
    T: Clone,
  {
    <SlimVec<T> as Extend<T>>::extend(self, other.iter().cloned());
  }

  #[inline]
  pub fn extend_from_within<R>(&mut self, src: R)
  where
    T: Clone,
    R: RangeBounds<usize> + Clone,
  {
    let range = conform_range(src, self.len());
    let additional = range.len();
    self.reserve(additional);
    for index in range {
      debug_assert!(self.raw.is_allocated() || T::IS_ZST);
      unsafe {
        let element: T = self.get_unchecked(index).clone();
        self.raw.push_unchecked(element);
      };
    }
  }

  /// Retains only the elements specified by the predicate
  ///
  /// Visits in order each element exactly once. Preserves the relative
  /// ordering of retained elements.
  #[inline]
  pub fn retain<F>(&mut self, mut keep: F)
  where
    F: FnMut(&T) -> bool,
  {
    let count = self.len();
    let mut new_len = 0;
    unsafe { self.raw.set_length(0) };
    for i in 0..count {
      let v: &mut T = unsafe { self.raw.element_ptr(i).as_mut() };
      if keep(v) {
        let v: T = unsafe { self.raw.read(i) };
        unsafe { self.raw.write(new_len, v) };
        new_len += 1;
      } else {
        unsafe { ptr::drop_in_place(v) };
      }
    }
    unsafe { self.raw.set_length(new_len) };
  }

  /// Retains only the elements specified by the predicate
  ///
  /// Visits in order each element exactly once. Preserves the relative
  /// ordering of retained elements. The elements may be mutated by the
  /// predicate closure.
  #[inline]
  pub fn retain_mut<F>(&mut self, mut keep: F)
  where
    F: FnMut(&mut T) -> bool,
  {
    let count = self.len();
    let mut new_len = 0;
    unsafe { self.raw.set_length(0) };
    for i in 0..count {
      let v: &mut T = unsafe { self.raw.element_ptr(i).as_mut() };
      if keep(v) {
        let v: T = unsafe { self.raw.read(i) };
        unsafe { self.raw.write(new_len, v) };
        new_len += 1;
      } else {
        unsafe { ptr::drop_in_place(v) };
      }
    }
    unsafe { self.raw.set_length(new_len) };
  }

  #[inline]
  pub fn resize(&mut self, new_length: usize, value: T)
  where
    T: Clone,
  {
    if new_length > self.len() {
      self.extend(::core::iter::repeat_n(value, new_length - self.len()));
    } else {
      self.truncate(new_length);
    }
  }

  #[inline]
  pub fn resize_with<F>(&mut self, new_length: usize, f: F)
  where
    F: FnMut() -> T,
  {
    if new_length > self.len() {
      let additional = new_length - self.len();
      self.extend(::core::iter::repeat_with(f).take(additional));
    } else {
      self.truncate(new_length);
    }
  }

  #[inline]
  pub fn shrink_to_fit(&mut self) {
    self.shrink_to(0);
  }

  #[inline]
  pub fn shrink_to(&mut self, min_capacity: usize) {
    if min_capacity < self.capacity() {
      self.raw.shrink_to(min_capacity);
    }
  }

  #[inline]
  pub fn split_off(&mut self, at: usize) -> SlimVec<T> {
    assert!(at <= self.len(), "invalid index");
    let range = at..self.len();
    let mut other = SlimVec::with_capacity(range.len());
    if !range.is_empty() {
      unsafe {
        ptr::NonNull::copy_from_nonoverlapping(
          other.raw.buffer_ptr(),
          self.raw.element_ptr(range.start),
          range.len(),
        );
        self.raw.set_length(at);
        other.raw.set_length(range.len());
      };
    }
    other
  }

  /// Removes from the vector all but the first of consecutive elements
  ///
  /// Elements are compared according to `PartialEq`.
  #[inline]
  pub fn dedup(&mut self)
  where
    T: PartialEq,
  {
    self.dedup_by(|a, b| a == b)
  }

  /// Removes from the vector all but the first of consecutive elements which
  /// resolve to the same key
  ///
  /// Keys are compared according to `PartialEq`.
  #[inline]
  pub fn dedup_by_key<F, K>(&mut self, mut key: F)
  where
    F: FnMut(&mut T) -> K,
    K: PartialEq,
  {
    self.dedup_by(|a, b| key(a) == key(b))
  }

  /// Removes from the vector all but the first of consecutive elements which
  /// satisfy the provided equality relation
  ///
  /// The `is_equal` closure is passed references to two elements in the
  /// vector, and its result decides whether the second element will be
  /// removed.
  ///
  /// Note: For conformance with the behaviour of [`Vec::dedup_by`] the
  /// arguments to `is_equal` are passed in the opposite order from their order
  /// in the vector.
  #[inline]
  pub fn dedup_by<F>(&mut self, mut is_equal: F)
  where
    F: FnMut(&mut T, &mut T) -> bool,
  {
    let count = self.len();
    if count == 0 {
      return;
    }
    let mut new_len = 0;
    unsafe { self.raw.set_length(0) };

    let mut bucket_e: ptr::NonNull<T> = unsafe { self.raw.element_ptr(0) };
    for i in 1..count {
      let mut v: ptr::NonNull<T> = unsafe { self.raw.element_ptr(i) };
      if !is_equal(unsafe { v.as_mut() }, unsafe { bucket_e.as_mut() }) {
        unsafe { self.raw.element_ptr(new_len).copy_from(bucket_e, 1) };
        bucket_e = v;
        new_len += 1;
      } else {
        unsafe { ptr::NonNull::drop_in_place(v) };
      }
    }
    unsafe { self.raw.element_ptr(new_len).copy_from(bucket_e, 1) };
    new_len += 1;
    unsafe { self.raw.set_length(new_len) };
  }

  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
  where
    R: RangeBounds<usize>,
  {
    Drain::new(self, range)
  }

  #[inline]
  pub fn extract_if<F, R>(
    &mut self,
    range: R,
    filter: F,
  ) -> ExtractIf<'_, T, F>
  where
    F: FnMut(&mut T) -> bool,
    R: RangeBounds<usize>,
  {
    ExtractIf::new(self, range, filter)
  }

  #[inline]
  pub fn splice<R, I>(
    &mut self,
    range: R,
    replace_with: I,
  ) -> Splice<'_, I::IntoIter>
  where
    R: RangeBounds<usize>,
    I: IntoIterator<Item = T>,
  {
    Splice::new(self, range, replace_with)
  }

  #[inline]
  pub fn leak<'a>(self) -> &'a mut [T] {
    if !(self.raw.is_allocated() || T::IS_ZST) {
      return &mut [];
    }

    let (buffer_ptr, length, _capacity) = self.raw.into_parts();
    unsafe { slice::from_raw_parts_mut(buffer_ptr.as_ptr(), length) }
  }

  /// Get a mutable slice to the spare capacity at the tail of the `SlimVec` as
  /// [`MaybeUninit<T>`](mem::MaybeUninit)
  ///
  /// This may be used in combination with [`set_len`](Self::set_len).
  #[inline]
  pub fn spare_capacity_mut(&mut self) -> &mut [mem::MaybeUninit<T>] {
    let (length, capacity) = (self.len(), self.capacity());
    if T::IS_ZST {
      unsafe {
        slice::from_raw_parts_mut(ptr::dangling_mut(), capacity - length)
      }
    } else if self.raw.is_allocated() {
      unsafe {
        slice::from_raw_parts_mut(
          self.raw.element_ptr(length).as_ptr().cast(),
          capacity - length,
        )
      }
    } else {
      &mut []
    }
  }

  /// Manually set the length of the `SlimVec`
  ///
  /// This is a low-level utility that may be used in combination with
  /// [`spare_capacity_mut`](`Self::spare_capacity_mut`).
  ///
  /// # Safety
  ///
  /// - `new_length` must be less than or equal to `self.capacity()`.
  /// - All elements in the range `0..new_length` must be initialised.
  #[inline]
  pub unsafe fn set_len(&mut self, new_length: usize) {
    debug_assert!(
      new_length <= self.capacity(),
      "new_length must be less than or equal to self.capacity()"
    );
    // Note: If the vector is not allocated then the capacity is zero. The
    // only valid new_length the caller could have passed in this case is
    // also zero. So there is nothing to do.
    if self.raw.is_allocated() || T::IS_ZST {
      // safety:
      // - The vector is allocated or `T` is zero-sized.
      // - Caller promises that `new_length <= capacity`.
      // - Caller promises that elements in `0..new_length` are initialised.
      unsafe { self.raw.set_length(new_length) };
    }
  }
}

impl<T, const N: usize> SlimVec<[T; N]> {
  #[inline]
  pub fn into_flattened(self) -> SlimVec<T> {
    let (ptr, len, cap): (ptr::NonNull<[T; N]>, usize, usize) =
      self.raw.into_parts();

    let (new_len, new_cap) = if T::IS_ZST {
      let new_len = N
        .checked_mul(len)
        .filter(|&new_len| new_len <= cap)
        .expect("slimvec len overflow");
      (new_len, cap)
    } else {
      (N * len, N * cap)
    };

    let raw =
      unsafe { RawSlimVec::<T>::from_parts(ptr.cast(), new_len, new_cap) };
    SlimVec { raw }
  }
}

mod macros {
  use crate::SlimVec;

  /// Creates a new [`SlimVec`] using array-expression syntax
  ///
  /// The `slimvec!` macro allows a `SlimVec` to be created using the same
  /// syntax as the Standard Library's [`vec!`] macro.
  ///
  /// [`vec!`]: https://doc.rust-lang.org/alloc/macro.vec.html
  #[macro_export]
  macro_rules! slimvec {
    // `slimvec![]`
    () => {
      $crate::SlimVec::new()
    };
    // `slimvec![T; N];`
    ($element:expr; $count:expr) => {
      $crate::SlimVec::from_element($element, $count)
    };
    // `slimvec![a, b, c, ..];`
    ($($element:expr),+ $(,)?) => {
      // Convert from array since that code-path does only a single allocation,
      // and moves elements via a single mem-copy in the worst case.
      $crate::SlimVec::from([$($element),+])
    };
  }

  impl<T> SlimVec<T> {
    /// Create a new `SlimVec<T>` with `count` clones of `element`
    ///
    /// This is used by the `slimvec!` macro.
    #[doc(hidden)]
    #[inline]
    pub fn from_element(element: T, count: usize) -> Self
    where
      T: Clone,
    {
      let mut slimvec = SlimVec::new();
      slimvec.reserve_exact(count);
      slimvec.extend(::core::iter::repeat_n(element, count));
      slimvec
    }
  }
}

mod ops {
  use {
    crate::SlimVec,
    ::core::{
      ops::{Deref, DerefMut, Index, IndexMut},
      slice::SliceIndex,
    },
  };

  impl<T, I> Index<I> for SlimVec<T>
  where
    I: SliceIndex<[T]>,
  {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
      <[T]>::index(self.as_slice(), index)
    }
  }

  impl<T, I> IndexMut<I> for SlimVec<T>
  where
    I: SliceIndex<[T]>,
  {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
      <[T]>::index_mut(self.as_mut_slice(), index)
    }
  }

  impl<T> Deref for SlimVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
      self.as_slice()
    }
  }

  impl<T> DerefMut for SlimVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
      self.as_mut_slice()
    }
  }
}

mod iter {
  use {
    crate::{IntoIter, SlimVec},
    ::core::{
      iter::{Extend, FromIterator, IntoIterator},
      slice,
    },
  };

  impl<T> FromIterator<T> for SlimVec<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> SlimVec<T> {
      let mut slim = SlimVec::new();
      slim.extend(iter);
      slim
    }
  }

  impl<T> IntoIterator for SlimVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> IntoIter<T> {
      IntoIter::new(self)
    }
  }

  impl<'v, T> IntoIterator for &'v SlimVec<T> {
    type Item = &'v T;
    type IntoIter = slice::Iter<'v, T>;

    #[inline]
    fn into_iter(self) -> slice::Iter<'v, T> {
      self.as_slice().iter()
    }
  }

  impl<'v, T> IntoIterator for &'v mut SlimVec<T> {
    type Item = &'v mut T;
    type IntoIter = slice::IterMut<'v, T>;

    #[inline]
    fn into_iter(self) -> slice::IterMut<'v, T> {
      self.as_mut_slice().iter_mut()
    }
  }

  impl<T> Extend<T> for SlimVec<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
      let mut iter = iter.into_iter();

      // Preallocate using the size-hint if it appears to be exact.
      if let (size_hint, Some(upper)) = iter.size_hint()
        && size_hint == upper
      {
        self.reserve(size_hint);
      }

      // Avoid bounds checks.
      for element in iter.by_ref().take(self.capacity() - self.len()) {
        // safety: the range of the for-loop guarantees there is sufficient
        // capacity.
        unsafe { self.raw.push_unchecked(element) };
      }

      // Handle any remaining elements.
      for element in iter {
        self.push(element);
      }
    }
  }

  impl<'a, T> Extend<&'a T> for SlimVec<T>
  where
    T: Copy,
  {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
      Extend::<T>::extend(self, iter.into_iter().copied());
    }
  }
}

#[cfg(feature = "std")]
mod io {
  use {crate::SlimVec, ::std::io};

  impl io::Write for SlimVec<u8> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
      self.extend(buf);
      Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }
}

mod default {
  use {crate::SlimVec, ::core::default::Default};

  impl<T> Default for SlimVec<T> {
    #[inline]
    fn default() -> Self {
      Self::new()
    }
  }
}

mod fmt {
  use {crate::SlimVec, ::core::fmt};

  impl<T> fmt::Debug for SlimVec<T>
  where
    T: fmt::Debug,
  {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      fmt::Debug::fmt(self.as_slice(), f)
    }
  }
}

mod hash {
  use {
    crate::SlimVec,
    ::core::hash::{Hash, Hasher},
  };

  impl<T> Hash for SlimVec<T>
  where
    T: Hash,
  {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
      H: Hasher,
    {
      Hash::hash(self.as_slice(), state)
    }
  }
}

mod borrow {
  use {
    crate::SlimVec,
    ::core::borrow::{Borrow, BorrowMut},
  };

  impl<T> Borrow<[T]> for SlimVec<T> {
    #[inline]
    fn borrow(&self) -> &[T] {
      self.as_slice()
    }
  }

  impl<T> BorrowMut<[T]> for SlimVec<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
      self.as_mut_slice()
    }
  }
}

mod convert {
  use {
    crate::SlimVec,
    ::alloc::{boxed::Box, vec::Vec},
    ::core::{
      convert::{AsMut, AsRef},
      mem, ptr,
    },
  };

  impl<T> AsRef<Self> for SlimVec<T> {
    #[inline]
    fn as_ref(&self) -> &Self {
      self
    }
  }

  impl<T> AsMut<Self> for SlimVec<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
      self
    }
  }

  impl<T> AsRef<[T]> for SlimVec<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
      self.as_slice()
    }
  }

  impl<T> AsMut<[T]> for SlimVec<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
      self.as_mut_slice()
    }
  }

  impl<T, const N: usize> From<[T; N]> for SlimVec<T> {
    #[inline]
    fn from(array: [T; N]) -> SlimVec<T> {
      let mut slim = SlimVec::new();
      if N == 0 {
        return slim;
      }

      // move values from `array` into `slim`.
      slim.reserve_exact(N);
      let array_ptr: ptr::NonNull<[T; N]> = ptr::NonNull::from(&array);
      unsafe {
        ptr::NonNull::copy_from_nonoverlapping(
          slim.raw.buffer_ptr(),
          array_ptr.cast(),
          N,
        )
      };
      // drop array without calling destructors for its elements.
      _ = mem::ManuallyDrop::new(array);

      unsafe { slim.raw.set_length(N) }
      slim
    }
  }

  impl<T> From<Box<[T]>> for SlimVec<T> {
    #[inline]
    fn from(mut slice: Box<[T]>) -> SlimVec<T> {
      let count = slice.len();
      let mut slim = SlimVec::new();
      if count == 0 {
        return slim;
      }

      // move values from `slice` into `slim`.
      slim.reserve_exact(count);
      unsafe {
        ptr::copy_nonoverlapping(
          slice.as_mut_ptr(),
          slim.raw.buffer_ptr().as_ptr(),
          count,
        );
      }

      // drop boxed-slice memory without calling destructors for its elements.
      let box_ptr = Box::into_raw(slice) as *mut [mem::ManuallyDrop<T>];
      let _: Box<[mem::ManuallyDrop<T>]> = unsafe { Box::from_raw(box_ptr) };

      unsafe { slim.raw.set_length(count) };
      slim
    }
  }

  impl<T> From<Vec<T>> for SlimVec<T> {
    #[inline]
    fn from(mut other: Vec<T>) -> Self {
      let count = other.len();
      let mut slim = SlimVec::new();
      if count == 0 {
        return slim;
      };

      // move values from `other` into `slim`.
      slim.reserve_exact(count);
      unsafe {
        ptr::copy_nonoverlapping(
          other.as_ptr(),
          slim.raw.buffer_ptr().as_ptr(),
          count,
        )
      };
      unsafe { other.set_len(0) };
      unsafe { slim.raw.set_length(count) };
      slim
    }
  }

  impl<T> From<&[T]> for SlimVec<T>
  where
    T: Clone,
  {
    #[inline]
    fn from(slice: &[T]) -> SlimVec<T> {
      let mut slim = SlimVec::new();
      slim.extend(slice.iter().cloned());
      slim
    }
  }

  impl<T> From<&mut [T]> for SlimVec<T>
  where
    T: Clone,
  {
    #[inline]
    fn from(slice: &mut [T]) -> SlimVec<T> {
      let slice: &[T] = slice;
      SlimVec::<T>::from(slice)
    }
  }

  impl<T, const N: usize> From<&[T; N]> for SlimVec<T>
  where
    T: Clone,
  {
    #[inline]
    fn from(array: &[T; N]) -> SlimVec<T> {
      let slice: &[T] = array;
      SlimVec::<T>::from(slice)
    }
  }

  impl<T, const N: usize> From<&mut [T; N]> for SlimVec<T>
  where
    T: Clone,
  {
    #[inline]
    fn from(array: &mut [T; N]) -> SlimVec<T> {
      let slice: &[T] = array;
      SlimVec::<T>::from(slice)
    }
  }
}

mod cmp {
  use {crate::SlimVec, ::alloc::vec::Vec, ::core::cmp::Ordering};

  impl<T> Ord for SlimVec<T>
  where
    T: Ord,
  {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
      self.as_slice().cmp(other.as_slice())
    }
  }

  impl<T> PartialOrd for SlimVec<T>
  where
    T: PartialOrd,
  {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      self.as_slice().partial_cmp(other.as_slice())
    }
  }

  impl<T> Eq for SlimVec<T> where T: Eq {}

  impl<T, U> PartialEq<SlimVec<T>> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &SlimVec<T>) -> bool {
      self.as_slice() == other.as_slice()
    }
  }

  impl<T, U> PartialEq<[T]> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &[T]) -> bool {
      self.as_slice() == other
    }
  }

  impl<T, U> PartialEq<SlimVec<T>> for [U]
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &SlimVec<T>) -> bool {
      self == other.as_slice()
    }
  }

  impl<T, U> PartialEq<&[T]> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &&[T]) -> bool {
      self.as_slice() == *other
    }
  }

  impl<T, U> PartialEq<SlimVec<T>> for &[U]
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &SlimVec<T>) -> bool {
      *self == other.as_slice()
    }
  }

  impl<T, U> PartialEq<&mut [T]> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &&mut [T]) -> bool {
      self.as_slice() == *other
    }
  }

  impl<T, U> PartialEq<SlimVec<T>> for &mut [U]
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &SlimVec<T>) -> bool {
      *self == other.as_slice()
    }
  }

  impl<T, U, const N: usize> PartialEq<[T; N]> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &[T; N]) -> bool {
      self.as_slice() == other.as_slice()
    }
  }

  impl<T, U, const N: usize> PartialEq<&[T; N]> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &&[T; N]) -> bool {
      self.as_slice() == other.as_slice()
    }
  }

  impl<T, U> PartialEq<Vec<T>> for SlimVec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &Vec<T>) -> bool {
      self.as_slice() == other.as_slice()
    }
  }

  impl<T, U> PartialEq<SlimVec<T>> for Vec<U>
  where
    U: PartialEq<T>,
  {
    #[inline]
    fn eq(&self, other: &SlimVec<T>) -> bool {
      self.as_slice() == other.as_slice()
    }
  }
}
