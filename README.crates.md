# SpongeHash-AES256

![SpongeHash-AES256](https://raw.githubusercontent.com/lordmulder/sponge-hash-aes256/master/.assets/images/sponge-hash-aes256.png)

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

This hash function has a *variable* output size.

## Installation

```
[dependencies]
sponge-hash-aes256 = "1.3.4"
```

## Usage

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

Download the **`sponge256sum`** application here:  
<https://github.com/lordmulder/sponge-hash-aes256/releases>

## License

This software is released under the BSD Zero Clause (“0BSD”) License.

Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;.
