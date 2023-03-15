build:
	TARGET_CC=x86_64-linux-musl-gcc \
	cargo build --release \
		--target x86_64-unknown-linux-gnu \
		--target x86_64-apple-darwin \
		--target x86_64-pc-windows-gnu \
		--target aarch64-apple-darwin
	cp target/aarch64-apple-darwin/release/palinter npm/bin/darwin-arm64
	cp target/x86_64-apple-darwin/release/palinter npm/bin/darwin-x64
	cp target/x86_64-pc-windows-gnu/release/palinter.exe npm/bin/win-x64
	cp target/x86_64-unknown-linux-gnu/release/palinter npm/bin/linux-x64

publish_current:
	cd npm \
	&& pnpm build \
	&& pnpm publish --access public


publish_minor:
	make build
	cd npm \
	&& pnpm version minor --git-tag-version
	make publish_current

publish_patch:
	make build
	cd npm \
	&& pnpm version patch --git-tag-version
	make publish_current

jestor_test:
	cargo run -- --config projects_test/jestor_store_folder.yaml --root ../jestor/web-app

flamegraph:
	cargo flamegraph -- --config projects_test/jestor_store_folder.yaml --root ../jestor/web-app
