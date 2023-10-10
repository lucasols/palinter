build:
	cargo test --color=always -- --test-threads=1 --nocapture --color=always -q
	TARGET_CC=x86_64-linux-musl-gcc \
	cargo build --release \
		--target x86_64-unknown-linux-gnu \
		--target aarch64-unknown-linux-gnu \
		--target x86_64-apple-darwin \
		--target x86_64-pc-windows-gnu \
		--target aarch64-apple-darwin
	rm -rf npm/bin
	mkdir -p npm/bin
	cp target/aarch64-apple-darwin/release/palinter npm/bin/darwin-arm64
	cp target/x86_64-apple-darwin/release/palinter npm/bin/darwin-x64
	cp target/x86_64-pc-windows-gnu/release/palinter.exe npm/bin/win-x64.exe
	cp target/x86_64-unknown-linux-gnu/release/palinter npm/bin/linux-x64
	cp target/aarch64-unknown-linux-gnu/release/palinter npm/bin/linux-arm64

publish_current:
	cd npm \
	&& pnpm build \
	&& pnpm publish --access public

publish_minor:
	make build
	cd npm \
	&& pnpm version minor
	git commit -am "minor version bump"
	make publish_current

publish_patch:
	make build
	cd npm \
	&& pnpm version patch
	git commit -am "patch version bump"
	make publish_current

jestor_test:
	cargo run --release -- --root ../jestor/web-app --config ../jestor/web-app/palinter.yaml

flamegraph:
	cargo flamegraph -- --config projects_test/jestor_store_folder.yaml --root ../jestor/web-app

delete_unused_snapshots:
	cargo insta test --unreferenced=delete

jestor_test_circular_dep:
	cargo run --release -- circular-deps '@src/pages/modals/Find.tsx' --config ../jestor/web-app/palinter.yaml --root ../jestor/web-app

jestor_test_config:
	cargo run -- test-config ../jestor/web-app/tests/palinter-config --config ../jestor/web-app/palinter.yaml
