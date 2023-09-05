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
echo 'hello hello hello' > ./disk/hello.txt
find ./disk/ -type f | tar --xform='s/.*\///g' -cf disk.tar --format=ustar --files-from=/dev/stdin

# カーネルをビルド＆QEMU起動
cargo run
