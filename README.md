# kanios

kanios is a toy operating system.

## Getting started

### Requirements

- `qemu-system-riscv64`
- `riscv64-unknown-elf-gcc`
- `cargo`
- `rustc` for riscv64-unknown-none-elf
  - `rustup target add riscv-64-unknown-none-elf`, see [the section](https://rust-lang.github.io/rustup/cross-compilation.html)

### Build and Run

`./run.sh`

## Acknowledgements

kanios is inspired by [nuta/operating-system-in-1000-lines](https://github.com/nuta/operating-system-in-1000-lines).
