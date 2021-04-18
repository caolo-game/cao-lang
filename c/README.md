# Cao-Lang C bindings

## Dependencies

-   [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
-   C compiler, I'll use [clang](https://clang.llvm.org/) in the examples

## Compiling the example

```sh
# Build the Rust library and the C header
# Pass --release flag to build in release mode
cargo build

# Compile and link the C app
# The library will be located in the target/release directory if you compiled with --release
clang test.c -l ../target/debug/cao_lang_c
```

Note that the generated header file will be placed in the cao-lang/c directory. You can copy it whereever you please.
