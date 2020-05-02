# or-iterator
[![Crates.io](https://img.shields.io/crates/v/or-iterator.svg)](https://crates.io/crates/or-iterator)
[![docs.rs](https://docs.rs/or-iterator/badge.svg)](https://docs.rs/or-iterator/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/vbrandl/merging-iterator/blob/master/LICENSE-MIT)

Rust iterator which takes two iterators and return not empty one

```rust
use or_iterator::OrIterator;

let v1 = vec![1, 2, 3];
let v2 = vec![4, 5];
let or = v1.iter().or(v2.iter());
assert_eq!(3, or.count());

let v1 = vec![];
let v2 = vec![4, 5];
let or = v1.iter().or(v2.iter());
assert_eq!(2, or.count());
```

#### Thanks to
Frank Steffahn (https://internals.rust-lang.org/u/steffahn)
