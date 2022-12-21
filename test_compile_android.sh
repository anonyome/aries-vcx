#!/usr/bin/env bash
export ANDROID_BUILD_FOLDER=/tmp/android_build
mkdir -p $ANDROID_BUILD_FOLDER

source wrappers/java/ci/setup.android.env.sh

TARGET_ARCH=arm64
#prepare deps:
pushd "${ANDROID_BUILD_FOLDER}"
        download_and_unzip_if_missed "openssl_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/openssl/openssl_$TARGET_ARCH.zip"
        # download_and_unzip_if_missed "libsodium_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/sodium/libsodium_$TARGET_ARCH.zip"
        download_and_unzip_if_missed "libzmq_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/zmq/libzmq_$TARGET_ARCH.zip"
popd


# generate_arch_flags arm64
# export TOOLCHAIN_SYSROOT_LIB="lib"

#setup_dependencies_env_vars:
export OPENSSL_DIR=${ANDROID_BUILD_FOLDER}/openssl_$TARGET_ARCH
# export SODIUM_DIR=${ANDROID_BUILD_FOLDER}/libsodium_$1
export LIBZMQ_DIR=${ANDROID_BUILD_FOLDER}/libzmq_$TARGET_ARCH

# set_env_vars:
# export PKG_CONFIG_ALLOW_CROSS=1

# export OPENSSL_STATIC=1

# export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
# export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include

export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
# export LIBZMQ_PREFIX=${LIBZMQ_DIR}

cargo ndk -t arm64-v8a build --release --package aries-vcx
# cargo build --release --target aarch64-linux-android --package aries-vcx