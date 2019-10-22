#!/bin/bash

cd "$HOME"
export ARCH="riscv64"
sudo apt-get update
sudo apt-get install -y autoconf automake libglib2.0-dev autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev libusb-1.0-0-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev device-tree-compiler pkg-config libexpat-dev \
    gcc cgdb make
sudo apt-get install -y build-essential zlib1g-dev pkg-config libglib2.0-dev binutils-dev libboost-all-dev autoconf libtool libssl-dev libpixman-1-dev libpython-dev python-pip python-capstone virtualenv

# export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
# export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
wget -O rustup_init.sh https://sh.rustup.rs
sh rustup_init.sh -y --default-toolchain nightly

source "$HOME/.cargo/env"
rustup component add rust-src llvm-tools-preview
(test -x $HOME/.cargo/bin/cargo-objdump || cargo install cargo-binutils)
(test -x $HOME/.cargo/bin/cargo-xbuild || cargo install cargo-xbuild)

# install qemu riscv32/64
[ ! -d qemu-4.1.0/$ARCH-softmmu ] && wget https://download.qemu.org/qemu-4.1.0.tar.xz && tar xJf qemu-4.1.0.tar.xz > /dev/null && cd qemu-4.1.0 && ./configure --target-list=${ARCH}-softmmu && make && cd ..;
echo 'export PATH=$PATH:$HOME/qemu-4.1.0/$ARCH-softmmu:$HOME/qemu-4.1.0' >> "$HOME/.bashrc"

[ ! -d riscv64-linux-musl-cross ] && wget https://musl.cc/riscv64-linux-musl-cross.tgz && tar xvzf riscv64-linux-musl-cross.tgz
echo 'export PATH=$HOME/riscv64-linux-musl-cross/bin:$PATH' >> $HOME/.bashrc
