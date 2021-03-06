include .env
export $(shell sed 's/=.*//' .env)

format:
	cargo fmt

define build
    cargo clean
	CROSS_COMPILE=x86_64-linux-musl cargo build --bin $(1) --release --target x86_64-unknown-linux-musl
	mv ./target/x86_64-unknown-linux-musl/release/$(1) ./target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j $(1).zip ./target/x86_64-unknown-linux-musl/release/bootstrap
	cargo clean
endef

build-categories:
	$(call build,categories)

build-questions:
	$(call build,questions)

build-new-questions:
	$(call build,new_question)

build-update-category-active:
	$(call build,update_category_active)

build: format build-categories build-questions build-new-questions build-update-category-active

test:
	@-TEST_CONN_STRING=$(TEST_CONN_STRING) cargo test -- --nocapture

test-fn:
	@-TEST_CONN_STRING=$(TEST_CONN_STRING) cargo test  -- $(FN_NAME) --nocapture