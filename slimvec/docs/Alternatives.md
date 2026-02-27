#### `Box<Vec<T>>`

A dependency-free alternative is to simply place a `Vec<T>` inside a `Box`.
This will indeed reduce the inline size to a single pointer, however also
implies a double-indirection for each access.

#### `thin-vec`

[`thin-vec`]: https://crates.io/crates/thin-vec

The [`thin-vec`] crate implements a very similar data-structure that is
described by its authors as follows:
> ThinVec is a Vec that stores its length and capacity inline, making it take
> up less space.
>
> Currently this crate mostly exists to facilitate Gecko (Firefox) FFI, but it
> works perfectly fine as a native rust library as well.

`thin-vec` code is in my opinion a bit strange due to its Gecko-ness. It also
seems to be missing some features & APIs compared to `Vec` & `SlimVec`.

#### `fillet`

[`fillet`]: https://crates.io/crates/fillet

The [`fillet`] crate implements a similar data-structure described by it's
author as follows:
> Fillet is null/zero when empty, making it ideal for scenarios where most
> collections are empty and you're storing many references to them. It handles
> zero-sized types (ZSTs) without heap allocations, using `usize` for length.
>
> Fillet is always pointer-sized and zero when empty.
>
> Fillet does not reserve capacity, so repeated `push` operations can be slow
> as they always invoke the allocator and compute layouts. However, `Extend`
> and `FromIterator` use amortized growth and perform similarly to `Vec`.

Fillet does not generally have an amortised growth strategy like `Vec` &
`SlimVec`. Additionally at this time `fillet` does not cover much of the API
surface of `Vec`, & has fewer features.
