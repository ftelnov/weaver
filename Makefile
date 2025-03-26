OS := $(shell uname -s)
ifeq ($(OS), Linux)
	LIB_EXT = so
else
	ifeq ($(OS), Darwin)
		LIB_EXT = dylib
	endif
endif

TEST_LIB=./target/debug/libtests.$(LIB_EXT)

build:
	cargo build --all

test: build
	pytest

lint:
	cargo clippy

publish-dry-run:
	cargo publish --dry-run -p weaver --all-features

publish:
	cargo publish -p weaver --all-features
