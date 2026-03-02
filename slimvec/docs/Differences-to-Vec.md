`slimvec` replicates the interface of the Rust Standard Library's [`Vec`]
collection. However there are some differences & limitations which are
described here, hopefully exhaustively.

[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html

If you discover other differences in the APIs of `SlimVec` & `Vec`, then please
create an issue.

### `try_reserve`

`Vec` provides the following two methods for fallible reallocation:
- `fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;`
- `fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError>;`

At this time these methods are not replicated for `SlimVec`, however they are
likely to be provided in a future release.

### Comparisons

The vast majority of `PartialEq` comparisons that are implemented for `Vec` are
also implemented for `SlimVec`. However the following are yet missing:
- `PartialEq<SlimVec<U>> for VecDeque<T> where T: PartialEq<U>`
- `PartialEq<SlimVec<U>> for Cow<'_, [T]> where T: PartialEq<U> + Clone`

These may be provided in a future release.

### Conversions

Many of the `From` conversions that are implemented for `Vec<T>` are also
implemented for `SlimVec<T>`. However the following are yet missing:
- `From<SlimVec<T>> for Box<[T]>`
- `From<SlimVec<T>> for Arc<[T]>`
- `From<SlimVec<T>> for Rc<[T]>`
- `From<&str> for SlimVec<u8>`
- `From<String> for SlimVec<u8>`
- `From<CString> for SlimVec<u8>`
- `From<SlimVec<NonZero<u8>>> for CString`
- `From<SlimVec<T>> for BinaryHeap<T> where T: Ord`
- `From<BinaryHeap<T>> for SlimVec<T>`
- `From<SlimVec<T>> for VecDeque<T>`
- `From<VecDeque<T>> for SlimVec<T>`

Similarly the following `TryFrom` implementations are yet missing:
- `TryFrom<SlimVec<T>> for [T; N]`
- `TryFrom<SlimVec<T>> for Box<[T; N]>`
- `TryFrom<SlimVec<u8>> for String`

These may be provided in a future release.

#### `Cow<[T]>`

Due to the definition & interaction of `Cow` & `ToOwned` the following is not
possible to implemented for `slimvec`, and so is omitted.

- `From<SlimVec<T>> for Cow<'a, [T]> where T: Clone`
- `From<Cow<'a, [T]>> for SlimVec<T> where [T]: ToOwned<Owned = ThinVec<T>>`
- `From<&'a SlimVec<T>> for Cow<'a, [T]> where T: Clone`

If there was interest in a copy-on-write equivalent that would work for
`SlimVec`, a new structure could be provided. If this would benefit you, please
create an issue and describe your use-case.

#### `into_boxed_slice`

`Vec` also includes the following method for conversion to a `Box<[T]>`
- `fn into_boxed_slice(self) -> Box<[T]>;`

This is redundant with `From<SlimVec<T>> for Box<[T]>` and likewise may be
provided for `slimvec` in a future release.


#### `From<Box<[T]>>`

The `From<Box<[T]>>` implementation for `Vec<T>` is able to directly transfer
ownership of the existing heap allocation in a zero-cost way. This same
optimisation is not possible in the equivalent implementation for `SlimVec<T>`
since the memory layouts differ.

### Slices

The Standard Library includes methods on `[T]` that return `Vec<T>`. These
include:
- `fn into_vec(self: Box<[T]>) -> Vec<T>;`
- `fn to_vec(&self) -> Vec<T> where T: Clone;`
- `fn repeat(&self, n: usize) -> Vec<T> where T: Copy;`

`slimvec` includes the trait `SliceExt` which provides similar functionality,
however the method names differ. The interface is as follows:
- `fn into_slimvec(self: Box<[T]>) -> SlimVec<T>;`
- `fn to_slimvec(&self) -> SlimVec<T> where T: Clone;`
- `fn repeat_to_slimvec(&self, n: usize) -> SlimVec<T> where T: Copy;`

Note well that `into_slimvec` can not perform a zero-cost conversion like
`into_vec`. Due to the differences in layout between `SlimVec<T>` & `Box<[T]>`,
`into_slimvec` necessarily involves creating a new allocation and then moving
the elements via mem-copy.

The Standard Library additionally includes the following two methods on `[u8]`
slices:
- `fn to_ascii_uppercase(&self) -> Vec<u8>;`
- `fn to_ascii_lowercase(&self) -> Vec<u8>;`

At this time equivalent methods are not provided in `slimvec`, however they may
be added to `slimvec::SliceExt` in a future release.

### Raw pointers

`Vec` includes the following methods involving raw-pointers:
- `fn as_ptr(&self) -> *const T;`
- `fn as_mut_ptr(&mut self) -> *mut T;`
- `fn into_raw_parts(self) -> (*mut T, usize, usize);`
- `unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Vec<T>;`

Internally `SlimVec` has implementations of similar functions, however they are
not made public. In particular `from-`/`into_raw_parts` have different
semantics & safety requirements.

If you need this functionality, please create an issue and describe your
use-case.

### Variance of Drain

The Standard Library's version of the `Drain` iterator is *covariant*, where-as
`slimvec`'s is *invariant*.

This is because `Drain` holds onto a mutable borrow of the underlying vector.
Long ago the Standard Library decided that so long as `Drain` never hands out
mutable references to its elements, it could in fact be made covariant.

*It is possible* for `slimvec` to also make its `Drain` covariant. However
doing so would make it more difficult to reason about, and I believe it has the
potential to cause UB other iterator adapters - In the past this happened in
the Standard Libary.

If you would benefit from `slimvec::Drain` being covariant, please create an
issue and describe your use-case.

### `#[may_dangle]`

The Standary Library has long since included an unstable way for destructors of
generic collections to declare that they will not access their elements. This
allows the borrow-checker to "close-one-eye" when it comes to the strictness of
the drop order of certain values; sometimes these inner values "may dangle".

More information about `may_dangle` can be found in the [nomicon].

[nomicon]: https://doc.rust-lang.org/nomicon/dropck.html

`SlimVec` will implement its destructor in terms of `may_dangle` when stable.

### Unwind safety

`Vec` goes to great lengths to avoid unnecessarily leaking elements during a
panic. On the other hand`SlimVec` is unwind-safe only to the extent that a
panic handler will not observe UB or any broken invariants of the SlimVec
itself - but elements may be leaked in a few more situations; this may be
referred to as "leak amplification".

It would be possible for `slimvec` to provide similar capabilities in this
regard at the cost of more complex implementation. If this is important to you,
please create an issue and describe your use-case.

### Unstable features

At this time, `slimvec` does not implement features that are unstable in the
Standard Library. This includes:

- `allocator_api` [#32838]
- `try_with_capacity` [#91913]
- `vec_push_within_capacity` [#100486]
- `box_vec_non_null` [#130364]
- `vec_try_remove` [#146954]
- `vec_peek_mut` [#122742]
- `vec_split_at_spare` [#81944]
- `vec_into_chunks` [#142137]
- `vec_recycle` [#148227]
- `push_mut` [#135974]
- `drain_keep_rest` [#101122]

### `allocator_api2`

While the Standard Library's `allocator_api` feature is unstable, `slimvec` is
very interested in supporting the `allocator_api2` library which replicates the
interface in stable Rust.
