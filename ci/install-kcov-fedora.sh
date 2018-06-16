#!/bin/bash
#  Needed for `cargo kcov`.
#  See https://github.com/SimonKagstrom/kcov/blob/master/INSTALL.md
sudo dnf install -y gcc-c++ elfutils-libelf-devel libcurl-devel binutils-devel elfutils-devel

rm -r ci/tmp/
mkdir -p ci/tmp/
cd ci/tmp/
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz

cd kcov-master
mkdir build
cd build
cmake ..
make
sudo make install
