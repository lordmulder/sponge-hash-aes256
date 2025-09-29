# SpongeHash-AES256

[![no_std](https://img.shields.io/badge/rust-no__std-orchid?logo=rust)](https://docs.rust-embedded.org/book/intro/no-std.html)
[![Crates.io](https://img.shields.io/crates/v/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256)
[![Downloads](https://img.shields.io/crates/d/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256)
[![Release Date](https://img.shields.io/github/release-date/lordmulder/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256/versions)
[![Docs.rs](https://img.shields.io/docsrs/sponge-hash-aes256)](https://docs.rs/sponge-hash-aes256/latest/)
[![License](https://img.shields.io/crates/l/sponge-hash-aes256)](https://opensource.org/license/0BSD)

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

This hash function has a *variable* output size and can produce outputs of *any* non-zero size.

Please see the [documentation](https://docs.rs/sponge-hash-aes256/latest/) for details! &#x1F4A1;

## Installation

In order to use this crate, you have to add it under `[dependencies]` to your **`Cargo.toml`**:

```
[dependencies]
sponge-hash-aes256 = "1.1.0"
```

## Usage

Here is a simple example that demonstrates how to use SpongeHash-AES256 in your code:

```rust
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};

fn main() {
    // Create new hash instance
    let mut hash = SpongeHash256::default();

    // Process message
    hash.update(b"The quick brown fox jumps over the lazy dog");

    // Retrieve the final digest
    let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();

    // Print result
    println!("{:02X?}", &digest);
}
```

## License

This software is released under the BSD Zero Clause (“0BSD”) License.

Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;.
