# SpongeHash-AES256

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

## Installation

```
[dependencies]
sponge-hash-aes256 = "1.0.0"
```

## Usage

```rust
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};

fn main() {
    // Create new hash instance
    let mut hash = SpongeHash256::new();

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
