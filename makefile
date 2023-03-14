build:
	cargo build --release
	cp target/release/palinter npm/bin
	cp target/release/palinter /Users/lucasoliveirasantos/Github/jestor/web-app/node_modules/.pnpm/palinter@0.2.0/node_modules/palinter/bin

publish:
	make build
	cd npm && pnpm build &&	pnpm publish --access public

jestor_test:
	cargo run -- --config projects_test/jestor_store_folder.yaml --root ../jestor/web-app
