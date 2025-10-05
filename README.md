# SpongeHash-AES256

[![no_std](https://img.shields.io/badge/rust-no__std-orchid?logo=rust)](https://docs.rust-embedded.org/book/intro/no-std.html)
[![Crates.io](https://img.shields.io/crates/v/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256)
[![Downloads](https://img.shields.io/crates/d/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256)
[![Release Date](https://img.shields.io/github/release-date/lordmulder/sponge-hash-aes256)](https://crates.io/crates/sponge-hash-aes256/versions)
[![Docs.rs](https://img.shields.io/docsrs/sponge-hash-aes256)](https://docs.rs/sponge-hash-aes256/latest/)
[![License](https://img.shields.io/crates/l/sponge-hash-aes256)](https://opensource.org/license/0BSD)

![SpongeHash-AES256](.assets/images/sponge-hash-aes256.png)

A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).

This hash function has a *variable* output size and can produce outputs of *any* non-zero size.

Please see the [documentation](https://docs.rs/sponge-hash-aes256/latest/) for details! &#x1F4A1;

## Library

The “core” hash algorithm is implemented in the **`sponge-hash-aes256`** crate.

### Installation

In order to use this crate, you have to add it under `[dependencies]` to your **`Cargo.toml`**:

```
[dependencies]
sponge-hash-aes256 = "1.3.0"
```

### Usage

Here is a simple example that demonstrates how to use `SpongeHash256` in your code:

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

## Command-line tool

The **`sponge256sum`** command-line tool can be used as follows:

```
Usage: sponge256sum [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Files to be processed

Options:
  -b, --binary           Read the input file(s) in binary mode, i.e., default mode
  -t, --text             Read the input file(s) in text mode
  -k, --keep-going       Keep going, even if an input file can not be read
  -l, --length <LENGTH>  Digest output size, in bits (default: 256, maximum: 1024)
  -i, --info <INFO>      Include additional context information
  -s, --snail...         Enable "snail" mode, i.e., slow down the hash computation
  -q, --quiet            Do not output any error messages or warnings
  -p, --plain            Print digest(s) in plain format, i.e., without file names
  -f, --flush            Explicitely flush 'stdout' stream after printing a digest
  -h, --help             Print help
  -V, --version          Print version

If no input files are specified, reads input data from 'stdin' stream.
```

## License

This software is released under the BSD Zero Clause (“0BSD”) License.

Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;.
