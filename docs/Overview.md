`SlimVec` is a dynamic-array containing a generic element type, mirroring the
api of the Standard Library's `Vec` with a smaller inline stack size.

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

An empty `SlimVec` may be created using `SlimVec::new()`, and like `Vec` it
will not allocate until elements are inserted or capacity is explicity
reserved. `SlimVec::new` is also a `const fn`.

```rust
use slimvec::SlimVec;
let mut v = SlimVec::new(); // Has not allocated yet
v.push(8); // Allocates when pushing the first element
```

The `slimvec!` macro may be used to create & populate a `SlimVec` using the
same syntax as the Standard Library's `vec!` macro:
```rust
use slimvec::slimvec;
slimvec![1f32; 5];
slimvec![1usize, 2, 3, 4, 5];
```

`SlimVec<T>` dereferences to `[T]`, so all slice methods are freely available.
This includes utilities such as sorting, searching, and iteration.

`slimvec` support `no_std` environments but `alloc` is required.


<details>
<summary>

### Alternatives

</summary>

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

</details>
