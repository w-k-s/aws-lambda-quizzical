format:
	cargo fmt

build-categories:
	cargo clean
	CROSS_COMPILE=x86_64-linux-musl cargo build --bin categories --release --target x86_64-unknown-linux-musl
	mv ./target/x86_64-unknown-linux-musl/release/categories ./target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j categories.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
	cargo clean

build-questions:
	cargo clean
	CROSS_COMPILE=x86_64-linux-musl cargo build --bin questions --release --target x86_64-unknown-linux-musl
	mv ./target/x86_64-unknown-linux-musl/release/questions ./target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j questions.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
	cargo clean

build: format build-categories build-questions