# SpongeHash-AES256

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

This hash function has a *variable* output size and can produce outputs of *any* size in the range from 8 to 16,384 (inclusive) bits.

&#x1F4A1; Please see the documentation for details!

# Installation

In order to use this crate, you have to add it under `[dependencies]` to your **`Cargo.toml`**:

```
[dependencies]
sponge-hash-aes256 = "1.0.0"
```

# Usage

Here is a simple example that demonstrates how to use SpongeHash-AES256 in your code:

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
    println!("{:02X?}", &digest)
}
```

# License

Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
