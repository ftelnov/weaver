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

build-release:
	cargo build --release --all

doctest:
	cargo test --doc

test: build doctest
	./tests/integration/run_tests.sh

lint:
	cargo fmt --check
	cargo clippy

publish-dry-run:
	cargo publish --dry-run -p weaver --all-features

publish:
	cargo publish -p weaver --all-features

bench: build-release
	echo "Running weaver"
	./tests/bench/exec.sh weaver 19000
	sleep 10
	echo "Running shors"
	./tests/bench/exec.sh shors 19001
