#!/bin/bash
echo "Compiling containers!"
cargo b
echo "Creating the root file system!"
mkdir root
mkdir rootfs
cd rootfs
echo "Downloading alpine linux"
wget http://nl.alpinelinux.org/alpine/v3.7/releases/x86_64/alpine-minirootfs-3.7.0-x86_64.tar.gz -O fs.tar.gz
tar -xzvf fs.tar.gz
rm fs.tar.gz
export ROOTFS=$(pwd)
echo "Done!"
