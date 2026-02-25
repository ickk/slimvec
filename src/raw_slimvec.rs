// © ickk 2026.

use {
  crate::utils::TypeMeta,
  ::alloc::alloc::{Layout, alloc, dealloc, realloc},
  ::core::{mem, num::NonZero, ops::Range, panic::UnwindSafe, ptr},
};

pub(crate) struct RawSlimVec<T> {
  /// This pointer must not be dereferenced or used for any memory accesses
  /// unless [`Self::is_allocated`] returns `true`.
  ///
  /// ---
  ///
  /// This pointer is managed according to [Strict Provenance].
  ///
  /// For zero-sized T:
  /// - This pointer directly encodes the length of the vector inline, using
  ///   [`without_provenance`].
  ///
  /// For non-zero-sized T, either:
  /// - This pointer lacks provenace, and has the value
  ///   [`SENTINEL_UNALLOCATED`] indicating there is no allocation; or
  /// - This pointer is valid for accesses to an allocated [`HeapData<T>`] and
  ///   its associated buffer.
  ///
  /// [Strict Provenance]: https://doc.rust-lang.org/std/ptr/index.html#strict-provenance
  /// [`without_provenance`]: ::core::ptr::without_provenance
  /// [`SENTINEL_UNALLOCATED`]: Self::SENTINEL_UNALLOCATED
  ptr: ptr::NonNull<HeapData<T>>,
}

#[repr(C)]
pub(crate) struct HeapData<T> {
  length: usize,
  capacity: NonZero<usize>,
  buffer: Buffer<T>,
}

/// A placeholder type with the correct alignment for a buffer containing a
/// list of elements of type `T`
///
/// The actual buffer is dynamically-sized with explicit memory management.
/// However this zero-sized place-holder helps force the correct alignment and
/// offset within `HeapData`.
#[repr(transparent)]
pub(crate) struct Buffer<T>([mem::MaybeUninit<T>; 0]);

macro_rules! assert_zst {
  ($t:ty) => {
    assert!(
      ::core::mem::size_of::<$t>() == 0,
      stringify!($t must be zero-sized)
    )
  };
}
macro_rules! assert_not_zst {
  ($t:ty) => {
    assert!(
      ::core::mem::size_of::<$t>() != 0,
      stringify!($t must not be zero-sized)
    )
  };
}

/// Encode a length into a NonNull pointer
///
/// # Safety
///
/// `length` must be less than or equal to [`MAX_CAPACITY`].
#[inline]
const unsafe fn zst_encode_length<T>(
  length: usize,
) -> ptr::NonNull<HeapData<T>> {
  debug_assert!(
    length <= RawSlimVec::<T>::MAX_CAPACITY,
    "safety criteria must be met"
  );

  let encoded_length = unsafe { NonZero::new(!length).unwrap_unchecked() };
  ptr::NonNull::without_provenance(encoded_length)
}

/// Decode a length from a NonNull pointer
///
/// # Panics
///
/// Panics if `T` is not zero-sized.
#[inline]
fn zst_decode_length<T>(ptr: ptr::NonNull<HeapData<T>>) -> usize {
  assert_zst!(T);
  !ptr.addr().get()
}

impl<T> Buffer<T> {
  /// Compute the [`Layout`] for a buffer able to contain `buffer_capacity`
  /// elements of type `T`
  ///
  /// # Panics
  ///
  /// - Panics if `T` is zero-sized.
  /// - Panics if the layout cannot be created.
  #[inline]
  const fn buffer_layout(buffer_capacity: NonZero<usize>) -> Layout {
    assert_not_zst!(T);

    assert!(
      mem::align_of::<T>() == mem::align_of::<Buffer<T>>(),
      "The alignment of the buffer must match the alignment of its elements"
    );
    assert!(
      (mem::align_of::<HeapData<T>>() + mem::offset_of!(HeapData<T>, buffer))
        .is_multiple_of(mem::align_of::<T>()),
      "The buffer field must be correctly aligned",
    );

    let Ok(layout) = Layout::array::<T>(buffer_capacity.get()) else {
      panic!("Failed to create Layout for buffer")
    };
    layout
  }
}

impl<T> HeapData<T> {
  const HEADER_LAYOUT: Layout = {
    let layout = Layout::new::<HeapData<T>>();
    assert!(layout.size() != 0, "HEADER_LAYOUT must have non-zero size");
    layout
  };

  /// Compute the combined [`Layout`] for `HeapData<T>` and its buffer
  ///
  /// # Panics
  ///
  /// - Panics if the layout cannot be created.
  /// - Panics if `T` is zero-sized.
  #[inline]
  const fn compute_layout(buffer_capacity: NonZero<usize>) -> Layout {
    assert_not_zst!(T);
    let Ok((layout, _)) =
      Self::HEADER_LAYOUT.extend(Buffer::<T>::buffer_layout(buffer_capacity))
    else {
      panic!("Failed to create layout for HeapData");
    };
    layout
  }

  /// Allocate a new `HeapData<T>` as well as its associated buffer, with
  /// capacity for exactly `buffer_capacity` elements
  ///
  /// # Panics
  ///
  /// - Panics if allocation fails.
  /// - Panics if `T` is a zero-sized type.
  ///
  /// TODO: use [`::alloc::alloc::handle_alloc_error`] instead of panicking.
  #[inline]
  fn allocate(buffer_capacity: NonZero<usize>) -> ptr::NonNull<Self> {
    assert_not_zst!(T);

    let layout = HeapData::<T>::compute_layout(buffer_capacity);
    assert!(layout.size() != 0);
    // safety: layout has a non-zero size.
    let allocation = unsafe { alloc(layout) };
    let ptr: ptr::NonNull<HeapData<T>> = ptr::NonNull::new(allocation)
      .expect("Allocation failed")
      .cast();

    // Initialise the fields.
    unsafe {
      // safety:
      // - ptr is valid for writes; it was just allocated.
      // - capacity is the same as the layout used for the allocation.
      HeapData::write_capacity(ptr, buffer_capacity);
      // safety:
      // - capacity was just initialised.
      // - 0 is a trivially valid length.
      HeapData::write_length(ptr, 0);
    }

    ptr
  }

  /// Reallocate a `HeapData<T>` as well as its associated buffer, with
  /// capacity for exactly `new_buffer_capacity` elements
  ///
  /// The new `length` will be `min(length, new_buffer_capacity)`. The range
  /// `0..length` of the new buffer is guaranteed to have the same values as
  /// the original buffer.
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for a block of memory allocated by
  ///   [`HeapData::<T>::allocate`] or [`HeapData::<T>::reallocate`].
  /// - The `capacity` field must be initialised & correct for this allocation.
  ///
  /// Ownership of `ptr` is transferred to this function. After this call, any
  /// dangling pointers will be invalid.
  ///
  /// # Panics
  ///
  /// - Panics if reallocation fails.
  /// - Panics if `T` is a zero-sized type.
  ///
  /// TODO: use [`::alloc::alloc::handle_alloc_error`] instead of panicking.
  #[inline]
  unsafe fn reallocate(
    ptr: ptr::NonNull<Self>,
    new_buffer_capacity: NonZero<usize>,
  ) -> ptr::NonNull<Self> {
    assert_not_zst!(T);

    // safety:
    // - Caller promises that `ptr` is valid.
    // - Caller promises that `capacity` is initialised and correct.
    let old_capacity = unsafe { Self::read_capacity(ptr) };
    let old_layout = HeapData::<T>::compute_layout(old_capacity);
    let new_computed_layout =
      HeapData::<T>::compute_layout(new_buffer_capacity);
    let new_layout =
      Layout::from_size_align(new_computed_layout.size(), old_layout.align())
        .expect("Failed to create layout for HeapData");
    assert!(
      new_computed_layout == new_layout,
      "The newly computed layout must match the definition for the layout that
      will be returned from `alloc::realloc`"
    );
    assert!(new_computed_layout.size() != 0);

    // safety:
    // - Caller promises that the pointer is valid.
    // - `old_layout` is the same, since the caller promises that `capacity` is
    //   correct.
    // - `new_size` has a non-zero size.
    let new_allocation = unsafe {
      realloc(ptr.as_ptr().cast(), old_layout, new_computed_layout.size())
    };
    let new_ptr: ptr::NonNull<HeapData<T>> = ptr::NonNull::new(new_allocation)
      .expect("Reallocation Failed")
      .cast();
    // Reinitialise the capacity.
    // safety:
    // - `new_ptr` is valid for reads and writes since it was just successfully
    //   reallocated.
    // - `new_buffer_capacity` is trivially correct for the new allocation; it
    //   was used to compute the new layout.
    unsafe { HeapData::write_capacity(new_ptr, new_buffer_capacity) };

    new_ptr
  }

  /// Deallocates the `HeapData<T>` and its associated buffer, without calling
  /// any destructors for its elements
  ///
  /// Call [`Self::drop_in_place`] first in order to drop elements.
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for a block of memory allocated by
  ///   [`HeapData::<T>::allocate`] or [`HeapData::<T>::reallocate`].
  /// - The `capacity` field must be initialised & correct for this allocation.
  ///
  /// After this call all dangling pointers will become invalid.
  #[inline]
  unsafe fn deallocate(ptr: ptr::NonNull<Self>) {
    // safety: The caller promises that `ptr` is valid.
    let capacity = unsafe { Self::read_capacity(ptr) };
    let layout = Self::compute_layout(capacity);
    let allocation = ptr.as_ptr().cast();
    // safety: The caller promises that `capacity` is correct, so the computed
    // layout will match the layout of the allocation.
    unsafe { dealloc(allocation, layout) }
  }

  /// Call the destructor for each element in `range`
  ///
  /// Elements are dropped in reverse order.
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for reads and writes.
  /// - `range.end` must be less than or equal to the `capacity` of the buffer.
  /// - All elements in `range` must be initialised before this call.
  ///
  /// After this call, all elements in `range` will be uninitialised.
  #[inline]
  unsafe fn drop_in_place(ptr: ptr::NonNull<Self>, range: Range<usize>) {
    debug_assert!(
      range.end <= unsafe { HeapData::read_capacity(ptr).get() },
      "safety criteria must be met"
    );

    let offset = mem::offset_of!(HeapData<T>, buffer);
    // safety: The caller promises that `ptr` is valid for reads and writes.
    let buffer_ptr: ptr::NonNull<T> = unsafe { ptr.byte_add(offset).cast() };
    for i in range.rev() {
      // safety: The caller promises that the range is valid for this buffer.
      let element_ptr = unsafe { buffer_ptr.add(i) };
      // safety:
      // - The caller promises that all elements to be dropped are initialised.
      unsafe { element_ptr.drop_in_place() };
    }
  }

  /// Get a pointer to the start of the buffer
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid.
  #[inline]
  unsafe fn buffer_ptr(ptr: ptr::NonNull<Self>) -> ptr::NonNull<T> {
    let offset = mem::offset_of!(HeapData<T>, buffer);
    // safety:
    // - The caller promises that `ptr` is valid.
    // - This also means adding the offset will not overflow `isize`.
    unsafe { ptr.byte_add(offset).cast() }
  }

  /// Write to the `length` field
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for memory writes.
  /// - all elements of the buffer in the range `0..length` must be
  ///   initialised.
  /// - `length` must be less than the capacity.
  /// - `capacity` must be initialised.
  #[inline]
  unsafe fn write_length(ptr: ptr::NonNull<Self>, length: usize) {
    assert_not_zst!(T);
    debug_assert!(
      length <= unsafe { Self::read_capacity(ptr).get() },
      "length must not exceed capacity"
    );

    let offset = mem::offset_of!(HeapData<T>, length);
    unsafe {
      let field = ptr.byte_add(offset).cast();
      field.write(length);
    }
  }

  /// Write to the `capacity` field
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for memory writes.
  /// - `capacity` must match the `buffer_capacity` specified in the call to
  ///   [`Buffer::buffer_layout`] for `ptr`'s allocation.
  #[inline]
  unsafe fn write_capacity(ptr: ptr::NonNull<Self>, capacity: NonZero<usize>) {
    assert_not_zst!(T);
    debug_assert!(
      capacity.get() <= RawSlimVec::<T>::MAX_CAPACITY,
      "capacity must not exceed MAX_CAPACITY"
    );

    let offset = mem::offset_of!(HeapData<T>, capacity);
    unsafe {
      let field = ptr.byte_add(offset).cast();
      field.write(capacity);
    }
  }

  /// Read from the `capacity` field
  ///
  /// # Safety
  ///
  /// - `ptr` must be valid for memory reads.
  /// - `capacity` must be initialised.
  #[inline]
  unsafe fn read_capacity(ptr: ptr::NonNull<Self>) -> NonZero<usize> {
    assert_not_zst!(T);
    let offset = mem::offset_of!(HeapData<T>, capacity);
    unsafe { ptr.byte_add(offset).cast().read() }
  }
}

impl<T> RawSlimVec<T> {
  /// A sentinel value used to indicate when a `SlimVec` has not yet allocated
  ///
  /// This pointer lacks provenance and is never aligned for `HeapData<T>`.
  const SENTINEL_UNALLOCATED: ptr::NonNull<HeapData<T>> = {
    let sentinel = NonZero::<usize>::MIN;
    assert!(
      !sentinel
        .get()
        .is_multiple_of(mem::align_of::<HeapData<T>>())
        && sentinel.get() < mem::align_of::<HeapData<T>>(),
      "SENTINEL_UNALLOCATED must not alias any bit-patterns for any pointer
      that could be valid for HeapData<T>"
    );
    ptr::NonNull::without_provenance(sentinel)
  };

  /// A pointer value encoding a length of zero, when T is a zero-sized-type
  const EMPTY_ZST_BUFFER: ptr::NonNull<HeapData<T>> = {
    // safety: `0` is trivially less than `MAX_CAPACITY`.
    unsafe { zst_encode_length::<T>(0) }
  };

  /// An empty `RawSlimVec<T>`
  pub(crate) const EMPTY: RawSlimVec<T> = {
    assert!(
      mem::size_of::<Self>() == mem::size_of::<Option<Self>>(),
      "SlimVec must contain a niche"
    );
    if T::IS_ZST {
      RawSlimVec {
        ptr: Self::EMPTY_ZST_BUFFER,
      }
    } else {
      RawSlimVec {
        ptr: Self::SENTINEL_UNALLOCATED,
      }
    }
  };

  /// The maximum buffer capacity
  ///
  /// For allocating-buffers the maximum capacity is guaranteed to be less than
  /// or equal to `isize::MAX` by [`core::alloc::GlobalAlloc`]. This same limit
  /// is enforced for buffers of zero-sized-types.
  pub(crate) const MAX_CAPACITY: usize = {
    if T::IS_ZST {
      isize::MAX as usize
    } else {
      isize::MAX as usize / mem::size_of::<T>()
    }
  };

  /// If [`Self::ptr`] is a valid pointer to an allocated `HeapData` then this
  /// will return `true`
  ///
  /// Otherwise there is no allocation and `Self::ptr` must not be dereferenced
  /// or used for any memory accesses.
  #[inline]
  pub(crate) fn is_allocated(&self) -> bool {
    if T::IS_ZST {
      return false;
    }

    self.ptr != Self::SENTINEL_UNALLOCATED
  }

  /// Get a reference to the `HeapData` of the vector
  #[inline]
  pub(crate) fn get_heap_data(&self) -> Option<&HeapData<T>> {
    if self.is_allocated() {
      Some(unsafe { self.ptr.as_ref() })
    } else {
      None
    }
  }

  /// Get the capacity of the vector
  #[inline]
  pub(crate) fn capacity(&self) -> usize {
    if T::IS_ZST {
      return Self::MAX_CAPACITY;
    }

    if let Some(heap_data) = self.get_heap_data() {
      heap_data.capacity.get()
    } else {
      0
    }
  }

  /// Get the length of the vector
  #[inline]
  pub(crate) fn length(&self) -> usize {
    if T::IS_ZST {
      return zst_decode_length(self.ptr);
    }

    if let Some(heap_data) = self.get_heap_data() {
      heap_data.length
    } else {
      0
    }
  }

  /// Set the length
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero-sized.
  /// - `new_length` must be less than or equal to `self.capacity()`.
  /// - `new_length` must be less than or equal to [`MAX_CAPACITY`].
  /// - all elements in the range `0..new_length` must be initialised.
  ///
  /// After this call elements in the range `new_length..` are permitted to be
  /// uninitialised.
  #[inline]
  pub(crate) unsafe fn set_length(&mut self, new_length: usize) {
    debug_assert!(
      (self.is_allocated() || T::IS_ZST)
        && (new_length <= self.capacity())
        && (new_length <= Self::MAX_CAPACITY),
      "safety criteria must be met"
    );

    if T::IS_ZST {
      // safety: The caller promises `new_length <= MAX_CAPACITY`.
      self.ptr = unsafe { zst_encode_length(new_length) };
      return;
    }

    // safety:
    // - The caller promises that the vector is allocated.
    // - The caller promises that elements in 0..new_length are initialised.
    // - `ptr` is valid for writes since we have `&mut self`.
    unsafe { HeapData::write_length(self.ptr, new_length) };
  }

  /// Allocate
  ///
  /// If `T` is zero-sized, this reduces to an assertion that the requested
  /// capacity does not exceed [`MAX_CAPACITY`].
  ///
  /// This should only be called if `Self::is_allocated` returns `false`.
  ///
  /// The existing allocation, if any, is replaced with a new allocation that
  /// has a buffer with the requested capacity.
  ///
  /// Note: If there is an existing allocation, then it will leak and
  /// destructors may not run.
  ///
  /// Use [`Self::reallocate`] to replace an allocation, moving existing
  /// elements.
  ///
  /// Use [`Self::drop_in_place`] and [`Self::deallocate`] first in order to
  /// drop elements and free an existing allocation.
  ///
  /// # Safety
  ///
  /// After this call, all dangling pointers will be invalid.
  ///
  /// # Panics
  ///
  /// - Panics if allocation fails.
  #[inline]
  pub(crate) fn allocate(&mut self, buffer_capacity: NonZero<usize>) {
    if T::IS_ZST {
      assert!(
        buffer_capacity.get() <= Self::MAX_CAPACITY,
        "required capacity exceeds max buffer capacity"
      );
      return;
    }

    self.ptr = HeapData::<T>::allocate(buffer_capacity);
  }

  /// Reallocate
  ///
  /// If `T` is zero-sized, this reduces to an assertion that the requested
  /// capacity does not exceed [`MAX_CAPACITY`].
  ///
  /// Replaces the current allocation with a new allocation containing a buffer
  /// with the requested capacity.
  ///
  /// Moves existing elements from the original allocation to the new
  /// allocation.
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero-sized.
  ///
  /// After this call, all dangling pointers will be invalid.
  ///
  /// # Panics
  ///
  /// - Panics if allocation fails.
  pub(crate) unsafe fn reallocate(&mut self, buffer_capacity: NonZero<usize>) {
    debug_assert!(
      self.is_allocated() || T::IS_ZST,
      "safety criteria must be met"
    );

    if T::IS_ZST {
      assert!(
        buffer_capacity.get() <= Self::MAX_CAPACITY,
        "required capacity exceeds max buffer capacity"
      );
      return;
    }

    // If a panic occurs during realloc, prevent the destructor from running,
    // and maintain invariants for `UnwindSafe`.
    let ptr = self.ptr;
    self.ptr = Self::SENTINEL_UNALLOCATED;
    self.ptr = unsafe { HeapData::<T>::reallocate(ptr, buffer_capacity) };
  }

  /// Free the allocation
  ///
  /// If `T` is zero-sized, this is a no-op.
  ///
  /// Note: If there are elements, their destructors may not run. Call
  /// [`Self::drop_in_place`] to free them first.
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero-sized.
  ///
  /// After this call all dangling pointers will be invalid.
  #[inline]
  pub(crate) unsafe fn deallocate(&mut self) {
    debug_assert!(
      self.is_allocated() || T::IS_ZST,
      "safety criteria must be met"
    );

    if T::IS_ZST {
      return;
    }

    // If a panic occurs during dealloc, prevent the destructor from running,
    // and maintain invariants for `UnwindSafe`.
    let ptr = self.ptr;
    self.ptr = Self::SENTINEL_UNALLOCATED;
    unsafe { HeapData::deallocate(ptr) };
  }

  /// Call the destructor for each element in `range`
  ///
  /// Elements are dropped in reverse order.
  ///
  /// This is a no-op if `range` is empty.
  ///
  /// # Safety
  ///
  /// - `range.end` must be less than or equal to the `capacity` of the buffer.
  /// - All elements in `range` must be initialised before this call.
  ///
  /// After this call, all elements in `range` will be considered to be
  /// uninitialised. It is wise to first set `length` to some value less than
  /// `range.start` to maintain the validity of the `RawSlimVec`.
  #[inline]
  pub(crate) unsafe fn drop_in_place(&mut self, range: Range<usize>) {
    debug_assert!(range.end <= self.capacity(), "safety criteria must be met");

    if range.is_empty() {
      return;
    }

    if ::core::mem::needs_drop::<T>() {
      if T::IS_ZST {
        for _ in range.rev() {
          // safety:
          // - `T` has a size of 0, so is trivially valid for reads and writes.
          // -  `dangling` pointer is properly aligned.
          unsafe { ptr::NonNull::<T>::dangling().drop_in_place() };
        }
        return;
      }

      // safety:
      // - `self.ptr` is valid for reads and writes since we have `&mut self`.
      // - The caller promises `range` is valid for this buffer.
      // - `range` is non-empty, so `self.ptr` must be allocated based on the
      //   caller's promise.
      // - The caller promises all elements in `range` are initialised.
      unsafe { HeapData::drop_in_place(self.ptr, range) };
    }
  }

  /// Returns a `NonNull` pointer to the start of the buffer
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero-sized.
  #[inline]
  pub(crate) unsafe fn buffer_ptr(&self) -> ptr::NonNull<T> {
    debug_assert!(
      self.is_allocated() || T::IS_ZST,
      "safety criteria must be met"
    );

    // safety:
    // - `self.ptr` is valid since the caller promises that the vector has
    //   allocated if `T` is non-zero-sized.
    // - 0 is trivially less than the (NonZero) capacity of an allocated buffer
    //   of non-zero-sized `T`, and also trivially less than [`MAX_CAPACITY`]
    //   for zero-sized `T`.
    unsafe { self.element_ptr(0) }
  }

  /// Returns a `NonNull` pointer to the element at `index` in the vector's
  /// buffer
  ///
  /// Note that while an index equal to the capacity is valid, it is valid only
  /// for zero-length reads.
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero-sized.
  /// - `index` must be less than or equal to the `capacity`.
  #[inline]
  pub(crate) unsafe fn element_ptr(&self, index: usize) -> ptr::NonNull<T> {
    debug_assert!(
      (self.is_allocated() || T::IS_ZST) && index <= self.capacity(),
      "safety criteria must be met"
    );

    if T::IS_ZST {
      return ptr::NonNull::dangling();
    }

    // safety:
    // - `self.ptr` is valid since the caller promises that the vector has
    //   allocated.
    // - The caller promises that index is within bounds of the buffer.
    unsafe { HeapData::buffer_ptr(self.ptr).add(index) }
  }

  /// Perform a memory write to the element at `index` in the buffer
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero_sized.
  /// - `index` must be less than `capacity`.
  ///
  /// If there was already an initialised element at `index` then its
  /// destructor will not run.
  #[inline]
  pub(crate) unsafe fn write(&mut self, index: usize, value: T) {
    debug_assert!(
      (self.is_allocated() || T::IS_ZST) && index < self.capacity(),
      "safety criteria must be met"
    );

    unsafe { self.element_ptr(index).write(value) }
  }

  /// Perform a memory read of the the element at `index` in the buffer
  ///
  /// # Safety
  ///
  /// - The vector must have allocated, or `T` must be zero_sized.
  /// - `index` must be less than `capacity`.
  /// - The element at `index` must be initialised.
  ///
  /// After this call, the value in the buffer at `index` will be considered to
  /// be uninitialised.
  #[inline]
  pub(crate) unsafe fn read(&self, index: usize) -> T {
    debug_assert!(
      (self.is_allocated() || T::IS_ZST) && index < self.capacity(),
      "safety criteria must be met"
    );

    unsafe { self.element_ptr(index).read() }
  }

  /// Decomposes a `RawSlimVec<T>` into `(pointer, length, capacity)`
  ///
  /// The `pointer` is a `NonNull<T>` to the start of the vector's buffer, the
  /// `length` is the number of elements in the vector, and the `capacity` is
  /// the allocated capacity of the buffer (in elements).
  ///
  /// The caller may pass these components back to `Self::from_parts` in order
  /// to reconstitute the vector and allow destructors to run.
  #[inline]
  pub(crate) fn into_parts(self) -> (ptr::NonNull<T>, usize, usize) {
    let length = self.length();
    let capacity = self.capacity();
    let ptr = if T::IS_ZST {
      ptr::NonNull::dangling()
    } else if self.is_allocated() {
      // safety: `self.ptr` is valid since the vector is allocated.
      unsafe { HeapData::buffer_ptr(self.ptr) }
    } else {
      ptr::NonNull::dangling()
    };

    mem::forget(self);
    (ptr, length, capacity)
  }

  /// Reconstitutes a `RawSlimVec<T>` from its parts
  ///
  /// It is only valid to create a `RawSlimVec` from parts created by
  /// `RawSlimVec::into_parts`. Reconstiting a `RawSlimVec` allows destructors
  /// to run and memory to be freed.
  ///
  /// # Safety
  ///
  /// - `ptr` must correspond to a live pointer created by `Self::into_parts`.
  /// - All elements of the vector in the range `0..length` must be
  ///   initialised.
  /// - `capacity` must match the capacity of the original `RawSlimVec`'s
  ///   allocation.
  /// - `length` must be less than or equal to `capacity`.
  /// - `capacity` must be less than or equal to [`MAX_CAPACITY`].
  ///
  /// Ownership of `ptr` is transferred to this function. The pointer must not
  /// be used afterwards.
  #[inline]
  pub(crate) unsafe fn from_parts(
    ptr: ptr::NonNull<T>,
    length: usize,
    capacity: usize,
  ) -> Self {
    debug_assert!(
      (length <= capacity) && (capacity <= Self::MAX_CAPACITY),
      "safety criteria must be met"
    );

    if T::IS_ZST {
      // safety: Caller promises `length <= capacity <= MAX_CAPACITY`.
      let ptr = unsafe { zst_encode_length(length) };
      return RawSlimVec { ptr };
    }

    if let Some(capacity) = NonZero::new(capacity) {
      let ptr: ptr::NonNull<HeapData<T>> = {
        let offset = mem::offset_of!(HeapData<T>, buffer);
        // safety: caller promises that `ptr` is valid for access to a
        // `RawSlimVec<T>`, so the negative offset from the buffer's ptr to the
        // `HeapData`'s ptr is also valid.
        unsafe { ptr.byte_sub(offset).cast() }
      };
      // Note: The capacity and the length must be updated, since `into_parts`
      // and `from_parts` may be used to effectively transmute between
      // `RawSlimVec<T>` and `RawSlimVec<[T; N]>` (or other similarly
      // compatible layouts).

      // safety: caller promises `ptr` is valid and that `capacity` & `length`
      // are correct.
      unsafe {
        HeapData::write_capacity(ptr, capacity);
        HeapData::write_length(ptr, length);
      }

      RawSlimVec { ptr }
    } else {
      Self::EMPTY
    }
  }

  /// Shrink the capacity of the vector to `new_capacity`
  ///
  /// The capacity will remain at least as large as `self.length()`.
  #[inline]
  pub(crate) fn shrink_to(&mut self, new_capacity: usize) {
    let new_capacity = self.length().max(new_capacity);
    if new_capacity >= self.capacity() {
      return;
    }
    if self.is_allocated() {
      if new_capacity == 0 {
        // safety: The vector is allocated.
        unsafe { self.deallocate() }
      } else {
        let new_capacity =
          NonZero::new(new_capacity).unwrap_or_else(|| unreachable!());
        // safety:
        // - The vector is allocated.
        // - Receiver is `&mut self`, so there are no dangling references.
        unsafe { self.reallocate(new_capacity) };
      }
    }
  }

  /// Grow the capacity of the vector to `new_capacity`
  ///
  /// The capacity will remain at least as large as `self.capacity()`.
  #[inline]
  pub(crate) fn grow_to(&mut self, new_capacity: usize) {
    if new_capacity <= self.capacity() {
      return;
    }
    let new_capacity =
      NonZero::new(new_capacity).unwrap_or_else(|| unreachable!());
    if self.is_allocated() {
      // safety:
      // - The vector is allocated.
      // - Receiver is `&mut self`, so there are no dangling references.
      unsafe { self.reallocate(new_capacity) };
    } else {
      self.allocate(new_capacity);
    }
  }

  /// # Safety
  ///
  /// `self.capacity()` must be greater than `self.length()`.
  #[inline]
  pub(crate) unsafe fn push_unchecked(&mut self, v: T) {
    debug_assert!(
      self.capacity() > self.length(),
      "safety criteria must be met"
    );

    let count = self.length();
    unsafe {
      self.write(count, v);
      self.set_length(count + 1);
    }
  }

  /// # Safety
  ///
  /// `self.length()` must be greater than `0`.
  #[inline]
  pub(crate) unsafe fn pop_unchecked(&mut self) -> T {
    debug_assert!(self.length() > 0, "safety criteria must be met");

    let new_length = self.length() - 1;
    unsafe {
      self.set_length(new_length);
      self.read(new_length)
    }
  }
}

// FUTURE-WORK: Properly implement drop-glue when `#[may_dangle]` (or
// equivalent) is stabilised: https://github.com/rust-lang/rust/issues/34761.
impl<T> Drop for RawSlimVec<T> {
  #[inline]
  fn drop(&mut self) {
    if T::IS_ZST || self.is_allocated() {
      let count = self.length();
      unsafe {
        self.set_length(0);
        self.drop_in_place(0..count);
        self.deallocate();
      }
    }
  }
}

impl<T> Clone for RawSlimVec<T>
where
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    if T::IS_ZST {
      return RawSlimVec { ptr: self.ptr };
    }

    let mut clone = RawSlimVec::EMPTY;
    let Some(length) = NonZero::new(self.length()) else {
      return clone;
    };
    clone.allocate(length);
    for i in 0..length.get() {
      unsafe {
        let v: &T = self.element_ptr(i).as_ref();
        clone.write(i, v.clone());
      }
    }
    unsafe { clone.set_length(length.get()) };
    clone
  }
}

unsafe impl<T> Send for RawSlimVec<T> where T: Send {}
unsafe impl<T> Sync for RawSlimVec<T> where T: Sync {}
impl<T> Unpin for RawSlimVec<T> where T: Unpin {}

// `RawSlimVec` implementation tries to maintain its invariants at all times.
//
// For non-zero-sized elements this is achieved mostly through the following
// strategies:
//
// 1. The `ptr` field of a `RawSlimVec` is always either a valid pointer or
//    `SENTINEL_UNALLOCATED`.
//
//    This is achieved by temporarily unsetting `ptr` as `SENTINEL_UNALLOCATED`
//    any time it may be invalidated, e.g. by alloc/realloc, and only setting
//    it to a real pointer again afterwards. All methods that dereference `ptr`
//    must first check for this sentinel value.
//
//    This protects against situations when the allocator itself panics.
//
// 2. All element from `0..length` of `HeapData` are always valid.
//
//    This is often achieved by setting the `length` to `0` before messing with
//    elements in the `Buffer`, or by waiting until after the ith-element is
//    valid before setting the `length` to `i+1`.
//
//    This ensures that if a panic occurs at any point all elements of
//    `RawSlimVec`, which the length promises are valid, are indeed valid.
//
// For zero-sized elements the invariants are trivially maintained.
impl<T> UnwindSafe for RawSlimVec<T> where T: UnwindSafe {}
