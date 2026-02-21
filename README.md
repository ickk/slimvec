`slimvec`
=========

[github](https://github.com/ickk/slimvec)

[`SlimVec`] is analogous to the Standard Library's [`Vec`] collection type,
however it has a smaller inline-size.

[`Vec`]: https://doc.rust-lang.org/alloc/vec/


Overview
--------

`SlimVec` is a dynamic-array containing a generic element type, like the
Standard Library's `Vec`.

An empty `SlimVec` may be created using `SlimVec::new()`, and it will not
allocate until elements are inserted or capacity is explicity reserved.
`SlimVec::new` is a `const fn`.

The `slimvec!` macro may be used to create & populate a `SlimVec` using the
same syntax as the Standard Library's `vec!` macro:
```rust
# use ::slimvec::slimvec;
slimvec![1f32; 5];
slimvec![1usize, 2, 3, 4, 5];
```

`SlimVec<T>` dereferences to `[T]`, so all slice methods are freely available.
This includes utilities such as sorting, searching, and iteration.


Architecture
------------

### Preamble

Dynamic arrays are composed of two main parts; the *buffer* - to contain its
elements; and the *length* & *capacity* - respectively tracking the current
number of elements, and maximum number of elements that may be stored before
needing to reallocate. The *buffer* is placed in dynamically allocated
heap-memory, and the owner holds onto a pointer to this memory.

#### `Vec`

The Standard Library's `Vec` uses its allocated heap-memory solely for the
buffer of elements. Inline it stores three values; the pointer to the buffer,
the *length*, & the *capacity*. This results in an inline-size for each `Vec`
equivalent to three `usize`s as depicted:

```text
    Vec<T>              Allocation
+-------------+       +-------------+
| ptr: 0xabcd | ----> | 0:        A |
|-------------|       |-------------|
| length:   2 |       | 1:        B |
|-------------|       |-------------|
| capacity: 4 |       | 2: <uninit> |
+-------------+       |-------------|
                      | 3: <uninit> |
                      +-------------+
```

### Motivations

The choices that the Standard Library's `Vec` makes are great in the general
case. However there are some circumstances where it may possibly be suboptimal.

This library endeavors to provide an alternative structure for dynamic-arrays
with a reduced inline-size, in order to mitigate this cost when the trade-off
for doing so may be beneficial.

#### As building blocks

Dynamic-arrays are often used as mere building-blocks of more complex
data-structures. The inline-size of such data-structures may end up fairly
large when they are composed of multiple collections.

This larger inline-size is a little bit more expensive to store and move around
on the stack, which in some ways is slightly at odds with idiomatic Rust which
tends to use the stack a lot.

#### As user-data

At times certain kinds of libraries (physics-engines, graphics-APIs,
frameworks, &c.) will provide the application with the ability to associate a
small amount of data, often referred to as "user-data", with an entity or
object otherwise managed by the library; frequently this user-data is large
enough to store only a mere `usize`.

If the application needs to store more data than it can encode in the limited
bits, then it may store in the user-data a pointer to the actual larger data.
Applications that want to use the user-data to associate a `Vec<T>` would in
fact have to store a `Box<Vec<T>>`, since the Standard Library's `Vec` is too
large. This bestows an unfortunate double-indirection when accessing the
elements of the `Vec`.

### `SlimVec`

`SlimVec` stores its *length* & *capacity* alongside the buffer on the heap, as
illustrated:

```text
  SlimVec<T>            Allocation
+-------------+       +-------------+
| ptr: 0xabcd | ----> | length:   2 |
+-------------+       |-------------|
                      | capacity: 4 |
                      |=============|
                      | 0:        A |
                      |-------------|
                      | 1:        B |
                      |-------------|
                      | 2: <uninit> |
                      |-------------|
                      | 3: <uninit> |
                      +-------------+
```

This means the inline-size of a `SlimVec` is just a single pointer:
```rust
# use {::slimvec::SlimVec, ::core::mem::size_of}; struct T([i64; 1024]);
assert!(size_of::<SlimVec<T>>() == size_of::<usize>());
```

`SlimVec`'s inline-data contains a niche, so that `Option`-like enums
containing a `SlimVec` have the same size:
```rust
# use {::slimvec::SlimVec, ::core::mem::size_of}; struct T([i64; 1024]);
assert!(size_of::<SlimVec<T>>() == size_of::<Option<SlimVec<T>>>());
```

`SlimVec` includes a specialisation for the case that its elements are
Zero-Sized-Types (ZSTs), such that it need never allocate. In this special case
the *length* is stored inline.

Otherwise for regular (non-ZST) elements, a `SlimVec` contains a non-null
pointer to its memory-allocation, or a special sentinel-value when the
`SlimVec` has a *capacity* of zero.

The trade-off is that accessing the *length* or *capacity* may be more
expensive as it requires dereferencing the pointer. In return the *length* &
*capacity* need not be lugged around by the owner.


`#![no_std]`
------------

`slimvec` may be used in `no_std` environments by disabling default-features.
It however does require `alloc`.


Testing
-------

`slimvec` is testing against a suite of smoke-tests, a suite of tests ensuring
it implements the same traits as the Standard Library's `Vec`.


Future Work
-----------

Almost all of the API surface area of `Vec` has been replicated for `SlimVec`.

In the future `SlimVec` would like to add support for the currently unstable
Rust features `allocator-api` & `#[may_dangle]`.

Support for ecosystem Traits, e.g. serialisation, is also desirable.

It is desirable to test against the Standard Library's test suite for `Vec`.


Alternatives
------------

### `Box<Vec<T>>`

An obvious dependency-free alternative is to simply place a `Vec<T>` inside a
`Box`. This will indeed reduce the inline size to a single pointer, however
also implies a double-indirection for each access.

### Mozilla's `thin-vec` crate

[`thin-vec`]: https://crates.io/crates/thin-vec

The [`thin-vec`] crate implements a very similar data-structure that is
described by its authors as follows:
> ThinVec is a Vec that stores its length and capacity inline, making it take
> up less space.
>
> Currently this crate mostly exists to facilitate Gecko (Firefox) FFI, but it
> works perfectly fine as a native rust library as well.

The `thin-vec` crate differs to `slimvec` in a few ways:

1. It is concerned with being compatible, to some extent, with a similar C++
   data-structure for the purposes of FFI. There are a lot of subtleties &
   caveats to this in practice; variance in platform endianness, smaller
   maximum capacity, &c.

   Ultimately it is more inscrutible compared to a straight forward
   implementation (which `slimvec` aspires to be); it may be harder to audit,
   maintain, or extend.

2. To avoid allocating, every `ThinVec` with a capacity of zero points to a
   `static` containing a singleton.

   In contrast, `SlimVec` simply uses an inline sentinel-value for this case.

3. `ThinVec::new()` is not `const`.

4. It does not special-case ZST elements, therefore it always needs to allocate
   before the length becomes non-zero.

   In contrast, `SlimVec` is able to avoid allocating for ZST elements.

5. `slimvec` is more complete than `thin-vec` with respect to the Standard
   Library; of note `slimvec` includes an implementation of the `ExtractIf`
   iterator.


Licensing
---------

This library may be licensed permissively; see [LICENSE.md] for details.

[LICENSE.md]: ./LICENSE.md


-------------------------------------------------------------------------------
<footer><small>© ickk 2026.</small></footer>
