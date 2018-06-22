#!/bin/sh
set -ex

case "$TARGET" in
    aarch64-unknown-linux-gnu)
        dpkg --add-architecture arm64
        ;;

    i686-unknown-linux-gnu)
        dpkg --add-architecture i386
        ;;
esac

rustup install "stable-$TARGET"
rustup target add "$TARGET" || true

apt-get update -q

case "$TARGET" in
    x86_64-unknown-linux-gnu)
        apt-get install -qy \
            libseccomp-dev \
            libdbus-1-dev \
            libzmq3-dev
        ;;
    aarch64-unknown-linux-gnu)
        apt-get install -qy gcc-6-aarch64-linux-gnu \
            libseccomp-dev:arm64 \
            libdbus-1-dev:arm64 \
            libzmq3-dev:arm64
        ;;
    i686-unknown-linux-gnu)
        apt-get install -qy gcc-multilib \
            libseccomp-dev:i386 \
            libdbus-1-dev:i386 \
            libzmq3-dev:i386
        ;;
    *)
        echo "UNKNOWN TARGET: $TARGET"
        exit 1
        ;;
esac
