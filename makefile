build:
	cargo build --release
	cp target/release/palinter npm/bin
	chmod +x npm/bin/palinter

publish:
	make build
	cd npm && pnpm build &&	pnpm publish --access public

jestor_test:
	cargo run -- --config projects_test/jestor_store_folder.yaml --root ../jestor/web-app
