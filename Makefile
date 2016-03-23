# A Makefile for building semi-portable credentials-to-env binaries that
# should work on as many Linux distros as possible.  To do this, we build
# on an older Linux distro, and rely on the fact that we only link against
# common, stable shared libraries.

# We use this command to run our build inside a sandboxed Linux
# distribution which has a fairly old libc.  With luck, this should allow
# us to use the same binary on pretty much any modern Linux distribution,
# because it will only rely on an ancient build of libc.
SANDBOXED = docker run -v `pwd`:/src --rm credentials-to-env-dev

# The name the zip archive we use for binary releases.
ZIP = credentials-to-env-$(shell git describe --tags)-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -p).zip

all: build

# Build the docker image we'll need for our sandbox.
image:
	docker build -t credentials-to-env-dev .

# Compile in our sandbox.
build:
	$(SANDBOXED) cargo build --release

package: build
	rm -f $(ZIP)
	zip -j $(ZIP) target/release/credentials-to-env

clean:
	cargo clean

.PHONY: all image build package clean
