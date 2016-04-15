# We're going to set up a generic Linux system with a Rust musl toolchain,
# which will allow us to produce static Rust binaries.
FROM ubuntu:14.04

# Make sure we have basic dev tools for building C libraries, and any
# shared library dev packages we'll need.
RUN apt-get update && \
    apt-get install -y build-essential curl file sudo git xutils-dev

# Set up our path to include both musl-gcc and our Rust toolchain (once
# they're installed).
ENV PATH=/root/.cargo/bin:/usr/local/musl/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# Install musl-gcc toolchain for building things against a static libc, and
# add it to our path.
WORKDIR /musl
RUN git clone git://git.musl-libc.org/musl && \
    cd musl && \
    ./configure && make install

# Build and install a static version of OpenSSL, and let the Rust
# openssl-sys library know where to find it.
RUN VERS=1.0.2g && \
    curl -O https://www.openssl.org/source/openssl-$VERS.tar.gz && \
    tar xvzf openssl-$VERS.tar.gz && \
    cd openssl-$VERS && \
    env CC=musl-gcc ./config --prefix=/usr/local/musl && \
    make depend && make && make install
ENV OPENSSL_INCLUDE_DIR=/usr/local/musl/include/ \
    OPENSSL_LIB_DIR=/usr/local/musl/lib/ \
    OPENSSL_STATIC=1

# We'll mount our source code on /src for the build.
VOLUME ["/src"]
WORKDIR /src

# Install a Rust compiler and the Rust libraries needed to build against
# musl.  We need to patch the downloaded rustup install script so that it
# doesn't try to use /dev/tty.
RUN curl https://sh.rustup.rs -sSf | sed 's,run "$_file" < /dev/tty,yes | run "$_file",' | sh && \
    . $HOME/.cargo/env && \
    rustup target add x86_64-unknown-linux-musl
