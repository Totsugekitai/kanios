#!/bin/bash
set -xue

QEMU=qemu-system-riscv64
CC=riscv64-unknown-elf-gcc

# CFLAGS="-std=c11 -O2 -g3 -Wall -Wextra --target=riscv64 -ffreestanding -nostdlib"
CFLAGS="-std=c11 -O2 -g3 -Wall -Wextra -ffreestanding -nostdlib"

# シェルをビルド
$CC $CFLAGS -Wl,-Tsrc/user/user.ld -Wl,-Map=src/user/shell.map -o src/user/shell.elf src/user/shell.c src/user/user.c src/user/common.c
cp src/user/shell.elf ./disk/shell.elf

# tarファイルを作成
find ./disk/ -type f | tar --xform='s/.*\///g' -cf disk.tar --format=ustar --files-from=/dev/stdin

# カーネルをビルド
cargo run

# $QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
#     -drive id=drive0,file=disk.tar,format=raw \
#     -device virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0 \
#     -kernel kernel.elf
