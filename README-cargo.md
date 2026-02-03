# SpongeHash-AES256

![SpongeHash-AES256](https://raw.githubusercontent.com/lordmulder/sponge-hash-aes256/master/.assets/images/sponge-hash-aes256.png)

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

This hash function has a *variable* output size.

## Installation

In order to use this crate, add it under `[dependencies]` to your **`Cargo.toml`**:

```
[dependencies]
sponge-hash-aes256 = "1.8.4"
```

## Usage

Here is a simple example that demonstrates how to use `SpongeHash256`:

```rust
use hex::encode_to_slice;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};

fn main() {
    // Create new hash instance
    let mut hash = SpongeHash256::default();

    // Process message
    hash.update(b"The quick brown fox jumps over the lazy dog");

    // Retrieve the final digest
    let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();

    // Encode to hex
    let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
    encode_to_slice(&digest, &mut hex_buffer).unwrap();

    // Print the digest (hex format)
    println!("0x{}", str::from_utf8(&hex_buffer).unwrap());
}
```

## Command-line application

[![sponge256sum](https://raw.githubusercontent.com/lordmulder/sponge-hash-aes256/main/.assets/images/sponge256sum-512px.png)](https://raw.githubusercontent.com/lordmulder/sponge-hash-aes256/main/.assets/images/sponge256sum.png)

Download the **`sponge256sum`** application here:  
&#128279; <https://github.com/lordmulder/sponge-hash-aes256/releases>  
&#128279; <https://codeberg.org/MuldeR/sponge-hash-aes256/releases>  
&#128279; <https://gitlab.com/lord_mulder/sponge-hash-aes256/-/releases>

## License

This software is released under the BSD Zero Clause (“0BSD”) License.

Copyright (C) 2025-2026 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;.
