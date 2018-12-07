format:
	cargo fmt

build-categories:
	cargo clean
	CROSS_COMPILE=x86_64-linux-musl cargo build --bin categories --release --target x86_64-unknown-linux-musl
	mv ./target/x86_64-unknown-linux-musl/release/categories ./target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j categories.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
	cargo clean

build: format build-categories