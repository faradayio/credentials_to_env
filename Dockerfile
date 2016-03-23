# We can use this file to set up a Linux distribution with an ancient C
# library so that the compiled credentials-to-env binary will work on as
# many modern distros as possible.
#
# Wheezy was released around May 4, 2013.
FROM debian:wheezy

# Make sure we have basic dev tools for building C libraries, and any
# shared library dev packages we'll need.
RUN apt-get update && \
    apt-get install -y build-essential libssl-dev curl file sudo

# Install a Rust compiler.
RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh

# We'll mount our source code on /src for the build.
VOLUME ["/src"]
WORKDIR /src
