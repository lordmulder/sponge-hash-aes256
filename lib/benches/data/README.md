# SpongeHash-AES256 Stress Test

Please put the input file for stress testing called **`input.txt`** into this directory!

Alternatively, the input file may be specified via the environment variable:  
`SPONGE_BENCH_INPUT_FILE`

## Processing

Every line in the input file will be hashed as a separate input string.

All generated hash values (digests) are added to a hash table to check them for uniqueness.

If any collisions, i.e., duplicate hash values, are found, the test fails!

## Test Files

Test files containing 512 million random and unique passwords are available here:

* <https://github.com/lordmulder/hash-function-test-files>

* <https://codeberg.org/MuldeR/hash-function-test-files>
