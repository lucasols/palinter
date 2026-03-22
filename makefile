build:
	cargo test --color=always -- --test-threads=1 --nocapture --color=always -q
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

test:
	cargo test --color=always -- --test-threads=1 --nocapture --color=always -q	

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

delete_unused_snapshots:
	cargo insta test --unreferenced=delete

-include makefile.local
