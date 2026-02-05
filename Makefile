.PHONY: fmt install clean build

all: build

build:
	cargo build --release

install: build
	cp target/release/connect-db ~/bin/

clean:
	cargo clean

fmt:
	cargo fmt