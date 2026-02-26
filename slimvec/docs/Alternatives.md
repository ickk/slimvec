#### `Box<Vec<T>>`

A dependency-free alternative is to simply place a `Vec<T>` inside a `Box`.
This will indeed reduce the inline size to a single pointer, however also
implies a double-indirection for each access.

#### Mozilla's `thin-vec` crate

[`thin-vec`]: https://crates.io/crates/thin-vec

The [`thin-vec`] crate implements a very similar data-structure that is
described by its authors as follows:
> ThinVec is a Vec that stores its length and capacity inline, making it take
> up less space.
>
> Currently this crate mostly exists to facilitate Gecko (Firefox) FFI, but it
> works perfectly fine as a native rust library as well.

`thin-vec` code is in my opinion a bit strange due to this. It also seems to be
missing some features & APIs compared to the Standard Library `Vec`.
