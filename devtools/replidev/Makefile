target/release/replidev: Cargo.toml Cargo.lock src/
	cargo build --release

$(HOME)/bin/replidev: target/release/replidev
	ln -sf $(CURDIR)/target/release/replidev $(HOME)/bin/replidev

install: $(HOME)/bin/replidev
